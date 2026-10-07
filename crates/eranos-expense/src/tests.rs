//! An expense records who paid, who shares the cost, and any tip on top.
//! These tests keep every constructor setting its payers and its split, keep
//! invalid expenses out (a blank name, a non-positive amount, items or
//! shares that do not add up), keep the payers covering the amount plus the
//! tip while the participants divide the amount alone, keep the last setter
//! in force, also between the forms of payers and of splits, keep a
//! replaced cost's rate only when the new cost has one, and keep the way a
//! cost is shared through `FromExpenseSplit`.

use eranos_money::{Currency, Percentage};

use super::*;

// Four members, in ID order.
const ANA: MemberId = MemberId::from_bytes([1; 16]);
const BOB: MemberId = MemberId::from_bytes([2; 16]);
const CAROL: MemberId = MemberId::from_bytes([3; 16]);
const DAN: MemberId = MemberId::from_bytes([4; 16]);

fn euros(minor_units: i64) -> Money {
    Money::from_minor_units(minor_units, Currency::EUR)
}

fn per_cent(text: &str) -> Percentage {
    Percentage::parse(text).unwrap()
}

fn september_23() -> CalendarDate {
    CalendarDate::new(2026, 9, 23).unwrap()
}

/// A 90.00 EUR dinner on 23 September 2026 that Ana paid and shared with
/// Bob and Carol; tests replace the payers and the split they check.
fn dinner() -> ExpenseBuilder {
    Expense::builder(
        "Dinner",
        euros(9_000),
        ANA,
        Split::equal([ANA, BOB, CAROL]),
        september_23(),
    )
}

/// A line of `minor_units` shared by Ana.
fn item(name: &str, minor_units: i64) -> ExpenseItem {
    ExpenseItem::builder(name, euros(minor_units), [ANA])
        .build()
        .unwrap()
}

/// The dinner with every optional field set and converted at 1 EUR =
/// 4.9765 RON: a 10 % tip, Ana and Bob paying the 99.00 total between them,
/// and three members sharing the 90.00 by percentage.
fn full_expense() -> Expense {
    dinner()
        .cost(euros(9_000).at(ExchangeRate::parse("4.9765").unwrap()))
        .payers(Split::amounts([
            MemberAmount::new(ANA, euros(5_000)),
            MemberAmount::new(BOB, euros(4_900)),
        ]))
        .split(Split::percentages([
            MemberPercentage::new(ANA, per_cent("50")),
            MemberPercentage::new(BOB, per_cent("25")),
            MemberPercentage::new(CAROL, per_cent("25")),
        ]))
        .tip(Tip::percentage(per_cent("10")))
        .time(TimeOfDay::new(20, 30).unwrap())
        .category(CategoryId::from_bytes([9; 16]))
        .description("Birthday")
        .build()
        .unwrap()
}

/// The dinner as a receipt paid by Bob: a pizza for Ana and two glasses of
/// wine for Ana and Carol.
fn itemised_expense() -> Expense {
    let wine = ExpenseItem::builder("Wine", euros(2_000), [ANA, CAROL])
        .quantity(2)
        .build()
        .unwrap();
    dinner()
        .paid_by(BOB)
        .items([item("Pizza", 5_000), wine])
        .build()
        .unwrap()
}

/// A plain expense would gain details nobody entered, or lose required ones.
#[test]
fn a_minimal_expense_keeps_its_fields_and_defaults() {
    let expense = dinner()
        .paid_by(ANA)
        .split(Split::equal([ANA, BOB, CAROL]))
        .build()
        .unwrap();
    // Required fields; a plain amount is a local cost, at a rate of 1.
    assert_eq!(expense.name(), "Dinner");
    assert_eq!(expense.cost(), Cost::from(euros(9_000)));
    assert_eq!(expense.amount(), euros(9_000));
    assert_eq!(expense.exchange_rate(), ExchangeRate::ONE);
    assert_eq!(expense.date(), september_23());
    assert_eq!(expense.payers(), &Split::equal([ANA]));
    assert_eq!(
        expense.split(),
        &ExpenseSplit::Whole(Split::equal([ANA, BOB, CAROL]))
    );
    // Optional fields are absent or at their defaults: the built-in
    // category and no tip.
    assert_eq!(expense.description(), None);
    assert_eq!(expense.category(), CategoryId::UNCATEGORISED);
    assert_eq!(expense.tip(), Tip::default());
    assert_eq!(expense.time(), None);
    // Without a tip, the total is the amount.
    assert_eq!(expense.total(), euros(9_000));
    assert_eq!(expense.tip_amount(), euros(0));
}

