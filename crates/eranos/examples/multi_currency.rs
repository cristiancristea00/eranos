//! A trip across currencies: the group keeps its books in EUR, expenses and
//! a cash repayment in RON and HUF carry a rate fixed when they are entered,
//! and the group later moves its books to RON.
//!
//! Run with `cargo run -p eranos --example multi_currency`.

use std::error::Error;

use eranos::{
    Balance, Currency, Date, ExchangeRate, Expense, ExpenseShares, Group, Member, MemberId, Money,
    Split, Transfer,
};
use tabled::settings::object::Columns;
use tabled::settings::{Alignment, Style};
use tabled::{Table, Tabled};

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

/// Prints a section heading with a line under it.
fn section(title: &str) {
    println!("\n{title}\n{}", "─".repeat(title.chars().count()));
}

/// Prints `rows` with rounded borders, right-aligning the columns from
/// `numbers_from` on.
fn print_table<Row: Tabled>(rows: impl IntoIterator<Item = Row>, numbers_from: usize) {
    let mut table = Table::new(rows);
    table
        .with(Style::rounded())
        .modify(Columns::new(numbers_from..), Alignment::right());
    println!("{table}");
}

fn name(group: &Group, member: MemberId) -> String {
    group
        .member(member)
        .map_or("?", Member::first_name)
        .to_owned()
}

/// Every expense and transfer, as entered and in the group currency.
fn print_entries(group: &Group) {
    let expenses = group.expenses().map(|(id, expense)| EntryRow {
        name: expense.name().to_owned(),
        payer: expense
            .payers()
            .members()
            .map(|member| name(group, member))
            .collect::<Vec<_>>()
            .join(", "),
        original: expense.total(),
        rate: expense.exchange_rate(),
        converted: group
            .expense_shares(id)
            .map_or_else(|| Money::zero(group.currency()), ExpenseShares::total),
    });
    let transfers = group.transfers().map(|(id, transfer)| EntryRow {
        name: format!("Transfer to {}", name(group, transfer.to())),
        payer: name(group, transfer.from()),
        original: transfer.amount(),
        rate: transfer.exchange_rate(),
        converted: group
            .transfer_amount(id)
            .unwrap_or_else(|| Money::zero(group.currency())),
    });
    print_table(expenses.chain(transfers), 2);
}

fn print_balances(group: &Group) {
    let summary = group.summary();
    print_table(
        group.members().map(|(id, member)| {
            let totals = summary.member(id);
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

fn expenses(group: &mut Group, ana: MemberId, bob: MemberId) -> Result<(), Box<dyn Error>> {
    section("1. Expenses in three currencies");
    let both = Split::equal([ana, bob]);
    group.add_expense(
        Expense::builder(
            "Fuel",
            Money::parse("400.00", Currency::RON)?.at(ExchangeRate::parse("0.2")?),
            Date::new(2026, 8, 1)?,
        )
        .paid_by(ana)
        .split(both.clone())
        .build()?,
    )?;
    group.add_expense(
        Expense::builder(
            "Dinner",
            Money::parse("24000.00", Currency::HUF)?.at(ExchangeRate::parse("0.0025")?),
            Date::new(2026, 8, 2)?,
        )
        .paid_by(bob)
        .split(both.clone())
        .build()?,
    )?;
    group.add_expense(
        Expense::builder(
            "Ferry",
            Money::parse("60.00", Currency::EUR)?,
            Date::new(2026, 8, 3)?,
        )
        .paid_by(bob)
        .split(both)
        .build()?,
    )?;
    print_entries(group);
    println!(
        "The rate says how many {} one unit of the entry's currency is worth: 400.00 RON at\n\
         0.2 is 80.00 EUR. The ferry was paid in the group currency, a local cost at 1.",
        group.currency()
    );
    Ok(())
}

fn repayment(group: &mut Group, ana: MemberId, bob: MemberId) -> Result<(), Box<dyn Error>> {
    section("3. A repayment in cash, in another currency");
    let cash = Transfer::builder(
        ana,
        bob,
        Money::parse("100.00", Currency::RON)?.at(ExchangeRate::parse("0.2")?),
        Date::new(2026, 8, 4)?,
    )
    .note("Cash at the border")
    .build()?;
    group.add_transfer(cash)?;
    println!(
        "Ana pays Bob 100.00 RON in cash, which at 0.2 is exactly the 20.00 EUR she owed.\n\
         Transfers move balances but are not spending, so the total stays at {}:",
        group.summary().total_spent()
    );
    print_balances(group);
    Ok(())
}

fn new_currency(group: &mut Group) -> Result<(), Box<dyn Error>> {
    section("4. Moving the books to another currency");
    println!(
        "Changing the group currency needs a rate into the new currency for every other\n\
         currency in use: 1 EUR = 5 RON and 1 HUF = 0.0125 RON. Entries in those currencies\n\
         take the new rate; entries already in RON become local costs, at 1."
    );
    group.change_currency(
        Currency::RON,
        &[
            (Currency::EUR, ExchangeRate::parse("5")?),
            (Currency::HUF, ExchangeRate::parse("0.0125")?),
        ],
    )?;
    print_entries(group);
    println!(
        "In {}, the trip cost {}, and everyone is still settled:",
        group.currency(),
        group.summary().total_spent()
    );
    print_balances(group);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut group = Group::builder("Danube road trip", Currency::EUR).build()?;
    let ana = group.add_member(Member::builder("Ana").build()?);
    let bob = group.add_member(Member::builder("Bob").build()?);

    println!("{}: expenses in several currencies", group.name());
    println!("═════════════════════════════════════════════════");
    println!(
        "Ana and Bob drive from Romania to Hungary. The group keeps its books in {}; an\n\
         expense in another currency carries the exchange rate used when it was entered,\n\
         and is converted once, rounding half away from zero to the cent.",
        group.currency()
    );
    expenses(&mut group, ana, bob)?;

    section("2. Balances");
    println!(
        "The group spent {}. Bob paid 120.00 of it but consumed half, so he gets back the\n\
         difference:",
        group.summary().total_spent()
    );
    print_balances(&group);
    assert_eq!(
        group.summary().member(bob).balance(),
        Balance::GetsBack(Money::parse("20.00", Currency::EUR)?)
    );

    repayment(&mut group, ana, bob)?;
    assert_eq!(group.summary().member(bob).balance(), Balance::Settled);

    new_currency(&mut group)?;
    // 400 RON of fuel, 300 of dinner, 300 of ferry; still settled.
    assert_eq!(
        group.summary().total_spent(),
        Money::parse("1000.00", Currency::RON)?
    );
    assert_eq!(group.summary().member(bob).balance(), Balance::Settled);
    Ok(())
}
