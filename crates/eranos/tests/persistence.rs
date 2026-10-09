//! The serialised form of a group (feature `serde`). The apps persist and
//! sync groups in this form, so a round trip must be the identity, the JSON
//! shape is pinned (renaming a field breaks every stored group), and a
//! tampered or corrupted file must be refused rather than loaded into an
//! inconsistent group.
#![forbid(unsafe_code)]
#![cfg(feature = "serde")]
#![expect(
    clippy::unwrap_used,
    reason = "a failing test helper panics to report the failure, as clippy.toml allows inside test functions"
)]

use std::future::IntoFuture;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

use eranos::cooperative::block_on;
use eranos::error::ExpenseGroupError;
use eranos::{
    CalendarDate, Category, CategoryId, CategoryTotal, CurrencyCode, Discount, DiscountScope,
    Email, ExchangeRate, Expense, ExpenseGroup, ExpenseGroupId, ExpenseId, ExpenseItem, Iban,
    Language, Member, MemberAmount, MemberId, MemberParts, MemberPercentage, Money, Parts,
    PaymentAccount, Percentage, PhoneNumber, Quantity, SavedExpenseGroup, SettlementStrategy,
    Split, TimeOfDay, TipDistribution, TipRule, Transfer, TransferId, Username,
};
use serde_json::{Value, json};

fn eur(text: &str) -> Money {
    Money::parse(text, CurrencyCode::EUR).unwrap()
}

fn day() -> CalendarDate {
    CalendarDate::new(2026, 9, 23).unwrap()
}

/// The group of `trip`, with Ana, who gives every member detail, and Carol,
/// who is archived.
struct Trip {
    group: ExpenseGroup,
    ana: MemberId,
    carol: MemberId,
}

/// A group using every serialised feature, and the two members the tests
/// look up.
fn trip() -> Trip {
    let mut group = ExpenseGroup::builder("Lisbon", CurrencyCode::EUR)
        .note("Autumn trip")
        .language(Language::parse("pt-PT").unwrap())
        .build()
        .unwrap();
    let ana = group
        .add_member(
            Member::builder("Ana")
                .last_name("Popescu")
                .email(Email::parse("ana@example.com").unwrap())
                .phone(PhoneNumber::parse("+40712345678").unwrap())
                .account(
                    PaymentAccount::new("ING", Iban::parse("RO49AAAA1B31007593840000").unwrap())
                        .unwrap(),
                )
                .account(PaymentAccount::new("Revolut", Username::new("@ana").unwrap()).unwrap())
                .build()
                .unwrap(),
        )
        .unwrap();
    let bob = group
        .add_member(Member::builder("Bob").build().unwrap())
        .unwrap();
    let carol = group
        .add_member(Member::builder("Carol").build().unwrap())
        .unwrap();
    let food = group.add_category(Category::new("Food").unwrap()).unwrap();

    let bill = Expense::builder(
        "Bill",
        eur("19.50"),
        bob,
        [
            ExpenseItem::builder("Bacalhau", eur("12.00"), [ana])
                .build()
                .unwrap(),
            // 1.5 l of wine at 6.00 a litre, 1.00 off each litre.
            ExpenseItem::builder(
                "Wine",
                eur("6.00"),
                Split::parts([
                    MemberParts::new(ana, Parts::ONE),
                    MemberParts::new(bob, Parts::parse("1.5").unwrap()),
                ]),
            )
            .quantity(Quantity::parse("1.5").unwrap())
            .discount(Discount::fixed(eur("1.00")).scoped(DiscountScope::Unit))
            .build()
            .unwrap(),
        ],
        day(),
    )
    .note("Tasca")
    .time(TimeOfDay::new(20, 30).unwrap())
    .category(food)
    .tip(
        TipRule::percentage(Percentage::parse("12.5").unwrap()).distributed(TipDistribution::Equal),
    )
    .build()
    .unwrap();
    group.add_expense(bill).unwrap();
    let tram = Expense::builder(
        "Tram",
        Money::parse("30.00", CurrencyCode::RON)
            .unwrap()
            .at(ExchangeRate::parse("0.2011").unwrap()),
        Split::amounts([
            MemberAmount::new(ana, Money::parse("20.00", CurrencyCode::RON).unwrap()),
            MemberAmount::new(carol, Money::parse("10.00", CurrencyCode::RON).unwrap()),
        ]),
        Split::percentages([
            MemberPercentage::new(ana, Percentage::parse("50").unwrap()),
            MemberPercentage::new(carol, Percentage::parse("50").unwrap()),
        ]),
        day(),
    )
    .build()
    .unwrap();
    group.add_expense(tram).unwrap();
    group
        .add_transfer(
            Transfer::builder(ana, bob, eur("5.00"), day())
                .note("Wine")
                .build()
                .unwrap(),
        )
        .unwrap();
    group.archive_member(carol).unwrap();
    Trip { group, ana, carol }
}

