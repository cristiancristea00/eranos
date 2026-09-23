//! An itemized restaurant bill: every line is split among the people who
//! had it, quantities multiply the unit price, and the tip is shared either
//! in proportion to what each person ordered or equally.
//!
//! Run with `cargo run -p eranos --example restaurant_bill`.

use std::error::Error;

use eranos::{
    Currency, Date, Expense, ExpenseShares, Group, Item, Member, MemberId, MemberShare, Money,
    Parts, Percentage, Split, Tip, TipDistribution,
};
use tabled::settings::object::Columns;
use tabled::settings::{Alignment, Style};
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct LineRow {
    #[tabled(rename = "Item")]
    name: String,
    #[tabled(rename = "Shared by")]
    participants: String,
    #[tabled(rename = "Quantity")]
    quantity: u32,
    #[tabled(rename = "Unit price")]
    unit_price: Money,
    #[tabled(rename = "Line total")]
    line_total: Money,
}

#[derive(Tabled)]
struct TipRow {
    #[tabled(rename = "Member")]
    name: String,
    #[tabled(rename = "Ordered")]
    subtotal: Money,
    #[tabled(rename = "Tip by consumption")]
    proportional_tip: Money,
    #[tabled(rename = "Pays")]
    proportional_total: Money,
    #[tabled(rename = "Tip shared equally")]
    equal_tip: Money,
    #[tabled(rename = "Pays")]
    equal_total: Money,
}

fn eur(text: &str) -> Result<Money, Box<dyn Error>> {
    Ok(Money::parse(text, Currency::EUR)?)
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

fn receipt(ana: MemberId, bob: MemberId, carol: MemberId) -> Result<Vec<Item>, Box<dyn Error>> {
    Ok(vec![
        Item::builder("Ciorbă", eur("8.00")?)
            .shared_by([ana])
            .build()?,
        Item::builder("Sarmale", eur("14.00")?)
            .quantity(2)
            .shared_by([bob, carol])
            .build()?,
        Item::builder("Papanași", eur("9.00")?)
            .split(Split::parts([
                (ana, Parts::ONE),
                (carol, Parts::from_whole(2)?),
            ]))
            .build()?,
        Item::builder("Lemonade", eur("3.00")?)
            .quantity(3)
            .shared_by([ana, bob, carol])
            .build()?,
    ])
}

/// How `expense` would divide if it were added to `group`, computed on a
/// copy so the group itself is unchanged.
fn preview(group: &Group, expense: Expense) -> Result<ExpenseShares, Box<dyn Error>> {
    let mut copy = group.clone();
    let id = copy.add_expense(expense)?;
    Ok(copy
        .expense_shares(id)
        .ok_or("the expense was just added")?
        .clone())
}

fn consumed(shares: &ExpenseShares, member: MemberId) -> Result<Money, Box<dyn Error>> {
    Ok(shares
        .member(member)
        .map(MemberShare::share)
        .ok_or("every member ordered")?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut group = Group::builder("Dinner at Caru' cu Bere", Currency::EUR).build()?;
    let ana = group.add_member(Member::builder("Ana").build()?);
    let bob = group.add_member(Member::builder("Bob").build()?);
    let carol = group.add_member(Member::builder("Carol").build()?);
    let day = Date::new(2026, 9, 23)?;

    println!("{}: an itemized bill", group.name());
    println!("{}", "═".repeat(group.name().chars().count() + 19));
    println!(
        "Three friends share a Romanian dinner. Instead of splitting the whole bill evenly,\n\
         every line of the receipt is split among the people who had it."
    );

    section("1. The receipt");
    let lines = receipt(ana, bob, carol)?;
    print_table(
        lines.iter().map(|item| LineRow {
            name: item.name().to_owned(),
            participants: names(&group, item.split().members()),
            quantity: item.quantity(),
            unit_price: item.unit_price(),
            line_total: item.line_total(),
        }),
        2,
    );
    println!(
        "The papanași are split by parts: Carol ate two of the three. The line totals add up\n\
         to 54.00 EUR, and the expense's amount must match them exactly."
    );

    section("2. Who ordered what, and the tip");
    let bill = |distribution| -> Result<Expense, Box<dyn Error>> {
        Ok(Expense::builder("Dinner", eur("54.00")?, day)
            .paid_by(ana)
            .items(receipt(ana, bob, carol)?)
            .tip(Tip::percentage(Percentage::parse("10")?).distributed(distribution))
            .build()?)
    };
    let untipped = Expense::builder("Dinner", eur("54.00")?, day)
        .paid_by(ana)
        .items(receipt(ana, bob, carol)?)
        .build()?;
    let ordered = preview(&group, untipped)?;
    let proportional = group.add_expense(bill(TipDistribution::Proportional)?)?;
    let by_consumption = group
        .expense_shares(proportional)
        .ok_or("the bill was added")?;
    let even = preview(&group, bill(TipDistribution::Equal)?)?;
    println!(
        "Ana paid the bill plus a 10 % tip ({}). A tip can be shared in proportion to what\n\
         each person ordered, or equally; both are shown:",
        by_consumption.tip()
    );
    let mut rows = Vec::new();
    for (member, first_name) in [(ana, "Ana"), (bob, "Bob"), (carol, "Carol")] {
        let subtotal = consumed(&ordered, member)?;
        let proportional_total = consumed(by_consumption, member)?;
        let equal_total = consumed(&even, member)?;
        rows.push(TipRow {
            name: first_name.to_owned(),
            subtotal,
            proportional_tip: proportional_total.checked_sub(subtotal)?,
            proportional_total,
            equal_tip: equal_total.checked_sub(subtotal)?,
            equal_total,
        });
    }
    print_table(rows, 1);
    println!(
        "By consumption, Carol, who ordered the most, adds the largest part of the tip;\n\
         shared equally, everyone adds the same 1.80."
    );

    section("3. A total that does not match is refused");
    let mismatched = Expense::builder("Dinner", eur("55.00")?, day)
        .paid_by(ana)
        .items(receipt(ana, bob, carol)?)
        .build();
    println!(
        "An amount of 55.00 EUR with these lines is refused when the expense is built:\n  \
         error: {}",
        mismatched.err().ok_or("the mismatch must be refused")?
    );

    // Subtotals: Ana 8 + 3 + 3 = 14.00, Bob 14 + 3 = 17.00, Carol
    // 14 + 6 + 3 = 23.00. A proportional tip adds 10 % to each; an equal one
    // 1.80 to everyone.
    assert_eq!(consumed(by_consumption, ana)?, eur("15.40")?);
    assert_eq!(consumed(by_consumption, bob)?, eur("18.70")?);
    assert_eq!(consumed(by_consumption, carol)?, eur("25.30")?);
    assert_eq!(consumed(&even, carol)?, eur("24.80")?);
    assert_eq!(even.total(), eur("59.40")?);
    Ok(())
}
