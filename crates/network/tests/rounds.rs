//! The beacon's rounds on cuts and rule 6 (`docs/04` §The beacon on cuts, §The withholders
//! on cuts, T76 step 2): what a cut counts, each round's end, whom a cut excludes.

use network::beacon::{RoundError, RoundId};
use network::cid::cid;
use network::consortium::{Consortium, Member};
use network::cut::{collect_with, sign_cut, Collected, Cut, MemberObject};
use network::log::TransparencyLog;
use network::replica::{FeedWriter, Replica, WriterSet};
use network::rounds::{Rounds, Uncounted};

const NET: [u8; 32] = [9; 32];
const SEEDS: [u8; 3] = [50, 51, 52];

fn members() -> Vec<Member> {
    SEEDS.iter().map(|s| Member::from_seed([*s; 32])).collect()
}

fn consortium() -> Consortium {
    Consortium::new(members().iter().map(|m| m.public()).collect(), 2)
}

fn round(epoch: u64) -> RoundId {
    RoundId {
        network_id: NET,
        member_set_hash: consortium().member_set_hash(),
        epoch,
    }
}

fn cut(number: u64, epoch: u64, closes: bool) -> Cut {
    Cut {
        number,
        epoch,
        closes,
        marks: Vec::new(),
    }
}

fn commit(m: usize, epoch: u64) -> MemberObject {
    MemberObject::Commit(members()[m].beacon_commit(round(epoch), m).0)
}

fn reveal(m: usize, epoch: u64) -> MemberObject {
    let reveal = members()[m].beacon_commit(round(epoch), m).1;
    MemberObject::Reveal { epoch, reveal }
}

/// Epoch 0's round: every member committing in closing cut 0, `revealing` revealing in cut 1.
fn epoch_zero(revealing: &[usize]) -> (Rounds, Vec<Cut>) {
    let c = consortium();
    let mut r = Rounds::new();
    let cuts = vec![cut(0, 0, true), cut(1, 1, false)];
    for m in 0..3 {
        r.count(&c, NET, &commit(m, 0), &cuts[0], None).unwrap();
    }
    r.finish(&c, NET, &cuts[0], None);
    for &m in revealing {
        r.count(&c, NET, &reveal(m, 0), &cuts[1], Some(&cuts[0]))
            .unwrap();
    }
    r.finish(&c, NET, &cuts[1], Some(&cuts[0]));
    (r, cuts)
}

/// AT-NET-20: a round with a value excludes its withholders from the cuts after its end in
/// that cut's epoch, the closing one included, and from no other cut.
#[test]
fn at_net_20_the_window_of_rule_6() {
    let (r, cuts) = epoch_zero(&[0, 1]);
    let outcome = r.outcome(0).unwrap();
    assert!(outcome.value().is_some());
    assert_eq!(outcome.withheld(), [2]);
    assert!(r.excluded(&cuts[1]).is_empty(), "the cut the round ends at");
    assert_eq!(r.excluded(&cut(2, 1, false)), vec![2]);
    assert_eq!(r.excluded(&cut(3, 1, true)), vec![2]);
    assert!(r.excluded(&cut(4, 2, false)).is_empty(), "the next epoch");
    assert!(r.excluded(&cut(2, 2, false)).is_empty());
}

/// AT-NET-20: a round with fewer than `t` reveals has no value and excludes no one.
#[test]
fn at_net_20_no_value_no_exclusion() {
    let (r, _) = epoch_zero(&[0]);
    let outcome = r.outcome(0).unwrap();
    assert!(outcome.value().is_none());
    assert_eq!(outcome.withheld(), [1, 2]);
    assert!(r.excluded(&cut(2, 1, false)).is_empty());
}

/// AT-NET-20: a commit counts only in its epoch's cuts, a reveal only in the cut after the
/// closing one; the round refuses a repeat; other objects are no beacon message.
#[test]
fn at_net_20_what_a_cut_counts() {
    let c = consortium();
    let mut r = Rounds::new();
    let closing = cut(0, 0, true);
    assert_eq!(
        r.count(&c, NET, &commit(0, 1), &closing, None),
        Err(Uncounted::OutOfWindow)
    );
    assert_eq!(r.count(&c, NET, &commit(0, 0), &closing, None), Ok(()));
    assert_eq!(
        r.count(&c, NET, &commit(0, 0), &closing, None),
        Err(Uncounted::Beacon(RoundError::DuplicateCommit))
    );
    assert_eq!(
        r.count(&c, NET, &reveal(0, 0), &closing, None),
        Err(Uncounted::OutOfWindow)
    );
    let open = cut(1, 1, false);
    assert_eq!(
        r.count(&c, NET, &reveal(0, 0), &open, Some(&cut(0, 0, false))),
        Err(Uncounted::OutOfWindow),
        "the cut before did not close"
    );
    let change = MemberObject::ViewChange {
        number: 0,
        view: 1,
        member: 0,
    };
    assert_eq!(r.count(&c, NET, &change, &closing, None), Ok(()));
    r.finish(&c, NET, &closing, None);
    assert!(
        r.outcome(0).is_none(),
        "the closing cut does not finish its round"
    );
}

/// AT-NET-20: collecting counts no signature of an excluded member.
#[test]
fn at_net_20_collection_skips_the_excluded() {
    let c = consortium();
    let writers: Vec<FeedWriter> = SEEDS
        .iter()
        .map(|s| FeedWriter::from_seed(NET, [*s; 32]))
        .collect();
    let keys: Vec<_> = writers.iter().map(|w| w.public()).collect();
    let mut replica = Replica::new(WriterSet::new(NET, &keys));
    let mut logs: Vec<TransparencyLog> = (0..3).map(|_| TransparencyLog::new()).collect();
    let mut publish = |replica: &mut Replica, m: usize, bytes: Vec<u8>| {
        let entry = logs[m].append(cid(&bytes)).clone();
        replica.insert(writers[m].sign(&entry), bytes).unwrap();
    };
    publish(&mut replica, 0, b"entry".to_vec());
    let target = Cut::of(&replica, 0);
    for m in [0, 2] {
        let o = sign_cut(&members()[m], m, NET, &c, &target).encode();
        publish(&mut replica, m, o);
    }
    let none = |_: &Cut| Vec::new();
    let two = |_: &Cut| vec![2];
    assert!(matches!(
        collect_with(&replica, &c, NET, 0, &none),
        Collected::Ready(..)
    ));
    assert_eq!(collect_with(&replica, &c, NET, 0, &two), Collected::Pending);
    let o = sign_cut(&members()[1], 1, NET, &c, &target).encode();
    publish(&mut replica, 1, o);
    assert!(matches!(
        collect_with(&replica, &c, NET, 0, &two),
        Collected::Ready(..)
    ));
}
