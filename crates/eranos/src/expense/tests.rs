//! An expense records who paid, who shares the cost, and any tip on top.
//! These tests keep invalid expenses out (a blank name, a non-positive
//! amount, payers or splits missing or set twice, items or shares that do not
//! add up), keep the payers covering the amount plus the tip while the
//! participants divide the amount alone, and keep edits made through
//! `to_builder` free to switch between the forms of payers and of splits.

use super::*;

use crate::error::ValueError;
use crate::money::{Currency, Percentage};

// Four members, in ID order.
const ANA: MemberId = MemberId::from_bytes([1; 16]);
const BOB: MemberId = MemberId::from_bytes([2; 16]);
const CAROL: MemberId = MemberId::from_bytes([3; 16]);
const DAN: MemberId = MemberId::from_bytes([4; 16]);

fn euros(minor_units: i64) -> Money {
    Money::from_minor_units(minor_units, Currency::EUR)
}

fn percent(text: &str) -> Percentage {
    Percentage::parse(text).unwrap()
}

fn september_23() -> Date {
    Date::new(2026, 9, 23).unwrap()
}

/// A 90.00 EUR dinner on 23 September 2026, without payers or a split.
fn dinner() -> ExpenseBuilder {
    Expense::builder("Dinner", euros(9_000), september_23())
}

/// A line of `minor_units` shared by Ana.
fn item(name: &str, minor_units: i64) -> Item {
    Item::builder(name, euros(minor_units))
        .shared_by([ANA])
        .build()
        .unwrap()
}

/// The dinner with every optional field set: a 10 % tip, Ana and Bob paying
/// the 99.00 total between them, and three members sharing the 90.00 by
/// percentage.
fn full_expense() -> Expense {
    dinner()
        .description("Birthday")
        .category(CategoryId::from_bytes([9; 16]))
        .tip(Tip::percentage(percent("10")))
        .time(Time::new(20, 30).unwrap())
        .exchange_rate(ExchangeRate::parse("4.9765").unwrap())
        .payers(Split::amounts([(ANA, euros(5_000)), (BOB, euros(4_900))]))
        .split(Split::percentages([
            (ANA, percent("50")),
            (BOB, percent("25")),
            (CAROL, percent("25")),
        ]))
        .build()
        .unwrap()
}

/// The dinner as a receipt paid by Bob: a pizza for Ana and two glasses of
/// wine for Ana and Carol.
fn itemized_expense() -> Expense {
    let wine = Item::builder("Wine", euros(2_000))
        .quantity(2)
        .shared_by([ANA, CAROL])
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
    // Required fields.
    assert_eq!(expense.name(), "Dinner");
    assert_eq!(expense.amount(), euros(9_000));
    assert_eq!(expense.date(), september_23());
    assert_eq!(expense.payers(), &Split::equal([ANA]));
    assert_eq!(
        expense.split(),
        &ExpenseSplit::Whole(Split::equal([ANA, BOB, CAROL]))
    );
    // Optional fields are absent.
    assert_eq!(expense.description(), None);
    assert_eq!(expense.category(), None);
    assert_eq!(expense.tip(), None);
    assert_eq!(expense.time(), None);
    assert_eq!(expense.exchange_rate(), None);
    // Without a tip, the total is the amount.
    assert_eq!(expense.total(), euros(9_000));
    assert_eq!(expense.tip_amount(), euros(0));
}

/// Expenses would be listed with stray spaces or with no name at all.
#[test]
fn the_expense_name_is_trimmed_and_required() {
    // Surrounding whitespace.
    let expense = Expense::builder("  Dinner \n", euros(9_000), september_23())
        .paid_by(ANA)
        .split(Split::equal([ANA]))
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
            Err(ExpenseError::Value(ValueError::Empty {
                field: Field::ExpenseName
            })),
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
                .amount(amount)
                .paid_by(ANA)
                .split(Split::equal([ANA]))
                .build(),
            Err(ExpenseError::NonPositiveAmount)
        );
    }
}

/// An expense nobody paid for, or that nobody shares, would be accepted and
/// its amount would vanish from the balances.
#[test]
fn payers_and_a_split_are_required() {
    // No payers.
    assert_eq!(
        dinner().split(Split::equal([ANA])).build(),
        Err(ExpenseError::MissingPayers)
    );
    // No split.
    assert_eq!(
        dinner().paid_by(ANA).build(),
        Err(ExpenseError::MissingSplit)
    );
}

