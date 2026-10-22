//! A member's duties read from its replica (`docs/04` §Proposing and signing, §The beacon on
//! cuts, T74 step 2): when a proposer closes an epoch, waits, and gives up waiting.

use network::beacon::RoundId;
use network::cid::cid;
use network::consortium::{Consortium, Member};
use network::cut::{sign_cut, Cut, MemberObject};
use network::log::TransparencyLog;
use network::replica::{FeedWriter, Replica, WriterSet};
use p2p::member::{collected, Collector, MemberRole, Progress};
use std::time::Duration;

const NET: [u8; 32] = [2; 32];
const SEEDS: [u8; 3] = [10, 11, 12];

fn members() -> Vec<Member> {
    SEEDS.iter().map(|s| Member::from_seed([*s; 32])).collect()
}

fn consortium() -> Consortium {
    Consortium::new(members().iter().map(|m| m.public()).collect(), 2)
}

fn role(index: usize, cuts_per_epoch: u64, patience: u32) -> MemberRole {
    MemberRole {
        view_patience: 1000,
        ..viewing(index, cuts_per_epoch, patience)
    }
}

/// A member that changes view after three looks without the cut collected.
fn viewing(index: usize, cuts_per_epoch: u64, patience: u32) -> MemberRole {
    MemberRole {
        member: Member::from_seed([SEEDS[index]; 32]),
        index,
        consortium: consortium(),
        every: Duration::from_millis(100),
        cuts_per_epoch,
        patience,
        view_patience: 3,
    }
}

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
            logs: (0..3).map(|_| TransparencyLog::new()).collect(),
            writers,
        }
    }

    fn publish(&mut self, m: usize, object: &MemberObject) {
        let bytes = object.encode();
        let entry = self.logs[m].append(cid(&bytes)).clone();
        self.replica
            .insert(self.writers[m].sign(&entry), bytes)
            .unwrap();
    }

    fn commit(&mut self, m: usize, epoch: u64) {
        let (commit, _) = members()[m].beacon_commit(round(epoch), m);
        self.publish(m, &MemberObject::Commit(commit));
    }

    fn reveal(&mut self, m: usize, epoch: u64) {
        let (_, reveal) = members()[m].beacon_commit(round(epoch), m);
        self.publish(m, &MemberObject::Reveal { epoch, reveal });
    }

    /// Cut `cut` signed by members `by`.
    fn sign(&mut self, cut: &Cut, by: &[usize]) {
        for &m in by {
            let object = sign_cut(&members()[m], m, NET, &consortium(), cut);
            self.publish(m, &object);
        }
    }

    fn run(&mut self, role: &MemberRole, progress: &mut Progress) -> Vec<MemberObject> {
        let out = role.duties(&self.replica, NET, progress);
        for o in &out {
            self.publish(role.index, o);
        }
        out
    }
}

fn round(epoch: u64) -> RoundId {
    RoundId {
        network_id: NET,
        member_set_hash: consortium().member_set_hash(),
        epoch,
    }
}

fn proposal(out: &[MemberObject]) -> Option<&Cut> {
    out.iter().find_map(|o| match o {
        MemberObject::CutSignature { cut, .. } => Some(cut),
        _ => None,
    })
}

/// AT-NET-17: the proposer closes the epoch at its last cut, and waits for every member's
/// commit before closing it.
#[test]
fn at_net_17_the_proposer_closes_once_every_member_committed() {
    let mut net = Net::new();
    let cut0 = Cut::next(&net.replica, None, 0, false);
    net.sign(&cut0, &[0, 2]);
    net.commit(0, 0);
    let me = role(1, 2, 5);
    let mut waited = Progress::default();
    let out = net.run(&me, &mut waited);
    assert!(matches!(out[..], [MemberObject::Commit(_)]));
    assert_eq!(waited.waited, 1, "member 2 has not committed");
    net.commit(2, 0);
    let out = net.run(&me, &mut waited);
    let cut = proposal(&out).expect("every member committed");
    assert!(cut.closes && cut.epoch == 0 && cut.number == 1);
    assert_eq!(waited.waited, 0);
    assert!(
        net.run(&me, &mut waited).is_empty(),
        "one signature per number"
    );
}

/// AT-NET-17: a proposer that does not close proposes at once, commits or not.
#[test]
fn at_net_17_a_cut_that_does_not_close_does_not_wait() {
    let mut net = Net::new();
    let cut0 = Cut::next(&net.replica, None, 0, false);
    net.sign(&cut0, &[0, 2]);
    let mut waited = Progress::default();
    let out = net.run(&role(1, 3, 5), &mut waited);
    let cut = proposal(&out).expect("no commit is needed yet");
    assert!(!cut.closes);
}

/// AT-NET-17: the proposer after the closing cut waits for every committed member's
/// reveal; a member that never commits is not waited for.
#[test]
fn at_net_17_the_reveal_cut_waits_for_the_committed_reveals() {
    let mut net = Net::new();
    net.commit(0, 0);
    net.commit(1, 0);
    let cut0 = Cut::next(&net.replica, None, 0, true);
    net.sign(&cut0, &[0, 1]);
    assert_eq!(collected(&net.replica, &consortium(), NET).len(), 1);
    let me = role(1, 2, 5);
    let mut waited = Progress::default();
    let out = net.run(&me, &mut waited);
    assert!(out
        .iter()
        .any(|o| matches!(o, MemberObject::Reveal { epoch: 0, .. })));
    assert!(proposal(&out).is_none(), "member 0 has not revealed");
    net.reveal(0, 0);
    let out = net.run(&me, &mut waited);
    let cut = proposal(&out).expect("every committed member revealed");
    assert_eq!((cut.number, cut.epoch, cut.closes), (1, 1, false));
}

