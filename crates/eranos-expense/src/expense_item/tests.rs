//! Itemised receipts divide each line among the members who shared it. These
//! tests keep every line valid (a name, a positive price, a line total that
//! is the price times the quantity rounded half away from zero less its
//! discount, free when that comes to zero, a split that divides it), keep
//! both constructors naming who shares a line, and keep the last split
//! setter and the last discount in force, on a new item and on one edited
//! through `to_builder`.

use eranos_money::{CurrencyCode, Percentage};
use proptest::prelude::*;

use super::*;

use crate::{DiscountScope, MemberAmount};

// Two members, in identifier order.

/// Ana, a member.
fn ana() -> MemberId {
    MemberId::new("ana").unwrap()
}

/// Bob, a member.
fn bob() -> MemberId {
    MemberId::new("bob").unwrap()
}

fn euros(minor_units: i64) -> Money {
    Money::from_minor_units(minor_units, CurrencyCode::EUR)
}

/// `count` whole units.
fn units(count: u64) -> Quantity {
    Quantity::from_whole(count).unwrap()
}

/// The quantity written `text`.
fn quantity(text: &str) -> Quantity {
    Quantity::parse(text).unwrap()
}

/// Three beers at 4.50, shared by Ana and Bob.
fn beers() -> ExpenseItem {
    ExpenseItem::builder("Beer", euros(450), [ana(), bob()])
        .quantity(units(3))
        .build()
        .unwrap()
}

/// A single item would be counted as zero or several units.
#[test]
fn the_quantity_defaults_to_one() {
    // One unit, priced at the unit price.
    let pizza = ExpenseItem::builder("Pizza", euros(1_200), Split::equal([ana()]))
        .build()
        .unwrap();
    assert_eq!(pizza.quantity(), Quantity::ONE);
    assert_eq!(pizza.line_total(), euros(1_200));
    // The other fields are kept as given.
    assert_eq!(pizza.name(), "Pizza");
    assert_eq!(pizza.unit_price(), euros(1_200));
    assert_eq!(pizza.split(), &Split::equal([ana()]));
}

/// A receipt line would be charged at another amount than price times
/// quantity.
#[test]
fn the_line_total_is_the_unit_price_times_the_quantity() {
    // 3 × 4.50.
    assert_eq!(beers().line_total(), euros(1_350));
    assert_eq!(beers().quantity(), units(3));
}

/// A line weighed or measured in fractions would be charged another amount
/// than the receipt prints: rounded towards zero, to even, or to a whole
/// unit.
#[test]
fn a_fractional_line_total_rounds_half_away_from_zero() {
    let line = |unit_price, text| {
        ExpenseItem::builder("Cheese", euros(unit_price), [ana()])
            .quantity(quantity(text))
            .build()
            .map(|item| item.line_total())
    };
    // 0.35 kg at 42.00 is exact.
    assert_eq!(line(4_200, "0.35"), Ok(euros(1_470)));
    // 0.335 at 1.00 is 33.5 cents, which rounds up.
    assert_eq!(line(100, "0.335"), Ok(euros(34)));
    // 0.452 at 12.99 is 5.87148, which rounds down.
    assert_eq!(line(1_299, "0.452"), Ok(euros(587)));
    // 1.5 at 4.50, and the smallest quantity at the largest price.
    assert_eq!(line(450, "1.5"), Ok(euros(675)));
    assert_eq!(line(i64::MAX, "0.0001"), Ok(euros(922_337_203_685_478)));
}

/// Sharing a line equally would divide it differently from an explicit
/// equal split.
#[test]
fn shared_by_is_an_equal_split() {
    // Same item either way.
    let explicit = ExpenseItem::builder("Beer", euros(450), Split::equal([ana(), bob()]))
        .quantity(units(3))
        .build()
        .unwrap();
    assert_eq!(beers(), explicit);
    assert_eq!(beers().split(), &Split::equal([ana(), bob()]));
}

