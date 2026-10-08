//! Every feature of the library in one story: a group and its details,
//! members with contact details and accounts, categories, an expense for
//! every way of dividing one, another currency, edits and refusals, members
//! who come and go, the summary, a new group currency, settling up, saving
//! and loading (feature `serde`), the values on their own, and the group
//! shared between tasks and threads. Every section asserts its key numbers.
//!
//! Run with `cargo run -p eranos --features serde --example grand_tour`;
//! add the `sequential-ids` feature to print the same identifiers, and so
//! the same text, on every run.
#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::Display;
use std::pin::pin;
use std::task::{Context, Waker};

use eranos::cooperative::block_on;
use eranos::error::{ExpenseError, ExpenseGroupError, IdError, TransferError};
use eranos::expense::ExpenseBuilder;
use eranos::{
    Balance, CalendarDate, Category, CategoryId, CategoryTotal, Cost, CurrencyCode, CurrencyRate,
    Email, ExchangeRate, Expense, ExpenseGroup, ExpenseId, ExpenseItem, ExpenseShares,
    ExpenseSplit, Iban, Language, Member, MemberAmount, MemberId, MemberParts, MemberPercentage,
    MemberShare, Money, Parts, PaymentAccount, PaymentAccountDetails, PaymentAccountKind,
    Percentage, PhoneNumber, SavedExpenseGroup, SettlementStrategy, SharedExpenseGroup, Split,
    TimeOfDay, TipDistribution, TipRule, TipValue, Transfer, Username,
};
use tabled::settings::formatting::AlignmentStrategy;
use tabled::settings::object::Columns;
use tabled::settings::{Alignment, Style};
use tabled::{Table, Tabled};

/// The members of the trip.
#[derive(Clone, Copy)]
struct People {
    ana: MemberId,
    bob: MemberId,
    carol: MemberId,
    dan: MemberId,
    eva: MemberId,
    filip: MemberId,
}

/// The categories the group adds to the built-in one.
#[derive(Clone, Copy)]
struct Categories {
    food: CategoryId,
    lodging: CategoryId,
    transport: CategoryId,
}

/// The seven expenses, one for every way of dividing one.
struct Expenses {
    apartment: ExpenseId,
    fado: ExpenseId,
    groceries: ExpenseId,
    museum: ExpenseId,
    seafood: ExpenseId,
    tour: ExpenseId,
    train: ExpenseId,
}

/// A change made to a saved group, and the text it leaves.
struct Tampering {
    change: &'static str,
    text: String,
}

#[derive(Tabled)]
struct DetailRow {
    #[tabled(rename = "Detail")]
    detail: &'static str,
    #[tabled(rename = "Value")]
    value: String,
}

#[derive(Tabled)]
struct MemberRow {
    #[tabled(rename = "Member")]
    name: String,
    #[tabled(rename = "Email")]
    email: String,
    #[tabled(rename = "Phone")]
    phone: String,
    #[tabled(rename = "Accounts")]
    accounts: String,
}

#[derive(Tabled)]
struct CheckRow {
    #[tabled(rename = "Input")]
    input: String,
    #[tabled(rename = "Checked as")]
    kind: &'static str,
    #[tabled(rename = "Result")]
    result: String,
}

#[derive(Tabled)]
struct RefusalRow {
    #[tabled(rename = "Change")]
    change: &'static str,
    #[tabled(rename = "Refused with")]
    error: String,
}

#[derive(Tabled)]
struct ShareRow {
    #[tabled(rename = "Expense")]
    expense: String,
    #[tabled(rename = "Split")]
    how: String,
    #[tabled(rename = "Member")]
    members: String,
    #[tabled(rename = "Paid")]
    paid: String,
    #[tabled(rename = "Consumed")]
    consumed: String,
}

#[derive(Tabled)]
struct ItemRow {
    #[tabled(rename = "Item")]
    name: String,
    #[tabled(rename = "Shared by")]
    shared_by: String,
    #[tabled(rename = "Quantity")]
    quantity: u32,
    #[tabled(rename = "Unit price")]
    unit_price: Money,
    #[tabled(rename = "Line total")]
    line_total: Money,
}

#[derive(Tabled)]
struct EntryRow {
    #[tabled(rename = "Entry")]
    name: String,
    #[tabled(rename = "Paid by")]
    payer: String,
    #[tabled(rename = "Entered as")]
    original: Money,
    #[tabled(rename = "Rate")]
    rate: ExchangeRate,
    #[tabled(rename = "In the group currency")]
    converted: Money,
}

#[derive(Tabled)]
struct EditRow {
    #[tabled(rename = "Entry")]
    entry: String,
    #[tabled(rename = "Edit")]
    edit: String,
}

#[derive(Tabled)]
struct BalanceRow {
    #[tabled(rename = "Member")]
    name: String,
    #[tabled(rename = "Paid")]
    paid: Money,
    #[tabled(rename = "Consumed")]
    consumed: Money,
    #[tabled(rename = "Sent")]
    sent: Money,
    #[tabled(rename = "Received")]
    received: Money,
    #[tabled(rename = "Balance")]
    balance: Balance,
}

#[derive(Tabled)]
struct StatusRow {
    #[tabled(rename = "Member")]
    name: String,
    #[tabled(rename = "Status")]
    status: &'static str,
    #[tabled(rename = "Balance")]
    balance: Balance,
}

#[derive(Tabled)]
struct CategoryRow {
    #[tabled(rename = "Category")]
    name: String,
    #[tabled(rename = "Expenses")]
    expenses: usize,
    #[tabled(rename = "Total")]
    total: Money,
}

#[derive(Tabled)]
struct MemberCategoryRow {
    #[tabled(rename = "Member")]
    name: String,
    #[tabled(rename = "Food")]
    food: Money,
    #[tabled(rename = "Lodging")]
    lodging: Money,
    #[tabled(rename = "Getting around")]
    transport: Money,
    #[tabled(rename = "Uncategorised")]
    uncategorised: Money,
}

#[derive(Tabled)]
struct StrategyRow {
    #[tabled(rename = "Strategy")]
    strategy: &'static str,
    #[tabled(rename = "Computed by")]
    computed_by: &'static str,
    #[tabled(rename = "Payments")]
    payments: usize,
}

#[derive(Tabled)]
struct PaymentRow {
    #[tabled(rename = "From")]
    from: String,
    #[tabled(rename = "To")]
    to: String,
    #[tabled(rename = "Amount")]
    amount: Money,
}

#[derive(Tabled)]
struct LoadRow {
    #[tabled(rename = "Change to the file")]
    change: &'static str,
    #[tabled(rename = "Message")]
    message: ExpenseGroupError,
}

#[derive(Tabled)]
struct CurrencyRow {
    #[tabled(rename = "Code")]
    code: &'static str,
    #[tabled(rename = "Name")]
    name: &'static str,
    #[tabled(rename = "Numeric code")]
    numeric_code: u16,
    #[tabled(rename = "Decimal places")]
    decimal_places: u8,
}

#[derive(Tabled)]
struct ValueRow {
    #[tabled(rename = "Input")]
    input: &'static str,
    #[tabled(rename = "Result")]
    result: String,
}

/// Prints a section heading with a line under it.
fn section(title: &str) {
    println!("\n{title}\n{}", "─".repeat(title.chars().count()));
}

/// Prints `rows` with rounded borders, right-aligning the columns from
/// `numbers_from` on, each line of a cell on its own.
fn print_table<Row: Tabled>(rows: impl IntoIterator<Item = Row>, numbers_from: usize) {
    let mut table = Table::new(rows);
    table
        .with(Style::rounded())
        .with(AlignmentStrategy::PerLine)
        .modify(Columns::new(numbers_from..), Alignment::right());
    println!("{table}");
}