fn json_of(group: &ExpenseGroup) -> Value {
    serde_json::to_value(group).unwrap()
}

/// Loading a saved group must give back the same group, the same totals,
/// and the same plans; anything else silently changes what people owe.
#[test]
fn a_group_round_trips_through_json() {
    let Trip { group, carol, .. } = trip();
    let text = serde_json::to_string(&group).unwrap();
    let loaded: ExpenseGroup = serde_json::from_str(&text).unwrap();
    assert_eq!(loaded, group);
    assert_eq!(loaded.summary(), group.summary());
    for &strategy in SettlementStrategy::all() {
        assert_eq!(
            loaded.settlement(strategy).unwrap(),
            group.settlement(strategy).unwrap()
        );
    }
    // The archived member stays archived, and keeps her history.
    assert_eq!(loaded.is_archived(carol), Some(true));
    assert_eq!(serde_json::to_string(&loaded).unwrap(), text);
}

/// The field names and value shapes are the stored format; renaming one
/// would make every saved group unreadable.
#[test]
fn the_json_shape_is_pinned() {
    let Trip { group, ana, carol } = trip();
    let value = json_of(&group);
    assert_eq!(value["format"], json!(4));
    assert_eq!(value["id"], json!(group.id().to_string()));
    assert_eq!(value["name"], json!("Lisbon"));
    assert_eq!(value["currency"], json!("EUR"));
    assert_eq!(value["language"], json!("pt-PT"));
    // The version leads, then the identifier, then what describes the group
    // and what it holds; every entry starts with its own identifier.
    let text = serde_json::to_string(&group).unwrap();
    let start = format!(
        r#"{{"format":4,"id":"{}","name":"Lisbon","currency":"EUR","note":"Autumn trip","language":"pt-PT","members":[{{"id":"#,
        group.id()
    );
    assert!(text.starts_with(&start), "{text}");
    let first_member = format!(r#""members":[{{"id":"{ana}","first_name":"Ana","#);
    assert!(text.contains(&first_member), "{text}");
    // Members carry their details; the archived ones are listed apart.
    let members = value["members"].as_array().unwrap();
    assert_eq!(members[0]["id"], json!(ana.to_string()));
    assert_eq!(members[2]["id"], json!(carol.to_string()));
    assert_eq!(value["archived"], json!([carol.to_string()]));
    assert_eq!(members[0].get("archived"), None);
    assert_eq!(
        members[0]["accounts"],
        json!([
            {"details": {"iban": "RO49AAAA1B31007593840000"}, "name": "ING"},
            {"details": {"username": "@ana"}, "name": "Revolut"},
        ])
    );
    assert_eq!(members[0]["phone"], json!("+40712345678"));
    // Entries carry their identifier; amounts are minor units plus a code.
    let bill_id = group.expenses().next().unwrap().id();
    let bill = &value["expenses"][0];
    assert_eq!(bill["id"], json!(bill_id.to_string()));
    assert_eq!(
        bill["amount"],
        json!({"currency": "EUR", "minor_units": 1950})
    );
    assert_eq!(bill["date"], json!("2026-09-23"));
    assert_eq!(bill["time"], json!("20:30"));
    assert_eq!(
        bill["tip"],
        json!({"value": {"percentage": 1250}, "distribution": "equal"})
    );
    assert_eq!(
        bill["payers"],
        json!({"equal": [value["members"][1]["id"]]})
    );
    assert_eq!(
        bill["split"]["itemised"][1]["split"]["parts"][1][1],
        json!(150)
    );
    // A quantity is its decimal text, left out at one; a discount is its
    // value and its scope, left out without one; a line total, before the
    // discount or after it, is derived, so never written.
    let items = &bill["split"]["itemised"];
    assert_eq!(items[1]["quantity"], json!("1.5"));
    assert_eq!(items[0].get("quantity"), None);
    assert_eq!(
        items[1]["discount"],
        json!({"value": {"fixed": {"minor_units": 100, "currency": "EUR"}}, "scope": "unit"})
    );
    assert_eq!(items[0].get("discount"), None);
    let wine = r#""quantity":"1.5","discount":{"value":{"fixed":{"minor_units":100,"currency":"EUR"}},"scope":"unit"}}"#;
    assert!(text.contains(wine), "{text}");
    assert_eq!(items[1].get("line_total_before_discount"), None);
    assert_eq!(items[1].get("line_total"), None);
    assert_eq!(bill.get("total"), None);
    let tram = &value["expenses"][1];
    assert_eq!(tram["exchange_rate"], json!("0.2011"));
    assert_eq!(tram["split"]["whole"]["percentages"][0][1], json!(5000));
    assert_eq!(value["transfers"][0]["note"], json!("Wine"));
    // The built-in category is not written, and neither is an expense's
    // category when it is the built-in one.
    let food = group
        .categories()
        .find(|category| "Food" == category.name())
        .unwrap()
        .id();
    assert_eq!(
        value["categories"],
        json!([{"id": food.to_string(), "name": "Food"}])
    );
    assert_eq!(bill["category"], json!(food.to_string()));
    assert_eq!(tram.get("category"), None);
    // Neither is a tip when it is the default, no tip at all.
    assert_eq!(tram.get("tip"), None);
}

/// Loading a file with an entry the library would never accept must fail,
/// or a corrupted file would produce balances that do not add up.
#[test]
fn a_tampered_file_is_refused() {
    let Trip { group, .. } = trip();
    let tamper = |change: &dyn Fn(&mut Value)| {
        let mut value = json_of(&group);
        change(&mut value);
        serde_json::from_value::<ExpenseGroup>(value)
            .map(|_| ())
            .unwrap_err()
            .to_string()
    };

    // Percentages that no longer add up to 100 %.
    let unbalanced = tamper(&|value| {
        value["expenses"][1]["split"]["whole"]["percentages"][0][1] = json!(4999);
    });
    assert!(
        unbalanced.contains("percentages add up to 9999"),
        "{unbalanced}"
    );
    // A discount larger than the price it reduces, and one above 100 %
    // saved without its scope.
    let above_price = tamper(&|value| {
        value["expenses"][0]["split"]["itemised"][1]["discount"]["value"]["fixed"]["minor_units"] =
            json!(601);
    });
    assert!(
        above_price.contains("the discount is larger than the price it reduces"),
        "{above_price}"
    );
    let above_hundred = tamper(&|value| {
        value["expenses"][0]["split"]["itemised"][0]["discount"] =
            json!({"value": {"percentage": 10_001}});
    });
    assert!(
        above_hundred.contains("a discount cannot be more than 100%"),
        "{above_hundred}"
    );
    // A discount the items no longer add up to the amount with.
    let rediscounted = tamper(&|value| {
        value["expenses"][0]["split"]["itemised"][1]["discount"]["scope"] = json!("line");
    });
    assert!(
        rediscounted.contains("the items add up to 20.00 EUR instead of 19.50 EUR"),
        "{rediscounted}"
    );
    // An expense paid by someone who is not a member.
    let stranger = "stranger";
    let unknown_payer = tamper(&|value| {
        value["expenses"][0]["payers"] = json!({"equal": [stranger]});
    });
    assert!(
        unknown_payer.contains("no member has this identifier"),
        "{unknown_payer}"
    );
    // An IBAN with a typo.
    let typo = tamper(&|value| {
        value["members"][0]["accounts"][0]["details"]["iban"] = json!("RO49AAAA1B31007593840001");
    });
    assert!(typo.contains("check digits"), "{typo}");
    // An IBAN with a digit where the bank code has letters.
    let malformed = tamper(&|value| {
        value["members"][0]["accounts"][0]["details"]["iban"] = json!("RO491AAA1B31007593840000");
    });
    assert!(malformed.contains("this country's format"), "{malformed}");
    // A language tag the apps' locale APIs throw on.
    let language = tamper(&|value| {
        value["language"] = json!("en-US-US");
    });
    assert!(language.contains("invalid language tag"), "{language}");
    // An email address with a doubled dot.
    let email = tamper(&|value| {
        value["members"][0]["email"] = json!("ana..popescu@example.com");
    });
    assert!(email.contains("invalid email address"), "{email}");
    // A phone number in a range its country does not have.
    let phone = tamper(&|value| {
        value["members"][0]["phone"] = json!("+40612345678");
    });
    assert!(phone.contains("invalid phone number"), "{phone}");
    // A blank first name, and a foreign expense that lost its rate.
    let blank_name = tamper(&|value| {
        value["members"][1]["first_name"] = json!("  ");
    });
    assert!(blank_name.contains("first name"), "{blank_name}");
    let unrated = tamper(&|value| {
        value["expenses"][1]
            .as_object_mut()
            .unwrap()
            .remove("exchange_rate");
    });
    assert!(unrated.contains("exchange rate from RON"), "{unrated}");
    // A newer format is refused instead of read with fields dropped, and an
    // older one because no earlier shape is read.
    let newer = tamper(&|value| {
        value["format"] = json!(5);
    });
    assert!(newer.contains("unsupported group format 5"), "{newer}");
    let older = tamper(&|value| {
        value["format"] = json!(2);
    });
    assert!(
        older.contains("unsupported group format 2: this version reads format 4"),
        "{older}"
    );
}

/// A file must also be refused when each entry is valid on its own but the
/// group as a whole is not, or loading would merge or drop entries.
#[test]
fn an_inconsistent_file_is_refused() {
    let Trip { group, .. } = trip();
    let tamper = |change: &dyn Fn(&mut Value)| {
        let mut value = json_of(&group);
        change(&mut value);
        serde_json::from_value::<ExpenseGroup>(value)
            .map(|_| ())
            .unwrap_err()
            .to_string()
    };

    // A group without a name.
    let nameless = tamper(&|value| value["name"] = json!("  "));
    assert!(nameless.contains("group name"), "{nameless}");
    // Two members with one ID.
    let twins = tamper(&|value| value["members"][1]["id"] = value["members"][0]["id"].clone());
    assert!(twins.contains("already exists"), "{twins}");
    // Two categories with one name, in different cases.
    let other = "other";
    let homonyms = tamper(&|value| {
        let copy = json!({"id": other, "name": "FOOD"});
        value["categories"].as_array_mut().unwrap().push(copy);
    });
    assert!(homonyms.contains("already has this name"), "{homonyms}");
    // A transfer from someone who is not a member.
    let stranger = "stranger";
    let unknown_sender = tamper(&|value| value["transfers"][0]["sender"] = json!(stranger));
    assert!(
        unknown_sender.contains("no member has this identifier"),
        "{unknown_sender}"
    );
    // The built-in category listed as if it were the group's own.
    let built_in = tamper(&|value| {
        let copy = json!({"id": CategoryId::UNCATEGORISED.to_string(), "name": "Uncategorised"});
        value["categories"].as_array_mut().unwrap().push(copy);
    });
    assert!(built_in.contains("already exists"), "{built_in}");
    // An expense listed twice.
    let repeated = tamper(&|value| {
        let copy = value["expenses"][0].clone();
        value["expenses"].as_array_mut().unwrap().push(copy);
    });
    assert!(repeated.contains("already exists"), "{repeated}");
    // Someone archived who is not a member.
    let archived_stranger = tamper(&|value| value["archived"] = json!([stranger]));
    assert!(
        archived_stranger.contains("no member has this identifier"),
        "{archived_stranger}"
    );
}

/// An app that loads through `SavedExpenseGroup` must learn why the group refused a
/// file, or it could not tell a file from a newer release from a damaged
/// one.
#[test]
fn a_saved_group_names_why_the_group_refuses_it() {
    let Trip { group, ana, .. } = trip();
    let load = |change: &dyn Fn(&mut Value)| {
        let mut value = json_of(&group);
        change(&mut value);
        serde_json::from_value::<SavedExpenseGroup>(value)
            .unwrap()
            .into_expense_group()
            .map(|loaded| json_of(&loaded))
    };

    // A file this version wrote loads unchanged.
    assert_eq!(load(&|_| ()), Ok(json_of(&group)));
    // A newer format and an older one, each named in the error.
    assert_eq!(
        load(&|value| value["format"] = json!(5)),
        Err(ExpenseGroupError::UnsupportedFormat { format: 5 })
    );
    assert_eq!(
        load(&|value| value["format"] = json!(2)),
        Err(ExpenseGroupError::UnsupportedFormat { format: 2 })
    );
    // A newer file of a shape this version does not know still names its
    // format: the reader stops at the key, wherever it stands, and reads
    // nothing else.
    let unknown_shape = serde_json::from_value::<SavedExpenseGroup>(json!({
        "entries": {"shape": "of a later release"},
        "format": 5,
        "members": "none of this is read",
    }))
    .unwrap()
    .into_expense_group();
    assert_eq!(
        unknown_shape.map(|loaded| json_of(&loaded)),
        Err(ExpenseGroupError::UnsupportedFormat { format: 5 })
    );
    // A key this version does not know is skipped, so a file of this format
    // written by a later release loads.
    assert_eq!(
        load(&|value| value["future"] = json!({"any": "shape"})),
        Ok(json_of(&group))
    );
    // A key the file repeats, or a key it lacks, is a damaged file.
    let read = |text: &str| {
        serde_json::from_str::<SavedExpenseGroup>(text)
            .map(|_| ())
            .unwrap_err()
            .to_string()
    };
    let text = serde_json::to_string(&group).unwrap();
    let repeated = read(&text.replacen(r#""format":4,"#, r#""format":4,"format":4,"#, 1));
    assert!(repeated.contains("duplicate field `format`"), "{repeated}");
    let without_format = read(&text.replacen(r#""format":4,"#, "", 1));
    assert!(
        without_format.contains("missing field `format`"),
        "{without_format}"
    );
    let without_id = read(&text.replacen(&format!(r#""id":"{}","#, group.id()), "", 1));
    assert!(without_id.contains("missing field `id`"), "{without_id}");
    let without_name = read(&text.replacen(r#""name":"Lisbon","#, "", 1));
    assert!(
        without_name.contains("missing field `name`"),
        "{without_name}"
    );
    let without_currency = read(&text.replacen(r#""currency":"EUR","#, "", 1));
    assert!(
        without_currency.contains("missing field `currency`"),
        "{without_currency}"
    );
    // A blank group name, and two members with one ID.
    assert_eq!(
        load(&|value| value["name"] = json!("  ")),
        Err(ExpenseGroupError::EmptyName)
    );
    assert_eq!(
        load(&|value| value["members"][1]["id"] = value["members"][0]["id"].clone()),
        Err(ExpenseGroupError::DuplicateMember(ana))
    );
    // Someone archived who is not a member.
    let stranger = MemberId::new("stranger").unwrap();
    assert_eq!(
        load(&|value| value["archived"] = json!([stranger.to_string()])),
        Err(ExpenseGroupError::UnknownMember(stranger))
    );
    // An entry that could never have been built fails the read itself.
    let mut typo = json_of(&group);
    typo["members"][0]["accounts"][0]["details"]["iban"] = json!("RO49AAAA1B31007593840001");
    assert!(serde_json::from_value::<SavedExpenseGroup>(typo).is_err());
}

/// A file whose identifiers are other text than UUIDs would not load, would
/// come back with its identifiers changed, or would list a category whose
/// identifier sorts before the nil UUID text ahead of Uncategorised.
#[test]
fn identifiers_load_as_written_and_uncategorised_stays_first() {
    let mut group = ExpenseGroup::builder("Lisbon", CurrencyCode::EUR)
        .build()
        .unwrap();
    let ana = group
        .add_member(Member::builder("Ana").build().unwrap())
        .unwrap();
    let bob = group
        .add_member(Member::builder("Bob").build().unwrap())
        .unwrap();
    let food = group.add_category(Category::new("Food").unwrap()).unwrap();
    let everyone = Split::equal([ana, bob]);
    let dinner = Expense::builder("Dinner", eur("30.00"), ana, everyone.clone(), day())
        .category(food)
        .build()
        .unwrap();
    group.add_expense(dinner).unwrap();
    let taxi = Expense::builder("Taxi", eur("10.00"), bob, everyone, day())
        .build()
        .unwrap();
    group.add_expense(taxi).unwrap();
    let text = serde_json::to_string(&group)
        .unwrap()
        .replace(&group.id().to_string(), "Lisbon 2026")
        .replace(&ana.to_string(), "ana")
        .replace(&bob.to_string(), "Bob")
        .replace(&food.to_string(), "-food");

    let loaded: ExpenseGroup = serde_json::from_str(&text).unwrap();
    assert_eq!(loaded.id().as_str(), "Lisbon 2026");
    // Members in the byte order of their identifiers: uppercase first.
    assert_eq!(
        loaded
            .members()
            .map(|member| member.id().to_string())
            .collect::<Vec<_>>(),
        ["Bob", "ana"]
    );
    // `-food` sorts before the nil UUID text, and is listed after it.
    let listed = [CategoryId::UNCATEGORISED, CategoryId::new("-food").unwrap()];
    assert!(CategoryId::UNCATEGORISED > listed[1]);
    assert_eq!(
        loaded.categories().map(Category::id).collect::<Vec<_>>(),
        listed
    );
    let summary = loaded.summary();
    assert_eq!(
        summary
            .category_totals()
            .map(CategoryTotal::category)
            .collect::<Vec<_>>(),
        listed
    );
    assert_eq!(
        summary
            .member_category_totals(MemberId::new("Bob").unwrap())
            .map(CategoryTotal::category)
            .collect::<Vec<_>>(),
        listed
    );
    // Saved again, the identifiers read back as they were written.
    let saved = serde_json::to_string(&loaded).unwrap();
    assert_eq!(
        serde_json::from_str::<ExpenseGroup>(&saved).unwrap(),
        loaded
    );
    // An identifier the rules refuse fails the read, naming the rule.
    let padded = text.replace("\"ana\"", "\" ana\"");
    let refused = serde_json::from_str::<ExpenseGroup>(&padded).unwrap_err();
    assert!(
        refused
            .to_string()
            .contains("an identifier cannot start or end with whitespace"),
        "{refused}"
    );
}

/// A group whose app gave it and its entries identifiers of its own would
/// not save, or would load under other identifiers, in another order, or
/// with other totals.
#[test]
fn a_group_with_given_identifiers_round_trips() {
    let mut group = ExpenseGroup::builder_with_id(
        ExpenseGroupId::new("zz").unwrap(),
        "Lisbon",
        CurrencyCode::EUR,
    )
    .build()
    .unwrap();
    let member = |id: MemberId, name: &str| Member::builder_with_id(id, name).build().unwrap();
    let ana = group
        .add_member(member(MemberId::new("!a").unwrap(), "Ana"))
        .unwrap();
    let bob = group
        .add_member(member(MemberId::new("Bøb").unwrap(), "Bob"))
        .unwrap();
    let carol = group
        .add_member(Member::builder("Carol").build().unwrap())
        .unwrap();
    let food = Category::with_id(CategoryId::new("-food").unwrap(), "Food").unwrap();
    let food = group.add_category(food).unwrap();
    let fuel = group.add_category(Category::new("Fuel").unwrap()).unwrap();
    let dinner = Expense::builder_with_id(
        ExpenseId::new("晚餐").unwrap(),
        "Dinner",
        eur("30.00"),
        ana,
        Split::equal([ana, bob, carol]),
        day(),
    )
    .category(food)
    .build()
    .unwrap();
    let dinner = group.add_expense(dinner).unwrap();
    let taxi = Expense::builder(
        "Taxi",
        eur("12.00"),
        carol,
        Split::equal([bob, carol]),
        day(),
    )
    .category(fuel)
    .build()
    .unwrap();
    let taxi = group.add_expense(taxi).unwrap();
    let repayment = Transfer::builder_with_id(
        TransferId::new("zz").unwrap(),
        bob,
        ana,
        eur("10.00"),
        day(),
    )
    .build()
    .unwrap();
    let repayment = group.add_transfer(repayment).unwrap();

    let json = serde_json::to_string(&group).unwrap();
    for text in [r#""zz""#, r#""!a""#, r#""Bøb""#, r#""-food""#, r#""晚餐""#] {
        assert!(json.contains(text), "{text}");
    }
    let loaded: ExpenseGroup = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded, group);
    assert_eq!(loaded.summary(), group.summary());
    assert_eq!(loaded.id().as_str(), "zz");
    // Byte order: `!` before the digits of generated text, before `B`.
    assert_eq!(
        loaded.members().map(Member::id).collect::<Vec<_>>(),
        [ana, carol, bob]
    );
    // Uncategorised first, then `-food` and the generated text.
    assert_eq!(
        loaded.categories().map(Category::id).collect::<Vec<_>>(),
        [CategoryId::UNCATEGORISED, food, fuel]
    );
    // A generated identifier's digits before the bytes of `晚`.
    assert_eq!(
        loaded.expenses().map(Expense::id).collect::<Vec<_>>(),
        [taxi, dinner]
    );
    assert_eq!(
        loaded.transfers().map(Transfer::id).collect::<Vec<_>>(),
        [repayment]
    );
    assert_eq!(serde_json::to_string(&loaded).unwrap(), json);
}

/// A load would hold its thread for the whole file, or leave half a group
/// behind when it is dropped: it yields once per 1 024 replayed steps, a
/// dropped load leaves nothing, and a copy of the saved form loads the
/// same group.
#[test]
fn a_saved_group_loads_asynchronously() {
    // 3 members, 2 000 expenses, and 100 transfers: 2 103 steps, so the
    // load yields after the 1 024th and the 2 048th.
    let mut group = ExpenseGroup::builder("Club", CurrencyCode::EUR)
        .build()
        .unwrap();
    let members: Vec<MemberId> = ["Ana", "Bob", "Carol"]
        .into_iter()
        .map(|name| {
            group
                .add_member(Member::builder(name).build().unwrap())
                .unwrap()
        })
        .collect();
    let everyone = Split::equal(members.iter().copied());
    for _ in 0..2_000 {
        let expense = Expense::builder("Coffee", eur("3.00"), members[0], everyone.clone(), day());
        group.add_expense(expense.build().unwrap()).unwrap();
    }
    for _ in 0..100 {
        let transfer = Transfer::builder(members[1], members[0], eur("1.00"), day());
        group.add_transfer(transfer.build().unwrap()).unwrap();
    }
    let saved: SavedExpenseGroup = serde_json::from_value(json_of(&group)).unwrap();

    let mut context = Context::from_waker(Waker::noop());
    {
        let mut load = pin!(saved.clone().into_expense_group_async().into_future());
        assert!(load.as_mut().poll(&mut context).is_pending());
    }
    let mut load = pin!(saved.clone().into_expense_group_async().into_future());
    for _ in 0..2 {
        assert!(load.as_mut().poll(&mut context).is_pending());
    }
    let Poll::Ready(Ok(loaded)) = load.as_mut().poll(&mut context) else {
        panic!("the load did not complete after its two yields");
    };
    assert_eq!(loaded, group);
    assert_eq!(loaded.summary(), group.summary());
    assert_eq!(block_on(saved.into_expense_group_async()), Ok(group));
}

/// A group with none of the optional details round-trips as well, without
/// gaining any.
#[test]
fn a_bare_group_round_trips_through_json() {
    let group = ExpenseGroup::builder("Bare", CurrencyCode::EUR)
        .build()
        .unwrap();
    let text = serde_json::to_string(&group).unwrap();
    assert!(!text.contains("note"), "{text}");
    let loaded: ExpenseGroup = serde_json::from_str(&text).unwrap();
    assert_eq!(loaded, group);
    assert_eq!(loaded.note(), None);
    assert_eq!(loaded.language(), None);
}

/// Account details are stored in clear text, for the app's secure storage,
/// and a log that printed a member would hold them: the saved form holds
/// them in full, and `Debug` holds none of them.
#[test]
fn account_details_serialise_in_clear_text_and_stay_out_of_debug_output() {
    let Trip { group, ana, .. } = trip();
    let text = serde_json::to_string(&group).unwrap();
    assert!(text.contains("RO49AAAA1B31007593840000"));
    assert!(text.contains("@ana"));
    let debug = format!("{:?}", group.member(ana).unwrap());
    assert_eq!(debug, format!("Member {{ id: {ana:?}, .. }}"));
}

/// Value types serialise on their own in the same shapes as inside a
/// group.
#[test]
fn value_types_serialise_on_their_own() {
    assert_eq!(
        serde_json::to_value(eur("12.34")).unwrap(),
        json!({"minor_units": 1234, "currency": "EUR"})
    );
    assert_eq!(
        serde_json::to_value(Percentage::parse("33.33").unwrap()).unwrap(),
        json!(3333)
    );
    assert_eq!(
        serde_json::from_value::<Parts>(json!(50)).unwrap(),
        Parts::parse("0.5").unwrap()
    );
    assert!(serde_json::from_value::<Parts>(json!(0)).is_err());
    assert!(serde_json::from_value::<CurrencyCode>(json!("XYZ")).is_err());
    assert!(serde_json::from_value::<CalendarDate>(json!("2026-02-30")).is_err());
    assert_eq!(
        serde_json::from_value::<TimeOfDay>(json!("07:05")).unwrap(),
        TimeOfDay::new(7, 5).unwrap()
    );
    assert!(serde_json::from_value::<ExchangeRate>(json!("0")).is_err());
    assert_eq!(
        serde_json::to_value(Discount::percentage(Percentage::parse("10").unwrap())).unwrap(),
        json!({"value": {"percentage": 1000}, "scope": "line"})
    );
    assert_eq!(
        serde_json::from_value::<Discount>(json!({"value": {"percentage": 1000}})).unwrap(),
        Discount::percentage(Percentage::parse("10").unwrap())
    );
}

/// A group the library accepted must load: the entries replay in the order
/// of their identifiers, not the order they were added in, and the sums
/// between two of them can leave the range of an amount where the group's
/// never did, so the totals are range-checked once, after the last entry.
#[test]
fn a_group_the_library_accepted_loads_whatever_order_its_entries_replay_in() {
    let mut group = ExpenseGroup::builder("Edge", CurrencyCode::EUR)
        .build()
        .unwrap();
    let maria = group
        .add_member(
            Member::builder_with_id(MemberId::new("maria").unwrap(), "Maria")
                .build()
                .unwrap(),
        )
        .unwrap();
    let nina = group
        .add_member(
            Member::builder_with_id(MemberId::new("nina").unwrap(), "Nina")
                .build()
                .unwrap(),
        )
        .unwrap();
    let huge = Money::from_minor_units(i64::MAX - 2, CurrencyCode::EUR);
    // M pays one cent for N, sends N nearly the range, gets it back, and
    // pays ten cents for N: every total stays in range in this order.
    let expenses = ["first", "second"].map(|id| {
        Expense::builder_with_id(
            ExpenseId::new(id).unwrap(),
            id,
            eur("0.01"),
            maria,
            Split::equal([nina]),
            day(),
        )
        .build()
        .unwrap()
    });
    let [first, second] = expenses;
    let second = second.to_builder().cost(eur("0.10")).build().unwrap();
    group.add_expense(first).unwrap();
    group
        .add_transfer(
            Transfer::builder_with_id(TransferId::new("t1").unwrap(), maria, nina, huge, day())
                .build()
                .unwrap(),
        )
        .unwrap();
    group
        .add_transfer(
            Transfer::builder_with_id(TransferId::new("t2").unwrap(), nina, maria, huge, day())
                .build()
                .unwrap(),
        )
        .unwrap();
    group.add_expense(second).unwrap();
    // Replayed as saved, both expenses come before the first transfer, whose
    // amount then leaves the range for a moment.
    let loaded: ExpenseGroup =
        serde_json::from_str(&serde_json::to_string(&group).unwrap()).unwrap();
    assert_eq!(loaded, group);
    // A file whose entries together leave the range is still refused, as
    // the typed error, once every entry has been checked.
    let mut value = json_of(&group);
    let mut again = value["transfers"][0].clone();
    again["id"] = json!("t3");
    value["transfers"].as_array_mut().unwrap().push(again);
    let refused = serde_json::from_value::<SavedExpenseGroup>(value)
        .unwrap()
        .into_expense_group()
        .map(|refused_group| refused_group.id());
    assert_eq!(refused, Err(ExpenseGroupError::Overflow));
}
