//! Members with contact details and payment accounts: IBANs, emails, and
//! phone numbers are validated, and a member who leaves is archived rather
//! than removed.
//!
//! Run with `cargo run -p eranos --example members_and_accounts`.

use std::error::Error;

use eranos::error::GroupError;
use eranos::{
    Account, AccountDetails, Balance, CalendarDate, Currency, Email, Expense, Group, Iban, Member,
    MemberId, Money, PhoneNumber, Split, Username,
};
use tabled::settings::object::Columns;
use tabled::settings::{Alignment, Style};
use tabled::{Table, Tabled};

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
    #[tabled(rename = "Input")]
    input: &'static str,
    #[tabled(rename = "Checked as")]
    kind: &'static str,
    #[tabled(rename = "Result")]
    result: String,
}

#[derive(Tabled)]
struct BalanceRow {
    #[tabled(rename = "Balance", order = 3)]
    balance: Balance,
    #[tabled(rename = "Consumed", order = 2)]
    consumed: Money,
    #[tabled(rename = "Member", order = 0)]
    name: String,
    #[tabled(rename = "Status", order = 1)]
    status: &'static str,
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

/// An account with its label and its details.
fn describe(account: &Account) -> String {
    let details = match account.details() {
        AccountDetails::Iban(iban) => iban.as_str().to_owned(),
        AccountDetails::Username(username) => username.as_str().to_owned(),
        _ => format!("{:?}", account.kind()),
    };
    format!("{}: {details}", account.name())
}

fn outcome<Value, Error: std::fmt::Display>(
    result: Result<Value, Error>,
    shown: impl Fn(Value) -> String,
) -> String {
    result.map_or_else(
        |error| format!("refused: {error}"),
        |value| format!("valid: {}", shown(value)),
    )
}

fn print_members(group: &Group) {
    print_table(
        group.members().map(|member| MemberRow {
            accounts: member
                .accounts()
                .iter()
                .map(describe)
                .collect::<Vec<_>>()
                .join("\n"),
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

fn print_balances(group: &Group) {
    let summary = group.summary();
    print_table(
        group.members().map(|member| BalanceRow {
            balance: summary.member(member.id()).balance(),
            consumed: summary.member(member.id()).share(),
            name: member.first_name().to_owned(),
            status: if group.is_archived(member.id()) {
                "archived"
            } else {
                "active"
            },
        }),
        2,
    );
}

fn members(group: &mut Group) -> Result<(MemberId, MemberId), Box<dyn Error>> {
    section("1. Members");
    let ana = group.add_member(
        Member::builder("Ana")
            .last_name("Popescu")
            .email(Email::parse("ana@example.com")?)
            .phone(PhoneNumber::parse("+40 712 345 678")?)
            .account(Account::new(
                Iban::parse("ro49 aaaa 1b31 0075 9384 0000")?,
                "ING",
            )?)
            .account(Account::new(Username::new("@ana")?, "Revolut")?)
            .build()?,
    )?;
    let bob = group.add_member(Member::builder("Bob").build()?)?;
    print_members(group);
    println!(
        "The phone number was stored in E.164 form, +40712345678, and the table shows it\n\
         grouped for reading; the IBAN was stored in its electronic form, uppercase and\n\
         without spaces, the way banking apps accept it."
    );
    Ok((ana, bob))
}

fn validation() {
    section("2. What validation accepts");
    println!(
        "IBANs are checked against the SWIFT registry: the country, its length, which\n\
         positions hold digits and which letters, and the mod-97 check digits, which catch\n\
         any single mistyped character. Emails are checked against the rules of RFC 5321\n\
         and 5322, which refuse the doubled dots and stray commas of hurried typing, and\n\
         phone numbers against their country's numbering plan, stored in E.164 form."
    );
    let iban = |input: &'static str| CheckRow {
        input,
        kind: "IBAN",
        result: outcome(Iban::parse(input), |iban| iban.as_str().to_owned()),
    };
    let email = |input: &'static str| CheckRow {
        input,
        kind: "email",
        result: outcome(Email::parse(input), |email| email.as_str().to_owned()),
    };
    let phone = |input: &'static str| CheckRow {
        input,
        kind: "phone",
        result: outcome(PhoneNumber::parse(input), |phone| phone.as_str().to_owned()),
    };
    print_table(
        [
            iban("ro49 aaaa 1b31 0075 9384 0000"),
            iban("RO49 AAAA 1B31 0075 9384 0001"),
            iban("RO49 AAAA 1B31 0075"),
            iban("DE46 37O4 0044 0532 0130 00"),
            email("ana@example.com"),
            email("ana.example.com"),
            email("ana..popescu@example.com"),
            phone("+40 (712) 345-678"),
            phone("0712 345 678"),
            phone("+40 612 345 678"),
            phone("+44 (0)20 7946 0958"),
        ],
        3,
    );
}

fn moving_out(group: &mut Group, ana: MemberId, bob: MemberId) -> Result<(), Box<dyn Error>> {
    section("3. A flatmate moves out");
    let rent = Expense::builder(
        Money::parse("900.00", Currency::EUR)?,
        CalendarDate::new(1, 9, 2026)?,
        "Rent",
        ana,
        Split::equal([ana, bob]),
    )
    .build()?;
    let rent = group.add_expense(rent)?;
    group.archive_member(bob)?;
    println!(
        "Bob shared September's rent, then moved out. He is archived, not removed: his\n\
         history and his balance stay, and he can still pay Ana back."
    );
    print_balances(group);

    let utilities = Expense::builder(
        Money::parse("60.00", Currency::EUR)?,
        CalendarDate::new(1, 10, 2026)?,
        "Utilities",
        ana,
        Split::equal([ana, bob]),
    )
    .build()?;
    let refused = group.add_expense(utilities);
    println!(
        "October's utilities cannot include him:\n  error: {}",
        refused
            .as_ref()
            .err()
            .ok_or("the archived member must be refused")?
    );
    let removal = group.remove_member(bob);
    println!(
        "Nor can he be removed while the rent references him:\n  error: {}",
        removal
            .as_ref()
            .err()
            .ok_or("the referenced member must stay")?
    );

    assert_eq!(refused, Err(GroupError::ArchivedMember(bob)));
    assert!(matches!(removal, Err(GroupError::MemberInUse { .. })));
    assert_eq!(
        group.summary().member(bob).balance(),
        Balance::Owes(Money::parse("450.00", Currency::EUR)?)
    );
    assert!(group.expense(rent).is_some());
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut group = Group::builder(Currency::EUR, "Flat 4B").build()?;
    println!("Flat 4B: members and their accounts");
    println!("═══════════════════════════════════");
    println!(
        "Ana and Bob share a flat. A member needs only a first name; the last name,\n\
         email, phone, and the accounts they can be paid to are optional."
    );
    let (ana, bob) = members(&mut group)?;
    validation();
    moving_out(&mut group, ana, bob)
}