fn eur(text: &str) -> Result<Money, Box<dyn Error>> {
    Ok(Money::parse(text, CurrencyCode::EUR)?)
}

fn ron(text: &str) -> Result<Money, Box<dyn Error>> {
    Ok(Money::parse(text, CurrencyCode::RON)?)
}

/// A date of the trip, in October 2026.
fn october(date: u8) -> Result<CalendarDate, Box<dyn Error>> {
    Ok(CalendarDate::new(2026, 10, date)?)
}

fn name(group: &ExpenseGroup, member: MemberId) -> String {
    group
        .member(member)
        .map_or("?", Member::first_name)
        .to_owned()
}

/// The first names of `members`, joined by `separator`.
fn names(
    group: &ExpenseGroup,
    members: impl IntoIterator<Item = MemberId>,
    separator: &str,
) -> String {
    members
        .into_iter()
        .map(|member| name(group, member))
        .collect::<Vec<_>>()
        .join(separator)
}

/// `lines`, one below the other in a table cell.
fn cell(lines: impl IntoIterator<Item = String>) -> String {
    lines.into_iter().collect::<Vec<_>>().join("\n")
}

/// "valid: …" with `shown` of the value, or "refused: …" with the message.
fn outcome<Value, Failure: Display>(
    result: Result<Value, Failure>,
    shown: impl Fn(Value) -> String,
) -> String {
    result.map_or_else(
        |error| format!("refused: {error}"),
        |value| format!("valid: {}", shown(value)),
    )
}

/// The message of the error `result` holds, or "accepted".
fn message<Value, Failure: Display>(result: &Result<Value, Failure>) -> String {
    result
        .as_ref()
        .map_or_else(ToString::to_string, |_| "accepted".to_owned())
}

/// The sum of `amounts`, all in `currency`.
fn sum(
    currency: CurrencyCode,
    amounts: impl IntoIterator<Item = Money>,
) -> Result<Money, Box<dyn Error>> {
    amounts
        .into_iter()
        .try_fold(Money::zero(currency), |total, amount| {
            Ok(total.checked_add(amount)?)
        })
}

fn print_balances(group: &ExpenseGroup) {
    let summary = group.summary();
    print_table(
        group.members().map(|member| {
            let totals = summary.member(member.id());
            BalanceRow {
                name: member.first_name().to_owned(),
                paid: totals.paid(),
                consumed: totals.share(),
                sent: totals.sent(),
                received: totals.received(),
                balance: totals.balance(),
            }
        }),
        1,
    );
}

fn print_categories(group: &ExpenseGroup) {
    let summary = group.summary();
    print_table(
        summary.category_totals().map(|total| CategoryRow {
            name: group
                .category(total.category())
                .map_or("?", Category::name)
                .to_owned(),
            expenses: group
                .expenses()
                .filter(|expense| expense.category() == total.category())
                .count(),
            total: total.total(),
        }),
        1,
    );
}

/// Every expense and transfer in a currency `shown` accepts, as entered and
/// in the group currency.
fn print_entries(group: &ExpenseGroup, shown: impl Fn(CurrencyCode) -> bool) {
    let expenses = group
        .expenses()
        .filter(|expense| shown(expense.amount().currency()))
        .map(|expense| EntryRow {
            name: expense.name().to_owned(),
            payer: names(group, expense.payers().members(), ", "),
            original: expense.total(),
            rate: expense.exchange_rate(),
            converted: group
                .expense_shares(expense.id())
                .map_or_else(|| Money::zero(group.currency()), ExpenseShares::total),
        });
    let transfers = group
        .transfers()
        .filter(|transfer| shown(transfer.amount().currency()))
        .map(|transfer| EntryRow {
            name: format!("Transfer to {}", name(group, transfer.to())),
            payer: name(group, transfer.from()),
            original: transfer.amount(),
            rate: transfer.exchange_rate(),
            converted: group
                .transfer_amount(transfer.id())
                .unwrap_or_else(|| Money::zero(group.currency())),
        });
    print_table(expenses.chain(transfers), 2);
}

fn the_group() -> Result<ExpenseGroup, Box<dyn Error>> {
    section("1. The group");
    let mut group = ExpenseGroup::builder("Lisbon", CurrencyCode::EUR)
        .description("A week by the Atlantic")
        .language(Language::parse("pt-pt")?)
        .build()?;
    let blank = group.set_name("   ");
    group.set_name("Lisbon and Porto")?;
    group.set_description("A week by the Atlantic, with a day in Porto");
    print_table(
        [
            DetailRow {
                detail: "Name",
                value: group.name().to_owned(),
            },
            DetailRow {
                detail: "Currency",
                value: group.currency().to_string(),
            },
            DetailRow {
                detail: "Language",
                value: group
                    .language()
                    .map_or_else(String::new, ToString::to_string),
            },
            DetailRow {
                detail: "Description",
                value: group.description().unwrap_or_default().to_owned(),
            },
        ],
        2,
    );
    println!(
        "The group started as Lisbon and was renamed when a day in Porto joined the plan.\n\
         Every total is kept in its currency, {}. The language is a BCP 47 tag the apps\n\
         show the group in, stored in canonical form: pt-pt became {}. A blank name is\n\
         refused:\n  error: {}",
        group.currency(),
        group.language().map_or("", Language::as_str),
        message(&blank)
    );

    assert_eq!(blank, Err(ExpenseGroupError::EmptyName));
    assert_eq!(group.name(), "Lisbon and Porto");
    assert_eq!(group.language().map(Language::as_str), Some("pt-PT"));
    Ok(group)
}

/// An account with its label, its kind, and its details.
fn describe(account: &PaymentAccount) -> String {
    let kind = match account.kind() {
        PaymentAccountKind::Iban => "IBAN",
        PaymentAccountKind::Username => "username",
        _ => "account",
    };
    let details = match account.details() {
        PaymentAccountDetails::Iban(iban) => iban.as_str(),
        PaymentAccountDetails::Username(username) => username.as_str(),
        _ => "",
    };
    format!("{}, {kind} {details}", account.name())
}

fn print_members(group: &ExpenseGroup) {
    print_table(
        group.members().map(|member| MemberRow {
            name: [Some(member.first_name()), member.last_name()]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" "),
            email: member.email().map_or("", Email::as_str).to_owned(),
            phone: member
                .phone()
                .map_or_else(String::new, PhoneNumber::international),
            accounts: cell(member.accounts().iter().map(describe)),
        }),
        4,
    );
}

fn validation() {
    println!(
        "IBANs are checked against the SWIFT registry and their mod-97 check digits,\n\
         emails against RFC 5321 and 5322, and phone numbers against their country's\n\
         numbering plan. Required text must not be blank:"
    );
    let iban = |input: &'static str| CheckRow {
        input: format!("\"{input}\""),
        kind: "IBAN",
        result: outcome(Iban::parse(input), |iban| {
            format!("{}, country {}", iban.as_str(), iban.country_code())
        }),
    };
    let email = |input: &'static str| CheckRow {
        input: format!("\"{input}\""),
        kind: "email",
        result: outcome(Email::parse(input), |email| email.as_str().to_owned()),
    };
    let phone = |input: &'static str| CheckRow {
        input: format!("\"{input}\""),
        kind: "phone",
        result: outcome(PhoneNumber::parse(input), |phone| {
            format!("{}, stored as {}", phone.international(), phone.as_str())
        }),
    };
    print_table(
        [
            iban("ro49 aaaa 1b31 0075 9384 0000"),
            iban("RO49 AAAA 1B31 0075 9384 0001"),
            email("ana..popescu@example.com"),
            phone("+351 912 345 678"),
            phone("+40 612 345 678"),
            CheckRow {
                input: "\"   \"".to_owned(),
                kind: "username",
                result: outcome(Username::new("   "), |username| {
                    username.as_str().to_owned()
                }),
            },
            CheckRow {
                input: "\"  \"".to_owned(),
                kind: "first name",
                result: outcome(Member::builder("  ").build(), |member| {
                    member.first_name().to_owned()
                }),
            },
        ],
        3,
    );
}

