//! Every name, note, and description goes through these helpers: the tests
//! keep stray whitespace out of stored text, keep blank optional fields
//! absent rather than empty, and keep blank required fields rejected with the
//! field that was blank.

use super::*;

/// Every field a required text can be.
const FIELDS: [Field; 7] = [
    Field::GroupName,
    Field::FirstName,
    Field::AccountName,
    Field::Username,
    Field::CategoryName,
    Field::ExpenseName,
    Field::ItemName,
];

/// Names typed with a stray space would be stored and shown with it.
#[test]
fn required_text_is_trimmed() {
    // Spaces on both sides.
    assert_eq!(
        required("  Ana  ", Field::FirstName),
        Ok(String::from("Ana"))
    );
    // Tabs, newlines, and non-breaking spaces.
    assert_eq!(
        required("\t\u{a0}Ana Maria\n", Field::FirstName),
        Ok(String::from("Ana Maria"))
    );
    // Text without surrounding whitespace.
    assert_eq!(required("Ana", Field::FirstName), Ok(String::from("Ana")));
}

/// A blank name would be saved, or the error would point at the wrong field.
#[test]
fn blank_required_text_is_rejected_with_its_field() {
    // Empty text.
    for field in FIELDS {
        assert_eq!(required("", field), Err(ValueError::Empty { field }));
    }
    // Whitespace only.
    for field in FIELDS {
        assert_eq!(
            required(" \t\n\u{a0}", field),
            Err(ValueError::Empty { field })
        );
    }
}

/// A note typed with a stray space would be stored and shown with it.
#[test]
fn optional_text_is_trimmed() {
    // Spaces on both sides.
    assert_eq!(optional("  Dinner  "), Some(String::from("Dinner")));
    // Inner whitespace stays.
    assert_eq!(
        optional("\nDinner at Ana's\t"),
        Some(String::from("Dinner at Ana's"))
    );
}

/// A blank note would be stored as an empty string and shown as an empty
/// line.
#[test]
fn blank_optional_text_is_absent() {
    // Empty text.
    assert_eq!(optional(""), None);
    // Whitespace only.
    assert_eq!(optional(" \t\n\u{a0}"), None);
}

/// Text that needs no trimming would be copied, doubling the allocations of
/// every entry built from owned strings.
#[test]
fn trimmed_text_keeps_its_allocation() {
    // Optional text.
    let text = String::from("Dinner");
    let pointer = text.as_ptr();
    let kept = optional(text).unwrap();
    assert_eq!(kept.as_ptr(), pointer);
    // Required text.
    let first_name = String::from("Ana");
    let first_name_pointer = first_name.as_ptr();
    let kept_first_name = required(first_name, Field::FirstName).unwrap();
    assert_eq!(kept_first_name.as_ptr(), first_name_pointer);
}
