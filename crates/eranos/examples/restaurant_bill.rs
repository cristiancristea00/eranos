//! An itemised restaurant bill: every line is split among the people who
//! had it, quantities multiply the unit price, and the tip is shared either
//! in proportion to what each person ordered or equally; then a line
//! weighed on a scale, and discounts off the line and off each unit.
//!
//! Run with `cargo run -p eranos --example restaurant_bill`.
#![forbid(unsafe_code)]

use std::error::Error;

use eranos::{
    CalendarDate, CurrencyCode, Discount, DiscountScope, Expense, ExpenseGroup, ExpenseItem,
    ExpenseShares, Member, MemberId, MemberParts, MemberShare, Money, Parts, Percentage, Quantity,
    Split, TipDistribution, TipRule,
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
    quantity: Quantity,
    #[tabled(rename = "Unit price")]
    unit_price: Money,
    #[tabled(rename = "Line total")]
    line_total: Money,
}

#[derive(Tabled)]
struct DiscountRow {
    #[tabled(rename = "Item")]
    name: String,
    #[tabled(rename = "Quantity")]
    quantity: Quantity,
    #[tabled(rename = "Unit price")]
    unit_price: Money,
    #[tabled(rename = "Before discount")]
    line_total_before_discount: Money,
    #[tabled(rename = "Discount")]
    discount_amount: Money,
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
    Ok(Money::parse(text, CurrencyCode::EUR)?)
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

fn receipt(
    ana: MemberId,
    bob: MemberId,
    carol: MemberId,
) -> Result<Vec<ExpenseItem>, Box<dyn Error>> {
    Ok(vec![
        ExpenseItem::builder("Ciorbă", eur("8.00")?, [ana]).build()?,
        ExpenseItem::builder("Sarmale", eur("14.00")?, [bob, carol])
            .quantity(Quantity::from_whole(2)?)
            .build()?,
        ExpenseItem::builder(
            "Papanași",
            eur("9.00")?,
            Split::parts([
                MemberParts::new(ana, Parts::ONE),
                MemberParts::new(carol, Parts::from_whole(2)?),
            ]),
        )
        .build()?,
        ExpenseItem::builder("Lemonade", eur("3.00")?, [ana, bob, carol])
            .quantity(Quantity::from_whole(3)?)
            .build()?,
    ])
}

/// A second round: telemea weighed on a scale with 10 % off the line,
/// three lemonades with 1.00 off each, and papanași on the house.
fn second_round(
    ana: MemberId,
    bob: MemberId,
    carol: MemberId,
) -> Result<Vec<ExpenseItem>, Box<dyn Error>> {
    // The scale reports the weight as a double.
    let weight = Quantity::from_f64(0.35)?;
    Ok(vec![
        ExpenseItem::builder("Telemea", eur("42.00")?, [ana, bob])
            .quantity(weight)
            .discount(Discount::percentage(Percentage::parse("10")?))
            .build()?,
        ExpenseItem::builder("Lemonade", eur("3.00")?, [ana, bob, carol])
            .quantity(Quantity::from_whole(3)?)
            .discount(Discount::fixed(eur("1.00")?).scoped(DiscountScope::Unit))
            .build()?,
        ExpenseItem::builder("Papanași", eur("9.00")?, [carol])
            .discount(Discount::percentage(Percentage::HUNDRED))
            .build()?,
    ])
}

/// How `expense` would divide if it were added to `group`, computed on a
/// copy so the group itself is unchanged.
fn preview(group: &ExpenseGroup, expense: Expense) -> Result<ExpenseShares, Box<dyn Error>> {
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

/// Section 4: the second round's lines, each with its total before the
/// discount, the discount, and its line total, and a discount refused for
/// being larger than its line.
fn weighed_lines_and_discounts(
    ana: MemberId,
    bob: MemberId,
    carol: MemberId,
) -> Result<(), Box<dyn Error>> {
    section("4. Weighed lines and discounts");
    let round = second_round(ana, bob, carol)?;
    print_table(
        round.iter().map(|item| DiscountRow {
            name: item.name().to_owned(),
            quantity: item.quantity(),
            unit_price: item.unit_price(),
            line_total_before_discount: item.line_total_before_discount(),
            discount_amount: item.discount_amount(),
            line_total: item.line_total(),
        }),
        1,
    );
    let [telemea, lemonade, papanasi] = round.as_slice() else {
        return Err("the second round has three lines".into());
    };
    let (nine, ten) = (eur("9.00")?, eur("10.00")?);
    let refused = ExpenseItem::builder("Papanași", nine, [carol])
        .discount(Discount::fixed(ten))
        .build();
    println!(
        "The scale reports {weight} kg as a double just below it; the quantity reads its shortest\n\
         form, {weight}, so the telemea costs exactly {before} before its discount. Taking {ten}\n\
         off a {nine} line is refused when the item is built:\n  \
         error: {error}",
        weight = telemea.quantity(),
        before = telemea.line_total_before_discount(),
        error = refused.err().ok_or("the discount must be refused")?
    );

    // 10 % of 14.70 is 1.47; 1.00 off each of three lemonades is 3.00; the
    // papanași on the house leave a free line.
    assert_eq!(telemea.line_total_before_discount(), eur("14.70")?);
    assert_eq!(telemea.discount_amount(), eur("1.47")?);
    assert_eq!(telemea.line_total(), eur("13.23")?);
    assert_eq!(lemonade.discount_amount(), eur("3.00")?);
    assert_eq!(lemonade.line_total(), eur("6.00")?);
    assert_eq!(papanasi.discount_amount(), eur("9.00")?);
    assert_eq!(papanasi.line_total(), eur("0.00")?);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut group = ExpenseGroup::builder("Dinner at Caru' cu Bere", CurrencyCode::EUR).build()?;
    let ana = group.add_member(Member::builder("Ana").build()?)?;
    let bob = group.add_member(Member::builder("Bob").build()?)?;
    let carol = group.add_member(Member::builder("Carol").build()?)?;
    let day = CalendarDate::new(2026, 9, 23)?;

    let title = format!("{}: an itemised bill", group.name());
    println!("{title}\n{}", "═".repeat(title.chars().count()));
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
        Ok(
            Expense::builder("Dinner", eur("54.00")?, ana, receipt(ana, bob, carol)?, day)
                .tip(TipRule::percentage(Percentage::parse("10")?).distributed(distribution))
                .build()?,
        )
    };
    let untipped =
        Expense::builder("Dinner", eur("54.00")?, ana, receipt(ana, bob, carol)?, day).build()?;
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
    for member in [ana, bob, carol] {
        let subtotal = consumed(&ordered, member)?;
        let proportional_total = consumed(by_consumption, member)?;
        let equal_total = consumed(&even, member)?;
        rows.push(TipRow {
            name: names(&group, [member]),
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
    let mismatched =
        Expense::builder("Dinner", eur("55.00")?, ana, receipt(ana, bob, carol)?, day).build();
    println!(
        "An amount of 55.00 EUR with these lines is refused when the expense is built:\n  \
         error: {}",
        mismatched.err().ok_or("the mismatch must be refused")?
    );

    weighed_lines_and_discounts(ana, bob, carol)?;

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