fn members(group: &mut ExpenseGroup) -> Result<People, Box<dyn Error>> {
    section("2. Members");
    let ana = group.add_member(
        Member::builder("Ana")
            .last_name("Popescu")
            .email(Email::parse("ana@example.com")?)
            .phone(PhoneNumber::parse("+40 712 345 678")?)
            .account(PaymentAccount::new(
                "ING",
                Iban::parse("RO49 AAAA 1B31 0075 9384 0000")?,
            )?)
            .account(PaymentAccount::new("Revolut", Username::new("@ana")?)?)
            .build()?,
    )?;
    let mut add = |first_name| -> Result<MemberId, Box<dyn Error>> {
        Ok(group.add_member(Member::builder(first_name).build()?)?)
    };
    let (bob, carol, dan) = (add("Bob")?, add("Carol")?, add("Dan")?);
    let (eva, filip) = (add("Eva")?, add("Filip")?);
    // An edit starts from a copy of the member, which keeps its identifier.
    let edited = group
        .member(bob)
        .ok_or("Bob is a member")?
        .to_builder()
        .last_name("Ionescu")
        .email(Email::parse("bob@example.com")?)
        .build()?;
    let before = group.replace_member(edited)?;
    print_members(group);
    println!(
        "Only a first name is required. Ana gave every detail; Bob added his last name and\n\
         email later, through an edit that kept his identifier:\n  {bob}\n\
         Eva is Carol's daughter, and Filip was added by mistake."
    );
    validation();

    assert_eq!(before.last_name(), None);
    assert_eq!(
        group.member(bob).and_then(Member::last_name),
        Some("Ionescu")
    );
    assert_eq!(
        group.member(ana).map(|member| member.accounts().len()),
        Some(2)
    );
    Ok(People {
        ana,
        bob,
        carol,
        dan,
        eva,
        filip,
    })
}

fn categories(group: &mut ExpenseGroup) -> Result<Categories, Box<dyn Error>> {
    section("3. Categories");
    let food = group.add_category(Category::new("Food")?)?;
    // An app that keeps its categories elsewhere gives them its own keys.
    let lodging = group.add_category(Category::with_id(CategoryId::new("lodging")?, "Lodging")?)?;
    let transport = group.add_category(Category::new("Transport")?)?;
    let homonym = group.add_category(Category::new("FOOD")?);
    let renamed = group
        .category(transport)
        .ok_or("Transport was added")?
        .renamed("Getting around")?;
    group.replace_category(renamed)?;
    let other = group
        .category(CategoryId::UNCATEGORISED)
        .ok_or("every group has the built-in category")?
        .renamed("Other")?;
    let built_in = group.replace_category(other);
    println!(
        "Every group has the built-in Uncategorised; this one adds Food, Lodging, under the\n\
         identifier lodging that the app gives it, and Transport, then renames Transport, which\n\
         keeps its identifier. In identifier order, where lodging follows the generated ones:\n  \
         {}\n\
         Names are compared caselessly, and the built-in category keeps its name:",
        group
            .categories()
            .map(Category::name)
            .collect::<Vec<_>>()
            .join(", ")
    );
    print_table(
        [
            RefusalRow {
                change: "Add a category named FOOD",
                error: message(&homonym),
            },
            RefusalRow {
                change: "Rename Uncategorised to Other",
                error: message(&built_in),
            },
        ],
        2,
    );

    assert_eq!(
        homonym,
        Err(ExpenseGroupError::DuplicateCategoryName { existing: food })
    );
    assert_eq!(built_in, Err(ExpenseGroupError::BuiltInCategory));
    assert_eq!(
        group.category(transport).map(Category::name),
        Some("Getting around")
    );
    assert_eq!(lodging.as_str(), "lodging");
    assert_eq!(
        group.categories().map(Category::id).collect::<Vec<_>>(),
        [CategoryId::UNCATEGORISED, food, transport, lodging]
    );
    Ok(Categories {
        food,
        lodging,
        transport,
    })
}

/// How an expense divides, in a few words.
fn how(expense: &Expense) -> String {
    let split = match expense.split() {
        ExpenseSplit::Whole(split) => match split {
            Split::Equal(_) => "equally",
            Split::Parts(_) => "by parts",
            Split::Percentages(_) => "by percentage",
            Split::Amounts(_) => "by amounts",
            _ => "another way",
        }
        .to_owned(),
        ExpenseSplit::Itemised(items) => format!("{} items", items.len()),
        _ => "another way".to_owned(),
    };
    let payers = match expense.payers().len() {
        1 => String::new(),
        count => format!(", {count} payers"),
    };
    let tip = if expense.tip_amount().is_zero() {
        String::new()
    } else {
        match expense.tip().value() {
            TipValue::Percentage(rate) => format!(", {rate} tip"),
            TipValue::Fixed(amount) => format!(", {amount} tip"),
            _ => ", a tip".to_owned(),
        }
    };
    format!("{split}{payers}{tip}")
}

/// Each expense of `expenses`, with every involved member's payment and
/// consumption.
fn print_shares(group: &ExpenseGroup, expenses: &[ExpenseId]) {
    let rows = expenses.iter().filter_map(|&id| {
        let expense = group.expense(id)?;
        let shares = group.expense_shares(id)?;
        let paid = |share: &MemberShare| {
            if share.paid().is_zero() {
                String::new()
            } else {
                share.paid().to_string()
            }
        };
        Some(ShareRow {
            expense: expense.name().to_owned(),
            how: how(expense),
            members: cell(
                shares
                    .members()
                    .iter()
                    .map(|share| name(group, share.member())),
            ),
            paid: cell(shares.members().iter().map(paid)),
            consumed: cell(
                shares
                    .members()
                    .iter()
                    .map(|share| share.share().to_string()),
            ),
        })
    });
    print_table(rows, 3);
}

fn print_items(group: &ExpenseGroup, id: ExpenseId) {
    let Some(ExpenseSplit::Itemised(items)) = group.expense(id).map(Expense::split) else {
        return;
    };
    print_table(
        items.iter().map(|item| ItemRow {
            name: item.name().to_owned(),
            shared_by: names(group, item.split().members(), ", "),
            quantity: item.quantity(),
            unit_price: item.unit_price(),
            line_total: item.line_total(),
        }),
        2,
    );
}

