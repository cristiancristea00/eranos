//! A weekend trip in a few lines: three friends, two shared expenses, the
//! totals, and who pays whom.
//!
//! Run with `cargo run -p eranos --example quickstart`.
#![forbid(unsafe_code)]

use std::error::Error;

use eranos::{
    Balance, CalendarDate, CurrencyCode, Expense, ExpenseGroup, Member, MemberId, Money,
    SettlementStrategy, Split,
};
use tabled::settings::object::Columns;
use tabled::settings::{Alignment, Style};
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct ExpenseRow {
    #[tabled(rename = "Expense")]
    name: String,
    #[tabled(rename = "Paid by")]
    payers: String,
    #[tabled(rename = "Shared by")]
    participants: String,
    #[tabled(rename = "Amount")]
    amount: Money,
}

#[derive(Tabled)]
struct MemberRow {
    #[tabled(rename = "Member")]
    name: String,
    #[tabled(rename = "Paid")]
    paid: Money,
    #[tabled(rename = "Consumed")]
    consumed: Money,
    #[tabled(rename = "Balance")]
    balance: Balance,
}

#[derive(Tabled)]
struct PaymentRow {
    #[tabled(rename = "From")]
    sender: String,
    #[tabled(rename = "To")]
    receiver: String,
    #[tabled(rename = "Amount")]
    amount: Money,
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

/// The first names of `members`, joined by commas.
fn names(group: &ExpenseGroup, members: impl IntoIterator<Item = MemberId>) -> String {
    members
        .into_iter()
        .map(|member| group.member(member).map_or("?", Member::first_name))
        .collect::<Vec<_>>()
        .join(", ")
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut group = ExpenseGroup::builder("Weekend in Sinaia", CurrencyCode::EUR).build()?;
    println!("{}", group.name());
    println!("{}", "═".repeat(group.name().chars().count()));
    println!(
        "Three friends rent a cabin and buy groceries. The group keeps its books in {}, so\n\
         every total, balance, and payment below is in {}.",
        group.currency(),
        group.currency()
    );

    section("1. Members");
    let ana = group.add_member(Member::builder("Ana").build()?)?;
    let bob = group.add_member(Member::builder("Bob").build()?)?;
    let carol = group.add_member(Member::builder("Carol").build()?)?;
    println!(
        "{} people joined the group: {}.",
        group.members().len(),
        names(&group, [ana, bob, carol])
    );

    section("2. Expenses");
    println!(
        "Each expense records who paid and who shares it. Here everyone shares everything\n\
         equally."
    );
    let friday = CalendarDate::new(2026, 9, 25)?;
    let eur = |text| Money::parse(text, CurrencyCode::EUR);
    let everyone = Split::equal([ana, bob, carol]);
    let cabin = Expense::builder("Cabin", eur("240.00")?, ana, everyone.clone(), friday);
    let groceries = Expense::builder("Groceries", eur("60.00")?, bob, everyone, friday);
    group.add_expense(cabin.build()?)?;
    group.add_expense(groceries.build()?)?;
    print_table(
        group.expenses().map(|expense| ExpenseRow {
            name: expense.name().to_owned(),
            payers: names(&group, expense.payers().members()),
            participants: names(&group, expense.split().participants()),
            amount: expense.total(),
        }),
        3,
    );

    section("3. Who paid and who consumed");
    let summary = group.summary();
    println!(
        "The group spent {} in total. A member's share of an expense is what they consumed,\n\
         and their balance is what they paid minus what they consumed.",
        summary.total_spent()
    );
    print_table(
        group.members().map(|member| {
            let totals = summary.member(member.id());
            MemberRow {
                name: member.first_name().to_owned(),
                paid: totals.paid(),
                consumed: totals.share(),
                balance: totals.balance(),
            }
        }),
        1,
    );
    println!(
        "Ana paid {} but consumed only {}, so the others owe her the difference.",
        summary.member(ana).paid(),
        summary.member(ana).share()
    );

    section("4. Settling up");
    let settlement = group.settlement(SettlementStrategy::MinimumTransfers)?;
    println!(
        "The fewest payments that bring every balance to zero ({} in total):",
        settlement.len()
    );
    print_table(
        settlement.payments().iter().map(|payment| PaymentRow {
            sender: names(&group, [payment.sender()]),
            receiver: names(&group, [payment.receiver()]),
            amount: payment.amount(),
        }),
        2,
    );
    println!("Once these payments are made, nobody owes anything.");

    // Everyone's share is 100.00: Ana gets back 140.00 from Bob (40.00) and
    // Carol (100.00).
    assert_eq!(
        summary.total_spent(),
        Money::parse("300.00", CurrencyCode::EUR)?
    );
    assert_eq!(
        summary.member(ana).balance(),
        Balance::GetsBack(Money::parse("140.00", CurrencyCode::EUR)?)
    );
    assert_eq!(settlement.len(), 2);
    Ok(())
}