/// An expense given both a single payer and a list of payers would keep one
/// of them silently, hiding the caller's mistake.
#[test]
fn paid_by_and_payers_together_conflict() {
    let payers = Split::amounts([(ANA, euros(9_000))]);
    // `paid_by` first.
    assert_eq!(
        dinner()
            .paid_by(ANA)
            .payers(payers.clone())
            .split(Split::equal([ANA]))
            .build(),
        Err(ExpenseError::ConflictingPayers)
    );
    // `payers` first.
    assert_eq!(
        dinner()
            .payers(payers)
            .paid_by(ANA)
            .split(Split::equal([ANA]))
            .build(),
        Err(ExpenseError::ConflictingPayers)
    );
}

/// An expense divided both as a whole and item by item would keep one of
/// the two silently, hiding the caller's mistake.
#[test]
fn split_and_items_together_conflict() {
    // `split` first.
    assert_eq!(
        dinner()
            .paid_by(ANA)
            .split(Split::equal([ANA]))
            .items([item("Pizza", 9_000)])
            .build(),
        Err(ExpenseError::ConflictingSplit)
    );
    // `items` first.
    assert_eq!(
        dinner()
            .paid_by(ANA)
            .items([item("Pizza", 9_000)])
            .split(Split::equal([ANA]))
            .build(),
        Err(ExpenseError::ConflictingSplit)
    );
}

/// A conflict would be forgotten when a setter is called once more after
/// it.
#[test]
fn a_later_setter_call_keeps_the_conflict() {
    let payers = || Split::amounts([(ANA, euros(9_000))]);
    // Payers: either setter called again.
    for builder in [
        dinner().paid_by(ANA).payers(payers()).paid_by(ANA),
        dinner().payers(payers()).paid_by(ANA).payers(payers()),
    ] {
        assert_eq!(
            builder.split(Split::equal([ANA])).build(),
            Err(ExpenseError::ConflictingPayers)
        );
    }
    // Splits: either setter called again.
    for builder in [
        dinner()
            .split(Split::equal([ANA]))
            .items([item("Pizza", 9_000)])
            .split(Split::equal([ANA])),
        dinner()
            .items([item("Pizza", 9_000)])
            .split(Split::equal([ANA]))
            .items([item("Pizza", 9_000)]),
    ] {
        assert_eq!(
            builder.paid_by(ANA).build(),
            Err(ExpenseError::ConflictingSplit)
        );
    }
}

/// Correcting the payers or the split before saving would be refused.
#[test]
fn repeating_a_setter_keeps_the_last_value() {
    // `paid_by` twice.
    let expense = dinner()
        .paid_by(ANA)
        .paid_by(BOB)
        .split(Split::equal([ANA]))
        .build()
        .unwrap();
    assert_eq!(expense.payers(), &Split::equal([BOB]));
    // `payers` twice.
    let payers_twice = dinner()
        .payers(Split::amounts([(ANA, euros(9_000))]))
        .payers(Split::amounts([(BOB, euros(9_000))]))
        .split(Split::equal([ANA]))
        .build()
        .unwrap();
    assert_eq!(
        payers_twice.payers(),
        &Split::amounts([(BOB, euros(9_000))])
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
        &ExpenseSplit::Itemized(vec![item("Wine", 9_000)])
    );
}

/// An itemized expense without lines would be accepted and nobody would
/// share its cost.
#[test]
fn an_itemized_expense_without_items_is_rejected() {
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
    assert!(matches!(expense.split(), ExpenseSplit::Itemized(items) if items.len() == 2));
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
    let dollars = Item::builder("Pizza", Money::from_minor_units(9_000, Currency::USD))
        .shared_by([ANA])
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
    let tipped = || dinner().paid_by(ANA).tip(Tip::percentage(percent("10")));
    // Shares of the 90.00 amount.
    let split = Split::amounts([(ANA, euros(4_500)), (BOB, euros(4_500))]);
    assert!(tipped().split(split).build().is_ok());
    // Shares of the 99.00 total.
    let shares_of_the_total = Split::amounts([(ANA, euros(4_950)), (BOB, euros(4_950))]);
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
            .tip(Tip::percentage(percent("10")))
    };
    // Payments adding up to the 99.00 total.
    let payers = Split::amounts([(ANA, euros(5_000)), (BOB, euros(4_900))]);
    assert!(tipped().payers(payers).build().is_ok());
    // Payments adding up to the 90.00 amount only.
    assert_eq!(
        tipped()
            .payers(Split::amounts([(ANA, euros(9_000))]))
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
    let expense = tipped(Tip::percentage(percent("10")));
    assert_eq!(expense.tip(), Some(Tip::percentage(percent("10"))));
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
    let cleared_tip_expense = zero_tip_expense.to_builder().tip(None).build().unwrap();
    assert_eq!(cleared_tip_expense.tip(), None);
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
        Expense::builder("Yacht", euros(i64::MAX), september_23())
            .paid_by(ANA)
            .split(Split::equal([ANA]))
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
        largest().tip(Tip::percentage(percent("200"))).build(),
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
    assert_eq!(expense.category(), Some(CategoryId::from_bytes([9; 16])));
    assert_eq!(expense.time(), Some(Time::new(20, 30).unwrap()));
    assert_eq!(
        expense.exchange_rate(),
        Some(ExchangeRate::parse("4.9765").unwrap())
    );
    // Cleared.
    let expense = expense
        .to_builder()
        .category(None)
        .time(None)
        .exchange_rate(None)
        .build()
        .unwrap();
    assert_eq!(expense.category(), None);
    assert_eq!(expense.time(), None);
    assert_eq!(expense.exchange_rate(), None);
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
    // Itemized expenses.
    assert_eq!(
        itemized_expense().members(),
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
        itemized_expense().split().participants(),
        BTreeSet::from([ANA, CAROL])
    );
}

