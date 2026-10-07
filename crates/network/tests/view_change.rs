//! Views of a cut (`docs/04` §Proposing and signing, §Changing view, `docs/08` PROTO-015,
//! T76): view-bound signatures, view changes, the proposals they justify and collection.

use network::cid::cid;
use network::consortium::{Consortium, Member};
use network::cut::{
    changed_view, collect, forced, promise, proposal, proposer_in, should_sign_in, sign_cut_in,
    signed_in, views_change, Collected, Cut, MemberObject, MissingViewChange,
};
use network::log::TransparencyLog;
use network::replica::{FeedWriter, Replica, WriterSet};

const NET: [u8; 32] = [8; 32];
const SEEDS: [u8; 4] = [40, 41, 42, 43];

fn members() -> Vec<Member> {
    SEEDS[..3]
        .iter()
        .map(|s| Member::from_seed([*s; 32]))
        .collect()
}

fn consortium() -> Consortium {
    Consortium::new(members().iter().map(|m| m.public()).collect(), 2)
}

/// Writers 0–2 are the members, 3 a relay; each keeps its log.
struct Net {
    writers: Vec<FeedWriter>,
    logs: Vec<TransparencyLog>,
    replica: Replica,
}

impl Net {
    fn new() -> Net {
        let writers: Vec<FeedWriter> = SEEDS
            .iter()
            .map(|s| FeedWriter::from_seed(NET, [*s; 32]))
            .collect();
        let keys: Vec<_> = writers.iter().map(|w| w.public()).collect();
        Net {
            replica: Replica::new(WriterSet::new(NET, &keys)),
            logs: (0..4).map(|_| TransparencyLog::new()).collect(),
            writers,
        }
    }

    fn publish(&mut self, w: usize, object: Vec<u8>) {
        let entry = self.logs[w].append(cid(&object)).clone();
        self.replica
            .insert(self.writers[w].sign(&entry), object)
            .unwrap();
    }

    fn sign(&mut self, m: usize, cut: &Cut, view: u64, named: &[usize]) {
        let o = sign_cut_in(
            &members()[m],
            m,
            NET,
            &consortium(),
            cut,
            view,
            named.to_vec(),
        );
        self.publish(m, o.encode());
    }

    fn change(&mut self, m: usize, number: u64, view: u64) {
        let o = MemberObject::ViewChange {
            number,
            view,
            member: m,
        };
        self.publish(m, o.encode());
    }

    /// Two different cuts numbered 0, over the relay's first one and first two entries.
    fn two_cuts(&mut self) -> (Cut, Cut) {
        self.publish(3, b"first".to_vec());
        let a = Cut::of(&self.replica, 0);
        self.publish(3, b"second".to_vec());
        (a, Cut::of(&self.replica, 0))
    }
}

/// AT-NET-19: view 0's checkpoint is the cut's own; a later view binds its number, so a
/// signature counts only in the view it was given in.
#[test]
fn at_net_19_a_signature_belongs_to_its_view() {
    let mut net = Net::new();
    let (cut, _) = net.two_cuts();
    let c = consortium();
    let msh = c.member_set_hash();
    assert_eq!(cut.checkpoint_in(0, NET, msh), cut.checkpoint(NET, msh));
    let heads: Vec<_> = (0..3)
        .map(|v| cut.checkpoint_in(v, NET, msh).head)
        .collect();
    assert!(heads[0] != heads[1] && heads[1] != heads[2] && heads[0] != heads[2]);
    let sig = members()[1].sign(&cut.checkpoint_in(1, NET, msh));
    let forged = MemberObject::CutSignature {
        cut: cut.clone(),
        view: 2,
        member: 1,
        signature: sig,
        named: vec![],
    };
    net.publish(1, forged.encode());
    net.sign(0, &cut, 2, &[]);
    assert_eq!(collect(&net.replica, &c, NET, 0), Collected::Pending);
}