/// Expenses would be listed with stray spaces or with no name at all.
#[test]
fn the_expense_name_is_trimmed_and_required() {
    // Surrounding whitespace.
    let expense = Expense::builder(
        "  Dinner \n",
        euros(9_000),
        ANA,
        Split::equal([ANA]),
        september_23(),
    )
    .build()
    .unwrap();
    assert_eq!(expense.name(), "Dinner");
    // Blank names.
    for name in ["", "   "] {
        assert_eq!(
            dinner()
                .name(name)
                .paid_by(ANA)
                .split(Split::equal([ANA]))
                .build(),
            Err(ExpenseError::EmptyName),
            "{name:?}"
        );
    }
}

/// A free or negative expense would be accepted and move balances the wrong
/// way.
#[test]
fn a_non_positive_amount_is_rejected() {
    // Zero and negative amounts.
    for amount in [euros(0), euros(-9_000)] {
        assert_eq!(
            dinner()
                .cost(amount)
                .paid_by(ANA)
                .split(Split::equal([ANA]))
                .build(),
            Err(ExpenseError::NonPositiveAmount)
        );
    }
}

/// An expense started with one constructor would come out with the payers
/// or the split of another.
#[test]
fn every_constructor_sets_its_payers_and_its_split() {
    let pizza = || item("Pizza", 9_000);
    let payers = Split::amounts([
        MemberAmount::new(ANA, euros(5_000)),
        MemberAmount::new(BOB, euros(4_000)),
    ]);
    let everyone = Split::equal([ANA, BOB, CAROL]);
    // One payer and a whole split.
    let whole = Expense::builder(
        "Dinner",
        euros(9_000),
        BOB,
        everyone.clone(),
        september_23(),
    )
    .build()
    .unwrap();
    assert_eq!(whole.payers(), &Split::equal([BOB]));
    assert_eq!(whole.split(), &ExpenseSplit::Whole(everyone.clone()));
    // Several payers and a whole split.
    let shared = Expense::builder_with_payers(
        "Dinner",
        euros(9_000),
        payers.clone(),
        everyone.clone(),
        september_23(),
    )
    .build()
    .unwrap();
    assert_eq!(shared.payers(), &payers);
    assert_eq!(shared.split(), &ExpenseSplit::Whole(everyone));
    // One payer and items.
    let itemised =
        Expense::itemised_builder("Dinner", euros(9_000), BOB, [pizza()], september_23())
            .build()
            .unwrap();
    assert_eq!(itemised.payers(), &Split::equal([BOB]));
    assert_eq!(itemised.split(), &ExpenseSplit::Itemised(vec![pizza()]));
    // Several payers and items.
    let shared_receipt = Expense::itemised_builder_with_payers(
        "Dinner",
        euros(9_000),
        payers.clone(),
        [pizza()],
        september_23(),
    )
    .build()
    .unwrap();
    assert_eq!(shared_receipt.payers(), &payers);
    assert_eq!(
        shared_receipt.split(),
        &ExpenseSplit::Itemised(vec![pizza()])
    );
}