/// Correcting who shares a line before saving it would keep the split the
/// line was started with.
#[test]
fn the_last_split_setter_wins() {
    let builder = || ExpenseItem::builder("Beer", euros(450), [ana()]);
    // `split` replaces the constructor's equal split.
    let item = builder().split(Split::equal([bob()])).build().unwrap();
    assert_eq!(item.split(), &Split::equal([bob()]));
    // `shared_by` after `split`, and `split` after `shared_by`.
    let shared_by_last = builder()
        .split(Split::equal([ana()]))
        .shared_by([bob()])
        .build()
        .unwrap();
    assert_eq!(shared_by_last.split(), &Split::equal([bob()]));
    let split_last = builder()
        .shared_by([bob()])
        .split(Split::equal([ana()]))
        .build()
        .unwrap();
    assert_eq!(split_last.split(), &Split::equal([ana()]));
    // `shared_by` replaces the split of `builder`.
    let shared_by_twice = ExpenseItem::builder("Beer", euros(450), Split::equal([ana()]))
        .shared_by([bob()])
        .build()
        .unwrap();
    assert_eq!(shared_by_twice.split(), &Split::equal([bob()]));
}

/// A line without a name would appear blank on the itemised receipt.
#[test]
fn the_item_name_is_trimmed_and_required() {
    // Surrounding whitespace.
    let item = ExpenseItem::builder("  Beer \n", euros(450), [ana()])
        .build()
        .unwrap();
    assert_eq!(item.name(), "Beer");
    // Blank names.
    for name in ["", "   "] {
        assert_eq!(
            ExpenseItem::builder(name, euros(450), [ana()]).build(),
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
            ExpenseItem::builder("Beer", unit_price, [ana()]).build(),
            Err(ExpenseError::NonPositiveAmount)
        );
    }
    // The smallest positive price.
    assert!(
        ExpenseItem::builder("Mint", euros(1), [ana()])
            .build()
            .is_ok()
    );
}

/// A line whose quantity is too small to cost a minor unit would be
/// refused though the receipt lists it, or charged a cent it does not cost;
/// and a split of amounts on it would charge its members for a line that
/// costs nothing.
#[test]
fn a_line_that_rounds_to_zero_is_free() {
    // 0.4 of a cent rounds to zero: a free line, shared as any other.
    let free = ExpenseItem::builder("Mint", euros(1), [ana(), bob()])
        .quantity(quantity("0.4"))
        .build()
        .unwrap();
    assert_eq!(free.line_total(), euros(0));
    assert_eq!(free.quantity(), quantity("0.4"));
    // Half a cent rounds up to one.
    assert_eq!(
        ExpenseItem::builder("Mint", euros(1), [ana()])
            .quantity(quantity("0.5"))
            .build()
            .map(|item| item.line_total()),
        Ok(euros(1))
    );
    // Positive amounts never add up to a free line's total of zero, and a
    // zero amount is refused as a share of any line.
    let one_cent = Split::amounts([MemberAmount::new(ana(), euros(1))]);
    assert_eq!(
        ExpenseItem::builder("Mint", euros(1), one_cent)
            .quantity(quantity("0.0001"))
            .build(),
        Err(ExpenseError::AmountsDoNotAddUp {
            expected: euros(0),
            actual: euros(1)
        })
    );
    let nothing = Split::amounts([MemberAmount::new(ana(), euros(0))]);
    assert_eq!(
        ExpenseItem::builder("Mint", euros(1), nothing)
            .quantity(quantity("0.0001"))
            .build(),
        Err(ExpenseError::NonPositiveShare { member: ana() })
    );
}

/// A line total too large for 64-bit minor units would wrap around to a
/// wrong amount.
#[test]
fn a_line_total_out_of_range_is_rejected() {
    // Two units of more than half the range.
    assert_eq!(
        ExpenseItem::builder("Yacht", euros(i64::MAX / 2 + 1), [ana()])
            .quantity(units(2))
            .build(),
        Err(ExpenseError::Overflow)
    );
    // A ten-thousandth more than one unit of the largest price.
    assert_eq!(
        ExpenseItem::builder("Yacht", euros(i64::MAX), [ana()])
            .quantity(quantity("1.0001"))
            .build(),
        Err(ExpenseError::Overflow)
    );
    // One unit of the largest price fits.
    let item = ExpenseItem::builder("Yacht", euros(i64::MAX), [ana()])
        .build()
        .unwrap();
    assert_eq!(item.line_total(), euros(i64::MAX));
}

