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
use eranos::error::{ExpenseError, GroupError, TransferError};
use eranos::expense::ExpenseBuilder;
use eranos::{
    Account, AccountDetails, AccountKind, Balance, CalendarDate, Category, CategoryId,
    CategoryTotal, Cost, Currency, CurrencyRate, Email, ExchangeRate, Expense, ExpenseId,
    ExpenseShares, ExpenseSplit, Group, Iban, Item, Language, Member, MemberAmount, MemberId,
    MemberParts, MemberPercentage, MemberShare, Money, Parts, Percentage, PhoneNumber, SavedGroup,
    SettlementStrategy, SharedGroup, Split, TimeOfDay, Tip, TipDistribution, TipValue, Transfer,
    Username,
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
    #[tabled(rename = "Detail", order = 0)]
    detail: &'static str,
    #[tabled(rename = "Value", order = 1)]
    value: String,
}

#[derive(Tabled)]
struct MemberRow {
    #[tabled(rename = "Accounts", order = 3)]
    accounts: String,
    #[tabled(rename = "Email", order = 1)]
    email: String,
    #[tabled(rename = "Member", order = 0)]
    name: String,
    #[tabled(rename = "Phone", order = 2)]
    phone: String,
}

#[derive(Tabled)]
struct CheckRow {
    #[tabled(rename = "Input", order = 0)]
    input: String,
    #[tabled(rename = "Checked as", order = 1)]
    kind: &'static str,
    #[tabled(rename = "Result", order = 2)]
    result: String,
}

#[derive(Tabled)]
struct RefusalRow {
    #[tabled(rename = "Change", order = 0)]
    change: &'static str,
    #[tabled(rename = "Refused with", order = 1)]
    error: String,
}

#[derive(Tabled)]
struct ShareRow {
    #[tabled(rename = "Consumed", order = 4)]
    consumed: String,
    #[tabled(rename = "Expense", order = 0)]
    expense: String,
    #[tabled(rename = "Split", order = 1)]
    how: String,
    #[tabled(rename = "Member", order = 2)]
    members: String,
    #[tabled(rename = "Paid", order = 3)]
    paid: String,
}

#[derive(Tabled)]
struct ItemRow {
    #[tabled(rename = "Line total", order = 4)]
    line_total: Money,
    #[tabled(rename = "Item", order = 0)]
    name: String,
    #[tabled(rename = "Quantity", order = 2)]
    quantity: u32,
    #[tabled(rename = "Shared by", order = 1)]
    shared_by: String,
    #[tabled(rename = "Unit price", order = 3)]
    unit_price: Money,
}

#[derive(Tabled)]
struct EntryRow {
    #[tabled(rename = "In the group currency", order = 4)]
    converted: Money,
    #[tabled(rename = "Entry", order = 0)]
    name: String,
    #[tabled(rename = "Entered as", order = 2)]
    original: Money,
    #[tabled(rename = "Paid by", order = 1)]
    payer: String,
    #[tabled(rename = "Rate", order = 3)]
    rate: ExchangeRate,
}

#[derive(Tabled)]
struct EditRow {
    #[tabled(rename = "Edit", order = 1)]
    edit: String,
    #[tabled(rename = "Entry", order = 0)]
    entry: String,
}

#[derive(Tabled)]
struct BalanceRow {
    #[tabled(rename = "Balance", order = 5)]
    balance: Balance,
    #[tabled(rename = "Consumed", order = 2)]
    consumed: Money,
    #[tabled(rename = "Member", order = 0)]
    name: String,
    #[tabled(rename = "Paid", order = 1)]
    paid: Money,
    #[tabled(rename = "Received", order = 4)]
    received: Money,
    #[tabled(rename = "Sent", order = 3)]
    sent: Money,
}

#[derive(Tabled)]
struct StatusRow {
    #[tabled(rename = "Balance", order = 2)]
    balance: Balance,
    #[tabled(rename = "Member", order = 0)]
    name: String,
    #[tabled(rename = "Status", order = 1)]
    status: &'static str,
}