/// AT-NET-19: signatures in different views are never added up; one cut ready in two views
/// is collected in the earlier; two cuts ready, in any views, are a conflict.
#[test]
fn at_net_19_collection_is_per_view() {
    let c = consortium();
    let mut net = Net::new();
    let (a, b) = net.two_cuts();
    net.sign(0, &a, 0, &[]);
    net.sign(1, &a, 1, &[]);
    assert_eq!(collect(&net.replica, &c, NET, 0), Collected::Pending);
    net.sign(2, &a, 1, &[]);
    assert!(matches!(collect(&net.replica, &c, NET, 0), Collected::Ready(ref x, 1, _) if *x == a));
    net.sign(1, &a, 0, &[]);
    assert!(matches!(collect(&net.replica, &c, NET, 0), Collected::Ready(ref x, 0, _) if *x == a));
    net.sign(0, &b, 3, &[]);
    net.sign(2, &b, 3, &[]);
    assert!(matches!(
        collect(&net.replica, &c, NET, 0),
        Collected::Conflict(..)
    ));
}

/// AT-NET-19: signatures in view `v ≥ 1` and view changes round-trip with one encoding;
/// view 0 in either, an unsorted list of named members, and cut bytes are refused.
#[test]
fn at_net_19_view_objects_have_one_encoding() {
    let mut net = Net::new();
    let (cut, _) = net.two_cuts();
    let c = consortium();
    let objects = [
        sign_cut_in(&members()[1], 1, NET, &c, &cut, 4, vec![0, 2]),
        sign_cut_in(&members()[2], 2, NET, &c, &cut, 1, vec![]),
        MemberObject::ViewChange {
            number: 7,
            view: 3,
            member: 2,
        },
    ];
    for o in &objects {
        let bytes = o.encode();
        assert_eq!(MemberObject::decode(&bytes), Some(o.clone()));
        for at in 0..bytes.len() {
            assert!(MemberObject::decode(&bytes[..at]).is_none(), "cut at {at}");
        }
        let mut longer = bytes.clone();
        longer.push(0);
        assert!(MemberObject::decode(&longer).is_none());
    }
    let view0 = sign_cut_in(&members()[1], 1, NET, &c, &cut, 0, vec![]).encode();
    assert_eq!(view0[1], 1, "view 0 keeps the first kind");
    let mut as_four = objects[0].encode();
    let at = as_four.len() - 64 - 8 - 8 - 8 - 2 * 8;
    as_four[at..at + 8].copy_from_slice(&0u64.to_le_bytes());
    assert!(MemberObject::decode(&as_four).is_none(), "kind 4 in view 0");
    let mut change = objects[2].encode();
    let len = change.len();
    change[len - 16..len - 8].copy_from_slice(&0u64.to_le_bytes());
    assert!(
        MemberObject::decode(&change).is_none(),
        "a view change to view 0"
    );
    let mut unsorted = objects[0].encode();
    let len = unsorted.len();
    unsorted[len - 16..len - 8].copy_from_slice(&2u64.to_le_bytes());
    unsorted[len - 8..].copy_from_slice(&0u64.to_le_bytes());
    assert!(
        MemberObject::decode(&unsorted).is_none(),
        "named out of order"
    );
    unsorted[len - 16..len - 8].copy_from_slice(&0u64.to_le_bytes());
    assert!(
        MemberObject::decode(&unsorted).is_none(),
        "a member named twice"
    );
    assert!(Cut::decode(&objects[2].encode()).is_none());
}