/// A line whose split does not divide its total would charge the members
/// more or less than the line costs.
#[test]
fn the_split_must_divide_the_line_total() {
    let builder = || ExpenseItem::builder("Beer", euros(450), [ana()]).quantity(units(3));
    // Amounts that add up to 13.50.
    let split = Split::amounts([
        MemberAmount::new(ana(), euros(1_000)),
        MemberAmount::new(bob(), euros(350)),
    ]);
    assert!(builder().split(split).build().is_ok());
    // Amounts that add up to the unit price instead.
    let shares_of_the_unit_price = Split::amounts([
        MemberAmount::new(ana(), euros(300)),
        MemberAmount::new(bob(), euros(150)),
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
        .split(Split::equal([bob()]))
        .build()
        .unwrap();
    assert_eq!(item.split(), &Split::equal([bob()]));
    // `shared_by` replaces a split set through `split`.
    let item = item.to_builder().shared_by([ana()]).build().unwrap();
    assert_eq!(item.split(), &Split::equal([ana()]));
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
        .quantity(units(2))
        .build()
        .unwrap();
    assert_eq!(wine.name(), "Wine");
    assert_eq!(wine.unit_price(), euros(2_400));
    assert_eq!(wine.quantity(), units(2));
    assert_eq!(wine.line_total(), euros(4_800));
    // The split is kept.
    assert_eq!(wine.split(), beers().split());
    // The last quantity wins, and the line total follows it.
    let half = wine
        .to_builder()
        .quantity(units(3))
        .quantity(quantity("0.5"))
        .build()
        .unwrap();
    assert_eq!(half.quantity(), quantity("0.5"));
    assert_eq!(half.line_total(), euros(1_200));
}

/// A discount would leave the line total as it was, or the receipt's
/// columns, before the discount, the discount, and the line total, would
/// not add up.
#[test]
fn a_discount_reduces_the_line_total() {
    let lemonade =
        || ExpenseItem::builder("Lemonade", euros(333), [ana(), bob()]).quantity(units(3));
    let half = Discount::percentage(Percentage::parse("50").unwrap());
    // 50 % off the line: 9.99 − 5.00.
    let off_the_line = lemonade().discount(half).build().unwrap();
    assert_eq!(off_the_line.line_total_before_discount(), euros(999));
    assert_eq!(off_the_line.discount(), Some(half));
    assert_eq!(off_the_line.discount_amount(), euros(500));
    assert_eq!(off_the_line.line_total(), euros(499));
    // 50 % off each unit: 3 × (3.33 − 1.67).
    let off_each_unit = lemonade()
        .discount(half.scoped(DiscountScope::Unit))
        .build()
        .unwrap();
    assert_eq!(off_each_unit.line_total_before_discount(), euros(999));
    assert_eq!(off_each_unit.discount_amount(), euros(501));
    assert_eq!(off_each_unit.line_total(), euros(498));
    // Without a discount, nothing is taken off.
    assert_eq!(beers().discount(), None);
    assert_eq!(beers().discount_amount(), euros(0));
    assert_eq!(beers().line_total_before_discount(), beers().line_total());
}

/// A dish on the house would be refused, charged to its members, or shared
/// by amounts that charge them for a line that costs nothing.
#[test]
fn a_discount_of_the_whole_price_makes_a_free_line() {
    let on_the_house = Discount::percentage(Percentage::HUNDRED);
    let free = ExpenseItem::builder("Papanași", euros(900), [ana(), bob()])
        .discount(on_the_house)
        .build()
        .unwrap();
    assert_eq!(free.line_total_before_discount(), euros(900));
    assert_eq!(free.discount_amount(), euros(900));
    assert_eq!(free.line_total(), euros(0));
    // Positive amounts never add up to a free line's total of zero.
    let shares = Split::amounts([MemberAmount::new(ana(), euros(900))]);
    assert_eq!(
        ExpenseItem::builder("Papanași", euros(900), shares)
            .discount(on_the_house)
            .build(),
        Err(ExpenseError::AmountsDoNotAddUp {
            expected: euros(0),
            actual: euros(900)
        })
    );
}

/// A discount the line cannot take would be stored and break the receipt:
/// one larger than what it reduces, above 100 %, below zero, or in another
/// currency.
#[test]
fn a_discount_the_line_cannot_take_is_refused() {
    let lemonade = || ExpenseItem::builder("Lemonade", euros(300), [ana()]).quantity(units(3));
    // 10.00 off a line of 9.00, and 3.01 off a unit of 3.00.
    assert_eq!(
        lemonade().discount(Discount::fixed(euros(1_000))).build(),
        Err(ExpenseError::DiscountAbovePrice)
    );
    assert_eq!(
        lemonade()
            .discount(Discount::fixed(euros(301)).scoped(DiscountScope::Unit))
            .build(),
        Err(ExpenseError::DiscountAbovePrice)
    );
    // A basis point above 100 %.
    assert_eq!(
        lemonade()
            .discount(Discount::percentage(Percentage::from_basis_points(10_001)))
            .build(),
        Err(ExpenseError::DiscountAboveHundredPercent)
    );
    // A cent below zero.
    assert_eq!(
        lemonade().discount(Discount::fixed(euros(-1))).build(),
        Err(ExpenseError::NegativeDiscount)
    );
    // A dollar.
    let dollar = Money::from_minor_units(100, CurrencyCode::USD);
    assert_eq!(
        lemonade().discount(Discount::fixed(dollar)).build(),
        Err(ExpenseError::CurrencyMismatch {
            expected: CurrencyCode::EUR,
            found: CurrencyCode::USD
        })
    );
    // The line is checked before its split.
    assert_eq!(
        lemonade()
            .shared_by([])
            .discount(Discount::fixed(euros(1_000)))
            .build(),
        Err(ExpenseError::DiscountAbovePrice)
    );
}

/// Editing a line would drop its discount, keep one the edit removed, or
/// keep the first of two discounts set.
#[test]
fn to_builder_keeps_the_discount_and_none_clears_it() {
    let tenth = Discount::percentage(Percentage::parse("10").unwrap());
    let discounted = beers().to_builder().discount(tenth).build().unwrap();
    // Rebuilt unchanged, discount included.
    assert_eq!(discounted.to_builder().build().unwrap(), discounted);
    assert_eq!(discounted.line_total(), euros(1_215));
    // `None` clears it, and the line total follows.
    let cleared = discounted.to_builder().discount(None).build().unwrap();
    assert_eq!(cleared, beers());
    // The last discount wins.
    let unit = Discount::fixed(euros(50)).scoped(DiscountScope::Unit);
    let last = beers()
        .to_builder()
        .discount(tenth)
        .discount(unit)
        .build()
        .unwrap();
    assert_eq!(last.discount(), Some(unit));
    assert_eq!(last.line_total(), euros(1_200));
}

/// Saved receipts would fail to load, or an invalid line would load.
#[cfg(feature = "serde")]
#[test]
fn items_round_trip_through_serde() {
    // Round trip.
    let json = serde_json::to_string(&beers()).unwrap();
    assert_eq!(serde_json::from_str::<ExpenseItem>(&json).unwrap(), beers());
    // Loading validates like the builder.
    let zero_quantity = json.replace(r#""quantity":"3""#, r#""quantity":"0""#);
    assert_ne!(zero_quantity, json);
    assert!(serde_json::from_str::<ExpenseItem>(&zero_quantity).is_err());
    // A record that is not an item's shape is refused before its checks.
    assert!(serde_json::from_str::<ExpenseItem>(r#"{"name":"Beer"}"#).is_err());
}

/// A saved quantity written as a number would lose its decimals in a
/// reader that parses JSON numbers as doubles, a line total written beside
/// it could disagree with it, and a quantity of one written on every line
/// would lengthen every saved receipt.
#[cfg(feature = "serde")]
#[test]
fn a_quantity_is_saved_as_text_and_left_out_at_one() {
    let cheese = ExpenseItem::builder("Cheese", euros(4_200), [ana()])
        .quantity(quantity("0.35"))
        .build()
        .unwrap();
    let saved_cheese = serde_json::to_value(&cheese).unwrap();
    assert_eq!(saved_cheese["quantity"], serde_json::json!("0.35"));
    assert_eq!(saved_cheese.get("line_total"), None);
    assert_eq!(
        serde_json::from_value::<ExpenseItem>(saved_cheese).unwrap(),
        cheese
    );
    // One unit is left out, and a record without a quantity reads as one.
    let pizza = ExpenseItem::builder("Pizza", euros(1_200), [ana()])
        .build()
        .unwrap();
    let saved_pizza = serde_json::to_value(&pizza).unwrap();
    assert_eq!(saved_pizza.get("quantity"), None);
    let loaded = serde_json::from_value::<ExpenseItem>(saved_pizza).unwrap();
    assert_eq!(loaded.quantity(), Quantity::ONE);
    assert_eq!(loaded.line_total(), euros(1_200));
}

/// A line's discount would be lost on saving, written on every line without
/// one, or saved beside totals that could disagree with it; and a tampered
/// discount would load into a line that costs less than nothing.
#[cfg(feature = "serde")]
#[test]
fn a_discount_is_saved_when_there_is_one() {
    let lemonade = ExpenseItem::builder("Lemonade", euros(300), [ana()])
        .quantity(units(3))
        .discount(Discount::fixed(euros(100)).scoped(DiscountScope::Unit))
        .build()
        .unwrap();
    let saved = serde_json::to_value(&lemonade).unwrap();
    assert_eq!(
        saved["discount"],
        serde_json::json!({"value": {"fixed": {"minor_units": 100, "currency": "EUR"}}, "scope": "unit"})
    );
    assert_eq!(saved.get("line_total_before_discount"), None);
    assert_eq!(saved.get("line_total"), None);
    assert_eq!(
        serde_json::from_value::<ExpenseItem>(saved.clone()).unwrap(),
        lemonade
    );
    // A line without a discount writes none, and reads back without one.
    let saved_beers = serde_json::to_value(beers()).unwrap();
    assert_eq!(saved_beers.get("discount"), None);
    assert_eq!(
        serde_json::from_value::<ExpenseItem>(saved_beers)
            .unwrap()
            .discount(),
        None
    );
    // Loading checks the discount as the builder does.
    let mut above = saved;
    above["discount"]["value"]["fixed"]["minor_units"] = serde_json::json!(301);
    let refused = serde_json::from_value::<ExpenseItem>(above).unwrap_err();
    assert!(
        refused
            .to_string()
            .contains("the discount is larger than the price it reduces"),
        "{refused}"
    );
}

proptest! {
    /// A whole quantity whose line total differed from the unit price times
    /// the count would change the totals, and the stored shares, of saved
    /// itemised bills.
    #[test]
    fn a_whole_quantity_gives_the_exact_product(
        unit_price in 1_i64..1_000_000_000,
        count in 1_u64..10_000,
    ) {
        let item = ExpenseItem::builder("Beer", euros(unit_price), [ana()])
            .quantity(units(count))
            .build()
            .unwrap();
        prop_assert_eq!(item.line_total(), euros(unit_price * i64::try_from(count).unwrap()));
    }

    /// A discount that took more than the line, or a receipt whose
    /// discount and line total did not add up to the total before the
    /// discount, would charge the line's members another amount than the
    /// receipt prints.
    #[test]
    fn a_discount_takes_off_between_nothing_and_the_whole_line(
        unit_price in 1_i64..1_000_000_000,
        ten_thousandths in 1_u64..1_000_000,
        percentage in proptest::option::of(0_u32..=10_000),
        fixed_share in 0_u32..=10_000,
        unit in any::<bool>(),
    ) {
        let quantity = Quantity::from_ten_thousandths(ten_thousandths).unwrap();
        let before_discount = ExpenseItem::builder("Cheese", euros(unit_price), [ana()])
            .quantity(quantity)
            .build()
            .unwrap()
            .line_total();
        let (scope, reduced) = if unit {
            (DiscountScope::Unit, unit_price)
        } else {
            (DiscountScope::Line, before_discount.minor_units())
        };
        // A fixed discount is a share of what it reduces, so it fits.
        let discount = percentage.map_or_else(
            || Discount::fixed(euros(reduced * i64::from(fixed_share) / 10_000)),
            |basis_points| Discount::percentage(Percentage::from_basis_points(basis_points)),
        );
        let item = ExpenseItem::builder("Cheese", euros(unit_price), [ana()])
            .quantity(quantity)
            .discount(discount.scoped(scope))
            .build()
            .unwrap();
        prop_assert_eq!(item.line_total_before_discount(), before_discount);
        prop_assert_eq!(
            item.line_total().minor_units() + item.discount_amount().minor_units(),
            before_discount.minor_units()
        );
        prop_assert!(0 <= item.line_total().minor_units());
        prop_assert!(item.line_total().minor_units() <= before_discount.minor_units());
    }
}
