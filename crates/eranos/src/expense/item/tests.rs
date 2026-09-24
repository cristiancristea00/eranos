//! Itemised receipts divide each line among the members who shared it. These
//! tests keep every line valid (a name, a positive price, at least one unit,
//! a split that divides the line total), keep the two ways of naming who
//! shares a line from silently overriding each other, and keep edits made
//! through `to_builder` free to replace the split.

use super::*;

use crate::error::ValueError;
use crate::money::Currency;

// Two members, in ID order.
const ANA: MemberId = MemberId::from_bytes([1; 16]);
const BOB: MemberId = MemberId::from_bytes([2; 16]);

fn euros(minor_units: i64) -> Money {
    Money::from_minor_units(minor_units, Currency::EUR)
}

/// Three beers at 4.50, shared by Ana and Bob.
fn beers() -> Item {
    Item::builder("Beer", euros(450))
        .quantity(3)
        .shared_by([ANA, BOB])
        .build()
        .unwrap()
}

/// A single item would be counted as zero or several units.
#[test]
fn the_quantity_defaults_to_one() {
    // One unit, priced at the unit price.
    let pizza = Item::builder("Pizza", euros(1_200))
        .split(Split::equal([ANA]))
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
    let explicit = Item::builder("Beer", euros(450))
        .quantity(3)
        .split(Split::equal([ANA, BOB]))
        .build()
        .unwrap();
    assert_eq!(beers(), explicit);
    assert_eq!(beers().split(), &Split::equal([ANA, BOB]));
}

/// A line given both a split and an equal sharing would keep one of them
/// silently, hiding the caller's mistake.
#[test]
fn split_and_shared_by_together_conflict() {
    let builder = || Item::builder("Beer", euros(450));
    // `split` first.
    assert_eq!(
        builder()
            .split(Split::equal([ANA]))
            .shared_by([BOB])
            .build(),
        Err(ExpenseError::ConflictingSplit)
    );
    // `shared_by` first.
    assert_eq!(
        builder()
            .shared_by([BOB])
            .split(Split::equal([ANA]))
            .build(),
        Err(ExpenseError::ConflictingSplit)
    );
    // Later calls keep the conflict.
    assert_eq!(
        builder()
            .split(Split::equal([ANA]))
            .shared_by([BOB])
            .split(Split::equal([ANA]))
            .shared_by([BOB])
            .build(),
        Err(ExpenseError::ConflictingSplit)
    );
}

/// Correcting who shares a line before saving it would be refused.
#[test]
fn repeating_a_split_setter_keeps_the_last_value() {
    let builder = || Item::builder("Beer", euros(450));
    // `split` twice.
    let item = builder()
        .split(Split::equal([ANA]))
        .split(Split::equal([BOB]))
        .build()
        .unwrap();
    assert_eq!(item.split(), &Split::equal([BOB]));
    // `shared_by` twice.
    let shared_by_twice = builder().shared_by([ANA]).shared_by([BOB]).build().unwrap();
    assert_eq!(shared_by_twice.split(), &Split::equal([BOB]));
}

/// A line nobody shares would be accepted and its cost would vanish.
#[test]
fn an_item_without_a_split_is_rejected() {
    // Neither setter called.
    assert_eq!(
        Item::builder("Beer", euros(450)).build(),
        Err(ExpenseError::MissingSplit)
    );
}

/// A line without a name would appear blank on the itemised receipt.
#[test]
fn the_item_name_is_trimmed_and_required() {
    // Surrounding whitespace.
    let item = Item::builder("  Beer \n", euros(450))
        .shared_by([ANA])
        .build()
        .unwrap();
    assert_eq!(item.name(), "Beer");
    // Blank names.
    for name in ["", "   "] {
        assert_eq!(
            Item::builder(name, euros(450)).shared_by([ANA]).build(),
            Err(ExpenseError::Value(ValueError::Empty {
                field: Field::ItemName
            })),
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
            Item::builder("Beer", unit_price).shared_by([ANA]).build(),
            Err(ExpenseError::NonPositiveAmount)
        );
    }
    // The smallest positive price.
    assert!(
        Item::builder("Mint", euros(1))
            .shared_by([ANA])
            .build()
            .is_ok()
    );
}

/// A line with zero units would be accepted with a zero total.
#[test]
fn a_zero_quantity_is_rejected() {
    // With a split.
    assert_eq!(
        Item::builder("Beer", euros(450))
            .quantity(0)
            .shared_by([ANA])
            .build(),
        Err(ExpenseError::ZeroQuantity)
    );
    // The quantity is checked before the split.
    assert_eq!(
        Item::builder("Beer", euros(450)).quantity(0).build(),
        Err(ExpenseError::ZeroQuantity)
    );
}

/// A line total too large for 64-bit minor units would wrap around to a
/// wrong amount.
#[test]
fn a_line_total_out_of_range_is_rejected() {
    // Two units of more than half the range.
    assert_eq!(
        Item::builder("Yacht", euros(i64::MAX / 2 + 1))
            .quantity(2)
            .shared_by([ANA])
            .build(),
        Err(ExpenseError::Overflow)
    );
    // One unit of the largest price fits.
    let item = Item::builder("Yacht", euros(i64::MAX))
        .shared_by([ANA])
        .build()
        .unwrap();
    assert_eq!(item.line_total(), euros(i64::MAX));
}

/// A line whose split does not divide its total would charge the members
/// more or less than the line costs.
#[test]
fn the_split_must_divide_the_line_total() {
    let builder = || Item::builder("Beer", euros(450)).quantity(3);
    // Amounts that add up to 13.50.
    let split = Split::amounts([(ANA, euros(1_000)), (BOB, euros(350))]);
    assert!(builder().split(split).build().is_ok());
    // Amounts that add up to the unit price instead.
    let shares_of_the_unit_price = Split::amounts([(ANA, euros(300)), (BOB, euros(150))]);
    assert_eq!(
        builder().split(shares_of_the_unit_price).build(),
        Err(ExpenseError::AmountsDoNotAddUp {
            actual: euros(450),
            expected: euros(1_350)
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

/// Editing who shares a line would be refused as a conflict with the split
/// it already has.
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
    // Both setters on a rebuilt item still conflict.
    assert_eq!(
        item.to_builder()
            .split(Split::equal([BOB]))
            .shared_by([ANA])
            .build(),
        Err(ExpenseError::ConflictingSplit)
    );
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
}
