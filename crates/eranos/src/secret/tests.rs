//! Account numbers and payment-app usernames are held as secrets. These tests
//! keep them out of debug output, and therefore out of logs and crash
//! reports, keep the text intact for the screens that show it, and keep the
//! caller's buffer as the only copy, so wiping it on drop leaves nothing
//! behind.

use super::*;

/// The account number every test holds.
const ACCOUNT_NUMBER: &str = "RO49AAAA1B31007593840000";

/// An account number logged through `{:?}` would leak into logs.
#[test]
fn debug_output_is_redacted() {
    let secret = Secret::new(String::from(ACCOUNT_NUMBER));
    // Plain and pretty debug output.
    assert_eq!(format!("{secret:?}"), "[REDACTED]");
    assert_eq!(format!("{secret:#?}"), "[REDACTED]");
    // Inside the debug output of another value.
    let nested = format!("{:?}", Some(&secret));
    assert_eq!(nested, "Some([REDACTED])");
    assert!(!nested.contains("RO49"));
}

/// Payment screens would show a wrong or truncated account number.
#[test]
fn expose_returns_the_whole_text() {
    // Owned text.
    assert_eq!(
        Secret::new(String::from(ACCOUNT_NUMBER)).expose(),
        ACCOUNT_NUMBER
    );
    // A zeroising buffer.
    let buffer = Zeroizing::new(String::from(ACCOUNT_NUMBER));
    assert_eq!(Secret::from_zeroising(buffer).expose(), ACCOUNT_NUMBER);
    // Empty text.
    assert_eq!(Secret::new(String::new()).expose(), "");
}

/// Two accounts with the same number would compare as different, or two
/// different numbers as the same account.
#[test]
fn equality_compares_the_text() {
    // Same text.
    assert_eq!(
        Secret::new(String::from(ACCOUNT_NUMBER)),
        Secret::from_zeroising(Zeroizing::new(String::from(ACCOUNT_NUMBER)))
    );
    // Different text.
    assert_ne!(
        Secret::new(String::from(ACCOUNT_NUMBER)),
        Secret::new(String::from("RO49AAAA1B31007593840001"))
    );
}

/// A copied account would hold different text than the original.
#[test]
fn a_clone_holds_the_same_text() {
    // Clone and original.
    let secret = Secret::new(String::from(ACCOUNT_NUMBER));
    let copy = secret.clone();
    assert_eq!(copy, secret);
    assert_eq!(copy.expose(), ACCOUNT_NUMBER);
}

/// A copy made on the way in would stay in memory, unwiped, after the secret
/// is dropped.
#[test]
fn a_secret_keeps_the_callers_buffer() {
    // Owned text.
    let text = String::from(ACCOUNT_NUMBER);
    let pointer = text.as_ptr();
    let secret = Secret::new(text);
    assert_eq!(secret.expose().as_ptr(), pointer);
    // A zeroising buffer.
    let buffer = Zeroizing::new(String::from(ACCOUNT_NUMBER));
    let buffer_pointer = buffer.as_ptr();
    let secret_from_buffer = Secret::from_zeroising(buffer);
    assert_eq!(secret_from_buffer.expose().as_ptr(), buffer_pointer);
}