/// The receipt of the fado night: every line with its own split.
fn fado_items(people: &People) -> Result<Vec<ExpenseItem>, Box<dyn Error>> {
    let People {
        ana,
        bob,
        carol,
        dan,
        eva,
        ..
    } = *people;
    Ok(vec![
        ExpenseItem::builder("Vinho verde", eur("12.00")?, [ana, bob, dan])
            .quantity(2)
            .build()?,
        ExpenseItem::builder("Bacalhau", eur("18.00")?, [carol]).build()?,
        ExpenseItem::builder_with_split(
            "Custard tarts",
            eur("1.50")?,
            Split::parts([
                MemberParts::new(ana, Parts::ONE),
                MemberParts::new(bob, Parts::ONE),
                MemberParts::new(carol, Parts::ONE),
                MemberParts::new(dan, Parts::ONE),
                MemberParts::new(eva, Parts::from_whole(2)?),
            ]),
        )
        .quantity(6)
        .build()?,
        ExpenseItem::builder("Juice", eur("3.00")?, [eva]).build()?,
    ])
}

/// The fado night, item by item, paid by Carol and Dan, with a 10 % tip,
/// under the identifier its constructor generates; the caller builds it.
fn fado_night(people: &People, categories: Categories) -> Result<ExpenseBuilder, Box<dyn Error>> {
    Ok(Expense::itemised_builder_with_payers(
        "Fado night",
        eur("54.00")?,
        Split::amounts([
            MemberAmount::new(people.carol, eur("39.40")?),
            MemberAmount::new(people.dan, eur("20.00")?),
        ]),
        fado_items(people)?,
        october(4)?,
    )
    .category(categories.food)
    .description("Tasca do Chico")
    .time(TimeOfDay::new(22, 15)?)
    .tip(TipRule::percentage(Percentage::parse("10")?)))
}

/// Adds one expense for every way of dividing one.
fn add_expenses(
    group: &mut ExpenseGroup,
    people: &People,
    categories: Categories,
) -> Result<Expenses, Box<dyn Error>> {
    let People {
        ana,
        bob,
        carol,
        dan,
        eva,
        ..
    } = *people;
    let everyone = [ana, bob, carol, dan, eva];
    let groceries = Expense::builder(
        "Groceries",
        eur("10.00")?,
        ana,
        Split::equal([ana, bob, carol]),
        october(1)?,
    )
    .category(categories.food);
    let apartment = Expense::builder(
        "Apartment",
        eur("800.00")?,
        ana,
        Split::percentages([
            MemberPercentage::new(ana, Percentage::parse("25")?),
            MemberPercentage::new(bob, Percentage::parse("25")?),
            MemberPercentage::new(carol, Percentage::parse("35")?),
            MemberPercentage::new(dan, Percentage::parse("15")?),
        ]),
        october(1)?,
    )
    .category(categories.lodging);
    let seafood = Expense::builder(
        "Seafood dinner",
        eur("90.00")?,
        carol,
        Split::parts([
            MemberParts::new(ana, Parts::ONE),
            MemberParts::new(bob, Parts::ONE),
            MemberParts::new(carol, Parts::ONE),
            MemberParts::new(dan, Parts::ONE),
            MemberParts::new(eva, Parts::parse("0.5")?),
        ]),
        october(2)?,
    )
    .category(categories.food);
    let train = Expense::builder(
        "Train to Porto",
        eur("88.50")?,
        bob,
        Split::amounts([
            MemberAmount::new(ana, eur("24.50")?),
            MemberAmount::new(bob, eur("24.50")?),
            MemberAmount::new(carol, eur("24.50")?),
            MemberAmount::new(eva, eur("15.00")?),
        ]),
        october(3)?,
    )
    .category(categories.transport);
    let museum = Expense::builder_with_payers(
        "Museum",
        eur("60.00")?,
        Split::equal([ana, bob]),
        Split::equal([ana, bob, carol, dan]),
        october(3)?,
    );
    let fado = fado_night(people, categories)?;
    let tour = Expense::builder(
        "Tuk-tuk tour",
        eur("45.00")?,
        dan,
        Split::equal(everyone),
        october(5)?,
    )
    .category(categories.transport)
    .tip(TipRule::fixed(eur("5.00")?).distributed(TipDistribution::Equal));
    let groceries = group.add_expense(groceries.build()?)?;
    let apartment = group.add_expense(apartment.build()?)?;
    let seafood = group.add_expense(seafood.build()?)?;
    let train = group.add_expense(train.build()?)?;
    let museum = group.add_expense(museum.build()?)?;
    let fado = group.add_expense(fado.build()?)?;
    let tour = group.add_expense(tour.build()?)?;
    Ok(Expenses {
        apartment,
        fado,
        groceries,
        museum,
        seafood,
        tour,
        train,
    })
}

/// The consumption of `member` in the expense `id`.
fn consumed(group: &ExpenseGroup, id: ExpenseId, member: MemberId) -> Option<Money> {
    group
        .expense_shares(id)?
        .member(member)
        .map(MemberShare::share)
}

fn expenses(
    group: &mut ExpenseGroup,
    people: &People,
    categories: Categories,
) -> Result<ExpenseId, Box<dyn Error>> {
    section("4. Expenses, one for every way of dividing");
    let Expenses {
        apartment,
        fado,
        groceries,
        museum,
        seafood,
        tour,
        train,
    } = add_expenses(group, people, categories)?;
    println!(
        "Seven expenses cover every way of dividing one: equally, by percentage, by parts\n\
         (Eva counts as half), by exact amounts, with two payers, item by item, and with a\n\
         tip. Every member's consumption adds up to the expense's total, to the cent:"
    );
    print_shares(
        group,
        &[groceries, apartment, seafood, train, museum, fado, tour],
    );
    let fado_expense = group.expense(fado).ok_or("the fado night was added")?;
    let tip = fado_expense.tip();
    let rule = match tip.value() {
        TipValue::Percentage(rate) => rate.to_string(),
        TipValue::Fixed(amount) => amount.to_string(),
        _ => "a tip".to_owned(),
    };
    let shared = match tip.distribution() {
        TipDistribution::Proportional => "in proportion to what each member ordered",
        TipDistribution::Equal => "equally",
        _ => "another way",
    };
    println!(
        "The groceries' 10.00 EUR did not divide into thirds, so one member consumed the\n\
         leftover cent; the expense's identifier picks which, so it moves from expense to\n\
         expense. The fado night, at {} on {} at {}, was itemised; its\n\
         {rule} tip of {} was shared {shared}:",
        fado_expense
            .time()
            .map_or_else(String::new, |time| time.to_string()),
        fado_expense.date(),
        fado_expense.description().unwrap_or_default(),
        fado_expense.tip_amount()
    );
    print_items(group, fado);
    println!("The tuk-tuk's fixed 5.00 EUR tip was shared equally instead.");

    let mut thirds: Vec<i64> = [people.ana, people.bob, people.carol]
        .into_iter()
        .filter_map(|member| consumed(group, groceries, member))
        .map(Money::minor_units)
        .collect();
    thirds.sort_unstable();
    assert_eq!(thirds, [333, 333, 334]);
    assert_eq!(
        consumed(group, apartment, people.carol),
        Some(eur("280.00")?)
    );
    assert_eq!(consumed(group, seafood, people.eva), Some(eur("10.00")?));
    assert_eq!(consumed(group, train, people.eva), Some(eur("15.00")?));
    let museum_shares = group.expense_shares(museum).ok_or("the museum was added")?;
    assert_eq!(
        museum_shares.member(people.bob).map(MemberShare::paid),
        Some(eur("30.00")?)
    );
    assert_eq!(consumed(group, museum, people.dan), Some(eur("15.00")?));
    assert_eq!(
        group.expense(museum).map(|expense| expense.members().len()),
        Some(4)
    );
    assert_eq!(fado_expense.total(), eur("59.40")?);
    assert_eq!(fado_expense.tip_amount(), eur("5.40")?);
    assert_eq!(consumed(group, fado, people.carol), Some(eur("21.45")?));
    assert_eq!(consumed(group, fado, people.eva), Some(eur("6.60")?));
    assert_eq!(consumed(group, tour, people.eva), Some(eur("10.00")?));
    Ok(seafood)
}

