//! Settling up with each strategy on a group where they differ, then
//! recording the chosen plan as transfers until everyone is settled.
//!
//! Run with `cargo run -p eranos --example settling_up`.

use std::error::Error;

use eranos::{
    Balance, CalendarDate, Currency, Expense, Group, Member, MemberId, Money, Settlement,
    SettlementStrategy, Split,
};
use tabled::settings::object::Columns;
use tabled::settings::{Alignment, Style};
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct ExpenseRow {
    #[tabled(rename = "Amount", order = 3)]
    amount: Money,
    #[tabled(rename = "Expense", order = 0)]
    name: String,
    #[tabled(rename = "Shared by", order = 2)]
    participants: String,
    #[tabled(rename = "Paid by", order = 1)]
    payer: String,
}

#[derive(Tabled)]
struct BalanceRow {
    #[tabled(rename = "Balance", order = 3)]
    balance: Balance,
    #[tabled(rename = "Consumed", order = 2)]
    consumed: Money,
    #[tabled(rename = "Member", order = 0)]
    name: String,
    #[tabled(rename = "Paid", order = 1)]
    paid: Money,
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
struct StrategyRow {
    #[tabled(rename = "Cost", order = 2)]
    cost: &'static str,
    #[tabled(rename = "Guarantee", order = 1)]
    guarantee: &'static str,
    #[tabled(rename = "Payments here", order = 3)]
    payments: usize,
    #[tabled(rename = "Strategy", order = 0)]
    strategy: &'static str,
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
fn names(group: &Group, members: impl IntoIterator<Item = MemberId>) -> String {
    members
        .into_iter()
        .map(|member| group.member(member).map_or("?", Member::first_name))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Records an expense of `amount` EUR paid by `payer` and split equally.
fn spend(
    group: &mut Group,
    name: &str,
    amount: &str,
    payer: MemberId,
    participants: &[MemberId],
) -> Result<(), Box<dyn Error>> {
    let expense = Expense::builder(
        Money::parse(amount, Currency::EUR)?,
        CalendarDate::new(20, 9, 2026)?,
        name,
        payer,
        Split::equal(participants.iter().copied()),
    )
    .build()?;
    group.add_expense(expense)?;
    Ok(())
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
            }
        }),
        1,
    );
}

fn print_payments(group: &Group, settlement: &Settlement) {
    print_table(
        settlement.payments().iter().map(|payment| PaymentRow {
            amount: payment.amount(),
            from: names(group, [payment.from()]),
            to: names(group, [payment.to()]),
        }),
        2,
    );
}

/// Prints each strategy's plan with what it promises; returns the number of
/// payments of each, in the order minimum, greedy, direct.
fn strategies(group: &Group) -> Result<[usize; 3], Box<dyn Error>> {
    section("3. Minimum transfers");
    let minimum = group.settlement(SettlementStrategy::MinimumTransfers)?;
    println!(
        "The fewest payments possible. Finding them is NP-hard; the solver does it exactly by\n\
         splitting the balances into as many groups that sum to zero as it can: here\n\
         {{Ana, Dan, Eva}} and {{Bob, Carol, Filip}}, each settled with one payment fewer than\n\
         its size. {} payments:",
        minimum.len()
    );
    print_payments(group, minimum);

    section("4. Greedy");
    let greedy = group.settlement(SettlementStrategy::Greedy)?;
    println!(
        "The largest debtor pays the largest creditor, again and again: fast, at most one\n\
         payment fewer than the members with a balance, but not always the minimum. {}\n\
         payments here:",
        greedy.len()
    );
    print_payments(group, greedy);

    section("5. Direct");
    let direct = group.settlement(SettlementStrategy::Direct)?;
    println!(
        "No simplification: everyone pays back the people who paid for them, with the debts\n\
         between each pair of members netted. The most payments ({}), but each one traces\n\
         back to shared expenses:",
        direct.len()
    );
    print_payments(group, direct);

    section("6. The strategies side by side");
    print_table(
        [
            StrategyRow {
                cost: "O(2ⁿ·n) for n open balances",
                guarantee: "the fewest payments",
                payments: minimum.len(),
                strategy: "MinimumTransfers",
            },
            StrategyRow {
                cost: "O(n log n)",
                guarantee: "at most n − 1 payments",
                payments: greedy.len(),
                strategy: "Greedy",
            },
            StrategyRow {
                cost: "O(pairs)",
                guarantee: "one payment per pair in debt",
                payments: direct.len(),
                strategy: "Direct",
            },
        ],
        3,
    );
    Ok([minimum.len(), greedy.len(), direct.len()])
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut group = Group::builder(Currency::EUR, "Ski week").build()?;
    let mut add = |name| -> Result<MemberId, Box<dyn Error>> {
        Ok(group.add_member(Member::builder(name).build()?)?)
    };
    let (ana, bob, carol) = (add("Ana")?, add("Bob")?, add("Carol")?);
    let (dan, eva, filip) = (add("Dan")?, add("Eva")?, add("Filip")?);

    println!("Ski week: three ways to settle up");
    println!("═════════════════════════════════");
    println!(
        "Six friends spent a week in the mountains. The library can turn their balances\n\
         into payments in three ways; the app picks one, and they differ here."
    );

    section("1. What was spent");
    spend(&mut group, "Cabin", "12.00", dan, &[dan, eva, filip])?;
    spend(&mut group, "Lift passes", "9.00", bob, &[bob, carol, filip])?;
    spend(&mut group, "Groceries", "6.00", carol, &[bob, dan])?;
    spend(&mut group, "Coffee", "2.00", ana, &[dan, eva])?;
    spend(&mut group, "Snacks", "1.00", filip, &[ana])?;
    print_table(
        group.expenses().map(|expense| ExpenseRow {
            amount: expense.total(),
            name: expense.name().to_owned(),
            participants: names(&group, expense.split().participants()),
            payer: names(&group, expense.payers().members()),
        }),
        3,
    );

    section("2. Balances");
    println!("Four members are owed money and two owe it:");
    print_balances(&group);

    let payments = strategies(&group)?;

    section("7. Settling up for real");
    let plan = group
        .settlement(SettlementStrategy::MinimumTransfers)?
        .clone();
    for payment in plan.payments() {
        group.add_transfer(payment.to_transfer(CalendarDate::new(27, 9, 2026)?))?;
    }
    println!(
        "Each payment of the minimum plan, once made, is recorded as a transfer. Afterwards\n\
         every balance is zero and a new settlement has nothing left to pay:"
    );
    print_balances(&group);
    println!(
        "Payments still needed: {}.",
        group
            .settlement(SettlementStrategy::MinimumTransfers)?
            .len()
    );

    // {Ana, Dan, Eva} and {Bob, Carol, Filip} each sum to zero, so four
    // payments suffice; greedy pays the largest amounts first and needs
    // five; the unsimplified debts between pairs need seven.
    assert_eq!(payments, [4, 5, 7]);
    for member in [ana, bob, carol, dan, eva, filip] {
        assert_eq!(group.summary().member(member).balance(), Balance::Settled);
    }
    Ok(())
}
