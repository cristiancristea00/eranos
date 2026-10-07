//! Itemised receipts divide each line among the members who shared it. These
//! tests keep every line valid (a name, a positive price, at least one unit,
//! a split that divides the line total), keep both constructors naming who
//! shares a line, and keep the last split setter in force, on a new item and
//! on one edited through `to_builder`.

use eranos_money::Currency;

use super::*;

use crate::MemberAmount;

// Two members, in ID order.
const ANA: MemberId = MemberId::from_bytes([1; 16]);
const BOB: MemberId = MemberId::from_bytes([2; 16]);

fn euros(minor_units: i64) -> Money {
    Money::from_minor_units(minor_units, Currency::EUR)
}

/// Three beers at 4.50, shared by Ana and Bob.
fn beers() -> Item {
    Item::builder("Beer", euros(450), [ANA, BOB])
        .quantity(3)
        .build()
        .unwrap()
}

/// A single item would be counted as zero or several units.
#[test]
fn the_quantity_defaults_to_one() {
    // One unit, priced at the unit price.
    let pizza = Item::builder_with_split("Pizza", euros(1_200), Split::equal([ANA]))
        .build()
        .unwrap();
    assert_eq!(pizza.quantity(), 1);
    assert_eq!(pizza.line_total(), euros(1_200));
    // The other fields are kept as given.
    assert_eq!(pizza.name(), "Pizza");
    assert_eq!(pizza.unit_price(), euros(1_200));
    assert_eq!(pizza.split(), &Split::equal([ANA]));
}

/// A receipt line would be charged at another amount than price times
/// quantity.
#[test]
fn the_line_total_is_the_unit_price_times_the_quantity() {
    // 3 × 4.50.
    assert_eq!(beers().line_total(), euros(1_350));
    assert_eq!(beers().quantity(), 3);
}

/// Sharing a line equally would divide it differently from an explicit
/// equal split.
#[test]
fn shared_by_is_an_equal_split() {
    // Same item either way.
    let explicit = Item::builder_with_split("Beer", euros(450), Split::equal([ANA, BOB]))
        .quantity(3)
        .build()
        .unwrap();
    assert_eq!(beers(), explicit);
    assert_eq!(beers().split(), &Split::equal([ANA, BOB]));
}

/// Correcting who shares a line before saving it would keep the split the
/// line was started with.
#[test]
fn the_last_split_setter_wins() {
    let builder = || Item::builder("Beer", euros(450), [ANA]);
    // `split` replaces the constructor's equal split.
    let item = builder().split(Split::equal([BOB])).build().unwrap();
    assert_eq!(item.split(), &Split::equal([BOB]));
    // `shared_by` after `split`, and `split` after `shared_by`.
    let shared_by_last = builder()
        .split(Split::equal([ANA]))
        .shared_by([BOB])
        .build()
        .unwrap();
    assert_eq!(shared_by_last.split(), &Split::equal([BOB]));
    let split_last = builder()
        .shared_by([BOB])
        .split(Split::equal([ANA]))
        .build()
        .unwrap();
    assert_eq!(split_last.split(), &Split::equal([ANA]));
    // `shared_by` replaces the split of `builder_with_split`.
    let shared_by_twice = Item::builder_with_split("Beer", euros(450), Split::equal([ANA]))
        .shared_by([BOB])
        .build()
        .unwrap();
    assert_eq!(shared_by_twice.split(), &Split::equal([BOB]));
}

/// A line without a name would appear blank on the itemised receipt.
#[test]
fn the_item_name_is_trimmed_and_required() {
    // Surrounding whitespace.
    let item = Item::builder("  Beer \n", euros(450), [ANA])
        .build()
        .unwrap();
    assert_eq!(item.name(), "Beer");
    // Blank names.
    for name in ["", "   "] {
        assert_eq!(
            Item::builder(name, euros(450), [ANA]).build(),
            Err(ExpenseError::EmptyItemName),
            "{name:?}"
        );
    }
}

/// A free or negatively priced line would be accepted and distort the
/// receipt's total.
#[test]
fn a_non_positive_unit_price_is_rejected() {
    // Zero and negative prices.
    for unit_price in [euros(0), euros(-450)] {
        assert_eq!(
            Item::builder("Beer", unit_price, [ANA]).build(),
            Err(ExpenseError::NonPositiveAmount)
        );
    }
    // The smallest positive price.
    assert!(Item::builder("Mint", euros(1), [ANA]).build().is_ok());
}

