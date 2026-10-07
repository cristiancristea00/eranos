//! Settling up with each strategy on a group where they differ, then
//! recording the chosen plan as transfers until everyone is settled.
//!
//! Run with `cargo run -p eranos --example settling_up`.
#![forbid(unsafe_code)]

use std::error::Error;

use eranos::{
    Balance, CalendarDate, Currency, Expense, ExpenseGroup, Member, MemberId, Money,
    SettlementPlan, SettlementStrategy, Split, Transfer,
};
use tabled::settings::object::Columns;
use tabled::settings::{Alignment, Style};
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct ExpenseRow {
    #[tabled(rename = "Expense")]
    name: String,
    #[tabled(rename = "Paid by")]
    payer: String,
    #[tabled(rename = "Shared by")]
    participants: String,
    #[tabled(rename = "Amount")]
    amount: Money,
}

#[derive(Tabled)]
struct BalanceRow {
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
    from: String,
    #[tabled(rename = "To")]
    to: String,
    #[tabled(rename = "Amount")]
    amount: Money,
}

#[derive(Tabled)]
struct StrategyRow {
    #[tabled(rename = "Strategy")]
    strategy: &'static str,
    #[tabled(rename = "Guarantee")]
    guarantee: &'static str,
    #[tabled(rename = "Cost")]
    cost: &'static str,
    #[tabled(rename = "Payments here")]
    payments: usize,
}

/// The number of payments of each strategy's plan.
#[derive(Debug, PartialEq, Eq)]
struct Counts {
    adaptive: usize,
    minimum: usize,
    greedy: usize,
    direct: usize,
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

/// Records an expense of `amount` EUR paid by `payer` and split equally.
fn spend(
    group: &mut ExpenseGroup,
    name: &str,
    amount: &str,
    payer: MemberId,
    participants: &[MemberId],
) -> Result<(), Box<dyn Error>> {
    let expense = Expense::builder(
        name,
        Money::parse(amount, Currency::EUR)?,
        payer,
        Split::equal(participants.iter().copied()),
        CalendarDate::new(2026, 9, 20)?,
    )
    .build()?;
    group.add_expense(expense)?;
    Ok(())
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
                balance: totals.balance(),
            }
        }),
        1,
    );
}

fn print_payments(group: &ExpenseGroup, settlement: &SettlementPlan) {
    print_table(
        settlement.payments().iter().map(|payment| PaymentRow {
            from: names(group, [payment.from()]),
            to: names(group, [payment.to()]),
            amount: payment.amount(),
        }),
        2,
    );
}

/// Prints each strategy's plan with what it promises; returns the number of
/// payments of each.
fn strategies(group: &ExpenseGroup) -> Result<Counts, Box<dyn Error>> {
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

    section("6. Adaptive");
    let adaptive = group.settlement(SettlementStrategy::Adaptive)?;
    println!(
        "The fewest payments while they are quick to find, greedy's beyond: up to 20 open\n\
         balances the minimum solver settles them, past that greedy does, and the plan\n\
         names the strategy whose plan it is. Six balances are well within the limit, so\n\
         the plan is the one {} gives, the same {} payments.",
        strategy_name(adaptive.strategy()),
        adaptive.len()
    );

    section("7. The strategies side by side");
    print_table(
        [
            StrategyRow {
                strategy: "MinimumTransfers",
                guarantee: "the fewest payments",
                cost: "O(2ⁿ·n) for n open balances",
                payments: minimum.len(),
            },
            StrategyRow {
                strategy: "Greedy",
                guarantee: "at most n − 1 payments",
                cost: "O(n log n)",
                payments: greedy.len(),
            },
            StrategyRow {
                strategy: "Direct",
                guarantee: "one payment per pair in debt",
                cost: "O(pairs)",
                payments: direct.len(),
            },
            StrategyRow {
                strategy: "Adaptive",
                guarantee: "the fewest up to 20 balances",
                cost: "as MinimumTransfers, then Greedy",
                payments: adaptive.len(),
            },
        ],
        3,
    );
    Ok(Counts {
        adaptive: adaptive.len(),
        minimum: minimum.len(),
        greedy: greedy.len(),
        direct: direct.len(),
    })
}