/// AT-NET-19: the proposer moves one member per view; views change only when two halves of
/// the members cannot both reach the threshold.
#[test]
fn at_net_19_who_proposes_and_when_views_change() {
    let c = consortium();
    let order: Vec<_> = (0..5).map(|v| proposer_in(&c, 4, v)).collect();
    assert_eq!(order, vec![1, 2, 0, 1, 2]);
    assert_eq!(proposer_in(&c, u64::MAX, u64::MAX), 0);
    let keys = |n: usize| -> Vec<_> {
        (0..n)
            .map(|i| Member::from_seed([60 + i as u8; 32]).public())
            .collect()
    };
    for (n, t, changes) in [
        (3, 2, true),
        (2, 1, false),
        (4, 2, false),
        (4, 3, true),
        (1, 1, true),
    ] {
        assert_eq!(
            views_change(&Consortium::new(keys(n), t)),
            changes,
            "{t} of {n}"
        );
    }
}

/// AT-NET-19: a member's promise is its latest view change or signature; it signs a
/// proposal once, and never one of a view before its promise.
#[test]
fn at_net_19_promises_bind_a_member() {
    let c = consortium();
    let mut net = Net::new();
    let (a, _) = net.two_cuts();
    assert_eq!(promise(&net.replica, &c, 2, 0), 0);
    net.sign(0, &a, 0, &[]);
    assert!(should_sign_in(&net.replica, &c, 2, None, &a, 0));
    net.change(2, 0, 2);
    assert_eq!(promise(&net.replica, &c, 2, 0), 2);
    assert_eq!(promise(&net.replica, &c, 2, 1), 0, "another number");
    assert!(
        !should_sign_in(&net.replica, &c, 2, None, &a, 0),
        "promised view 2"
    );
    net.sign(1, &a, 0, &[]);
    assert!(signed_in(&net.replica, &c, 1, 0, 0));
    assert!(
        !should_sign_in(&net.replica, &c, 1, None, &a, 0),
        "signed already"
    );
}

/// AT-NET-19: a proposal after view 0 rests on `t` view changes and carries the cut those
/// members signed in the latest view before them, or any cut if they signed none.
#[test]
fn at_net_19_a_proposal_carries_the_forced_cut() {
    let c = consortium();
    let mut net = Net::new();
    let (a, b) = net.two_cuts();
    net.change(1, 0, 1);
    net.change(2, 0, 1);
    net.sign(1, &b, 1, &[2]);
    assert_eq!(proposal(&net.replica, &c, 0, 1), None, "fewer than t named");
    let mut net = Net::new();
    let (a2, b2) = net.two_cuts();
    assert_eq!((&a2, &b2), (&a, &b));
    net.change(1, 0, 1);
    net.sign(1, &b, 1, &[1, 2]);
    assert_eq!(
        proposal(&net.replica, &c, 0, 1),
        None,
        "member 2 changed no view"
    );
    net.change(2, 0, 1);
    assert_eq!(
        proposal(&net.replica, &c, 0, 1),
        Some(b.clone()),
        "none signed: free"
    );

    let mut net = Net::new();
    net.two_cuts();
    net.sign(0, &a, 0, &[]);
    net.sign(2, &a, 0, &[]);
    net.change(2, 0, 1);
    net.change(1, 0, 1);
    assert_eq!(forced(&net.replica, &c, 0, 1, &[1, 2]), Ok(Some(a.clone())));
    net.sign(1, &b, 1, &[1, 2]);
    assert_eq!(
        proposal(&net.replica, &c, 0, 1),
        None,
        "b is not the forced cut"
    );
    assert_eq!(
        forced(&net.replica, &c, 0, 1, &[0, 1]),
        Err(MissingViewChange { member: 0 })
    );

    let mut net = Net::new();
    net.two_cuts();
    net.change(2, 0, 1);
    net.sign(2, &a, 0, &[]);
    net.change(1, 0, 1);
    assert_eq!(
        forced(&net.replica, &c, 0, 1, &[1, 2]),
        Ok(None),
        "a signature after the view change does not bind"
    );
}