/// A line with zero units would be accepted with a zero total.
#[test]
fn a_zero_quantity_is_rejected() {
    // With a split.
    assert_eq!(
        Item::builder("Beer", euros(450), [ANA]).quantity(0).build(),
        Err(ExpenseError::ZeroQuantity)
    );
    // The quantity is checked before the split, which would not divide a
    // line total of zero.
    let one_cent = Split::amounts([MemberAmount::new(ANA, euros(1))]);
    assert_eq!(
        Item::builder_with_split("Beer", euros(450), one_cent)
            .quantity(0)
            .build(),
        Err(ExpenseError::ZeroQuantity)
    );
}

/// A line total too large for 64-bit minor units would wrap around to a
/// wrong amount.
#[test]
fn a_line_total_out_of_range_is_rejected() {
    // Two units of more than half the range.
    assert_eq!(
        Item::builder("Yacht", euros(i64::MAX / 2 + 1), [ANA])
            .quantity(2)
            .build(),
        Err(ExpenseError::Overflow)
    );
    // One unit of the largest price fits.
    let item = Item::builder("Yacht", euros(i64::MAX), [ANA])
        .build()
        .unwrap();
    assert_eq!(item.line_total(), euros(i64::MAX));
}

/// A line whose split does not divide its total would charge the members
/// more or less than the line costs.
#[test]
fn the_split_must_divide_the_line_total() {
    let builder = || Item::builder("Beer", euros(450), [ANA]).quantity(3);
    // Amounts that add up to 13.50.
    let split = Split::amounts([
        MemberAmount::new(ANA, euros(1_000)),
        MemberAmount::new(BOB, euros(350)),
    ]);
    assert!(builder().split(split).build().is_ok());
    // Amounts that add up to the unit price instead.
    let shares_of_the_unit_price = Split::amounts([
        MemberAmount::new(ANA, euros(300)),
        MemberAmount::new(BOB, euros(150)),
    ]);
    assert_eq!(
        builder().split(shares_of_the_unit_price).build(),
        Err(ExpenseError::AmountsDoNotAddUp {
            expected: euros(1_350),
            actual: euros(450)
        })
    );
    // Any other invalid split.
    assert_eq!(
        builder().shared_by([]).build(),
        Err(ExpenseError::EmptySplit)
    );
}

/// Editing a line would silently drop fields that were not edited.
#[test]
fn to_builder_rebuilds_an_equal_item() {
    // Rebuilt unchanged.
    assert_eq!(beers().to_builder().build().unwrap(), beers());
}

/// Editing who shares a line would keep the split it already has.
#[test]
fn a_rebuilt_item_accepts_a_new_split() {
    // `split` replaces a split set through `shared_by`.
    let item = beers()
        .to_builder()
        .split(Split::equal([BOB]))
        .build()
        .unwrap();
    assert_eq!(item.split(), &Split::equal([BOB]));
    // `shared_by` replaces a split set through `split`.
    let item = item.to_builder().shared_by([ANA]).build().unwrap();
    assert_eq!(item.split(), &Split::equal([ANA]));
}

/// Editing a line's name, price, or quantity would be lost or change other
/// fields.
#[test]
fn builder_setters_replace_each_field() {
    // Every field replaced.
    let wine = beers()
        .to_builder()
        .name("Wine")
        .unit_price(euros(2_400))
        .quantity(2)
        .build()
        .unwrap();
    assert_eq!(wine.name(), "Wine");
    assert_eq!(wine.unit_price(), euros(2_400));
    assert_eq!(wine.quantity(), 2);
    assert_eq!(wine.line_total(), euros(4_800));
    // The split is kept.
    assert_eq!(wine.split(), beers().split());
}

/// Saved receipts would fail to load, or an invalid line would load.
#[cfg(feature = "serde")]
#[test]
fn items_round_trip_through_serde() {
    // Round trip.
    let json = serde_json::to_string(&beers()).unwrap();
    assert_eq!(serde_json::from_str::<Item>(&json).unwrap(), beers());
    // Loading validates like the builder.
    let zero_quantity = json.replace(r#""quantity":3"#, r#""quantity":0"#);
    assert_ne!(zero_quantity, json);
    assert!(serde_json::from_str::<Item>(&zero_quantity).is_err());
    // A record that is not an item's shape is refused before its checks.
    assert!(serde_json::from_str::<Item>(r#"{"name":"Beer"}"#).is_err());
}
