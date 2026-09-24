//! End-to-end scenarios through the public API, worked out by hand: a
//! weekend trip with every split method, an itemised restaurant bill, a
//! multi-currency trip, and a household where a member moves out. The
//! amounts divide evenly, so every expected number is exact; the numbers in
//! the comments are the hand calculation the assertions pin.
#![expect(
    clippy::unwrap_used,
    reason = "a failing test helper panics to report the failure, as clippy.toml allows inside test functions"
)]

use eranos::{
    Balance, Category, CategoryId, Currency, Date, ExchangeRate, Expense, Group, Item, Member,
    MemberId, MemberShare, Money, Parts, Percentage, SettlementStrategy, Split, Tip,
    TipDistribution, Transfer, error::GroupError,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn eur(text: &str) -> Money {
    Money::parse(text, Currency::EUR).unwrap()
}

fn day(day: u8) -> Date {
    Date::new(day, 9, 2026).unwrap()
}

fn member(name: &str) -> Member {
    Member::builder(name).build().unwrap()
}

/// (payer, recipient, amount) of a settlement.
fn payments(group: &Group, strategy: SettlementStrategy) -> Vec<(MemberId, MemberId, Money)> {
    group
        .settlement(strategy)
        .unwrap()
        .payments()
        .iter()
        .map(|payment| (payment.from(), payment.to(), payment.amount()))
        .collect()
}

/// Sorts expected payments the way settlements are sorted.
fn sorted(mut payments: Vec<(MemberId, MemberId, Money)>) -> Vec<(MemberId, MemberId, Money)> {
    payments.sort_by_key(|&(from, to, _)| (from, to));
    payments
}

struct Weekend {
    ana: MemberId,
    bob: MemberId,
    carol: MemberId,
    dan: MemberId,
    food: CategoryId,
    group: Group,
    lodging: CategoryId,
    transport: CategoryId,
}

/// Four friends, four expenses, one per split method.
fn weekend() -> Weekend {
    let mut group = Group::builder(Currency::EUR, "Weekend in Brașov")
        .build()
        .unwrap();
    let ana = group.add_member(member("Ana"));
    let bob = group.add_member(member("Bob"));
    let carol = group.add_member(member("Carol"));
    let dan = group.add_member(member("Dan"));
    let lodging = group
        .add_category(Category::new("Lodging").unwrap())
        .unwrap();
    let transport = group
        .add_category(Category::new("Transport").unwrap())
        .unwrap();
    let food = group.add_category(Category::new("Food").unwrap()).unwrap();
    let expenses = [
        // Hotel 400.00 by Ana, equal among four: 100.00 each.
        Expense::builder(eur("400.00"), day(19), "Hotel")
            .paid_by(ana)
            .split(Split::equal([ana, bob, carol, dan]))
            .category(lodging),
        // Fuel 90.00 by Bob, equal among the three who rode: 30.00 each.
        Expense::builder(eur("90.00"), day(19), "Fuel")
            .paid_by(bob)
            .split(Split::equal([ana, bob, carol]))
            .category(transport),
        // Groceries 60.00 by Carol, parts 1:1:1:3: 10, 10, 10, 30.
        Expense::builder(eur("60.00"), day(20), "Groceries")
            .paid_by(carol)
            .split(Split::parts([
                (ana, Parts::ONE),
                (bob, Parts::ONE),
                (carol, Parts::ONE),
                (dan, Parts::from_whole(3).unwrap()),
            ]))
            .category(food),
        // Dinner 100.00 + 10 % tip by Dan, 40/20/20/20: 44, 22, 22, 22.
        Expense::builder(eur("100.00"), day(20), "Dinner")
            .paid_by(dan)
            .split(Split::percentages([
                (ana, Percentage::parse("40").unwrap()),
                (bob, Percentage::parse("20").unwrap()),
                (carol, Percentage::parse("20").unwrap()),
                (dan, Percentage::parse("20").unwrap()),
            ]))
            .tip(Tip::percentage(Percentage::parse("10").unwrap()))
            .category(food),
    ];
    for expense in expenses {
        group.add_expense(expense.build().unwrap()).unwrap();
    }
    Weekend {
        ana,
        bob,
        carol,
        dan,
        food,
        group,
        lodging,
        transport,
    }
}

/// The summary a trip shows is exactly the hand calculation: spending,
/// paid, consumed, balances, and category totals.
#[test]
fn a_weekend_trip_adds_up_by_hand() {
    let Weekend {
        ana,
        bob,
        carol,
        dan,
        food,
        group,
        lodging,
        transport,
    } = weekend();
    let summary = group.summary();

    // 400 + 90 + 60 + 110 spent.
    assert_eq!(summary.total_spent(), eur("660.00"));
    // Consumed: Ana 100 + 30 + 10 + 44, Bob and Carol 100 + 30 + 10 + 22,
    // Dan 100 + 30 + 22.
    assert_eq!(summary.member(ana).share(), eur("184.00"));
    assert_eq!(summary.member(bob).share(), eur("162.00"));
    assert_eq!(summary.member(carol).share(), eur("162.00"));
    assert_eq!(summary.member(dan).share(), eur("152.00"));
    assert_eq!(summary.member(dan).paid(), eur("110.00"));
    // Paid minus consumed.
    assert_eq!(
        summary.member(ana).balance(),
        Balance::GetsBack(eur("216.00"))
    );
    assert_eq!(summary.member(bob).balance(), Balance::Owes(eur("72.00")));
    assert_eq!(
        summary.member(carol).balance(),
        Balance::Owes(eur("102.00"))
    );
    assert_eq!(summary.member(dan).balance(), Balance::Owes(eur("42.00")));
    // Per category, and per member per category.
    assert_eq!(summary.category_total(Some(lodging)), eur("400.00"));
    assert_eq!(summary.category_total(Some(transport)), eur("90.00"));
    assert_eq!(summary.category_total(Some(food)), eur("170.00"));
    assert_eq!(summary.category_total(None), eur("0"));
    assert_eq!(summary.member_category_total(dan, Some(food)), eur("52.00"));
    assert_eq!(
        summary.member_category_total(dan, Some(transport)),
        eur("0")
    );
}

/// Each strategy produces its documented plan for the trip.
#[test]
fn every_strategy_settles_the_weekend_its_own_way() {
    let Weekend {
        ana,
        bob,
        carol,
        dan,
        group,
        ..
    } = weekend();

    // One creditor: the simplified strategies pay Ana directly.
    let direct_to_ana = sorted(vec![
        (bob, ana, eur("72.00")),
        (carol, ana, eur("102.00")),
        (dan, ana, eur("42.00")),
    ]);
    assert_eq!(
        payments(&group, SettlementStrategy::MinimumTransfers),
        direct_to_ana
    );
    assert_eq!(payments(&group, SettlementStrategy::Greedy), direct_to_ana);
    // Unsimplified, every pair nets its own debts: Bob owes Ana 100 − 30,
    // Carol owes Ana 100 − 10 and Bob 30 − 10, Dan owes Ana 100 − 44 and
    // Carol 30 − 22, and Bob owes Dan 22.
    assert_eq!(
        payments(&group, SettlementStrategy::Direct),
        sorted(vec![
            (bob, ana, eur("70.00")),
            (carol, ana, eur("90.00")),
            (dan, ana, eur("56.00")),
            (carol, bob, eur("20.00")),
            (bob, dan, eur("22.00")),
            (dan, carol, eur("8.00")),
        ])
    );
}

/// Recording any strategy's payments as transfers settles everyone, and the
/// settlement is then empty.
#[test]
fn recording_a_settlement_settles_everyone() -> TestResult {
    for strategy in SettlementStrategy::all() {
        let Weekend {
            ana,
            bob,
            carol,
            dan,
            mut group,
            ..
        } = weekend();
        let plan = group.settlement(strategy)?.clone();
        for payment in plan.payments() {
            group.add_transfer(payment.to_transfer(day(21)))?;
        }
        for member in [ana, bob, carol, dan] {
            assert_eq!(group.summary().member(member).balance(), Balance::Settled);
        }
        assert!(group.settlement(strategy)?.is_empty());
        // Transfers are not spending.
        assert_eq!(group.summary().total_spent(), eur("660.00"));
    }
    Ok(())
}

/// An itemised bill charges each member for their own items, and the two
/// tip distributions differ exactly as documented.
#[test]
fn an_itemised_bill_charges_each_member_for_their_items() -> TestResult {
    let mut group = Group::builder(Currency::EUR, "Dinner").build()?;
    let ana = group.add_member(member("Ana"));
    let bob = group.add_member(member("Bob"));
    let carol = group.add_member(member("Carol"));
    let items = || -> Result<Vec<Item>, eranos::error::ExpenseError> {
        Ok(vec![
            // Ana's steak and Bob's pasta.
            Item::builder("Steak", eur("30.00"))
                .shared_by([ana])
                .build()?,
            Item::builder("Pasta", eur("15.00"))
                .shared_by([bob])
                .build()?,
            // Three bottles of water shared by everyone: 1.50 each.
            Item::builder("Water", eur("1.50"))
                .quantity(3)
                .shared_by([ana, bob, carol])
                .build()?,
            // A dessert for Bob and Carol, Carol eating two thirds.
            Item::builder("Dessert", eur("10.50"))
                .split(Split::parts([
                    (bob, Parts::ONE),
                    (carol, Parts::from_whole(2)?),
                ]))
                .build()?,
        ])
    };
    // 30 + 15 + 4.50 + 10.50 = 60.00: Ana 31.50, Bob 20.00, Carol 8.50.
    let bill = |distribution| -> Result<Expense, eranos::error::ExpenseError> {
        Expense::builder(eur("60.00"), day(23), "Bill")
            .paid_by(carol)
            .items(items()?)
            .tip(Tip::fixed(eur("6.00")).distributed(distribution))
            .build()
    };

    // Proportional: the 6.00 tip is 10 % of each subtotal.
    let proportional = group.add_expense(bill(TipDistribution::Proportional)?)?;
    let shares = group
        .expense_shares(proportional)
        .expect("the expense was added");
    let consumed = |member| shares.member(member).map(MemberShare::share);
    assert_eq!(consumed(ana), Some(eur("34.65")));
    assert_eq!(consumed(bob), Some(eur("22.00")));
    assert_eq!(consumed(carol), Some(eur("9.35")));
    assert_eq!(shares.tip(), eur("6.00"));

    // Equal: 2.00 each, whatever they ordered.
    let equal = group.add_expense(bill(TipDistribution::Equal)?)?;
    let even = group.expense_shares(equal).expect("the expense was added");
    let consumed_evenly = |member| even.member(member).map(MemberShare::share);
    assert_eq!(consumed_evenly(ana), Some(eur("33.50")));
    assert_eq!(consumed_evenly(bob), Some(eur("22.00")));
    assert_eq!(consumed_evenly(carol), Some(eur("10.50")));

    // Items that do not add up to the amount are refused.
    let short = Expense::builder(eur("61.00"), day(23), "Bill")
        .paid_by(carol)
        .items(items()?)
        .build();
    assert!(short.is_err());
    Ok(())
}

/// Foreign expenses and transfers convert at their own fixed rates, and a
/// currency change re-converts everything.
#[test]
fn a_multi_currency_trip_totals_in_the_group_currency() -> TestResult {
    let mut group = Group::builder(Currency::EUR, "Bucharest").build()?;
    let ana = group.add_member(member("Ana"));
    let bob = group.add_member(member("Bob"));
    let ron = |text| Money::parse(text, Currency::RON);

    // 500.00 RON at 0.2 EUR is 100.00 EUR, split in two.
    let hostel = Expense::builder(
        ron("500.00")?.at(ExchangeRate::parse("0.2")?),
        day(22),
        "Hostel",
    )
    .paid_by(ana)
    .split(Split::equal([ana, bob]))
    .build()?;
    group.add_expense(hostel)?;
    // A 40.00 EUR train in the group currency needs no rate.
    group.add_expense(
        Expense::builder(eur("40.00"), day(22), "Train")
            .paid_by(bob)
            .split(Split::equal([ana, bob]))
            .build()?,
    )?;
    assert_eq!(group.summary().total_spent(), eur("140.00"));
    assert_eq!(
        group.summary().member(bob).balance(),
        Balance::Owes(eur("30.00"))
    );

    // Bob pays back 150.00 RON in cash at 0.2: 30.00 EUR.
    let cash = Transfer::builder(
        ron("150.00")?.at(ExchangeRate::parse("0.2")?),
        day(23),
        bob,
        ana,
    )
    .build()?;
    group.add_transfer(cash)?;
    assert_eq!(group.summary().member(bob).balance(), Balance::Settled);

    // A rate is required for foreign entries and refused for local ones.
    let unrated = Expense::builder(ron("10.00")?, day(23), "Snack")
        .paid_by(ana)
        .split(Split::equal([ana]))
        .build()?;
    assert_eq!(
        group.add_expense(unrated),
        Err(GroupError::MissingExchangeRate {
            currency: Currency::RON
        })
    );

    // Moving the group to RON needs the EUR → RON rate for the train.
    group.change_currency(Currency::RON, &[(Currency::EUR, ExchangeRate::parse("5")?)])?;
    assert_eq!(group.summary().total_spent(), ron("700.00")?);
    assert_eq!(group.summary().member(ana).balance(), Balance::Settled);
    Ok(())
}

/// A flatmate who moves out settles up and is archived: their history
/// stays, new expenses cannot include them, and removal is refused while
/// their entries exist.
#[test]
fn a_flatmate_moves_out() -> TestResult {
    let mut group = Group::builder(Currency::EUR, "Flat 4B").build()?;
    let ana = group.add_member(member("Ana"));
    let bob = group.add_member(member("Bob"));
    let rent = group.add_expense(
        Expense::builder(eur("1200.00"), day(1), "Rent")
            .paid_by(ana)
            .split(Split::equal([ana, bob]))
            .build()?,
    )?;
    let deposit =
        group.add_transfer(Transfer::builder(eur("600.00"), day(2), bob, ana).build()?)?;
    group.archive_member(bob)?;

    // Bob's history and settled balance stay.
    assert_eq!(group.summary().member(bob).share(), eur("600.00"));
    assert_eq!(group.summary().member(bob).balance(), Balance::Settled);
    // He cannot join new expenses, nor be removed while referenced.
    let utilities = Expense::builder(eur("80.00"), day(3), "Utilities")
        .paid_by(ana)
        .split(Split::equal([ana, bob]))
        .build()?;
    assert_eq!(
        group.add_expense(utilities),
        Err(GroupError::ArchivedMember(bob))
    );
    assert_eq!(
        group.remove_member(bob),
        Err(GroupError::MemberInUse {
            expenses: vec![rent],
            member: bob,
            transfers: vec![deposit]
        })
    );
    // Correcting the old rent may keep him.
    let corrected = group
        .expense(rent)
        .expect("rent exists")
        .to_builder()
        .cost(eur("1100.00"))
        .build()?;
    group.replace_expense(rent, corrected)?;
    assert_eq!(
        group.summary().member(bob).balance(),
        Balance::GetsBack(eur("50.00"))
    );
    Ok(())
}