/// Correcting the payers or the split before saving would keep what the
/// constructor or an earlier call set.
#[test]
fn the_last_setter_wins() {
    let payers = || Split::amounts([MemberAmount::new(BOB, euros(9_000))]);
    // A list of payers after a single payer, and the reverse.
    let listed = dinner().paid_by(ANA).payers(payers()).build().unwrap();
    assert_eq!(listed.payers(), &payers());
    let single = dinner().payers(payers()).paid_by(CAROL).build().unwrap();
    assert_eq!(single.payers(), &Split::equal([CAROL]));
    // Items after a whole split, and the reverse.
    let receipt = dinner()
        .split(Split::equal([ANA]))
        .items([item("Pizza", 9_000)])
        .build()
        .unwrap();
    assert_eq!(
        receipt.split(),
        &ExpenseSplit::Itemised(vec![item("Pizza", 9_000)])
    );
    let whole = dinner()
        .items([item("Pizza", 9_000)])
        .split(Split::equal([BOB]))
        .build()
        .unwrap();
    assert_eq!(whole.split(), &ExpenseSplit::Whole(Split::equal([BOB])));
    // The same setter twice: `paid_by`.
    let expense = dinner()
        .paid_by(ANA)
        .paid_by(BOB)
        .split(Split::equal([ANA]))
        .build()
        .unwrap();
    assert_eq!(expense.payers(), &Split::equal([BOB]));
    // `payers` twice.
    let payers_twice = dinner()
        .payers(Split::amounts([MemberAmount::new(ANA, euros(9_000))]))
        .payers(Split::amounts([MemberAmount::new(BOB, euros(9_000))]))
        .split(Split::equal([ANA]))
        .build()
        .unwrap();
    assert_eq!(
        payers_twice.payers(),
        &Split::amounts([MemberAmount::new(BOB, euros(9_000))])
    );
    // `split` twice.
    let split_twice = dinner()
        .paid_by(ANA)
        .split(Split::equal([ANA]))
        .split(Split::equal([BOB]))
        .build()
        .unwrap();
    assert_eq!(
        split_twice.split(),
        &ExpenseSplit::Whole(Split::equal([BOB]))
    );
    // `items` twice.
    let items_twice = dinner()
        .paid_by(ANA)
        .items([item("Pizza", 9_000)])
        .items([item("Wine", 9_000)])
        .build()
        .unwrap();
    assert_eq!(
        items_twice.split(),
        &ExpenseSplit::Itemised(vec![item("Wine", 9_000)])
    );
}

/// An itemised expense without lines would be accepted and nobody would
/// share its cost.
#[test]
fn an_itemised_expense_without_items_is_rejected() {
    // An empty receipt.
    assert_eq!(
        dinner().paid_by(ANA).items([]).build(),
        Err(ExpenseError::NoItems)
    );
}

/// Receipt lines that miss the amount would leave part of it unshared or
/// charge too much.
#[test]
fn items_must_add_up_to_the_amount() {
    // Lines of 50.00 and 40.00 on a 90.00 receipt.
    let expense = dinner()
        .paid_by(ANA)
        .items([item("Pizza", 5_000), item("Wine", 4_000)])
        .build()
        .unwrap();
    assert!(matches!(expense.split(), ExpenseSplit::Itemised(items) if 2 == items.len()));
    // Lines of 50.00 and 30.00.
    assert_eq!(
        dinner()
            .paid_by(ANA)
            .items([item("Pizza", 5_000), item("Wine", 3_000)])
            .build(),
        Err(ExpenseError::ItemsDoNotAddUp {
            expected: euros(9_000),
            actual: euros(8_000)
        })
    );
    // Item totals whose sum leaves the 64-bit range.
    assert_eq!(
        dinner()
            .paid_by(ANA)
            .items([item("Yacht", i64::MAX), item("Yacht", i64::MAX)])
            .build(),
        Err(ExpenseError::Overflow)
    );
}

/// A line priced in dollars would be counted as euros on a euro receipt.
#[test]
fn an_item_in_another_currency_is_rejected() {
    // A dollar line on a euro receipt.
    let dollars = ExpenseItem::builder(
        "Pizza",
        Money::from_minor_units(9_000, Currency::USD),
        [ANA],
    )
    .build()
    .unwrap();
    assert_eq!(
        dinner().paid_by(ANA).items([dollars]).build(),
        Err(ExpenseError::CurrencyMismatch {
            expected: Currency::EUR,
            found: Currency::USD
        })
    );
}

