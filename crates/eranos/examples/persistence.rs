//! Saving and loading a group as JSON (feature `serde`). The saved form
//! holds the entries only; loading replays them through the group's checks
//! and recomputes every total.
//!
//! Run with `cargo run -p eranos --features serde --example persistence`.
#![forbid(unsafe_code)]

use std::error::Error;

use eranos::{
    Balance, CalendarDate, Currency, Expense, Group, Member, Money, Percentage, SavedGroup,
    SettlementStrategy, Split, Tip,
};
use tabled::settings::object::Columns;
use tabled::settings::{Alignment, Style};
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct ComparisonRow {
    #[tabled(rename = "Balance after loading", order = 2)]
    after: Balance,
    #[tabled(rename = "Balance before saving", order = 1)]
    before: Balance,
    #[tabled(rename = "Member", order = 0)]
    name: String,
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
    let mut group = Group::builder(Currency::EUR, "Road trip").build()?;
    let ana = group.add_member(Member::builder("Ana").build()?)?;
    let bob = group.add_member(Member::builder("Bob").build()?)?;
    group.add_expense(
        Expense::builder(
            Money::parse("70.00", Currency::EUR)?,
            CalendarDate::new(23, 9, 2026)?,
            "Fuel",
            ana,
            Split::equal([ana, bob]),
        )
        .build()?,
    )?;
    group.add_expense(
        Expense::builder(
            Money::parse("30.00", Currency::EUR)?,
            CalendarDate::new(23, 9, 2026)?,
            "Lunch",
            bob,
            Split::equal([ana, bob]),
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
    let loaded: Group = serde_json::from_str(&saved)?;
    println!(
        "Loading replays every entry through the same checks as adding it, then rebuilds\n\
         the totals. The loaded group equals the saved one, and so do its balances:"
    );
    let rows = [ana, bob].map(|member| ComparisonRow {
        after: loaded.summary().member(member).balance(),
        before: group.summary().member(member).balance(),
        name: group
            .member(member)
            .map_or("?", Member::first_name)
            .to_owned(),
    });
    print_table(rows, 1);
    let plan = loaded.settlement(SettlementStrategy::Greedy)?;
    println!(
        "Payments needed to settle the loaded group: {}.",
        plan.len()
    );

    section("3. A corrupted file is refused");
    println!(
        "Here the fuel's payer was edited to an identifier no member has. Reading the file\n\
         as a SavedGroup and loading it with into_group replays every entry through the\n\
         same checks as adding it, and refuses the file instead of producing balances that\n\
         do not add up:"
    );
    // The fuel is the first entry that names Ana: the expenses come before
    // the members in the saved form.
    let corrupted = saved.replacen(&ana.to_string(), "0199a000-0000-7000-8000-00000000ffff", 1);
    let refused = serde_json::from_str::<SavedGroup>(&corrupted)?.into_group();
    println!(
        "  error: {}",
        refused.err().ok_or("the file must be refused")?
    );

    assert_eq!(loaded, group);
    assert_eq!(loaded.summary(), group.summary());
    assert_eq!(plan.len(), 1);
    Ok(())
}