#[derive(Tabled)]
struct CategoryRow {
    #[tabled(rename = "Expenses", order = 1)]
    expenses: usize,
    #[tabled(rename = "Category", order = 0)]
    name: String,
    #[tabled(rename = "Total", order = 2)]
    total: Money,
}

#[derive(Tabled)]
struct MemberCategoryRow {
    #[tabled(rename = "Food", order = 1)]
    food: Money,
    #[tabled(rename = "Lodging", order = 2)]
    lodging: Money,
    #[tabled(rename = "Member", order = 0)]
    name: String,
    #[tabled(rename = "Getting around", order = 3)]
    transport: Money,
    #[tabled(rename = "Uncategorised", order = 4)]
    uncategorised: Money,
}

#[derive(Tabled)]
struct StrategyRow {
    #[tabled(rename = "Payments", order = 1)]
    payments: usize,
    #[tabled(rename = "Strategy", order = 0)]
    strategy: &'static str,
}

#[derive(Tabled)]
struct PaymentRow {
    #[tabled(rename = "Amount", order = 2)]
    amount: Money,
    #[tabled(rename = "From", order = 0)]
    from: String,
    #[tabled(rename = "To", order = 1)]
    to: String,
}

#[derive(Tabled)]
struct LoadRow {
    #[tabled(rename = "Change to the file", order = 0)]
    change: &'static str,
    #[tabled(rename = "Message", order = 1)]
    message: GroupError,
}

#[derive(Tabled)]
struct CurrencyRow {
    #[tabled(rename = "Code", order = 0)]
    code: &'static str,
    #[tabled(rename = "Decimal places", order = 3)]
    decimal_places: u8,
    #[tabled(rename = "Name", order = 1)]
    name: &'static str,
    #[tabled(rename = "Numeric code", order = 2)]
    numeric_code: u16,
}

#[derive(Tabled)]
struct ValueRow {
    #[tabled(rename = "Input", order = 0)]
    input: &'static str,
    #[tabled(rename = "Result", order = 1)]
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
    Ok(Money::parse(text, Currency::EUR)?)
}

fn ron(text: &str) -> Result<Money, Box<dyn Error>> {
    Ok(Money::parse(text, Currency::RON)?)
}

/// A date of the trip, in October 2026.
fn october(date: u8) -> Result<CalendarDate, Box<dyn Error>> {
    Ok(CalendarDate::new(date, 10, 2026)?)
}

fn name(group: &Group, member: MemberId) -> String {
    group
        .member(member)
        .map_or("?", Member::first_name)
        .to_owned()
}

/// The first names of `members`, joined by `separator`.
fn names(group: &Group, members: impl IntoIterator<Item = MemberId>, separator: &str) -> String {
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
    currency: Currency,
    amounts: impl IntoIterator<Item = Money>,
) -> Result<Money, Box<dyn Error>> {
    amounts
        .into_iter()
        .try_fold(Money::zero(currency), |total, amount| {
            Ok(total.checked_add(amount)?)
        })
}

fn print_balances(group: &Group) {
    let summary = group.summary();
    print_table(
        group.members().map(|member| {
            let totals = summary.member(member.id());
            BalanceRow {
                balance: totals.balance(),
                consumed: totals.share(),
                name: member.first_name().to_owned(),
                paid: totals.paid(),
                received: totals.received(),
                sent: totals.sent(),
            }
        }),
        1,
    );
}

fn print_categories(group: &Group) {
    let summary = group.summary();
    print_table(
        summary.category_totals().map(|total| CategoryRow {
            expenses: group
                .expenses()
                .filter(|expense| expense.category() == total.category())
                .count(),
            name: group
                .category(total.category())
                .map_or("?", Category::name)
                .to_owned(),
            total: total.total(),
        }),
        1,
    );
}