/// Participants would be charged the tip twice: once in their shares and
/// once in the tip itself.
#[test]
fn the_participants_divide_the_amount_before_the_tip() {
    let tipped = || dinner().paid_by(ANA).tip(Tip::percentage(per_cent("10")));
    // Shares of the 90.00 amount.
    let split = Split::amounts([
        MemberAmount::new(ANA, euros(4_500)),
        MemberAmount::new(BOB, euros(4_500)),
    ]);
    assert!(tipped().split(split).build().is_ok());
    // Shares of the 99.00 total.
    let shares_of_the_total = Split::amounts([
        MemberAmount::new(ANA, euros(4_950)),
        MemberAmount::new(BOB, euros(4_950)),
    ]);
    assert_eq!(
        tipped().split(shares_of_the_total).build(),
        Err(ExpenseError::AmountsDoNotAddUp {
            expected: euros(9_000),
            actual: euros(9_900)
        })
    );
    // Any other invalid split.
    assert_eq!(
        tipped().split(Split::equal([])).build(),
        Err(ExpenseError::EmptySplit)
    );
}

/// The payers' shares would leave the tip unpaid, and the balances would
/// not add up to zero.
#[test]
fn the_payers_cover_the_amount_plus_the_tip() {
    let tipped = || {
        dinner()
            .split(Split::equal([ANA, BOB]))
            .tip(Tip::percentage(per_cent("10")))
    };
    // Payments adding up to the 99.00 total.
    let payers = Split::amounts([
        MemberAmount::new(ANA, euros(5_000)),
        MemberAmount::new(BOB, euros(4_900)),
    ]);
    assert!(tipped().payers(payers).build().is_ok());
    // Payments adding up to the 90.00 amount only.
    assert_eq!(
        tipped()
            .payers(Split::amounts([MemberAmount::new(ANA, euros(9_000))]))
            .build(),
        Err(ExpenseError::AmountsDoNotAddUp {
            expected: euros(9_900),
            actual: euros(9_000)
        })
    );
    // Any other invalid payers.
    assert_eq!(
        tipped().payers(Split::equal([])).build(),
        Err(ExpenseError::EmptySplit)
    );
}

/// The apps would show a total without the tip, or a tip that differs from
/// the one entered.
#[test]
fn total_and_tip_amount_include_the_tip() {
    let tipped = |tip: Tip| {
        dinner()
            .paid_by(ANA)
            .split(Split::equal([ANA, BOB]))
            .tip(tip)
            .build()
            .unwrap()
    };
    // A percentage tip.
    let expense = tipped(Tip::percentage(per_cent("10")));
    assert_eq!(expense.tip(), Tip::percentage(per_cent("10")));
    assert_eq!(expense.tip_amount(), euros(900));
    assert_eq!(expense.total(), euros(9_900));
    // A fixed tip.
    let fixed_tip_expense = tipped(Tip::fixed(euros(500)));
    assert_eq!(fixed_tip_expense.tip_amount(), euros(500));
    assert_eq!(fixed_tip_expense.total(), euros(9_500));
    // A zero tip.
    let zero_tip_expense = tipped(Tip::fixed(euros(0)));
    assert_eq!(zero_tip_expense.tip_amount(), euros(0));
    assert_eq!(zero_tip_expense.total(), euros(9_000));
    // A cleared tip.
    let cleared_tip_expense = expense.to_builder().tip(Tip::default()).build().unwrap();
    assert_eq!(cleared_tip_expense.tip(), Tip::default());
    assert_eq!(cleared_tip_expense.total(), euros(9_000));
}

/// A negative or foreign tip would be added to the total.
#[test]
fn an_invalid_fixed_tip_is_rejected() {
    let builder = || dinner().paid_by(ANA).split(Split::equal([ANA]));
    // Negative.
    assert_eq!(
        builder().tip(Tip::fixed(euros(-100))).build(),
        Err(ExpenseError::NegativeTip)
    );
    // Another currency.
    let dollars = Money::from_minor_units(100, Currency::USD);
    assert_eq!(
        builder().tip(Tip::fixed(dollars)).build(),
        Err(ExpenseError::CurrencyMismatch {
            expected: Currency::EUR,
            found: Currency::USD
        })
    );
}

