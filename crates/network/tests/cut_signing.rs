//! Cuts signed on the members' feeds (`docs/04` §Members' objects, §Collecting a cut,
//! §Proposing and signing, `docs/08` PROTO-015, T74 step 2).

use network::beacon::RoundId;
use network::cid::cid;
use network::consortium::{Consortium, Member};
use network::cut::{
    added, collect, proposer, should_sign, sign_cut, Collected, Cut, CutError, MemberObject,
};
use network::log::TransparencyLog;
use network::replica::{FeedWriter, Replica, SignedEntry, WriterSet};

const NET: [u8; 32] = [6; 32];
const SEEDS: [u8; 4] = [10, 11, 12, 20];

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

    fn publish(&mut self, w: usize, object: Vec<u8>) -> SignedEntry {
        let entry = self.logs[w].append(cid(&object)).clone();
        let signed = self.writers[w].sign(&entry);
        self.replica.insert(signed.clone(), object).unwrap();
        signed
    }

    fn member_commit(&mut self, m: usize, round: RoundId) -> network::replica::EntryId {
        let (commit, _) = members()[m].beacon_commit(round, m);
        self.publish(m, MemberObject::Commit(commit).encode()).id()
    }

    fn sign(&mut self, m: usize, cut: &Cut) {
        let object = sign_cut(&members()[m], m, NET, &consortium(), cut).encode();
        self.publish(m, object);
    }
}

/// AT-NET-16: members' objects round-trip, never decode from other bytes, and bad kinds,
/// cuts and flags are refused.
#[test]
fn at_net_16_member_objects_have_one_encoding() {
    let mut net = Net::new();
    net.publish(3, b"a relay's entry".to_vec());
    let cut = Cut::of(&net.replica, 0);
    let round = RoundId {
        network_id: NET,
        member_set_hash: consortium().member_set_hash(),
        epoch: 3,
    };
    let (commit, reveal) = members()[1].beacon_commit(round, 1);
    let objects = vec![
        sign_cut(&members()[0], 0, NET, &consortium(), &cut),
        MemberObject::Commit(commit),
        MemberObject::Reveal { epoch: 3, reveal },
    ];
    for (o, member) in objects.iter().zip([0usize, 1, 1]) {
        assert_eq!(o.member(), member);
        let bytes = o.encode();
        assert_eq!(bytes[0], 0xC0);
        assert_eq!(MemberObject::decode(&bytes), Some(o.clone()));
        for at in 0..bytes.len() {
            assert!(MemberObject::decode(&bytes[..at]).is_none(), "cut at {at}");
        }
        let mut longer = bytes.clone();
        longer.push(0);
        assert!(MemberObject::decode(&longer).is_none());
        let mut tag = bytes.clone();
        tag[0] = 1;
        assert!(MemberObject::decode(&tag).is_none());
    }
    for kind in [0u8, 4] {
        assert!(MemberObject::decode(&[0xC0, kind]).is_none());
    }
    let mut bad_cut = objects[0].encode();
    bad_cut[10] = 9;
    assert!(
        MemberObject::decode(&bad_cut).is_none(),
        "a cut of another version"
    );
}

/// AT-NET-16: a cut is collected once `t` members signed it on their own feeds; signatures
/// on another's feed, for another network or number do not count; two cuts each signed by
/// `t` are a conflict.
#[test]
fn at_net_16_a_cut_is_collected_from_t_members() {
    let mut net = Net::new();
    net.publish(3, b"entry".to_vec());
    let cut = Cut::of(&net.replica, 0);
    let c = consortium();
    assert_eq!(collect(&net.replica, &c, NET, 0), Collected::Pending);
    net.sign(0, &cut);
    let forged = sign_cut(&members()[1], 1, NET, &c, &cut).encode();
    net.publish(3, forged);
    let elsewhere = sign_cut(&members()[1], 1, [7; 32], &c, &cut).encode();
    net.publish(1, elsewhere);
    let mut misfiled = sign_cut(&members()[2], 2, NET, &c, &cut);
    if let MemberObject::CutSignature { member, .. } = &mut misfiled {
        *member = 1;
    }
    net.publish(1, misfiled.encode());
    assert_eq!(collect(&net.replica, &c, NET, 0), Collected::Pending);
    net.sign(2, &cut);
    let Collected::Ready(got, sigs) = collect(&net.replica, &c, NET, 0) else {
        panic!("two members signed")
    };
    assert_eq!(got, cut);
    assert_eq!(sigs.iter().map(|(i, _)| *i).collect::<Vec<_>>(), vec![0, 2]);
    assert!(c.verify(&cut.checkpoint(NET, c.member_set_hash()), &sigs));
    assert_eq!(collect(&net.replica, &c, NET, 1), Collected::Pending);
    let other = Cut {
        epoch: 1,
        ..cut.clone()
    };
    net.sign(1, &other);
    net.sign(0, &other);
    let (a, b) = if cut.digest() < other.digest() {
        (cut, other)
    } else {
        (other, cut)
    };
    assert_eq!(collect(&net.replica, &c, NET, 0), Collected::Conflict(a, b));
}

