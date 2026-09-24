//! Categories sort a group's expenses, and their names must be unique within
//! the group whatever their letter case or the way an accent is encoded.
//! These tests keep names trimmed and present, keep the folded name that
//! uniqueness is checked on the same for names that differ only in case, in
//! any script, or in the encoding of an accent, and keep a category's
//! identifier through a rename and a save.

use super::*;

fn category(name: &str) -> Category {
    Category::new(name).unwrap()
}

/// Names typed with a stray space would be stored, shown, and compared with
/// it.
#[test]
fn category_names_are_trimmed() {
    // Surrounding whitespace.
    assert_eq!(category("  Food \n").name(), "Food");
    // Inner whitespace stays.
    assert_eq!(category("Eating out").name(), "Eating out");
}

/// A category without a name would appear as a blank entry in every picker.
#[test]
fn a_blank_category_name_is_rejected() {
    // Empty and whitespace-only names.
    for name in ["", "   ", "\t\n"] {
        assert_eq!(
            Category::new(name),
            Err(CategoryError::EmptyName),
            "{name:?}"
        );
    }
}

/// A group could hold both "Food" and "FOOD", which users see as the same
/// category.
#[test]
fn folded_names_ignore_case() {
    // ASCII letters.
    assert_eq!(
        category("Food").folded_name(),
        category("FOOD").folded_name()
    );
    assert_eq!(
        category("Food").folded_name(),
        category("food").folded_name()
    );
    // Letters outside ASCII.
    assert_eq!(
        category("Café").folded_name(),
        category("CAFÉ").folded_name()
    );
    assert_eq!(
        category("Ψώνια").folded_name(),
        category("ΨΏΝΙΑ").folded_name()
    );
}

/// Names that differ only by full case folding or by how an accent is
/// encoded would be accepted as two categories that users see as one.
#[test]
fn folded_names_follow_canonical_caseless_matching() {
    for (first, second) in [
        // `ß` uppercases to `SS`, and Swiss German writes `ss`.
        ("Straße", "STRASSE"),
        ("Straße", "Strasse"),
        // A final sigma typed as a medial one.
        ("ΟΔΟΣ", "οδοσ"),
        // A ligature and a long s.
        ("\u{fb01}nance", "FINANCE"),
        ("\u{17f}tate", "STATE"),
        // A composed and a decomposed accent, which look identical.
        ("Café", "Cafe\u{301}"),
        ("\u{390}", "\u{1fd3}"),
        // Two German words that full folding makes one: the price of
        // catching `Straße` and `STRASSE`.
        ("Maße", "Masse"),
    ] {
        assert_eq!(
            category(first).folded_name(),
            category(second).folded_name(),
            "{first} and {second}"
        );
    }
}

/// Distinct categories such as "Food" and "Fuel" would collide as
/// duplicates.
#[test]
fn different_names_fold_differently() {
    // Different words.
    assert_ne!(
        category("Food").folded_name(),
        category("Fuel").folded_name()
    );
    // Accents are part of the name.
    assert_ne!(
        category("Cafe").folded_name(),
        category("Café").folded_name()
    );
    // The dotted and the dotless i stay distinct without Turkic folding, and
    // full-width letters and superscript digits without compatibility
    // folding.
    for (first, second) in [
        ("İstanbul", "istanbul"),
        ("ısparta", "ISPARTA"),
        ("ＦＯＯＤ", "food"),
        ("Food²", "Food2"),
    ] {
        assert_ne!(
            category(first).folded_name(),
            category(second).folded_name(),
            "{first} and {second}"
        );
    }
    // The name itself keeps its case.
    assert_eq!(category("FOOD").name(), "FOOD");
}

/// Two categories would share an identifier, or a renamed one would lose its
/// expenses by getting a new one.
#[test]
fn a_rename_keeps_the_identifier() {
    let food = category("Food");
    assert_ne!(category("Food").id(), food.id());
    // Renamed, trimmed, and validated like `new`.
    let groceries = food.renamed(" Groceries ").unwrap();
    assert_eq!(groceries.id(), food.id());
    assert_eq!(groceries.name(), "Groceries");
    assert_eq!(food.renamed(" "), Err(CategoryError::EmptyName));
}

/// Saved categories would fail to load, load under another identifier, or
/// load with a blank name.
#[cfg(feature = "serde")]
#[test]
fn categories_round_trip_through_serde() {
    // Round trip, the identifier first.
    let food = category("Food");
    let json = serde_json::to_string(&food).unwrap();
    assert_eq!(json, format!(r#"{{"id":"{}","name":"Food"}}"#, food.id()));
    assert_eq!(serde_json::from_str::<Category>(&json).unwrap(), food);
    // Loading trims and validates like `new`.
    let padded = format!(r#"{{"id":"{}","name":" Food "}}"#, food.id());
    assert_eq!(serde_json::from_str::<Category>(&padded).unwrap(), food);
    let blank = format!(r#"{{"id":"{}","name":" "}}"#, food.id());
    assert!(serde_json::from_str::<Category>(&blank).is_err());
    // The identifier is required.
    assert!(serde_json::from_str::<Category>(r#"{"name":"Food"}"#).is_err());
}
