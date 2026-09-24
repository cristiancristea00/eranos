//! The serialised form of a group (feature `serde`). The apps persist and
//! sync groups in this form, so a round trip must be the identity, the JSON
//! shape is pinned (renaming a field breaks every stored group), and a
//! tampered or corrupted file must be refused rather than loaded into an
//! inconsistent group.
#![cfg(feature = "serde")]
#![expect(
    clippy::unwrap_used,
    reason = "a failing test helper panics to report the failure, as clippy.toml allows inside test functions"
)]

use eranos::{
    Account, CalendarDate, Category, CategoryId, Currency, Email, ExchangeRate, Expense, Group,
    Iban, Item, Language, Member, MemberAmount, MemberId, MemberParts, MemberPercentage, Money,
    Parts, Percentage, PhoneNumber, SettlementStrategy, Split, TimeOfDay, Tip, TipDistribution,
    Transfer, Username,
};
use serde_json::{Value, json};

fn eur(text: &str) -> Money {
    Money::parse(text, Currency::EUR).unwrap()
}

fn day() -> CalendarDate {
    CalendarDate::new(23, 9, 2026).unwrap()
}

/// A group using every serialised feature, and its members.
fn group() -> (Group, MemberId, MemberId, MemberId) {
    let mut group = Group::builder(Currency::EUR, "Lisbon")
        .description("Autumn trip")
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
                    Account::new(Iban::parse("RO49AAAA1B31007593840000").unwrap(), "ING").unwrap(),
                )
                .account(Account::new(Username::new("@ana").unwrap(), "Revolut").unwrap())
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

    let bill = Expense::itemised_builder(
        eur("21.00"),
        day(),
        [
            Item::builder([ana], "Bacalhau", eur("12.00"))
                .build()
                .unwrap(),
            Item::builder_with_split(
                "Wine",
                Split::parts([
                    MemberParts::new(ana, Parts::ONE),
                    MemberParts::new(bob, Parts::parse("1.5").unwrap()),
                ]),
                eur("4.50"),
            )
            .quantity(2)
            .build()
            .unwrap(),
        ],
        "Bill",
        bob,
    )
    .description("Tasca")
    .time(TimeOfDay::new(20, 30).unwrap())
    .category(food)
    .tip(Tip::percentage(Percentage::parse("12.5").unwrap()).distributed(TipDistribution::Equal))
    .build()
    .unwrap();
    group.add_expense(bill).unwrap();
    let tram = Expense::builder_with_payers(
        Money::parse("30.00", Currency::RON)
            .unwrap()
            .at(ExchangeRate::parse("0.2011").unwrap()),
        day(),
        "Tram",
        Split::amounts([
            MemberAmount::new(Money::parse("20.00", Currency::RON).unwrap(), ana),
            MemberAmount::new(Money::parse("10.00", Currency::RON).unwrap(), carol),
        ]),
        Split::percentages([
            MemberPercentage::new(ana, Percentage::parse("50").unwrap()),
            MemberPercentage::new(carol, Percentage::parse("50").unwrap()),
        ]),
    )
    .build()
    .unwrap();
    group.add_expense(tram).unwrap();
    group
        .add_transfer(
            Transfer::builder(eur("5.00"), day(), ana, bob)
                .note("Wine")
                .build()
                .unwrap(),
        )
        .unwrap();
    group.archive_member(carol).unwrap();
    (group, ana, bob, carol)
}

fn json_of(group: &Group) -> Value {
    serde_json::to_value(group).unwrap()
}

/// Loading a saved group must give back the same group, the same totals,
/// and the same plans; anything else silently changes what people owe.
#[test]
fn a_group_round_trips_through_json() {
    let (group, _, _, carol) = group();
    let text = serde_json::to_string(&group).unwrap();
    let loaded: Group = serde_json::from_str(&text).unwrap();
    assert_eq!(loaded, group);
    assert_eq!(loaded.summary(), group.summary());
    for strategy in SettlementStrategy::all() {
        assert_eq!(
            loaded.settlement(strategy).unwrap(),
            group.settlement(strategy).unwrap()
        );
    }
    // The archived member stays archived, and keeps her history.
    assert!(loaded.is_archived(carol));
    assert_eq!(serde_json::to_string(&loaded).unwrap(), text);
}