/// A total too large for 64-bit minor units would wrap around to a wrong
/// amount.
#[test]
fn a_total_out_of_range_is_rejected() {
    let largest = || {
        Expense::builder(
            "Yacht",
            euros(i64::MAX),
            ANA,
            Split::equal([ANA]),
            september_23(),
        )
    };
    // The largest amount without a tip.
    assert!(largest().build().is_ok());
    // A fixed tip on top of it.
    assert_eq!(
        largest().tip(Tip::fixed(euros(1))).build(),
        Err(ExpenseError::Overflow)
    );
    // A percentage tip that does not fit on its own.
    assert_eq!(
        largest().tip(Tip::percentage(per_cent("200"))).build(),
        Err(ExpenseError::Overflow)
    );
}

/// A description left blank would be stored as an empty string and shown as
/// an empty line.
#[test]
fn the_description_is_trimmed_and_blank_clears_it() {
    let builder = || dinner().paid_by(ANA).split(Split::equal([ANA]));
    // Surrounding whitespace.
    let expense = builder().description("  Birthday \n").build().unwrap();
    assert_eq!(expense.description(), Some("Birthday"));
    // Blank text is no description.
    let blank_description_expense = builder().description("   ").build().unwrap();
    assert_eq!(blank_description_expense.description(), None);
    // Blank text clears a description set before.
    let cleared_description_expense = full_expense().to_builder().description("").build().unwrap();
    assert_eq!(cleared_description_expense.description(), None);
}

/// Optional details could not be added, or not removed once added.
#[test]
fn optional_fields_are_set_and_cleared_with_none() {
    // Set.
    let expense = full_expense();
    assert_eq!(expense.category(), CategoryId::from_bytes([9; 16]));
    assert_eq!(expense.time(), Some(TimeOfDay::new(20, 30).unwrap()));
    // Cleared.
    let expense = expense
        .to_builder()
        .category(CategoryId::UNCATEGORISED)
        .time(None)
        .build()
        .unwrap();
    assert_eq!(expense.category(), CategoryId::UNCATEGORISED);
    assert_eq!(expense.time(), None);
}

/// Correcting an expense's amount would keep a rate the new cost does not
/// have, or drop the rate of a corrected conversion.
#[test]
fn a_replaced_cost_brings_its_own_rate() {
    let expense = full_expense();
    let rate = ExchangeRate::parse("4.9765").unwrap();
    assert_eq!(expense.cost(), euros(9_000).at(rate));
    assert_eq!(expense.exchange_rate(), rate);
    // A plain amount makes the cost local; Ana pays the new total.
    let local_expense = expense
        .to_builder()
        .cost(euros(4_000))
        .paid_by(ANA)
        .build()
        .unwrap();
    assert_eq!(local_expense.cost(), Cost::from(euros(4_000)));
    // A corrected conversion keeps its rate.
    let corrected_expense = expense
        .to_builder()
        .cost(expense.cost().with_amount(euros(4_000)))
        .paid_by(ANA)
        .build()
        .unwrap();
    assert_eq!(corrected_expense.cost(), euros(4_000).at(rate));
}

/// A member who only paid, or only shared, would be missing from the
/// expense's members and could be removed from the group.
#[test]
fn members_are_the_payers_and_the_participants() {
    // A payer who does not share.
    let expense = dinner()
        .paid_by(DAN)
        .split(Split::equal([ANA, BOB]))
        .build()
        .unwrap();
    assert_eq!(expense.members(), BTreeSet::from([ANA, BOB, DAN]));
    // A payer who also shares is counted once.
    let paid_by_a_participant = dinner()
        .paid_by(ANA)
        .split(Split::equal([ANA, BOB]))
        .build()
        .unwrap();
    assert_eq!(paid_by_a_participant.members(), BTreeSet::from([ANA, BOB]));
    // Itemised expenses.
    assert_eq!(
        itemised_expense().members(),
        BTreeSet::from([ANA, BOB, CAROL])
    );
}

/// Members who share only one line of a receipt would be missing from the
/// participants.
#[test]
fn participants_are_everyone_who_shares_the_cost() {
    // A whole split.
    let whole = ExpenseSplit::Whole(Split::equal([CAROL, ANA]));
    assert_eq!(whole.participants(), BTreeSet::from([ANA, CAROL]));
    // Items shared by overlapping members.
    assert_eq!(
        itemised_expense().split().participants(),
        BTreeSet::from([ANA, CAROL])
    );
}