/// Every expense and transfer in a currency `shown` accepts, as entered and
/// in the group currency.
fn print_entries(group: &Group, shown: impl Fn(Currency) -> bool) {
    let expenses = group
        .expenses()
        .filter(|expense| shown(expense.amount().currency()))
        .map(|expense| EntryRow {
            converted: group
                .expense_shares(expense.id())
                .map_or_else(|| Money::zero(group.currency()), ExpenseShares::total),
            name: expense.name().to_owned(),
            original: expense.total(),
            payer: names(group, expense.payers().members(), ", "),
            rate: expense.exchange_rate(),
        });
    let transfers = group
        .transfers()
        .filter(|transfer| shown(transfer.amount().currency()))
        .map(|transfer| EntryRow {
            converted: group
                .transfer_amount(transfer.id())
                .unwrap_or_else(|| Money::zero(group.currency())),
            name: format!("Transfer to {}", name(group, transfer.to())),
            original: transfer.amount(),
            payer: name(group, transfer.from()),
            rate: transfer.exchange_rate(),
        });
    print_table(expenses.chain(transfers), 2);
}

fn the_group() -> Result<Group, Box<dyn Error>> {
    section("1. The group");
    let mut group = Group::builder(Currency::EUR, "Lisbon")
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

    assert_eq!(blank, Err(GroupError::EmptyName));
    assert_eq!(group.name(), "Lisbon and Porto");
    assert_eq!(group.language().map(Language::as_str), Some("pt-PT"));
    Ok(group)
}

/// An account with its label, its kind, and its details.
fn describe(account: &Account) -> String {
    let kind = match account.kind() {
        AccountKind::Iban => "IBAN",
        AccountKind::Username => "username",
        _ => "account",
    };
    let details = match account.details() {
        AccountDetails::Iban(iban) => iban.as_str(),
        AccountDetails::Username(username) => username.as_str(),
        _ => "",
    };
    format!("{}, {kind} {details}", account.name())
}

fn print_members(group: &Group) {
    print_table(
        group.members().map(|member| MemberRow {
            accounts: cell(member.accounts().iter().map(describe)),
            email: member.email().map_or("", Email::as_str).to_owned(),
            name: [Some(member.first_name()), member.last_name()]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" "),
            phone: member
                .phone()
                .map_or_else(String::new, PhoneNumber::international),
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

fn members(group: &mut Group) -> Result<People, Box<dyn Error>> {
    section("2. Members");
    let ana = group.add_member(
        Member::builder("Ana")
            .last_name("Popescu")
            .email(Email::parse("ana@example.com")?)
            .phone(PhoneNumber::parse("+40 712 345 678")?)
            .account(Account::new(
                Iban::parse("RO49 AAAA 1B31 0075 9384 0000")?,
                "ING",
            )?)
            .account(Account::new(Username::new("@ana")?, "Revolut")?)
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

fn categories(group: &mut Group) -> Result<Categories, Box<dyn Error>> {
    section("3. Categories");
    let food = group.add_category(Category::new("Food")?)?;
    let lodging = group.add_category(Category::new("Lodging")?)?;
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
        "Every group has the built-in Uncategorised; this one adds Food, Lodging, and\n\
         Transport, then renames Transport, which keeps its identifier. In identifier order:\n  \
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
        Err(GroupError::DuplicateCategoryName { existing: food })
    );
    assert_eq!(built_in, Err(GroupError::BuiltInCategory));
    assert_eq!(
        group.category(transport).map(Category::name),
        Some("Getting around")
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
        ExpenseSplit::Itemised(items) => format!("{} items", items.len()),
        ExpenseSplit::Whole(split) => match split {
            Split::Amounts(_) => "by amounts",
            Split::Equal(_) => "equally",
            Split::Parts(_) => "by parts",
            Split::Percentages(_) => "by percentage",
            _ => "another way",
        }
        .to_owned(),
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
            TipValue::Fixed(amount) => format!(", {amount} tip"),
            TipValue::Percentage(rate) => format!(", {rate} tip"),
            _ => ", a tip".to_owned(),
        }
    };
    format!("{split}{payers}{tip}")
}

/// Each expense of `expenses`, with every involved member's payment and
/// consumption.
fn print_shares(group: &Group, expenses: &[ExpenseId]) {
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
            consumed: cell(
                shares
                    .members()
                    .iter()
                    .map(|share| share.share().to_string()),
            ),
            expense: expense.name().to_owned(),
            how: how(expense),
            members: cell(
                shares
                    .members()
                    .iter()
                    .map(|share| name(group, share.member())),
            ),
            paid: cell(shares.members().iter().map(paid)),
        })
    });
    print_table(rows, 3);
}