/// AT-NET-16: the proposer rotates; a member co-signs only the proposer's cut, extending
/// the last, whose entries it holds, and which counts its own commit and reveal.
#[test]
fn at_net_16_members_sign_only_what_they_should() {
    let c = consortium();
    assert_eq!(
        (0..6).map(|n| proposer(&c, n)).collect::<Vec<_>>(),
        vec![0, 1, 2, 0, 1, 2]
    );
    let mut net = Net::new();
    net.publish(3, b"entry".to_vec());
    let cut0 = Cut::next(&net.replica, None, 0, false);
    assert_eq!(cut0, Cut::of(&net.replica, 0));
    assert!(
        !should_sign(&net.replica, &c, 1, None, &cut0),
        "not proposed yet"
    );
    net.sign(1, &cut0);
    assert!(
        !should_sign(&net.replica, &c, 2, None, &cut0),
        "signed by a non-proposer"
    );
    net.sign(0, &cut0);
    assert!(should_sign(&net.replica, &c, 1, None, &cut0));
    assert!(
        !should_sign(&net.replica, &c, 9, None, &cut0),
        "not a member"
    );

    let round = RoundId {
        network_id: NET,
        member_set_hash: c.member_set_hash(),
        epoch: 0,
    };
    let (commit, reveal) = members()[2].beacon_commit(round, 2);
    let closing = Cut::next(&net.replica, Some(&cut0), 0, true);
    net.sign(1, &closing);
    assert!(should_sign(&net.replica, &c, 2, Some(&cut0), &closing));
    net.publish(2, MemberObject::Commit(commit).encode());
    assert!(
        !should_sign(&net.replica, &c, 2, Some(&cut0), &closing),
        "the closing cut leaves out member 2's commit"
    );
    assert!(should_sign(&net.replica, &c, 0, Some(&cut0), &closing));
    let closing = Cut::next(&net.replica, Some(&cut0), 0, true);
    net.sign(1, &closing);
    assert!(should_sign(&net.replica, &c, 2, Some(&cut0), &closing));

    let after = Cut::next(&net.replica, Some(&closing), 1, false);
    net.sign(2, &after);
    net.publish(2, MemberObject::Reveal { epoch: 0, reveal }.encode());
    assert!(
        !should_sign(&net.replica, &c, 2, Some(&closing), &after),
        "no reveal"
    );
    let after = Cut::next(&net.replica, Some(&closing), 1, false);
    net.sign(2, &after);
    assert!(should_sign(&net.replica, &c, 2, Some(&closing), &after));
    let lacking = {
        let mut r = Net::new();
        r.publish(3, b"entry".to_vec());
        r.replica
    };
    assert!(
        !should_sign(&lacking, &c, 2, Some(&closing), &after),
        "entries not held"
    );
    assert!(
        !should_sign(&net.replica, &c, 2, Some(&cut0), &after),
        "does not extend"
    );
}