fn another_currency(
    group: &mut ExpenseGroup,
    people: &People,
    categories: Categories,
) -> Result<(), Box<dyn Error>> {
    section("5. Another currency");
    let People { ana, bob, .. } = *people;
    let rate = ExchangeRate::parse("0.2")?;
    let parking = group.add_expense(
        Expense::builder(
            "Airport parking",
            ron("250.00")?.at(rate),
            ana,
            Split::equal([ana, bob]),
            october(6)?,
        )
        .category(categories.transport)
        .build()?,
    )?;
    let cash = group.add_transfer(
        Transfer::builder(bob, ana, ron("100.00")?.at(rate), october(6)?)
            .note("Cash for the parking")
            .time(TimeOfDay::new(7, 40)?)
            .build()?,
    )?;
    println!(
        "Back at the Bucharest airport, Ana paid the parking in RON and Bob paid her back in\n\
         cash. An entry in another currency carries the rate into the group currency, how\n\
         many EUR one RON is worth, and is converted once, when it is entered:"
    );
    print_entries(group, |currency| CurrencyCode::EUR != currency);
    let transfer = group.transfer(cash).ok_or("the cash was recorded")?;
    let converted = ExchangeRate::parse("0.2011")?.convert(ron("250.00")?, CurrencyCode::EUR)?;
    println!(
        "Bob's transfer, at {} (\"{}\"), counts as {}.\n\
         A conversion rounds half away from zero: 250.00 RON at 0.2011 is 50.275 EUR,\n\
         which becomes {converted}.",
        transfer
            .time()
            .map_or_else(String::new, |time| time.to_string()),
        transfer.note().unwrap_or_default(),
        group
            .transfer_amount(cash)
            .unwrap_or_else(|| Money::zero(CurrencyCode::EUR)),
    );

    assert_eq!(converted, eur("50.28")?);
    assert_eq!(
        group.expense_shares(parking).map(ExpenseShares::total),
        Some(eur("50.00")?)
    );
    assert_eq!(group.transfer_amount(cash), Some(eur("20.00")?));
    assert!(matches!(transfer.cost(), Cost::Converted { .. }));
    Ok(())
}

/// Replaces an expense and a transfer, and removes one of each; returns
/// what changed.
fn replace_and_remove(
    group: &mut ExpenseGroup,
    people: &People,
    categories: Categories,
    seafood: ExpenseId,
) -> Result<Vec<EditRow>, Box<dyn Error>> {
    let People {
        ana,
        bob,
        carol,
        dan,
        ..
    } = *people;
    // The dinner's receipt said 99.00.
    let corrected = group
        .expense(seafood)
        .ok_or("the dinner was added")?
        .to_builder()
        .cost(eur("99.00")?)
        .description("Cervejaria Ramiro")
        .build()?;
    let previous = group.replace_expense(corrected)?;
    // Bob entered the coffee twice.
    let coffee = || -> Result<Expense, Box<dyn Error>> {
        Ok(Expense::builder(
            "Coffee",
            eur("6.00")?,
            bob,
            Split::equal([ana, bob, carol]),
            october(5)?,
        )
        .category(categories.food)
        .build()?)
    };
    group.add_expense(coffee()?)?;
    let copy = group.add_expense(coffee()?)?;
    let removed = group.remove_expense(copy)?;
    // Dan's part of the apartment was 120.00, not 100.00.
    let first =
        group.add_transfer(Transfer::builder(dan, ana, eur("100.00")?, october(5)?).build()?)?;
    let fixed = group
        .transfer(first)
        .ok_or("the transfer was added")?
        .to_builder()
        .cost(eur("120.00")?)
        .note("Apartment")
        .build()?;
    let replaced = group.replace_transfer(fixed)?;
    // Carol sent 10.00 to the wrong member.
    let mistake =
        group.add_transfer(Transfer::builder(carol, bob, eur("10.00")?, october(5)?).build()?)?;
    let undone = group.remove_transfer(mistake)?;

    assert_eq!(previous.amount(), eur("90.00")?);
    assert!(group.expense(copy).is_none());
    assert_eq!(group.transfer_amount(first), Some(eur("120.00")?));
    assert!(group.transfer(mistake).is_none());
    Ok(vec![
        EditRow {
            entry: previous.name().to_owned(),
            edit: format!(
                "{} became {}, at {}",
                previous.amount(),
                group
                    .expense(seafood)
                    .map_or_else(|| Money::zero(CurrencyCode::EUR), Expense::amount),
                group
                    .expense(seafood)
                    .and_then(Expense::description)
                    .unwrap_or_default()
            ),
        },
        EditRow {
            entry: removed.name().to_owned(),
            edit: format!(
                "entered twice; the copy of {} was removed",
                removed.amount()
            ),
        },
        EditRow {
            entry: format!("{} to {}", name(group, dan), name(group, ana)),
            edit: format!(
                "{} became {}, noted {}",
                replaced.amount(),
                group
                    .transfer(first)
                    .map_or_else(|| Money::zero(CurrencyCode::EUR), Transfer::amount),
                group
                    .transfer(first)
                    .and_then(Transfer::note)
                    .unwrap_or_default()
            ),
        },
        EditRow {
            entry: format!("{} to {}", name(group, carol), name(group, bob)),
            edit: format!("{} sent to the wrong member; removed", undone.amount()),
        },
    ])
}

fn edits(
    group: &mut ExpenseGroup,
    people: &People,
    categories: Categories,
    seafood: ExpenseId,
) -> Result<(), Box<dyn Error>> {
    section("6. Edits");
    let People { ana, bob, .. } = *people;
    let changes = replace_and_remove(group, people, categories, seafood)?;
    println!(
        "Entries are values: an edit starts from a copy made with to_builder, keeps the\n\
         identifier, and replaces the entry; a removal takes it out with its totals."
    );
    print_table(changes, 2);
    println!("The balances after the edits:");
    print_balances(group);

    let before = group.summary().clone();
    let unbalanced = Expense::builder(
        "Taxi",
        eur("30.00")?,
        ana,
        Split::percentages([
            MemberPercentage::new(ana, Percentage::parse("50")?),
            MemberPercentage::new(bob, Percentage::parse("49.99")?),
        ]),
        october(6)?,
    )
    .build();
    let to_herself = Transfer::builder(ana, ana, eur("5.00")?, october(6)?).build();
    let unrated = group.add_expense(
        Expense::builder(
            "Souvenirs",
            ron("40.00")?,
            ana,
            Split::equal([ana]),
            october(6)?,
        )
        .build()?,
    );
    println!("A change that does not fit is refused, and the group stays as it was:");
    print_table(
        [
            RefusalRow {
                change: "A taxi split 50% and 49.99%",
                error: message(&unbalanced),
            },
            RefusalRow {
                change: "Ana paying herself back",
                error: message(&to_herself),
            },
            RefusalRow {
                change: "Souvenirs in RON, without a rate",
                error: message(&unrated),
            },
        ],
        2,
    );
    println!("The balances again, unchanged:");
    print_balances(group);

    assert_eq!(
        unbalanced,
        Err(ExpenseError::PercentagesDoNotAddUp {
            total_basis_points: 9_999
        })
    );
    assert_eq!(to_herself, Err(TransferError::SameMember));
    assert_eq!(
        unrated,
        Err(ExpenseGroupError::MissingExchangeRate {
            currency: CurrencyCode::RON
        })
    );
    assert_eq!(group.summary(), &before);
    Ok(())
}