fn print_items(group: &Group, id: ExpenseId) {
    let Some(ExpenseSplit::Itemised(items)) = group.expense(id).map(Expense::split) else {
        return;
    };
    print_table(
        items.iter().map(|item| ItemRow {
            line_total: item.line_total(),
            name: item.name().to_owned(),
            quantity: item.quantity(),
            shared_by: names(group, item.split().members(), ", "),
            unit_price: item.unit_price(),
        }),
        2,
    );
}

/// The receipt of the fado night: every line with its own split.
fn fado_items(people: People) -> Result<Vec<Item>, Box<dyn Error>> {
    let People {
        ana,
        bob,
        carol,
        dan,
        eva,
        ..
    } = people;
    Ok(vec![
        Item::builder([ana, bob, dan], "Vinho verde", eur("12.00")?)
            .quantity(2)
            .build()?,
        Item::builder([carol], "Bacalhau", eur("18.00")?).build()?,
        Item::builder_with_split(
            "Custard tarts",
            Split::parts([
                MemberParts::new(ana, Parts::ONE),
                MemberParts::new(bob, Parts::ONE),
                MemberParts::new(carol, Parts::ONE),
                MemberParts::new(dan, Parts::ONE),
                MemberParts::new(eva, Parts::from_whole(2)?),
            ]),
            eur("1.50")?,
        )
        .quantity(6)
        .build()?,
        Item::builder([eva], "Juice", eur("3.00")?).build()?,
    ])
}

/// The fado night, item by item, paid by Carol and Dan, with a 10 % tip; the
/// caller builds it, which gives it its identifier.
fn fado_night(people: People, categories: Categories) -> Result<ExpenseBuilder, Box<dyn Error>> {
    Ok(Expense::itemised_builder_with_payers(
        eur("54.00")?,
        october(4)?,
        fado_items(people)?,
        "Fado night",
        Split::amounts([
            MemberAmount::new(eur("39.40")?, people.carol),
            MemberAmount::new(eur("20.00")?, people.dan),
        ]),
    )
    .category(categories.food)
    .description("Tasca do Chico")
    .time(TimeOfDay::new(22, 15)?)
    .tip(Tip::percentage(Percentage::parse("10")?)))
}

