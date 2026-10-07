//! Tests for the direct strategy: one payment per netted pair, in the
//! direction the debt runs. A sign read the wrong way would send money from
//! the creditor to the debtor.

use eranos_core::MemberId;

use super::*;

/// The member `number`; members sort by number.
fn member(number: u8) -> MemberId {
    MemberId::new(&format!("member-{number:03}")).unwrap()
}

/// The pair of `from` owing or paying `to`.
const fn pair(from: MemberId, to: MemberId) -> MemberPair {
    MemberPair { from, to }
}

/// The sign of a pair decides who pays: positive means the lower member
/// owes the higher one, negative the reverse, and zero nothing.
#[test]
fn the_sign_of_a_pair_decides_who_pays() {
    let (ana, bob, carol) = (member(1), member(2), member(3));
    let pairs = BTreeMap::from([
        (pair(ana, bob), 700),
        (pair(ana, carol), -300),
        (pair(bob, carol), 0),
    ]);
    assert_eq!(
        settle(&pairs),
        vec![
            Flow {
                members: pair(ana, bob),
                amount: 700
            },
            Flow {
                members: pair(carol, ana),
                amount: 300
            }
        ]
    );
}

/// No netted debt means no payment.
#[test]
fn no_pairs_need_no_payment() {
    assert_eq!(settle(&BTreeMap::new()), Vec::<Flow>::new());
}