/// Two expenses would share an identifier, and with it their tie rotation,
/// or an edited expense would become another entry.
#[test]
fn an_expense_keeps_its_identifier_through_edits() {
    let expense = full_expense();
    assert_ne!(full_expense().id(), expense.id());
    let renamed = expense.to_builder().name("Supper").build().unwrap();
    assert_eq!(renamed.id(), expense.id());
}

/// Editing an expense would silently drop fields that were not edited.
#[test]
fn to_builder_rebuilds_an_equal_expense() {
    // Every optional field set, whole split.
    let expense = full_expense();
    assert_eq!(expense.to_builder().build().unwrap(), expense);
    // Itemised split.
    let itemised = itemised_expense();
    assert_eq!(itemised.to_builder().build().unwrap(), itemised);
}

/// Turning a whole expense into a receipt, or back, would be refused as a
/// conflict with the split it already has.
#[test]
fn a_rebuilt_expense_accepts_a_new_form_of_split() {
    // Whole into itemised.
    let expense = full_expense()
        .to_builder()
        .items([item("Pizza", 9_000)])
        .build()
        .unwrap();
    assert_eq!(
        expense.split(),
        &ExpenseSplit::Itemised(vec![item("Pizza", 9_000)])
    );
    // Itemised into whole.
    let whole_expense = itemised_expense()
        .to_builder()
        .split(Split::equal([BOB]))
        .build()
        .unwrap();
    assert_eq!(
        whole_expense.split(),
        &ExpenseSplit::Whole(Split::equal([BOB]))
    );
}

/// Changing who paid an expense would be refused as a conflict with the
/// payers it already has.
#[test]
fn a_rebuilt_expense_accepts_a_new_form_of_payers() {
    // A single payer into a list.
    let expense = itemised_expense()
        .to_builder()
        .payers(Split::amounts([
            MemberAmount::new(ANA, euros(4_500)),
            MemberAmount::new(BOB, euros(4_500)),
        ]))
        .build()
        .unwrap();
    assert_eq!(
        expense.payers(),
        &Split::amounts([
            MemberAmount::new(ANA, euros(4_500)),
            MemberAmount::new(BOB, euros(4_500))
        ])
    );
    // A list into a single payer.
    let single_payer_expense = full_expense().to_builder().paid_by(CAROL).build().unwrap();
    assert_eq!(single_payer_expense.payers(), &Split::equal([CAROL]));
}

/// Editing an expense's name, cost, or date would be lost or skip
/// validation.
#[test]
fn builder_setters_replace_each_field() {
    let base = || {
        dinner()
            .paid_by(ANA)
            .split(Split::equal([ANA, BOB]))
            .build()
            .unwrap()
    };
    // Name, cost, and date replaced.
    let lunch_date = CalendarDate::new(2026, 9, 24).unwrap();
    let lunch = base()
        .to_builder()
        .name("Lunch")
        .cost(euros(3_000))
        .date(lunch_date)
        .build()
        .unwrap();
    assert_eq!(lunch.name(), "Lunch");
    assert_eq!(lunch.amount(), euros(3_000));
    assert_eq!(lunch.total(), euros(3_000));
    assert_eq!(lunch.date(), lunch_date);
    // A replaced cost is validated again.
    assert_eq!(
        base().to_builder().cost(euros(0)).build(),
        Err(ExpenseError::NonPositiveAmount)
    );
}

/// The exchange rate fixed when a group changes currency would not be
/// stored, could not be removed, or would change something else.
#[test]
fn with_exchange_rate_replaces_or_clears_the_rate() {
    let rate = ExchangeRate::parse("0.2011").unwrap();
    // Replaced, and only the cost changes.
    let expense = full_expense();
    let converted = expense.clone().with_exchange_rate(Some(rate));
    assert_eq!(converted.cost(), euros(9_000).at(rate));
    assert_eq!(
        converted,
        expense
            .to_builder()
            .cost(euros(9_000).at(rate))
            .build()
            .unwrap()
    );
    // Cleared.
    let local = converted.with_exchange_rate(None);
    assert_eq!(local.cost(), Cost::from(euros(9_000)));
}