fn comings_and_goings(group: &mut ExpenseGroup, people: &People) -> Result<(), Box<dyn Error>> {
    section("7. Members come and go");
    let People {
        ana,
        bob,
        carol,
        dan,
        filip,
        ..
    } = *people;
    let removed = group.remove_member(filip)?;
    group.archive_member(dan)?;
    let farewell = group.add_expense(
        Expense::builder(
            "Farewell dinner",
            eur("40.00")?,
            ana,
            Split::equal([ana, bob, carol, dan]),
            october(6)?,
        )
        .build()?,
    );
    let in_use = group.remove_member(dan);
    println!(
        "{} was never part of an entry, so removing him leaves no trace. Dan flew home\n\
         early and was archived: his history and his balance stay, but a new expense\n\
         cannot include him, and he cannot be removed while entries reference him.",
        removed.first_name()
    );
    let summary = group.summary();
    print_table(
        group.members().map(|member| StatusRow {
            name: member.first_name().to_owned(),
            status: match group.is_archived(member.id()) {
                Some(true) => "archived",
                Some(false) => "active",
                None => "not in the group",
            },
            balance: summary.member(member.id()).balance(),
        }),
        2,
    );
    print_table(
        [
            RefusalRow {
                change: "A farewell dinner with Dan",
                error: message(&farewell),
            },
            RefusalRow {
                change: "Removing Dan",
                error: message(&in_use),
            },
        ],
        2,
    );
    group.unarchive_member(dan)?;
    println!("When Dan joins the next trip, unarchiving him makes him active again.");

    assert_eq!(removed.first_name(), "Filip");
    assert!(group.member(filip).is_none());
    assert_eq!(farewell, Err(ExpenseGroupError::ArchivedMember(dan)));
    assert!(matches!(in_use, Err(ExpenseGroupError::MemberInUse { .. })));
    assert_eq!(group.is_archived(dan), Some(false));
    Ok(())
}

fn summary(
    group: &ExpenseGroup,
    people: &People,
    categories: Categories,
) -> Result<(), Box<dyn Error>> {
    section("8. The summary");
    let summary = group.summary();
    println!(
        "The group spent {} in all. For each member, what they paid, what they\n\
         consumed, the transfers they sent and received, and what is left:",
        summary.total_spent()
    );
    print_balances(group);
    println!("The spending per category:");
    print_categories(group);
    println!("And each member's consumption per category:");
    print_table(
        group.members().map(|member| {
            let total = |category| summary.member_category_total(member.id(), category);
            MemberCategoryRow {
                name: member.first_name().to_owned(),
                food: total(categories.food),
                lodging: total(categories.lodging),
                transport: total(categories.transport),
                uncategorised: total(CategoryId::UNCATEGORISED),
            }
        }),
        1,
    );

    assert_eq!(summary.total_spent(), eur("1222.90")?);
    assert_eq!(summary.member(people.ana).paid(), eur("890.00")?);
    assert_eq!(summary.member(people.ana).received(), eur("140.00")?);
    assert_eq!(summary.member(people.bob).sent(), eur("20.00")?);
    assert_eq!(
        summary.member(people.dan).balance(),
        Balance::GetsBack(eur("12.55")?)
    );
    assert_eq!(
        summary.member(people.eva).balance(),
        Balance::Owes(eur("42.60")?)
    );
    assert_eq!(summary.category_total(categories.food), eur("174.40")?);
    assert_eq!(summary.category_total(categories.lodging), eur("800.00")?);
    assert_eq!(summary.category_total(categories.transport), eur("188.50")?);
    assert_eq!(
        summary.category_total(CategoryId::UNCATEGORISED),
        eur("60.00")?
    );
    for member in summary.members() {
        let by_category = sum(
            CurrencyCode::EUR,
            summary
                .member_category_totals(member.member())
                .map(CategoryTotal::total),
        )?;
        assert_eq!(by_category, member.share());
    }
    assert!(
        sum(
            CurrencyCode::EUR,
            summary.members().map(|member| member.net())
        )?
        .is_zero()
    );
    Ok(())
}

fn removing_a_category(
    group: &mut ExpenseGroup,
    categories: Categories,
) -> Result<(), Box<dyn Error>> {
    section("9. Removing a category");
    let removed = group.remove_category(categories.transport)?;
    let built_in = group.remove_category(CategoryId::UNCATEGORISED);
    println!(
        "Removing {} moves its expenses to Uncategorised and the category\n\
         totals with them; no balance changes. The built-in category cannot be removed:\n  \
         error: {}",
        removed.name(),
        message(&built_in)
    );
    print_categories(group);

    assert_eq!(built_in, Err(ExpenseGroupError::BuiltInCategory));
    assert!(group.category(categories.transport).is_none());
    assert_eq!(
        group.summary().category_total(CategoryId::UNCATEGORISED),
        eur("248.50")?
    );
    assert!(
        group
            .expenses()
            .all(|expense| expense.category() != categories.transport)
    );
    Ok(())
}

fn new_currency(group: &mut ExpenseGroup, people: &People) -> Result<(), Box<dyn Error>> {
    section("10. A new currency");
    let missing = group.change_currency(CurrencyCode::RON, &[]);
    group.change_currency(
        CurrencyCode::RON,
        &[CurrencyRate::new(
            CurrencyCode::EUR,
            ExchangeRate::parse("5")?,
        )],
    )?;
    println!(
        "Home again, the group moves its books to RON. The change needs a rate into RON\n\
         for every other currency in use, here only EUR, or it is refused:\n  error: {}\n\
         Entries in EUR take the new rate, and the RON parking and cash become local:",
        message(&missing)
    );
    print_entries(group, |_| true);
    println!(
        "Each entry keeps its currency and converts at the new rate. At 5 every share\n\
         converts exactly, so each balance is five times what it was. The trip cost\n\
         {}:",
        group.summary().total_spent()
    );
    print_balances(group);

    assert_eq!(
        missing,
        Err(ExpenseGroupError::MissingExchangeRate {
            currency: CurrencyCode::EUR
        })
    );
    assert_eq!(group.currency(), CurrencyCode::RON);
    assert_eq!(group.summary().total_spent(), ron("6114.50")?);
    assert_eq!(
        group.summary().member(people.dan).balance(),
        Balance::GetsBack(ron("62.75")?)
    );
    assert_eq!(
        group.summary().member(people.eva).balance(),
        Balance::Owes(ron("213.00")?)
    );
    Ok(())
}

/// The name of `strategy` as the apps show it.
const fn strategy_name(strategy: SettlementStrategy) -> &'static str {
    match strategy {
        SettlementStrategy::Adaptive => "Adaptive",
        SettlementStrategy::MinimumTransfers => "MinimumTransfers",
        SettlementStrategy::Greedy => "Greedy",
        SettlementStrategy::Direct => "Direct",
        _ => "another strategy",
    }
}