/// AT-NET-19: among the cuts the named members signed, the one of the latest view is forced.
#[test]
fn at_net_19_the_latest_view_is_forced() {
    let c = consortium();
    let mut net = Net::new();
    let (a, b) = net.two_cuts();
    net.sign(0, &a, 0, &[]);
    net.change(0, 0, 1);
    net.change(1, 0, 1);
    net.change(2, 0, 2);
    net.sign(1, &b, 1, &[0, 1]);
    net.change(1, 0, 2);
    net.change(0, 0, 2);
    assert_eq!(forced(&net.replica, &c, 0, 2, &[0, 1]), Ok(Some(b.clone())));
    net.sign(2, &a, 2, &[0, 1]);
    assert_eq!(proposal(&net.replica, &c, 0, 2), None);
}

/// AT-NET-19: the members holding a view change are those of that number and view only.
#[test]
fn at_net_19_who_changed_view() {
    let c = consortium();
    let mut net = Net::new();
    net.two_cuts();
    assert!(changed_view(&net.replica, &c, 0, 1).is_empty());
    net.change(2, 0, 1);
    net.change(0, 0, 1);
    net.change(1, 0, 2);
    net.change(1, 3, 1);
    assert_eq!(changed_view(&net.replica, &c, 0, 1), vec![0, 2]);
    assert_eq!(changed_view(&net.replica, &c, 0, 2), vec![1]);
    assert_eq!(changed_view(&net.replica, &c, 3, 1), vec![1]);
}

/// AT-NET-19: two cuts signed in one view, the forced one is the smallest digest, whichever
/// member signed it and in whichever order; a later view still wins over it.
#[test]
fn at_net_19_a_tie_forces_the_smallest_digest() {
    let c = consortium();
    let mut probe = Net::new();
    let (a, b) = probe.two_cuts();
    let (low, high) = if a.digest() < b.digest() {
        (a, b)
    } else {
        (b, a)
    };
    for (first, second) in [(&low, &high), (&high, &low)] {
        let mut net = Net::new();
        net.two_cuts();
        net.sign(0, first, 0, &[]);
        net.sign(1, second, 0, &[]);
        net.change(0, 0, 1);
        net.change(1, 0, 1);
        assert_eq!(
            forced(&net.replica, &c, 0, 1, &[0, 1]),
            Ok(Some(low.clone()))
        );
        net.change(2, 0, 2);
        net.sign(2, &high, 2, &[]);
        net.change(2, 0, 3);
        net.change(0, 0, 3);
        assert_eq!(
            forced(&net.replica, &c, 0, 3, &[0, 2]),
            Ok(Some(high.clone())),
            "the later view wins"
        );
    }
}

/// AT-NET-19: a signature given in the view of the member's own view change is not one it
/// gave before promising, and does not bind the proposal.
#[test]
fn at_net_19_a_signature_in_the_promised_view_does_not_bind() {
    let c = consortium();
    let mut net = Net::new();
    let (a, _) = net.two_cuts();
    net.sign(1, &a, 1, &[]);
    net.change(1, 0, 1);
    net.change(2, 0, 1);
    assert_eq!(forced(&net.replica, &c, 0, 1, &[1, 2]), Ok(None));
}

/// AT-NET-19: a proposal is the proposer's signature in that view, whatever it signed in
/// other views, and carrying the forced cut makes it valid.
#[test]
fn at_net_19_the_proposal_of_a_view_is_found_in_it() {
    let c = consortium();
    let mut net = Net::new();
    let (a, b) = net.two_cuts();
    net.sign(0, &a, 0, &[]);
    net.sign(1, &b, 0, &[]);
    net.change(0, 0, 1);
    net.change(2, 0, 1);
    assert_eq!(proposal(&net.replica, &c, 0, 0), Some(a.clone()));
    net.sign(1, &a, 1, &[0, 2]);
    assert_eq!(proposal(&net.replica, &c, 0, 1), Some(a.clone()));
    assert!(should_sign_in(&net.replica, &c, 2, None, &a, 1));
    assert!(!should_sign_in(&net.replica, &c, 2, None, &b, 1));
}