/// Adds one expense for every way of dividing one.
fn add_expenses(
    group: &mut Group,
    people: People,
    categories: Categories,
) -> Result<Expenses, Box<dyn Error>> {
    let People {
        ana,
        bob,
        carol,
        dan,
        eva,
        ..
    } = people;
    let everyone = [ana, bob, carol, dan, eva];
    let groceries = Expense::builder(
        eur("10.00")?,
        october(1)?,
        "Groceries",
        ana,
        Split::equal([ana, bob, carol]),
    )
    .category(categories.food);
    let apartment = Expense::builder(
        eur("800.00")?,
        october(1)?,
        "Apartment",
        ana,
        Split::percentages([
            MemberPercentage::new(ana, Percentage::parse("25")?),
            MemberPercentage::new(bob, Percentage::parse("25")?),
            MemberPercentage::new(carol, Percentage::parse("35")?),
            MemberPercentage::new(dan, Percentage::parse("15")?),
        ]),
    )
    .category(categories.lodging);
    let seafood = Expense::builder(
        eur("90.00")?,
        october(2)?,
        "Seafood dinner",
        carol,
        Split::parts([
            MemberParts::new(ana, Parts::ONE),
            MemberParts::new(bob, Parts::ONE),
            MemberParts::new(carol, Parts::ONE),
            MemberParts::new(dan, Parts::ONE),
            MemberParts::new(eva, Parts::parse("0.5")?),
        ]),
    )
    .category(categories.food);
    let train = Expense::builder(
        eur("88.50")?,
        october(3)?,
        "Train to Porto",
        bob,
        Split::amounts([
            MemberAmount::new(eur("24.50")?, ana),
            MemberAmount::new(eur("24.50")?, bob),
            MemberAmount::new(eur("24.50")?, carol),
            MemberAmount::new(eur("15.00")?, eva),
        ]),
    )
    .category(categories.transport);
    let museum = Expense::builder_with_payers(
        eur("60.00")?,
        october(3)?,
        "Museum",
        Split::equal([ana, bob]),
        Split::equal([ana, bob, carol, dan]),
    );
    let fado = fado_night(people, categories)?;
    let tour = Expense::builder(
        eur("45.00")?,
        october(5)?,
        "Tuk-tuk tour",
        dan,
        Split::equal(everyone),
    )
    .category(categories.transport)
    .tip(Tip::fixed(eur("5.00")?).distributed(TipDistribution::Equal));
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
fn consumed(group: &Group, id: ExpenseId, member: MemberId) -> Option<Money> {
    group
        .expense_shares(id)?
        .member(member)
        .map(MemberShare::share)
}

fn expenses(
    group: &mut Group,
    people: People,
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
        TipValue::Fixed(amount) => amount.to_string(),
        TipValue::Percentage(rate) => rate.to_string(),
        _ => "a tip".to_owned(),
    };
    let shared = match tip.distribution() {
        TipDistribution::Equal => "equally",
        TipDistribution::Proportional => "in proportion to what each member ordered",
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
    group: &mut Group,
    people: People,
    categories: Categories,
) -> Result<(), Box<dyn Error>> {
    section("5. Another currency");
    let People { ana, bob, .. } = people;
    let rate = ExchangeRate::parse("0.2")?;
    let parking = group.add_expense(
        Expense::builder(
            ron("250.00")?.at(rate),
            october(6)?,
            "Airport parking",
            ana,
            Split::equal([ana, bob]),
        )
        .category(categories.transport)
        .build()?,
    )?;
    let cash = group.add_transfer(
        Transfer::builder(ron("100.00")?.at(rate), october(6)?, bob, ana)
            .note("Cash for the parking")
            .time(TimeOfDay::new(7, 40)?)
            .build()?,
    )?;
    println!(
        "Back at the Bucharest airport, Ana paid the parking in RON and Bob paid her back in\n\
         cash. An entry in another currency carries the rate into the group currency, how\n\
         many EUR one RON is worth, and is converted once, when it is entered:"
    );
    print_entries(group, |currency| Currency::EUR != currency);
    let transfer = group.transfer(cash).ok_or("the cash was recorded")?;
    let converted = ExchangeRate::parse("0.2011")?.convert(ron("250.00")?, Currency::EUR)?;
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
            .unwrap_or_else(|| Money::zero(Currency::EUR)),
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
    group: &mut Group,
    people: People,
    categories: Categories,
    seafood: ExpenseId,
) -> Result<Vec<EditRow>, Box<dyn Error>> {
    let People {
        ana,
        bob,
        carol,
        dan,
        ..
    } = people;
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
            eur("6.00")?,
            october(5)?,
            "Coffee",
            bob,
            Split::equal([ana, bob, carol]),
        )
        .category(categories.food)
        .build()?)
    };
    group.add_expense(coffee()?)?;
    let copy = group.add_expense(coffee()?)?;
    let removed = group.remove_expense(copy)?;
    // Dan's part of the apartment was 120.00, not 100.00.
    let first =
        group.add_transfer(Transfer::builder(eur("100.00")?, october(5)?, dan, ana).build()?)?;
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
        group.add_transfer(Transfer::builder(eur("10.00")?, october(5)?, carol, bob).build()?)?;
    let undone = group.remove_transfer(mistake)?;

    assert_eq!(previous.amount(), eur("90.00")?);
    assert!(group.expense(copy).is_none());
    assert_eq!(group.transfer_amount(first), Some(eur("120.00")?));
    assert!(group.transfer(mistake).is_none());
    Ok(vec![
        EditRow {
            edit: format!(
                "{} became {}, at {}",
                previous.amount(),
                group
                    .expense(seafood)
                    .map_or_else(|| Money::zero(Currency::EUR), Expense::amount),
                group
                    .expense(seafood)
                    .and_then(Expense::description)
                    .unwrap_or_default()
            ),
            entry: previous.name().to_owned(),
        },
        EditRow {
            edit: format!(
                "entered twice; the copy of {} was removed",
                removed.amount()
            ),
            entry: removed.name().to_owned(),
        },
        EditRow {
            edit: format!(
                "{} became {}, noted {}",
                replaced.amount(),
                group
                    .transfer(first)
                    .map_or_else(|| Money::zero(Currency::EUR), Transfer::amount),
                group
                    .transfer(first)
                    .and_then(Transfer::note)
                    .unwrap_or_default()
            ),
            entry: format!("{} to {}", name(group, dan), name(group, ana)),
        },
        EditRow {
            edit: format!("{} sent to the wrong member; removed", undone.amount()),
            entry: format!("{} to {}", name(group, carol), name(group, bob)),
        },
    ])
}