fn settling_up(group: &mut ExpenseGroup) -> Result<(), Box<dyn Error>> {
    section("11. Settling up");
    let mut counts = Vec::new();
    for strategy in SettlementStrategy::all() {
        let settlement = group.settlement(strategy)?;
        // The group's open balances are well within the adaptive strategy's
        // limit, so it settles them exactly.
        let computed_by = if SettlementStrategy::Adaptive == strategy {
            SettlementStrategy::MinimumTransfers
        } else {
            strategy
        };
        assert_eq!(settlement.strategy(), computed_by);
        counts.push(StrategyRow {
            strategy: strategy_name(strategy),
            computed_by: strategy_name(settlement.strategy()),
            payments: settlement.len(),
        });
    }
    println!(
        "Each strategy turns the balances into payments: the direct one pays back every\n\
         pair's own debts, greedy pays the largest amounts first, the minimum one finds\n\
         the fewest payments, and the adaptive one finds the fewest while that is quick\n\
         and greedy's beyond, naming the strategy whose plan it chose."
    );
    print_table(counts, 2);
    let plan = group
        .settlement(SettlementStrategy::MinimumTransfers)?
        .clone();
    println!("The group follows the minimum plan:");
    print_table(
        plan.payments().iter().map(|payment| PaymentRow {
            from: name(group, payment.from()),
            to: name(group, payment.to()),
            amount: payment.amount(),
        }),
        2,
    );
    for payment in plan.payments() {
        group.add_transfer(Transfer::builder_from_payment(payment, october(7)?).build()?)?;
    }
    println!("Each payment, once made, is recorded as a transfer, and every balance is settled:");
    print_balances(group);

    let greedy = group.settlement(SettlementStrategy::Greedy)?;
    assert!(greedy.is_empty());
    assert_eq!(plan.len(), 4);
    for member in group.members() {
        assert_eq!(
            group.summary().member(member.id()).balance(),
            Balance::Settled
        );
    }
    Ok(())
}

/// What loading the saved form `text` refuses: the error of `into_expense_group`,
/// which replays every entry through the checks of adding it.
fn refusal(text: &str) -> Result<ExpenseGroupError, Box<dyn Error>> {
    let saved: SavedExpenseGroup = serde_json::from_str(text)?;
    Ok(saved
        .into_expense_group()
        .err()
        .ok_or("the change must be refused")?)
}

fn saving_and_loading(
    group: &ExpenseGroup,
    people: &People,
    categories: Categories,
) -> Result<(), Box<dyn Error>> {
    section("12. Saving and loading");
    let saved = serde_json::to_string_pretty(group)?;
    println!(
        "The group saves to {} bytes of JSON: its details and entries, never the totals.\n\
         Ana's accounts are saved in clear text, as the library holds them:",
        saved.len()
    );
    let ana = group.member(people.ana).ok_or("Ana is a member")?;
    for account in ana.accounts() {
        let details = match account.details() {
            PaymentAccountDetails::Iban(iban) => iban.as_str(),
            PaymentAccountDetails::Username(username) => username.as_str(),
            _ => "",
        };
        assert!(saved.contains(details), "{details} is saved as it is");
        println!("  {}: {details}", account.name());
    }
    let loaded: ExpenseGroup = serde_json::from_str(&saved)?;
    println!(
        "Loading replays every entry through the same checks as adding it and rebuilds the\n\
         totals: the loaded group equals the saved one. Reading a SavedExpenseGroup and\n\
         loading it with into_expense_group reports a refusal as a typed error, not as text:"
    );
    let tampered = [
        Tampering {
            change: "\"format\": 4",
            text: saved.replacen("\"format\": 3", "\"format\": 4", 1),
        },
        Tampering {
            change: "a blank group name",
            text: saved.replacen("\"name\": \"Lisbon and Porto\"", "\"name\": \" \"", 1),
        },
        Tampering {
            change: "Lodging renamed FOOD",
            text: saved.replacen("\"name\": \"Lodging\"", "\"name\": \"FOOD\"", 1),
        },
    ];
    let mut rows = Vec::new();
    let mut errors = Vec::new();
    for tampering in &tampered {
        assert_ne!(
            tampering.text, saved,
            "{} changes the file",
            tampering.change
        );
        let error = refusal(&tampering.text)?;
        rows.push(LoadRow {
            change: tampering.change,
            message: error.clone(),
        });
        errors.push(error);
    }
    print_table(rows, 2);

    assert_eq!(&loaded, group);
    assert_eq!(loaded.summary(), group.summary());
    assert_eq!(
        errors,
        [
            ExpenseGroupError::UnsupportedFormat { format: 4 },
            ExpenseGroupError::EmptyName,
            ExpenseGroupError::DuplicateCategoryName {
                existing: categories.food
            },
        ]
    );
    Ok(())
}

fn currencies() -> Result<(), Box<dyn Error>> {
    let mut rows = Vec::new();
    for code in ["EUR", "RON", "JPY", "BHD"] {
        let currency = CurrencyCode::from_code(code)?;
        rows.push(CurrencyRow {
            code: currency.code(),
            name: currency.name(),
            numeric_code: currency.numeric_code(),
            decimal_places: currency.decimal_places(),
        });
    }
    println!(
        "Currencies are the {} active ISO 4217 codes with a minor unit; an unknown code is\n\
         refused ({}):",
        CurrencyCode::all().len(),
        message(&CurrencyCode::from_code("XYZ"))
    );
    print_table(rows, 2);
    assert_eq!(CurrencyCode::from_code("BHD")?.decimal_places(), 3);
    Ok(())
}

fn amounts() -> Result<(), Box<dyn Error>> {
    let total = eur("12.34")?.checked_add(eur("0.66")?)?;
    let change = eur("5.00")?.checked_sub(eur("7.50")?)?;
    let mixed = eur("1.00")?.checked_add(ron("1.00")?);
    let yen = Money::parse("1500", CurrencyCode::JPY)?;
    let dinars = Money::from_minor_units(1_234, CurrencyCode::BHD);
    let nothing = Money::zero(CurrencyCode::RON);
    let cost = eur("10.00")?.at(ExchangeRate::parse("4.9765")?);
    println!("Amounts are exact minor units in their currency:");
    print_table(
        [
            ValueRow {
                input: "12.34 EUR + 0.66 EUR",
                result: total.to_string(),
            },
            ValueRow {
                input: "5.00 EUR − 7.50 EUR",
                result: format!("{change}, negative: {}", change.is_negative()),
            },
            ValueRow {
                input: "1.00 EUR + 1.00 RON",
                result: message(&mixed),
            },
            ValueRow {
                input: "1.005 EUR",
                result: message(&Money::parse("1.005", CurrencyCode::EUR)),
            },
            ValueRow {
                input: "1500 JPY",
                result: format!("{yen}, {} minor units", yen.minor_units()),
            },
            ValueRow {
                input: "1234 minor units of BHD",
                result: dinars.to_string(),
            },
            ValueRow {
                input: "zero RON",
                result: format!("{nothing}, zero: {}", nothing.is_zero()),
            },
            ValueRow {
                input: "10.00 EUR at 4.9765",
                result: format!(
                    "{} at {}, then {} at the same rate",
                    cost.amount(),
                    cost.exchange_rate(),
                    cost.with_amount(eur("20.00")?).amount()
                ),
            },
        ],
        2,
    );
    assert_eq!(total, eur("13.00")?);
    assert_eq!(change.minor_units(), -250);
    assert!(mixed.is_err());
    assert_eq!(dinars.to_string(), "1.234 BHD");
    Ok(())
}