/// Settles a group of 21 friends, more open balances than the adaptive
/// strategy settles exactly, and prints which strategy's plan it took;
/// returns the number of payments.
fn festival() -> Result<usize, Box<dyn Error>> {
    section("8. A larger group");
    let mut group = ExpenseGroup::builder("Festival", Currency::EUR).build()?;
    let mut friends = Vec::new();
    for number in 1..=21 {
        friends.push(group.add_member(Member::builder(format!("Friend {number}")).build()?)?);
    }
    // Friend i buys a round of 0.21·i² EUR for all 21, so each share is i²
    // cents with nothing to round, and friend i's balance is 21·i² − 3311
    // cents: never zero, and never the opposite of another's.
    for (number, &payer) in (1_i64..).zip(&friends) {
        let expense = Expense::builder(
            "Round",
            Money::from_minor_units(21 * number * number, Currency::EUR),
            payer,
            Split::equal(friends.iter().copied()),
            CalendarDate::new(2026, 9, 18)?,
        )
        .build()?;
        group.add_expense(expense)?;
    }
    let open = group
        .summary()
        .members()
        .filter(|member| !member.net().is_zero())
        .count();
    let adaptive = group.settlement(SettlementStrategy::Adaptive)?;
    let greedy = group.settlement(SettlementStrategy::Greedy)?;
    println!(
        "Twenty-one friends at a festival each bought one round for everyone, each round\n\
         at another price, so all {open} end with a balance and no two of them cancel out.\n\
         That is more than the 20 the adaptive strategy settles exactly, so it takes the\n\
         plan {} gives and says so: the same {} payments.",
        strategy_name(adaptive.strategy()),
        adaptive.len()
    );
    assert_eq!(open, 21);
    assert_eq!(adaptive.strategy(), SettlementStrategy::Greedy);
    assert_eq!(adaptive.payments(), greedy.payments());
    Ok(adaptive.len())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut group = ExpenseGroup::builder("Ski week", Currency::EUR).build()?;
    let mut add = |name| -> Result<MemberId, Box<dyn Error>> {
        Ok(group.add_member(Member::builder(name).build()?)?)
    };
    let (ana, bob, carol) = (add("Ana")?, add("Bob")?, add("Carol")?);
    let (dan, eva, filip) = (add("Dan")?, add("Eva")?, add("Filip")?);

    println!("Ski week: four ways to settle up");
    println!("════════════════════════════════");
    println!(
        "Six friends spent a week in the mountains. The library can turn their balances\n\
         into payments in four ways, and the app picks one."
    );

    section("1. What was spent");
    spend(&mut group, "Cabin", "12.00", dan, &[dan, eva, filip])?;
    spend(&mut group, "Lift passes", "9.00", bob, &[bob, carol, filip])?;
    spend(&mut group, "Groceries", "6.00", carol, &[bob, dan])?;
    spend(&mut group, "Coffee", "2.00", ana, &[dan, eva])?;
    spend(&mut group, "Snacks", "1.00", filip, &[ana])?;
    print_table(
        group.expenses().map(|expense| ExpenseRow {
            name: expense.name().to_owned(),
            payer: names(&group, expense.payers().members()),
            participants: names(&group, expense.split().participants()),
            amount: expense.total(),
        }),
        3,
    );

    section("2. Balances");
    println!("Four members are owed money and two owe it:");
    print_balances(&group);

    let counts = strategies(&group)?;
    let festival_payments = festival()?;

    section("9. Settling up for real");
    let plan = group
        .settlement(SettlementStrategy::MinimumTransfers)?
        .clone();
    for payment in plan.payments() {
        group.add_transfer(Transfer::from_payment(
            payment,
            CalendarDate::new(2026, 9, 27)?,
        ))?;
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
    // payments suffice, and the adaptive plan is that one; greedy pays the
    // largest amounts first and needs five; the unsimplified debts between
    // pairs need seven.
    assert_eq!(
        counts,
        Counts {
            adaptive: 4,
            minimum: 4,
            greedy: 5,
            direct: 7,
        }
    );
    // Greedy settles the festival's 21 balances in 19 payments.
    assert_eq!(festival_payments, 19);
    for member in [ana, bob, carol, dan, eva, filip] {
        assert_eq!(group.summary().member(member).balance(), Balance::Settled);
    }
    Ok(())
}