/// AT-NET-17: a proposer gives up waiting after `patience` looks and proposes anyway.
#[test]
fn at_net_17_patience_runs_out() {
    let mut net = Net::new();
    let cut0 = Cut::next(&net.replica, None, 0, false);
    net.sign(&cut0, &[0, 2]);
    let me = role(1, 2, 3);
    let mut waited = Progress::default();
    for look in 1..=3 {
        assert!(
            proposal(&net.run(&me, &mut waited)).is_none(),
            "look {look}"
        );
        assert_eq!(waited.waited, look);
    }
    let out = net.run(&me, &mut waited);
    assert!(proposal(&out).is_some_and(|c| c.closes));
    assert_eq!(waited.waited, 0);
}

fn changes(out: &[MemberObject]) -> Vec<(u64, u64)> {
    out.iter()
        .filter_map(|o| match o {
            MemberObject::ViewChange { number, view, .. } => Some((*number, *view)),
            _ => None,
        })
        .collect()
}

/// AT-NET-19: a member changes view after `view_patience` looks without the cut, and leaves
/// a view after view 0 only once `t` members reached it, its looks there counting.
#[test]
fn at_net_19_a_view_ends_after_its_patience() {
    let mut net = Net::new();
    let me = viewing(2, 2, 1);
    let mut progress = Progress::default();
    for look in 1..=3 {
        assert!(
            changes(&net.run(&me, &mut progress)).is_empty(),
            "look {look}"
        );
    }
    assert_eq!(changes(&net.run(&me, &mut progress)), vec![(0, 1)]);
    for look in 1..=10 {
        assert!(
            changes(&net.run(&me, &mut progress)).is_empty(),
            "alone in view 1, look {look}"
        );
    }
    net.publish(
        0,
        &MemberObject::ViewChange {
            number: 0,
            view: 1,
            member: 0,
        },
    );
    assert_eq!(
        changes(&net.run(&me, &mut progress)),
        vec![(0, 2)],
        "the looks spent alone count"
    );
}

/// AT-NET-19: a member joins the view `t − 1` others promised, and one ahead of them does
/// not move back.
#[test]
fn at_net_19_a_member_joins_and_never_moves_back() {
    let mut net = Net::new();
    net.publish(
        2,
        &MemberObject::ViewChange {
            number: 0,
            view: 3,
            member: 2,
        },
    );
    let mut progress = Progress::default();
    assert_eq!(
        changes(&net.run(&viewing(0, 2, 1), &mut progress)),
        vec![(0, 3)]
    );
    let mut progress = Progress::default();
    let ahead = viewing(2, 2, 1);
    for _ in 1..=3 {
        assert!(changes(&net.run(&ahead, &mut progress)).is_empty());
    }
}

/// AT-NET-19: the proposer of a view after view 0 proposes only on `t` view changes, and
/// names them.
#[test]
fn at_net_19_a_proposal_waits_for_t_view_changes() {
    let mut net = Net::new();
    net.publish(
        1,
        &MemberObject::ViewChange {
            number: 0,
            view: 1,
            member: 1,
        },
    );
    let me = viewing(1, 2, 1);
    let mut progress = Progress::default();
    assert!(proposal(&net.run(&me, &mut progress)).is_none());
    net.publish(
        2,
        &MemberObject::ViewChange {
            number: 0,
            view: 1,
            member: 2,
        },
    );
    let out = net.run(&me, &mut progress);
    let named = out.iter().find_map(|o| match o {
        MemberObject::CutSignature { view: 1, named, .. } => Some(named.clone()),
        _ => None,
    });
    assert_eq!(named, Some(vec![1, 2]));
}

/// AT-PRO-15: a member collects no cut of the next epoch that counts the signature of a
/// member who withheld its reveal, as the ledger does.
#[test]
fn at_pro_15_members_collect_without_the_withholders() {
    let mut net = Net::new();
    for m in 0..3 {
        net.commit(m, 0);
    }
    let closing = Cut::next(&net.replica, None, 0, true);
    net.sign(&closing, &[0, 1]);
    net.reveal(0, 0);
    net.reveal(1, 0);
    let after = Cut::next(&net.replica, Some(&closing), 1, false);
    net.sign(&after, &[0, 1]);
    let next = Cut::next(&net.replica, Some(&after), 1, false);
    net.sign(&next, &[0, 2]);
    assert_eq!(collected(&net.replica, &consortium(), NET).len(), 2);
    net.sign(&next, &[1]);
    assert_eq!(collected(&net.replica, &consortium(), NET).len(), 3);
}

/// AT-NET-21: the collector extended look after look holds the cuts collected from number 0;
/// a cut signed by `t` members for a number already collected keeps those before it.
#[test]
fn at_net_21_the_collector_matches_collecting_from_zero() {
    let mut net = Net::new();
    let mut collector = Collector::default();
    let mut last: Option<Cut> = None;
    for epoch in [0, 0, 1] {
        let cut = Cut::next(&net.replica, last.as_ref(), epoch, false);
        net.sign(&cut, &[0, 1]);
        let had = collector
            .update(&net.replica, &consortium(), NET)
            .unwrap()
            .to_vec();
        assert_eq!(had, collected(&net.replica, &consortium(), NET));
        last = Some(cut);
    }
    assert_eq!(collector.cuts().len(), 3);
    net.commit(2, 9);
    let other = Cut::next(&net.replica, Some(&collector.cuts()[0]), 0, false);
    net.sign(&other, &[1, 2]);
    let kept = collector
        .update(&net.replica, &consortium(), NET)
        .unwrap()
        .to_vec();
    assert_eq!(kept, collected(&net.replica, &consortium(), NET));
    assert_eq!(kept.len(), 1);
    assert!(collector.conflict());
}