/// Editing an expense would silently drop fields that were not edited.
#[test]
fn to_builder_rebuilds_an_equal_expense() {
    // Every optional field set, whole split.
    let expense = full_expense();
    assert_eq!(expense.to_builder().build().unwrap(), expense);
    // Itemized split.
    let itemized = itemized_expense();
    assert_eq!(itemized.to_builder().build().unwrap(), itemized);
}

/// Turning a whole expense into a receipt, or back, would be refused as a
/// conflict with the split it already has.
#[test]
fn a_rebuilt_expense_accepts_a_new_form_of_split() {
    // Whole into itemized.
    let expense = full_expense()
        .to_builder()
        .items([item("Pizza", 9_000)])
        .build()
        .unwrap();
    assert_eq!(
        expense.split(),
        &ExpenseSplit::Itemized(vec![item("Pizza", 9_000)])
    );
    // Itemized into whole.
    let whole_expense = itemized_expense()
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
    let expense = itemized_expense()
        .to_builder()
        .payers(Split::amounts([(ANA, euros(4_500)), (BOB, euros(4_500))]))
        .build()
        .unwrap();
    assert_eq!(
        expense.payers(),
        &Split::amounts([(ANA, euros(4_500)), (BOB, euros(4_500))])
    );
    // A list into a single payer.
    let single_payer_expense = full_expense().to_builder().paid_by(CAROL).build().unwrap();
    assert_eq!(single_payer_expense.payers(), &Split::equal([CAROL]));
}

/// Setting both forms on a rebuilt expense would keep one of them silently.
#[test]
fn a_rebuilt_expense_still_rejects_both_forms() {
    // Both forms of payers.
    assert_eq!(
        full_expense()
            .to_builder()
            .paid_by(ANA)
            .payers(Split::equal([BOB]))
            .build(),
        Err(ExpenseError::ConflictingPayers)
    );
    // Both forms of split.
    assert_eq!(
        full_expense()
            .to_builder()
            .split(Split::equal([ANA]))
            .items([item("Pizza", 9_000)])
            .build(),
        Err(ExpenseError::ConflictingSplit)
    );
}

/// Editing an expense's name, amount, or date would be lost or skip
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
    // Name, amount, and date replaced.
    let lunch_date = Date::new(2026, 9, 24).unwrap();
    let lunch = base()
        .to_builder()
        .name("Lunch")
        .amount(euros(3_000))
        .date(lunch_date)
        .build()
        .unwrap();
    assert_eq!(lunch.name(), "Lunch");
    assert_eq!(lunch.amount(), euros(3_000));
    assert_eq!(lunch.total(), euros(3_000));
    assert_eq!(lunch.date(), lunch_date);
    // A replaced amount is validated again.
    assert_eq!(
        base().to_builder().amount(euros(0)).build(),
        Err(ExpenseError::NonPositiveAmount)
    );
}

/// The exchange rate fixed when a group changes currency would not be
/// stored, or could not be removed.
#[test]
fn set_exchange_rate_replaces_or_clears_the_rate() {
    let rate = ExchangeRate::parse("0.2011").unwrap();
    // Replaced.
    let mut expense = full_expense();
    expense.set_exchange_rate(Some(rate));
    assert_eq!(expense.exchange_rate(), Some(rate));
    // Cleared.
    expense.set_exchange_rate(None);
    assert_eq!(expense.exchange_rate(), None);
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
    // An itemized split without a description.
    let itemized = itemized_expense();
    let itemized_json = serde_json::to_string(&itemized).unwrap();
    assert!(!itemized_json.contains("description"));
    assert_eq!(
        serde_json::from_str::<Expense>(&itemized_json).unwrap(),
        itemized
    );
    // Loading validates like the builder.
    let tampered = itemized_json.replace(
        r#""amount":{"minor_units":9000"#,
        r#""amount":{"minor_units":9001"#,
    );
    assert_ne!(tampered, itemized_json);
    assert!(serde_json::from_str::<Expense>(&tampered).is_err());
}