/// The field names and value shapes are the stored format; renaming one
/// would make every saved group unreadable.
#[test]
fn the_json_shape_is_pinned() {
    let (group, ana, _, carol) = group();
    let value = json_of(&group);
    assert_eq!(value["format"], json!(2));
    assert_eq!(value["id"], json!(group.id().to_string()));
    assert_eq!(value["name"], json!("Lisbon"));
    assert_eq!(value["currency"], json!("EUR"));
    assert_eq!(value["language"], json!("pt-PT"));
    // The version leads, then the identifier; every entry starts with its
    // own identifier.
    let text = serde_json::to_string(&group).unwrap();
    let start = format!(
        r#"{{"format":2,"id":"{}","categories":[{{"id":"#,
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
    let (bill_id, _) = group.expenses().next().unwrap();
    let bill = &value["expenses"][0];
    assert_eq!(bill["id"], json!(bill_id.to_string()));
    assert_eq!(
        bill["amount"],
        json!({"currency": "EUR", "minor_units": 2100})
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
    assert_eq!(bill.get("total"), None);
    let tram = &value["expenses"][1];
    assert_eq!(tram["exchange_rate"], json!("0.2011"));
    assert_eq!(tram["split"]["whole"]["percentages"][0][1], json!(5000));
    assert_eq!(value["transfers"][0]["note"], json!("Wine"));
    // The built-in category is not written, and neither is an expense's
    // category when it is the built-in one.
    let (food, _) = group
        .categories()
        .find(|(_, category)| category.name() == "Food")
        .unwrap();
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
    let (group, _, _, _) = group();
    let tamper = |change: &dyn Fn(&mut Value)| {
        let mut value = json_of(&group);
        change(&mut value);
        serde_json::from_value::<Group>(value)
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
    // An expense paid by someone who is not a member.
    let stranger = MemberId::from_bytes([200; 16]).to_string();
    let unknown_payer = tamper(&|value| {
        value["expenses"][0]["payers"] = json!({"equal": [stranger]});
    });
    assert!(unknown_payer.contains("unknown member"), "{unknown_payer}");
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
        value["format"] = json!(3);
    });
    assert!(newer.contains("unsupported group format 3"), "{newer}");
    let older = tamper(&|value| {
        value["format"] = json!(1);
    });
    assert!(
        older.contains("unsupported group format 1: this version reads format 2"),
        "{older}"
    );
}

/// A file must also be refused when each entry is valid on its own but the
/// group as a whole is not, or loading would merge or drop entries.
#[test]
fn an_inconsistent_file_is_refused() {
    let (group, _, _, _) = group();
    let tamper = |change: &dyn Fn(&mut Value)| {
        let mut value = json_of(&group);
        change(&mut value);
        serde_json::from_value::<Group>(value)
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
    let other = CategoryId::from_bytes([201; 16]).to_string();
    let homonyms = tamper(&|value| {
        let copy = json!({"id": other, "name": "FOOD"});
        value["categories"].as_array_mut().unwrap().push(copy);
    });
    assert!(homonyms.contains("already has this name"), "{homonyms}");
    // A transfer from someone who is not a member.
    let stranger = MemberId::from_bytes([200; 16]).to_string();
    let unknown_sender = tamper(&|value| value["transfers"][0]["from"] = json!(stranger));
    assert!(
        unknown_sender.contains("unknown member"),
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
        archived_stranger.contains("unknown member"),
        "{archived_stranger}"
    );
}

/// A group with none of the optional details round-trips as well, without
/// gaining any.
#[test]
fn a_bare_group_round_trips_through_json() {
    let group = Group::builder(Currency::EUR, "Bare").build().unwrap();
    let text = serde_json::to_string(&group).unwrap();
    assert!(!text.contains("description"), "{text}");
    let loaded: Group = serde_json::from_str(&text).unwrap();
    assert_eq!(loaded, group);
    assert_eq!(loaded.description(), None);
    assert_eq!(loaded.language(), None);
}

/// Account details are stored in clear text, for the app's secure storage,
/// and print in full, like every other value.
#[test]
fn account_details_serialise_and_print_in_clear_text() {
    let (group, ana, _, _) = group();
    let text = serde_json::to_string(&group).unwrap();
    assert!(text.contains("RO49AAAA1B31007593840000"));
    let debug = format!("{:?}", group.member(ana).unwrap());
    assert!(debug.contains("RO49AAAA1B31007593840000"), "{debug}");
    assert!(debug.contains("@ana"), "{debug}");
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
    assert!(serde_json::from_value::<Currency>(json!("XYZ")).is_err());
    assert!(serde_json::from_value::<CalendarDate>(json!("2026-02-30")).is_err());
    assert_eq!(
        serde_json::from_value::<TimeOfDay>(json!("07:05")).unwrap(),
        TimeOfDay::new(7, 5).unwrap()
    );
    assert!(serde_json::from_value::<ExchangeRate>(json!("0")).is_err());
}
