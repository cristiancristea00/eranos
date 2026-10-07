//! Categories sort a group's expenses, and their names must be unique within
//! the group whatever their letter case or the way an accent is encoded.
//! These tests keep names trimmed and present, keep the built-in category
//! the same everywhere, keep the folded name that uniqueness is checked on
//! the same for names that differ only in case, in any script, or in the
//! encoding of an accent, and keep a category's identifier, given or
//! generated, through a rename and a save.

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

/// Devices that named or identified the built-in category differently would
/// sort a synced expense into different categories.
#[test]
fn the_built_in_category_is_the_same_everywhere() {
    let built_in = Category::uncategorised();
    assert_eq!(built_in.id(), CategoryId::UNCATEGORISED);
    assert_eq!(built_in.name(), "Uncategorised");
    assert_eq!(built_in, Category::uncategorised());
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

/// Two names whose folded forms a test compares.
#[derive(Debug, Clone, Copy)]
struct NamePair {
    first: &'static str,
    second: &'static str,
}

/// Names that differ only by full case folding or by how an accent is
/// encoded would be accepted as two categories that users see as one.
#[test]
fn folded_names_follow_canonical_caseless_matching() {
    for NamePair { first, second } in [
        // `ß` uppercases to `SS`, and Swiss German writes `ss`.
        NamePair {
            first: "Straße",
            second: "STRASSE",
        },
        NamePair {
            first: "Straße",
            second: "Strasse",
        },
        // A final sigma typed as a medial one.
        NamePair {
            first: "ΟΔΟΣ",
            second: "οδοσ",
        },
        // A ligature and a long s.
        NamePair {
            first: "\u{fb01}nance",
            second: "FINANCE",
        },
        NamePair {
            first: "\u{17f}tate",
            second: "STATE",
        },
        // A composed and a decomposed accent, which look identical.
        NamePair {
            first: "Café",
            second: "Cafe\u{301}",
        },
        NamePair {
            first: "\u{390}",
            second: "\u{1fd3}",
        },
        // Two German words that full folding makes one: the price of
        // catching `Straße` and `STRASSE`.
        NamePair {
            first: "Maße",
            second: "Masse",
        },
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
    for NamePair { first, second } in [
        NamePair {
            first: "İstanbul",
            second: "istanbul",
        },
        NamePair {
            first: "ısparta",
            second: "ISPARTA",
        },
        NamePair {
            first: "ＦＯＯＤ",
            second: "food",
        },
        NamePair {
            first: "Food²",
            second: "Food2",
        },
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

/// A category from an app's own store would get another identifier than
/// its key, keep a stray space in its name, or be accepted without a name.
#[test]
fn a_category_takes_the_identifier_it_is_given() {
    let id = CategoryId::new("-food").unwrap();
    let food = Category::with_id(id, " Food ").unwrap();
    assert_eq!(food.id(), id);
    assert_eq!(food.name(), "Food");
    // A rename keeps it.
    assert_eq!(food.renamed("Groceries").unwrap().id(), id);
    // The name is checked as `new` checks it.
    assert_eq!(Category::with_id(id, "\t"), Err(CategoryError::EmptyName));
    // The built-in category's identifier is accepted here; a group refuses
    // to add a second category under it.
    let other = Category::with_id(CategoryId::UNCATEGORISED, "Other").unwrap();
    assert_eq!(other.id(), CategoryId::UNCATEGORISED);
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
    // An identifier given from outside reads back as it was written.
    let lodging = Category::with_id(CategoryId::new("lodging").unwrap(), "Lodging").unwrap();
    let lodging_json = serde_json::to_string(&lodging).unwrap();
    assert_eq!(lodging_json, r#"{"id":"lodging","name":"Lodging"}"#);
    assert_eq!(
        serde_json::from_str::<Category>(&lodging_json).unwrap(),
        lodging
    );
    // The identifier is required.
    assert!(serde_json::from_str::<Category>(r#"{"name":"Food"}"#).is_err());
}