fn edits(
    group: &mut Group,
    people: People,
    categories: Categories,
    seafood: ExpenseId,
) -> Result<(), Box<dyn Error>> {
    section("6. Edits");
    let People { ana, bob, .. } = people;
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
        eur("30.00")?,
        october(6)?,
        "Taxi",
        ana,
        Split::percentages([
            MemberPercentage::new(ana, Percentage::parse("50")?),
            MemberPercentage::new(bob, Percentage::parse("49.99")?),
        ]),
    )
    .build();
    let to_herself = Transfer::builder(eur("5.00")?, october(6)?, ana, ana).build();
    let unrated = group.add_expense(
        Expense::builder(
            ron("40.00")?,
            october(6)?,
            "Souvenirs",
            ana,
            Split::equal([ana]),
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
        Err(GroupError::MissingExchangeRate {
            currency: Currency::RON
        })
    );
    assert_eq!(group.summary(), &before);
    Ok(())
}

fn comings_and_goings(group: &mut Group, people: People) -> Result<(), Box<dyn Error>> {
    section("7. Members come and go");
    let People {
        ana,
        bob,
        carol,
        dan,
        filip,
        ..
    } = people;
    let removed = group.remove_member(filip)?;
    group.archive_member(dan)?;
    let farewell = group.add_expense(
        Expense::builder(
            eur("40.00")?,
            october(6)?,
            "Farewell dinner",
            ana,
            Split::equal([ana, bob, carol, dan]),
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
            balance: summary.member(member.id()).balance(),
            name: member.first_name().to_owned(),
            status: if group.is_archived(member.id()) {
                "archived"
            } else {
                "active"
            },
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
    assert_eq!(farewell, Err(GroupError::ArchivedMember(dan)));
    assert!(matches!(in_use, Err(GroupError::MemberInUse { .. })));
    assert!(!group.is_archived(dan));
    Ok(())
}

fn summary(group: &Group, people: People, categories: Categories) -> Result<(), Box<dyn Error>> {
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
                food: total(categories.food),
                lodging: total(categories.lodging),
                name: member.first_name().to_owned(),
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
            Currency::EUR,
            summary
                .member_category_totals(member.member())
                .map(CategoryTotal::total),
        )?;
        assert_eq!(by_category, member.share());
    }
    assert!(sum(Currency::EUR, summary.members().map(|member| member.net()))?.is_zero());
    Ok(())
}