/// AT-NET-16: epochs never go back, and a closed epoch is over; a proposal keeps a forked
/// writer at its last counted mark.
#[test]
fn at_net_16_epochs_advance_and_proposals_extend() {
    let mut net = Net::new();
    net.publish(3, b"a".to_vec());
    let cut0 = Cut::next(&net.replica, None, 2, false);
    let at = |epoch, closes| Cut {
        epoch,
        closes,
        ..Cut::next(&net.replica, Some(&cut0), 0, false)
    };
    assert_eq!(
        added(&net.replica, Some(&cut0), &at(1, false)),
        Err(CutError::Epoch)
    );
    assert!(added(&net.replica, Some(&cut0), &at(2, true)).is_ok());
    assert!(added(&net.replica, Some(&cut0), &at(3, false)).is_ok());
    let closed = at(2, true);
    let next = |epoch| Cut {
        number: 2,
        epoch,
        closes: false,
        marks: closed.marks.clone(),
    };
    assert_eq!(
        added(&net.replica, Some(&closed), &next(2)),
        Err(CutError::Epoch)
    );
    assert!(added(&net.replica, Some(&closed), &next(3)).is_ok());

    net.publish(3, b"b".to_vec());
    let cut1 = Cut::next(&net.replica, Some(&cut0), 2, false);
    assert_eq!(cut1.marks[0].len, 2);
    let relay = FeedWriter::from_seed(NET, [SEEDS[3]; 32]);
    let mut fork = TransparencyLog::new();
    fork.append(cid(b"a"));
    let o = b"fork".to_vec();
    let forked = relay.sign(&fork.append(cid(&o)).clone());
    net.replica.insert(forked, o).unwrap();
    net.publish(3, b"c".to_vec());
    let cut2 = Cut::next(&net.replica, Some(&cut1), 2, false);
    assert_eq!(
        cut2.marks, cut1.marks,
        "the forked writer stays at its last mark"
    );
    assert_eq!(cut2.number, 2);
    assert!(added(&net.replica, Some(&cut1), &cut2).unwrap().is_empty());
}

/// AT-NET-16: a proposal does not jump to another, longer branch of a writer: it keeps the
/// last counted mark.
#[test]
fn at_net_16_a_proposal_keeps_the_counted_branch() {
    let relay = FeedWriter::from_seed(NET, [SEEDS[3]; 32]);
    let branch = |tag: &str, n: usize| {
        let mut log = TransparencyLog::new();
        (0..n)
            .map(|i| {
                let o = format!("{tag}-{i}").into_bytes();
                (relay.sign(&log.append(cid(&o)).clone()), o)
            })
            .collect::<Vec<_>>()
    };
    let (a, b) = (branch("a", 2), branch("b", 3));
    let mut first = Net::new();
    for (e, o) in &a {
        first.replica.insert(e.clone(), o.clone()).unwrap();
    }
    let prev = Cut::next(&first.replica, None, 0, false);
    let mut other = Net::new();
    for (e, o) in &b {
        other.replica.insert(e.clone(), o.clone()).unwrap();
    }
    let proposal = Cut::next(&other.replica, Some(&prev), 0, false);
    assert_eq!(proposal.marks, prev.marks);
}

/// AT-NET-16: a closing cut whose mark stops right before the member's commit leaves it
/// out; a cut that does not close does not need it.
#[test]
fn at_net_16_the_commit_counts_only_inside_the_mark() {
    let c = consortium();
    let mut net = Net::new();
    net.publish(2, b"an earlier entry of member 2".to_vec());
    let cut0 = Cut::next(&net.replica, None, 0, false);
    net.sign(0, &cut0);
    let closing = Cut::next(&net.replica, Some(&cut0), 0, true);
    let open = Cut {
        closes: false,
        ..closing.clone()
    };
    net.sign(1, &closing);
    net.sign(1, &open);
    let round = RoundId {
        network_id: NET,
        member_set_hash: c.member_set_hash(),
        epoch: 0,
    };
    let commit = net.member_commit(2, round);
    let mark = closing
        .marks
        .iter()
        .find(|m| m.writer == commit.writer)
        .unwrap();
    assert_eq!(
        mark.len, commit.seq,
        "the mark ends right before the commit"
    );
    assert!(!should_sign(&net.replica, &c, 2, Some(&cut0), &closing));
    assert!(should_sign(&net.replica, &c, 2, Some(&cut0), &open));
}
