//! Every way to divide an expense: equally, by percentage, by parts
//! (including half a part for a child), by exact amounts, and with several
//! payers.
//!
//! Run with `cargo run -p eranos --example split_methods`.
#![forbid(unsafe_code)]

use std::error::Error;

use eranos::{
    CalendarDate, Currency, Expense, ExpenseId, Group, Member, MemberAmount, MemberId, MemberParts,
    MemberPercentage, MemberShare, Money, Parts, Percentage, Split,
};
use tabled::settings::object::Columns;
use tabled::settings::{Alignment, Style};
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct ShareRow {
    #[tabled(rename = "Member")]
    name: String,
    #[tabled(rename = "Rule")]
    rule: String,
    #[tabled(rename = "Paid")]
    paid: Money,
    #[tabled(rename = "Consumed")]
    consumed: Money,
}

#[derive(Tabled)]
struct TotalRow {
    #[tabled(rename = "Member")]
    name: String,
    #[tabled(rename = "Paid")]
    paid: Money,
    #[tabled(rename = "Consumed")]
    consumed: Money,
    #[tabled(rename = "Balance")]
    balance: String,
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

fn name(group: &Group, member: MemberId) -> String {
    group
        .member(member)
        .map_or("?", Member::first_name)
        .to_owned()
}

/// The rule by which a member shares an expense, as the report names it.
struct Rule {
    member: MemberId,
    text: &'static str,
}

/// Prints how one expense divides: every involved member's rule, what they
/// paid, and what they consumed.
fn report(group: &Group, id: ExpenseId, rules: &[Rule]) {
    let (Some(expense), Some(shares)) = (group.expense(id), group.expense_shares(id)) else {
        return;
    };
    println!("{}, {}:", expense.name(), shares.total());
    print_table(
        shares.members().iter().map(|share: &MemberShare| ShareRow {
            name: name(group, share.member()),
            rule: rules
                .iter()
                .find(|rule| rule.member == share.member())
                .map_or("(payer only)", |rule| rule.text)
                .to_owned(),
            paid: share.paid(),
            consumed: share.share(),
        }),
        2,
    );
}

/// The family and the day every expense happened.
struct Family {
    ana: MemberId,
    bob: MemberId,
    mia: MemberId,
    day: CalendarDate,
}

fn equally(group: &mut Group, family: &Family) -> Result<ExpenseId, Box<dyn Error>> {
    let Family { ana, bob, mia, day } = *family;
    section("1. Equally");
    println!(
        "Everyone pays the same. 10.00 does not divide by three, so one member pays the\n\
         leftover cent; which member it is rotates from expense to expense, so over a trip\n\
         nobody always pays it."
    );
    let taxi = group.add_expense(
        Expense::builder(
            "Taxi",
            eur("10.00")?,
            ana,
            Split::equal([ana, bob, mia]),
            day,
        )
        .build()?,
    )?;
    report(group, taxi, &thirds(family));
    Ok(taxi)
}

const fn thirds(family: &Family) -> [Rule; 3] {
    [
        Rule {
            member: family.ana,
            text: "an equal third",
        },
        Rule {
            member: family.bob,
            text: "an equal third",
        },
        Rule {
            member: family.mia,
            text: "an equal third",
        },
    ]
}

fn by_percentage(group: &mut Group, family: &Family) -> Result<ExpenseId, Box<dyn Error>> {
    let Family { ana, bob, mia, day } = *family;
    section("2. By percentage");
    println!(
        "Everyone pays a percentage of the amount; the percentages must add up to exactly\n\
         100 %. Mia's room is smaller, so she takes 20 %."
    );
    let rent = group.add_expense(
        Expense::builder(
            "Flat",
            eur("500.00")?,
            bob,
            Split::percentages([
                MemberPercentage::new(ana, Percentage::parse("40")?),
                MemberPercentage::new(bob, Percentage::parse("40")?),
                MemberPercentage::new(mia, Percentage::parse("20")?),
            ]),
            day,
        )
        .build()?,
    )?;
    report(
        group,
        rent,
        &[
            Rule {
                member: ana,
                text: "40%",
            },
            Rule {
                member: bob,
                text: "40%",
            },
            Rule {
                member: mia,
                text: "20%",
            },
        ],
    );
    Ok(rent)
}

fn by_parts(group: &mut Group, family: &Family) -> Result<ExpenseId, Box<dyn Error>> {
    let Family { ana, bob, mia, day } = *family;
    section("3. By parts");
    println!(
        "Everyone pays in proportion to their parts. Mia, a child, counts as half an adult:\n\
         2.5 parts in total, so one part is 125.00 / 2.5 = 50.00."
    );
    let food = group.add_expense(
        Expense::builder(
            "Food",
            eur("125.00")?,
            ana,
            Split::parts([
                MemberParts::new(ana, Parts::ONE),
                MemberParts::new(bob, Parts::ONE),
                MemberParts::new(mia, Parts::parse("0.5")?),
            ]),
            day,
        )
        .build()?,
    )?;
    report(
        group,
        food,
        &[
            Rule {
                member: ana,
                text: "1 part",
            },
            Rule {
                member: bob,
                text: "1 part",
            },
            Rule {
                member: mia,
                text: "0.5 parts",
            },
        ],
    );
    Ok(food)
}

fn by_amounts(group: &mut Group, family: &Family) -> Result<ExpenseId, Box<dyn Error>> {
    let Family { ana, bob, mia, day } = *family;
    section("4. By exact amounts");
    println!(
        "Everyone pays a fixed amount, such as a ticket price; the amounts must add up to\n\
         exactly the expense amount."
    );
    let tickets = group.add_expense(
        Expense::builder(
            "Museum",
            eur("35.00")?,
            bob,
            Split::amounts([
                MemberAmount::new(ana, eur("15.00")?),
                MemberAmount::new(bob, eur("15.00")?),
                MemberAmount::new(mia, eur("5.00")?),
            ]),
            day,
        )
        .build()?,
    )?;
    report(
        group,
        tickets,
        &[
            Rule {
                member: ana,
                text: "15.00 EUR",
            },
            Rule {
                member: bob,
                text: "15.00 EUR",
            },
            Rule {
                member: mia,
                text: "5.00 EUR",
            },
        ],
    );
    Ok(tickets)
}

fn several_payers(group: &mut Group, family: &Family) -> Result<ExpenseId, Box<dyn Error>> {
    let Family { ana, bob, mia, day } = *family;
    section("5. Several payers");
    println!(
        "The dinner bill went on two cards: Ana paid 50.00 and Bob 40.00. The payers are\n\
         divided with the same methods as the participants."
    );
    let dinner = group.add_expense(
        Expense::builder_with_payers(
            "Dinner",
            eur("90.00")?,
            Split::amounts([
                MemberAmount::new(ana, eur("50.00")?),
                MemberAmount::new(bob, eur("40.00")?),
            ]),
            Split::equal([ana, bob, mia]),
            day,
        )
        .build()?,
    )?;
    report(group, dinner, &thirds(family));
    Ok(dinner)
}

fn mistakes(family: &Family) -> Result<(), Box<dyn Error>> {
    section("6. Mistakes are refused");
    println!("A percentage split that adds up to 90 % is refused when the expense is built:");
    let invalid = Expense::builder(
        "Invalid",
        eur("10.00")?,
        family.ana,
        Split::percentages([
            MemberPercentage::new(family.ana, Percentage::parse("60")?),
            MemberPercentage::new(family.bob, Percentage::parse("30")?),
        ]),
        family.day,
    )
    .build();
    println!(
        "  error: {}",
        invalid.err().ok_or("the split must be refused")?
    );
    Ok(())
}

fn totals(group: &Group) {
    section("7. The holiday in total");
    let summary = group.summary();
    println!("{} spent over five expenses:", summary.total_spent());
    print_table(
        group.members().map(|member| {
            let member_totals = summary.member(member.id());
            TotalRow {
                name: member.first_name().to_owned(),
                paid: member_totals.paid(),
                consumed: member_totals.share(),
                balance: member_totals.balance().to_string(),
            }
        }),
        1,
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut group = Group::builder("Family holiday", Currency::EUR).build()?;
    let family = Family {
        ana: group.add_member(Member::builder("Ana").build()?)?,
        bob: group.add_member(Member::builder("Bob").build()?)?,
        mia: group.add_member(Member::builder("Mia").build()?)?,
        day: CalendarDate::new(2026, 7, 14)?,
    };

    println!("Family holiday: every way to split an expense");
    println!("═════════════════════════════════════════════");
    println!(
        "Ana, Bob, and their daughter Mia share a holiday. Each expense below uses a\n\
         different split method; whatever the method, the shares add up to exactly the\n\
         amount, to the cent."
    );
    equally(&mut group, &family)?;
    by_percentage(&mut group, &family)?;
    let food = by_parts(&mut group, &family)?;
    by_amounts(&mut group, &family)?;
    several_payers(&mut group, &family)?;
    mistakes(&family)?;
    totals(&group);

    // Parts: 125.00 over 2.5 parts is 50.00 per part.
    let shares = group.expense_shares(food).ok_or("food was added")?;
    assert_eq!(
        shares.member(family.mia).map(MemberShare::share),
        Some(eur("25.00")?)
    );
    Ok(())
}