/// The expenses of a removed category would lose their identifier or
/// another detail when they move to Uncategorised.
#[test]
fn with_category_moves_the_expense_and_keeps_the_rest() {
    let expense = full_expense();
    let moved = expense.clone().with_category(CategoryId::UNCATEGORISED);
    assert_eq!(moved.category(), CategoryId::UNCATEGORISED);
    assert_eq!(
        moved,
        expense
            .to_builder()
            .category(CategoryId::UNCATEGORISED)
            .build()
            .unwrap()
    );
}

/// Saved expenses would fail to load, load with another total, or an
/// invalid expense would load.
#[cfg(feature = "serde")]
#[test]
fn expenses_round_trip_through_serde() {
    // A whole split with every optional field; the total is derived.
    let expense = full_expense();
    let json = serde_json::to_string(&expense).unwrap();
    assert!(!json.contains("total"));
    assert_eq!(serde_json::from_str::<Expense>(&json).unwrap(), expense);
    // An itemised split without a description or a tip.
    let itemised = itemised_expense();
    let itemised_json = serde_json::to_string(&itemised).unwrap();
    assert!(!itemised_json.contains("description"));
    assert!(!itemised_json.contains(r#""tip":"#));
    // A fixed zero tip is not the default, so it is written.
    let zero_tip = itemised
        .to_builder()
        .tip(Tip::fixed(euros(0)))
        .build()
        .unwrap();
    let zero_tip_json = serde_json::to_string(&zero_tip).unwrap();
    assert!(zero_tip_json.contains(r#""tip":{"value":{"fixed":"#));
    assert_eq!(
        serde_json::from_str::<Expense>(&zero_tip_json).unwrap(),
        zero_tip
    );
    assert_eq!(
        serde_json::from_str::<Expense>(&itemised_json).unwrap(),
        itemised
    );
    // A conversion at 1 keeps its rate; a local cost is written without one.
    let pegged = itemised
        .to_builder()
        .cost(euros(9_000).at(ExchangeRate::ONE))
        .build()
        .unwrap();
    let pegged_json = serde_json::to_string(&pegged).unwrap();
    assert!(pegged_json.contains(r#""exchange_rate":"1""#));
    assert!(!itemised_json.contains("exchange_rate"));
    assert_eq!(
        serde_json::from_str::<Expense>(&pegged_json).unwrap(),
        pegged
    );
    // Loading validates like the builder.
    let tampered = itemised_json.replace(
        r#""amount":{"minor_units":9000,"currency":"EUR""#,
        r#""amount":{"minor_units":9001,"currency":"EUR""#,
    );
    assert_ne!(tampered, itemised_json);
    assert!(serde_json::from_str::<Expense>(&tampered).is_err());
    // A record that is not an expense's shape is refused before its checks.
    assert!(serde_json::from_str::<Expense>(r#"{"name":"Dinner"}"#).is_err());
}

/// An expense's division as a crate outside this one converts it.
#[derive(Debug, PartialEq, Eq)]
enum ConvertedExpenseSplit {
    Whole(Split),
    Itemised(Vec<ExpenseItem>),
}

impl FromExpenseSplit for ConvertedExpenseSplit {
    fn whole(split: &Split) -> Self {
        Self::Whole(split.clone())
    }

    fn itemised(items: &[ExpenseItem]) -> Self {
        Self::Itemised(items.to_vec())
    }
}

/// A conversion that called another variant's method would hand the
/// bindings an itemised bill as one split, or one split as a bill.
#[test]
fn an_expense_split_converts_through_the_method_of_its_variant() {
    let split = Split::equal([ANA, BOB]);
    assert_eq!(
        ConvertedExpenseSplit::from_expense_split(&ExpenseSplit::Whole(split.clone())),
        ConvertedExpenseSplit::Whole(split)
    );
    let wine = ExpenseItem::builder("Wine", euros(2_400), [ANA, BOB])
        .build()
        .unwrap();
    assert_eq!(
        ConvertedExpenseSplit::from_expense_split(&ExpenseSplit::Itemised(vec![wine.clone()])),
        ConvertedExpenseSplit::Itemised(vec![wine])
    );
}
