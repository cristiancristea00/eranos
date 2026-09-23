//! Tests for the direct strategy: one payment per netted pair, in the
//! direction the debt runs. A sign read the wrong way would send money from
//! the creditor to the debtor.

use super::*;

fn member(number: u8) -> MemberId {
    MemberId::from_bytes([number; 16])
}

/// The sign of a pair decides who pays: positive means the lower member
/// owes the higher one, negative the reverse, and zero nothing.
#[test]
fn the_sign_of_a_pair_decides_who_pays() {
    let (ana, bob, carol) = (member(1), member(2), member(3));
    let pairs = BTreeMap::from([((ana, bob), 700), ((ana, carol), -300), ((bob, carol), 0)]);
    assert_eq!(settle(&pairs), vec![(ana, bob, 700), (carol, ana, 300)]);
}

/// No netted debt means no payment.
#[test]
fn no_pairs_need_no_payment() {
    assert!(settle(&BTreeMap::new()).is_empty());
}