fn ratios() -> Result<(), Box<dyn Error>> {
    let percentage = Percentage::parse("12.5")?;
    let parts = Parts::parse("1.5")?;
    let rate = ExchangeRate::parse("4.97650")?;
    println!("Percentages, parts, and rates are exact decimals:");
    print_table(
        [
            ValueRow {
                input: "12.5%",
                result: format!("{percentage}, {} basis points", percentage.basis_points()),
            },
            ValueRow {
                input: "333 basis points",
                result: Percentage::from_basis_points(333).to_string(),
            },
            ValueRow {
                input: "33.333%",
                result: message(&Percentage::parse("33.333")),
            },
            ValueRow {
                input: "1.5 parts",
                result: format!("{parts}, {} hundredths", parts.hundredths()),
            },
            ValueRow {
                input: "3 whole parts",
                result: Parts::from_whole(3)?.to_string(),
            },
            ValueRow {
                input: "25 hundredths of a part",
                result: Parts::from_hundredths(25)?.to_string(),
            },
            ValueRow {
                input: "0 parts",
                result: message(&Parts::parse("0")),
            },
            ValueRow {
                input: "a rate of 4.97650",
                result: rate.to_string(),
            },
            ValueRow {
                input: "a rate of 0",
                result: message(&ExchangeRate::parse("0")),
            },
        ],
        2,
    );
    assert_eq!(percentage.basis_points(), 1_250);
    assert_eq!(parts.hundredths(), 150);
    assert_eq!(Parts::ONE.hundredths(), 100);
    // A rate keeps no trailing zero: 4.97650 is 49765 at scale 4.
    assert_eq!(rate.mantissa(), 49_765);
    assert_eq!(rate.scale(), 4);
    assert_eq!(ExchangeRate::from_decimal(2_011, 4)?.to_string(), "0.2011");
    Ok(())
}

fn dates_and_identifiers(people: &People) -> Result<(), Box<dyn Error>> {
    let date = CalendarDate::parse("2026-10-01")?;
    let time = TimeOfDay::parse("22:15")?;
    let text = people.ana.to_string();
    let parsed: MemberId = text.parse()?;
    let language = Language::parse("pt-pt")?;
    println!("Dates, times, identifiers, and language tags read and print the ISO forms:");
    print_table(
        [
            ValueRow {
                input: "2026-10-01",
                result: format!(
                    "{date}: day {}, month {}, year {}",
                    date.day(),
                    date.month(),
                    date.year()
                ),
            },
            ValueRow {
                input: "29 February 2027",
                result: message(&CalendarDate::new(2027, 2, 29)),
            },
            ValueRow {
                input: "22:15",
                result: format!("{time}: hour {}, minute {}", time.hour(), time.minute()),
            },
            ValueRow {
                input: "24:00",
                result: message(&TimeOfDay::new(24, 0)),
            },
            ValueRow {
                input: "Ana's identifier",
                result: format!(
                    "{parsed}, the same after a round trip: {}",
                    parsed == people.ana && MemberId::new(people.ana.as_str())? == people.ana
                ),
            },
            ValueRow {
                input: "empty text",
                result: message(&MemberId::new("")),
            },
            ValueRow {
                input: "ana after a space",
                result: message(&MemberId::new(" ana")),
            },
            ValueRow {
                input: "a tab between a and b",
                result: message(&MemberId::new("a\tb")),
            },
            ValueRow {
                input: "65 letters a",
                result: message(&MemberId::new(&"a".repeat(65))),
            },
            ValueRow {
                input: "pt-pt",
                result: language.as_str().to_owned(),
            },
            ValueRow {
                input: "pt_PT",
                result: message(&Language::parse("pt_PT")),
            },
        ],
        2,
    );
    assert_eq!(date.day(), 1);
    assert_eq!(date.month(), 10);
    assert_eq!(date.year(), 2026);
    assert_eq!(time.hour(), 22);
    assert_eq!(time.minute(), 15);
    assert_eq!(parsed, people.ana);
    assert_eq!(MemberId::new(""), Err(IdError::Empty));
    assert_eq!(MemberId::new(" ana"), Err(IdError::SurroundingWhitespace));
    assert_eq!(MemberId::new("a\tb"), Err(IdError::ControlCharacter));
    assert_eq!(MemberId::new(&"a".repeat(65)), Err(IdError::TooLong));
    Ok(())
}

fn values(people: &People) -> Result<(), Box<dyn Error>> {
    section("13. Values on their own");
    currencies()?;
    amounts()?;
    ratios()?;
    dates_and_identifiers(people)
}

fn sharing(group: ExpenseGroup, people: &People) -> Result<(), Box<dyn Error>> {
    section("14. Sharing the group");
    let shared = SharedExpenseGroup::new(group);
    // Another handle on the same group, as another task or thread holds.
    let other = shared.clone();
    let everyone = [people.ana, people.bob, people.carol, people.dan, people.eva];
    let coffee = Expense::builder(
        "Airport coffee",
        ron("75.00")?,
        people.bob,
        Split::equal(everyone),
        october(8)?,
    )
    .build()?;
    block_on(other.add_expense(coffee))?;
    let plan = block_on(shared.settlement(SettlementStrategy::MinimumTransfers))?;
    println!(
        "An app shares the group between its tasks and threads through a\n\
         SharedExpenseGroup, whose calls are futures any executor runs; here the library's\n\
         block_on runs each one on this thread. Bob adds the airport coffee through another\n\
         handle, and the plan that settles it comes through the first:"
    );
    {
        let reading = block_on(shared.read());
        print_table(
            plan.payments().iter().map(|payment| PaymentRow {
                from: name(&reading, payment.from()),
                to: name(&reading, payment.to()),
                amount: payment.amount(),
            }),
            2,
        );
    }

    let before = block_on(shared.summary());
    let rates = [CurrencyRate::new(
        CurrencyCode::RON,
        ExchangeRate::parse("0.2")?,
    )];
    let (waited, reported) = {
        // A report reads the group, holding the shared lock meanwhile.
        let report = block_on(shared.read());
        let mut change = pin!(shared.change_currency(CurrencyCode::EUR, rates));
        let waited = change
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
            .is_pending();
        (waited, report.summary().total_spent())
        // The block drops the waiting change, then the report's guard.
    };
    let after = block_on(shared.summary());
    println!(
        "While a report reads the group, a change of its currency to EUR waits for the\n\
         report. The app drops the waiting change, which cancels it: the group still\n\
         keeps its books in {}, and the trip cost {}, as the report read.",
        block_on(shared.currency()),
        after.total_spent()
    );
    drop(other);
    let returned = shared
        .into_expense_group()
        .map_err(|_| "another handle still shares the group")?;
    println!(
        "With the other handle gone, the app takes the group back: {} expenses.",
        returned.expenses().len()
    );

    assert_eq!(plan.len(), 4);
    for payment in plan.payments() {
        assert_eq!(payment.to(), people.bob);
        assert_eq!(payment.amount(), ron("15.00")?);
    }
    assert!(waited);
    assert_eq!(after, before);
    assert_eq!(reported, after.total_spent());
    assert_eq!(returned.currency(), CurrencyCode::RON);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Lisbon and Porto: a grand tour of eranos");
    println!("════════════════════════════════════════");
    println!(
        "Ana, Bob, Carol, Dan, and Carol's daughter Eva spend a week in Portugal. Each\n\
         section below uses another part of the library, from creating the group to saving\n\
         it, and checks its key numbers."
    );
    let mut group = the_group()?;
    let people = members(&mut group)?;
    let categories = categories(&mut group)?;
    let seafood = expenses(&mut group, &people, categories)?;
    another_currency(&mut group, &people, categories)?;
    edits(&mut group, &people, categories, seafood)?;
    comings_and_goings(&mut group, &people)?;
    summary(&group, &people, categories)?;
    removing_a_category(&mut group, categories)?;
    new_currency(&mut group, &people)?;
    settling_up(&mut group)?;
    saving_and_loading(&group, &people, categories)?;
    values(&people)?;
    sharing(group, &people)
}