fn removing_a_category(group: &mut Group, categories: Categories) -> Result<(), Box<dyn Error>> {
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

    assert_eq!(built_in, Err(GroupError::BuiltInCategory));
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

fn new_currency(group: &mut Group, people: People) -> Result<(), Box<dyn Error>> {
    section("10. A new currency");
    let missing = group.change_currency(Currency::RON, &[]);
    group.change_currency(
        Currency::RON,
        &[CurrencyRate::new(Currency::EUR, ExchangeRate::parse("5")?)],
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
        Err(GroupError::MissingExchangeRate {
            currency: Currency::EUR
        })
    );
    assert_eq!(group.currency(), Currency::RON);
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
        SettlementStrategy::Direct => "Direct",
        SettlementStrategy::Greedy => "Greedy",
        SettlementStrategy::MinimumTransfers => "MinimumTransfers",
        _ => "another strategy",
    }
}

fn settling_up(group: &mut Group) -> Result<(), Box<dyn Error>> {
    section("11. Settling up");
    let mut counts = Vec::new();
    for strategy in SettlementStrategy::all() {
        let settlement = group.settlement(strategy)?;
        assert_eq!(settlement.strategy(), strategy);
        counts.push(StrategyRow {
            payments: settlement.len(),
            strategy: strategy_name(settlement.strategy()),
        });
    }
    println!(
        "Each strategy turns the balances into payments: the direct one pays back every\n\
         pair's own debts, greedy pays the largest amounts first, and the minimum one\n\
         finds the fewest payments."
    );
    print_table(counts, 1);
    let plan = group
        .settlement(SettlementStrategy::MinimumTransfers)?
        .clone();
    println!("The group follows the minimum plan:");
    print_table(
        plan.payments().iter().map(|payment| PaymentRow {
            amount: payment.amount(),
            from: name(group, payment.from()),
            to: name(group, payment.to()),
        }),
        2,
    );
    for payment in plan.payments() {
        group.add_transfer(Transfer::from_payment(payment, october(7)?))?;
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

/// What loading the saved form `text` refuses: the error of `into_group`,
/// which replays every entry through the checks of adding it.
fn refusal(text: &str) -> Result<GroupError, Box<dyn Error>> {
    let saved: SavedGroup = serde_json::from_str(text)?;
    Ok(saved
        .into_group()
        .err()
        .ok_or("the change must be refused")?)
}

fn saving_and_loading(
    group: &Group,
    people: People,
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
            AccountDetails::Iban(iban) => iban.as_str(),
            AccountDetails::Username(username) => username.as_str(),
            _ => "",
        };
        assert!(saved.contains(details), "{details} is saved as it is");
        println!("  {}: {details}", account.name());
    }
    let loaded: Group = serde_json::from_str(&saved)?;
    println!(
        "Loading replays every entry through the same checks as adding it and rebuilds the\n\
         totals: the loaded group equals the saved one. Reading a SavedGroup and loading it\n\
         with into_group reports a refusal as a typed error instead of a message:"
    );
    let tampered = [
        Tampering {
            change: "\"format\": 3",
            text: saved.replacen("\"format\": 2", "\"format\": 3", 1),
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
            GroupError::UnsupportedFormat { format: 3 },
            GroupError::EmptyName,
            GroupError::DuplicateCategoryName {
                existing: categories.food
            },
        ]
    );
    Ok(())
}

