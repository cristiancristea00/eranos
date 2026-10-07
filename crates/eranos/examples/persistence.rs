//! Saving and loading a group as JSON (feature `serde`). The saved form
//! holds the entries only; loading replays them through the group's checks
//! and recomputes every total.
//!
//! Run with `cargo run -p eranos --features serde --example persistence`.
#![forbid(unsafe_code)]

use std::error::Error;

use eranos::{
    Balance, CalendarDate, Currency, Expense, ExpenseGroup, Member, Money, Percentage,
    SavedExpenseGroup, SettlementStrategy, Split, Tip,
};
use tabled::settings::object::Columns;
use tabled::settings::{Alignment, Style};
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct ComparisonRow {
    #[tabled(rename = "Member")]
    name: String,
    #[tabled(rename = "Balance before saving")]
    before: Balance,
    #[tabled(rename = "Balance after loading")]
    after: Balance,
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

fn main() -> Result<(), Box<dyn Error>> {
    let mut group = ExpenseGroup::builder("Road trip", Currency::EUR).build()?;
    let ana = group.add_member(Member::builder("Ana").build()?)?;
    let bob = group.add_member(Member::builder("Bob").build()?)?;
    group.add_expense(
        Expense::builder(
            "Fuel",
            Money::parse("70.00", Currency::EUR)?,
            ana,
            Split::equal([ana, bob]),
            CalendarDate::new(2026, 9, 23)?,
        )
        .build()?,
    )?;
    group.add_expense(
        Expense::builder(
            "Lunch",
            Money::parse("30.00", Currency::EUR)?,
            bob,
            Split::equal([ana, bob]),
            CalendarDate::new(2026, 9, 23)?,
        )
        .tip(Tip::percentage(Percentage::parse("10")?))
        .build()?,
    )?;

    println!("Road trip: saving and loading a group");
    println!("═════════════════════════════════════");
    println!(
        "With the `serde` feature a group serialises, and so does every type it stores.\n\
         It saves only what was entered: its details, members, categories, expenses,\n\
         and transfers. The totals, balances, and settlements are recomputed when it is\n\
         loaded."
    );

    section("1. The saved form");
    // Account details would be in clear text: store this in the platform's
    // secure storage (Keychain, Keystore, or an encrypted database).
    let saved = serde_json::to_string_pretty(&group)?;
    println!("{saved}");
    println!(
        "Amounts are minor units with their currency, dates are YYYY-MM-DD, and the\n\
         `format` number lets a future release refuse a file it cannot read instead of\n\
         dropping fields."
    );

    section("2. Loading it back");
    let loaded: ExpenseGroup = serde_json::from_str(&saved)?;
    println!(
        "Loading replays every entry through the same checks as adding it, then rebuilds\n\
         the totals. The loaded group equals the saved one, and so do its balances:"
    );
    let rows = [ana, bob].map(|member| ComparisonRow {
        name: group
            .member(member)
            .map_or("?", Member::first_name)
            .to_owned(),
        before: group.summary().member(member).balance(),
        after: loaded.summary().member(member).balance(),
    });
    print_table(rows, 1);
    let plan = loaded.settlement(SettlementStrategy::Greedy)?;
    println!(
        "Payments needed to settle the loaded group: {}.",
        plan.len()
    );

    section("3. A corrupted file is refused");
    println!(
        "Here Ana's identifier was edited in her member entry, so the fuel's payer is an\n\
         identifier no member has. Reading the file as a SavedExpenseGroup and loading it\n\
         with into_expense_group replays every entry through the same checks as adding it,\n\
         and refuses the file instead of producing balances that do not add up:"
    );
    // Ana's own entry is the first to name her: the members come before the
    // expenses in the saved form, so the fuel she paid is left naming an
    // identifier no member has.
    let corrupted = saved.replacen(&ana.to_string(), "0199a000-0000-7000-8000-00000000ffff", 1);
    let refused = serde_json::from_str::<SavedExpenseGroup>(&corrupted)?.into_expense_group();
    println!(
        "  error: {}",
        refused.err().ok_or("the file must be refused")?
    );

    assert_eq!(loaded, group);
    assert_eq!(loaded.summary(), group.summary());
    assert_eq!(plan.len(), 1);
    Ok(())
}