fn currencies() -> Result<(), Box<dyn Error>> {
    let mut rows = Vec::new();
    for code in ["EUR", "RON", "JPY", "BHD"] {
        let currency = Currency::from_code(code)?;
        rows.push(CurrencyRow {
            code: currency.code(),
            decimal_places: currency.decimal_places(),
            name: currency.name(),
            numeric_code: currency.numeric_code(),
        });
    }
    println!(
        "Currencies are the {} active ISO 4217 codes with a minor unit; an unknown code is\n\
         refused ({}):",
        Currency::all().len(),
        message(&Currency::from_code("XYZ"))
    );
    print_table(rows, 2);
    assert_eq!(Currency::from_code("BHD")?.decimal_places(), 3);
    Ok(())
}

fn amounts() -> Result<(), Box<dyn Error>> {
    let total = eur("12.34")?.checked_add(eur("0.66")?)?;
    let change = eur("5.00")?.checked_sub(eur("7.50")?)?;
    let mixed = eur("1.00")?.checked_add(ron("1.00")?);
    let yen = Money::parse("1500", Currency::JPY)?;
    let dinars = Money::from_minor_units(Currency::BHD, 1_234);
    let nothing = Money::zero(Currency::RON);
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
                result: message(&Money::parse("1.005", Currency::EUR)),
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

fn dates_and_identifiers(people: People) -> Result<(), Box<dyn Error>> {
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
                result: message(&CalendarDate::new(29, 2, 2027)),
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
                    parsed == people.ana && MemberId::from_bytes(parsed.to_bytes()) == people.ana
                ),
            },
            ValueRow {
                input: "not-a-uuid",
                result: message(&"not-a-uuid".parse::<MemberId>()),
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
    Ok(())
}

fn values(people: People) -> Result<(), Box<dyn Error>> {
    section("13. Values on their own");
    currencies()?;
    amounts()?;
    ratios()?;
    dates_and_identifiers(people)
}

fn sharing(group: Group, people: People) -> Result<(), Box<dyn Error>> {
    section("14. Sharing the group");
    let shared = SharedGroup::new(group);
    // Another handle on the same group, as another task or thread holds.
    let other = shared.clone();
    let everyone = [people.ana, people.bob, people.carol, people.dan, people.eva];
    let coffee = Expense::builder(
        ron("75.00")?,
        october(8)?,
        "Airport coffee",
        people.bob,
        Split::equal(everyone),
    )
    .build()?;
    block_on(other.add_expense(coffee))?;
    let plan = block_on(shared.settlement(SettlementStrategy::MinimumTransfers))?;
    println!(
        "An app shares the group between its tasks and threads through a SharedGroup,\n\
         whose calls are futures any executor runs; here the library's block_on runs\n\
         each one on this thread. Bob adds the airport coffee through another handle,\n\
         and the plan that settles it comes through the first:"
    );
    {
        let reading = block_on(shared.read());
        print_table(
            plan.payments().iter().map(|payment| PaymentRow {
                amount: payment.amount(),
                from: name(&reading, payment.from()),
                to: name(&reading, payment.to()),
            }),
            2,
        );
    }

    let before = block_on(shared.summary());
    let rates = [CurrencyRate::new(
        Currency::RON,
        ExchangeRate::parse("0.2")?,
    )];
    let (waited, reported) = {
        // A report reads the group, holding the shared lock meanwhile.
        let report = block_on(shared.read());
        let mut change = pin!(shared.change_currency(Currency::EUR, rates));
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
        .into_group()
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
    assert_eq!(returned.currency(), Currency::RON);
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
    let seafood = expenses(&mut group, people, categories)?;
    another_currency(&mut group, people, categories)?;
    edits(&mut group, people, categories, seafood)?;
    comings_and_goings(&mut group, people)?;
    summary(&group, people, categories)?;
    removing_a_category(&mut group, categories)?;
    new_currency(&mut group, people)?;
    settling_up(&mut group)?;
    saving_and_loading(&group, people, categories)?;
    values(people)?;
    sharing(group, people)
}
