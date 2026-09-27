//! The protocol state from the replicated set (`docs/04` §Cuts, `docs/08` PROTO-015, T74):
//! signed cuts fix the order, the first of conflicting events wins, arrival does not count.

use identity::credential::{AnonymousCredential, Credential, Issuer};
use identity::enrollment::Label;
use identity::nullifier::prove;
use identity::nym::Role;
use network::cid::{cid, Cid};
use network::consortium::{Consortium, Member};
use network::cut::{added, Cut, CutError};
use network::log::TransparencyLog;
use network::replica::{EntryId, FeedWriter, Replica, SignedEntry, WriterSet};
use proptest::prelude::*;
use protocol::deposit::{deposit_context, DepositRejected, Draft};
use protocol::events::NodeEvent;
use protocol::ledger::{Ledger, LedgerError, Refusal};
use protocol::lifecycle::{Event, State};
use protocol::node::Rejection;

const NET: [u8; 32] = [5; 32];
const EPOCH: u64 = 2;

type Item = (SignedEntry, Vec<u8>);

fn members() -> Vec<Member> {
    (0..3).map(|i| Member::from_seed([10 + i; 32])).collect()
}

fn consortium() -> Consortium {
    Consortium::new(members().iter().map(|m| m.public()).collect(), 2)
}

/// Writers 0–2 are the consortium members' keys, 3–4 relays.
fn writers() -> Vec<FeedWriter> {
    [10u8, 11, 12, 20, 21]
        .iter()
        .map(|s| FeedWriter::from_seed(NET, [*s; 32]))
        .collect()
}

fn writer_set() -> WriterSet {
    let keys: Vec<_> = writers().iter().map(|w| w.public()).collect();
    WriterSet::new(NET, &keys)
}

fn issuer() -> Issuer {
    Issuer::new([1u8; 32])
}

fn author(issuer: &Issuer, secret: u8) -> AnonymousCredential {
    let holder = Credential::from_secret([secret; 32]);
    let (req, pending) = holder.request_issuance(&Label([secret; 32]), &issuer.public());
    pending.finalize(issuer.issue(&req).unwrap())
}

fn draft(tag: &str) -> Draft {
    Draft {
        item: tag.as_bytes().to_vec(),
        primary_source: b"Gazzetta Ufficiale".to_vec(),
    }
}

/// A deposit of `tag` by author `secret`, with a quota of one per epoch.
fn deposit(issuer: &Issuer, secret: u8, tag: &str) -> NodeEvent {
    let draft = draft(tag);
    let proof = prove(
        &author(issuer, secret),
        &issuer.public(),
        Role::Propose,
        &deposit_context(draft.content_id(), EPOCH),
    );
    NodeEvent::Deposit {
        epoch: EPOCH,
        quota: 1,
        draft,
        proof,
    }
}

/// `w`'s feed of `objects`, continuing `log`.
fn feed(w: &FeedWriter, log: &mut TransparencyLog, objects: Vec<Vec<u8>>) -> Vec<Item> {
    objects
        .into_iter()
        .map(|o| (w.sign(&log.append(cid(&o)).clone()), o))
        .collect()
}

fn signed(cut: &Cut) -> Vec<(usize, ed25519_dalek::Signature)> {
    let cp = cut.checkpoint(NET, consortium().member_set_hash());
    members()[..2]
        .iter()
        .enumerate()
        .map(|(i, m)| (i, m.sign(&cp)))
        .collect()
}

fn ledger(issuer: &Issuer) -> Ledger {
    Ledger::new(NET, consortium(), issuer.public())
}

fn replica(items: &[Item]) -> Replica {
    let mut r = Replica::new(writer_set());
    for (e, o) in items {
        r.insert(e.clone(), o.clone()).unwrap();
    }
    r
}

/// Two relays carrying the same deposit, one author spending a quota of one on both, and
/// three plain deposits; then a second batch after the first cut.
struct Scenario {
    first: Vec<Item>,
    second: Vec<Item>,
}

fn scenario(issuer: &Issuer) -> Scenario {
    let ws = writers();
    let (mut r1, mut r2) = (TransparencyLog::new(), TransparencyLog::new());
    let same = deposit(issuer, 2, "same").encode();
    let mut first = feed(
        &ws[3],
        &mut r1,
        vec![
            same.clone(),
            deposit(issuer, 3, "q-a").encode(),
            deposit(issuer, 4, "p").encode(),
        ],
    );
    first.extend(feed(
        &ws[4],
        &mut r2,
        vec![
            deposit(issuer, 3, "q-b").encode(),
            same,
            deposit(issuer, 5, "r").encode(),
        ],
    ));
    let mut second = feed(&ws[3], &mut r1, vec![deposit(issuer, 6, "late").encode()]);
    second.extend(feed(&ws[4], &mut r2, vec![b"not an event".to_vec()]));
    Scenario { first, second }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    /// AT-PRO-13: the same cuts give the same state and the same refusals, whatever order
    /// the replica received the entries in.
    #[test]
    fn at_pro_13_arrival_order_does_not_count(
        order in Just((0..8usize).collect::<Vec<_>>()).prop_shuffle(),
    ) {
        let issuer = issuer();
        let s = scenario(&issuer);
        let all: Vec<Item> = s.first.iter().chain(&s.second).cloned().collect();
        let shuffled: Vec<Item> = order.iter().map(|i| all[*i].clone()).collect();
        let (early, late) = (replica(&s.first), replica(&all));
        let cut0 = Cut::of(&early, 0);
        let cut1 = Cut::of(&late, 1);
        let mut a = ledger(&issuer);
        a.apply(&early, &cut0, &signed(&cut0)).unwrap();
        a.apply(&late, &cut1, &signed(&cut1)).unwrap();
        let other = replica(&shuffled);
        let mut b = ledger(&issuer);
        b.apply(&other, &cut0, &signed(&cut0)).unwrap();
        b.apply(&other, &cut1, &signed(&cut1)).unwrap();
        prop_assert_eq!(a.state(), b.state());
        prop_assert_eq!(a.refused(), b.refused());
    }
}

/// Where the deposits of `tag` fall in `order`.
fn position(order: &[EntryId], tag: &str, items: &[Item]) -> Vec<usize> {
    items
        .iter()
        .filter(|(_, o)| {
            matches!(NodeEvent::decode(o), Some(NodeEvent::Deposit { draft, .. }) if draft.item == tag.as_bytes())
        })
        .map(|(e, _)| order.iter().position(|id| *id == e.id()).unwrap())
        .collect()
}

/// AT-PRO-13: of conflicting events the first in the cut's order wins; the others are
/// refused as a single node refuses them, and the cut goes on.
#[test]
fn at_pro_13_the_first_wins() {
    let issuer = issuer();
    let s = scenario(&issuer);
    let r = replica(&s.first);
    let cut = Cut::of(&r, 0);
    let order = added(&r, None, &cut).unwrap();
    let mut l = ledger(&issuer);
    let report = l.apply(&r, &cut, &signed(&cut)).unwrap();
    assert_eq!(report.applied.len(), 4);
    assert_eq!(report.refused.len(), 2);
    let same = position(&order, "same", &s.first);
    let later = order[same[0].max(same[1])];
    assert!(report.refused.contains(&(
        later,
        Refusal::Rejected(Rejection::Deposit(DepositRejected::DuplicateCid))
    )));
    let qa = position(&order, "q-a", &s.first)[0];
    let qb = position(&order, "q-b", &s.first)[0];
    let (winner, loser) = if qa < qb {
        ("q-a", order[qb])
    } else {
        ("q-b", order[qa])
    };
    assert!(report.refused.contains(&(
        loser,
        Refusal::Rejected(Rejection::Deposit(DepositRejected::OverQuota))
    )));
    for tag in [winner, "same", "p", "r"] {
        let id = draft(tag).content_id();
        assert_eq!(l.state().item(&id), Some(&State::Deposited), "{tag}");
    }
    assert_eq!(
        l.refused(),
        report.refused.iter().map(|(id, _)| *id).collect::<Vec<_>>()
    );
}

/// AT-PRO-13: entries past the last cut wait for the next; an object that is not an event
/// is refused without stopping the cut.
#[test]
fn at_pro_13_late_entries_wait_for_the_next_cut() {
    let issuer = issuer();
    let s = scenario(&issuer);
    let early = replica(&s.first);
    let cut0 = Cut::of(&early, 0);
    let mut l = ledger(&issuer);
    l.apply(&early, &cut0, &signed(&cut0)).unwrap();
    let late = replica(&s.first.iter().chain(&s.second).cloned().collect::<Vec<_>>());
    let late_item = draft("late").content_id();
    assert_eq!(l.state().item(&late_item), None);
    let cut1 = Cut::of(&late, 1);
    let report = l.apply(&late, &cut1, &signed(&cut1)).unwrap();
    assert_eq!(report.applied.len(), 1);
    assert_eq!(report.refused.len(), 1);
    assert_eq!(report.refused[0].1, Refusal::NotAnEvent);
    assert_eq!(l.state().item(&late_item), Some(&State::Deposited));
    assert_eq!(l.last(), Some(&cut1));
}

/// AT-PRO-13: a lifecycle step counts only from a consortium member's feed.
#[test]
fn at_pro_13_orchestration_only_from_the_consortium() {
    let issuer = issuer();
    let ws = writers();
    let d = deposit(&issuer, 2, "item");
    let item = draft("item").content_id();
    let step = NodeEvent::Step {
        item,
        event: Event::Admit {
            seed_from_beacon: true,
        },
    }
    .encode();
    let mut items = feed(
        &ws[3],
        &mut TransparencyLog::new(),
        vec![d.encode(), step.clone()],
    );
    let r = replica(&items);
    let cut0 = Cut::of(&r, 0);
    let mut l = ledger(&issuer);
    let report = l.apply(&r, &cut0, &signed(&cut0)).unwrap();
    assert_eq!(
        report.refused,
        vec![(items[1].0.id(), Refusal::NotAuthorized)]
    );
    assert_eq!(l.state().item(&item), Some(&State::Deposited));
    items.extend(feed(&ws[1], &mut TransparencyLog::new(), vec![step]));
    let r = replica(&items);
    let cut1 = Cut::of(&r, 1);
    let report = l.apply(&r, &cut1, &signed(&cut1)).unwrap();
    assert_eq!(report.applied, vec![items[2].0.id()]);
    assert_eq!(l.state().item(&item), Some(&State::Admitted));
}

/// AT-PRO-13: a cut unsigned, out of turn, malformed, retracting a counted entry, or naming
/// an entry the replica lacks is refused and leaves the ledger as it was; the missing entry
/// once synced lets it apply.
#[test]
fn at_pro_13_bad_cuts_are_refused() {
    let issuer = issuer();
    let s = scenario(&issuer);
    let r = replica(&s.first);
    let cut0 = Cut::of(&r, 0);
    let mut l = ledger(&issuer);
    let one = signed(&cut0)[..1].to_vec();
    assert_eq!(l.apply(&r, &cut0, &one).err(), Some(LedgerError::Unsigned));
    let other = Cut::of(&r, 1);
    assert_eq!(
        l.apply(&r, &cut0, &signed(&other)).err(),
        Some(LedgerError::Unsigned)
    );
    assert_eq!(
        l.apply(&r, &other, &signed(&other)).err(),
        Some(LedgerError::Cut(CutError::NotNext { expected: 0 }))
    );
    let mut unsorted = cut0.clone();
    unsorted.marks.reverse();
    assert_eq!(
        l.apply(&r, &unsorted, &signed(&unsorted)).err(),
        Some(LedgerError::Cut(CutError::Malformed))
    );
    let partial = replica(&s.first[1..]);
    let writer = s.first[0].0.writer;
    assert_eq!(
        l.apply(&partial, &cut0, &signed(&cut0)).err(),
        Some(LedgerError::Cut(CutError::Missing { writer }))
    );
    assert_eq!(l.last(), None);
    assert_eq!(l.state(), ledger(&issuer).state());
    l.apply(&r, &cut0, &signed(&cut0)).unwrap();
    let before = l.state().drafts().len();

    let mut shorter = Cut::of(&r, 1);
    shorter.marks[0].len -= 1;
    shorter.marks[0].head = r.feed(&shorter.marks[0].writer)[1].entry.hash;
    let dropped = Cut {
        number: 1,
        marks: cut0.marks[1..].to_vec(),
    };
    let ws = writers();
    let forked_items = feed(&ws[3], &mut TransparencyLog::new(), vec![b"fork".to_vec()]);
    let mut forked_r = r.clone();
    forked_r
        .insert(forked_items[0].0.clone(), forked_items[0].1.clone())
        .unwrap();
    let mut forked = Cut::of(&r, 1);
    let at = forked
        .marks
        .iter()
        .position(|m| m.writer == forked_items[0].0.writer)
        .unwrap();
    forked.marks[at].len = 1;
    forked.marks[at].head = forked_items[0].0.entry.hash;
    for (bad, rep) in [(&shorter, &r), (&dropped, &r), (&forked, &forked_r)] {
        let writer = match l.apply(rep, bad, &signed(bad)) {
            Err(LedgerError::Cut(CutError::Retracts { writer })) => writer,
            other => panic!("expected a retraction, got {other:?}"),
        };
        assert!(cut0.marks.iter().any(|m| m.writer == writer));
    }
    assert_eq!(l.last(), Some(&cut0));
    assert_eq!(l.state().drafts().len(), before);
}

/// AT-PRO-13: a cut's entries apply round by round in a writer rank drawn from the previous
/// cut, each writer's in its own order.
#[test]
fn at_pro_13_the_order_interleaves_writers_by_a_rank_each_cut_redraws() {
    let ws = writers();
    let mut logs: Vec<TransparencyLog> = (0..5).map(|_| TransparencyLog::new()).collect();
    let mut items = Vec::new();
    for (w, n) in [(0usize, 3usize), (2, 1), (3, 2), (4, 3)] {
        let objects = (0..n).map(|i| format!("{w}-{i}").into_bytes()).collect();
        items.extend(feed(&ws[w], &mut logs[w], objects));
    }
    let r = replica(&items);
    let cut0 = Cut::of(&r, 0);
    let order = added(&r, None, &cut0).unwrap();
    assert_eq!(order.len(), 9);
    let firsts: std::collections::BTreeSet<_> = order[..4].iter().map(|id| id.writer).collect();
    assert_eq!(
        firsts.len(),
        4,
        "the first round takes one entry of each writer"
    );
    assert!(order[..4].iter().all(|id| id.seq == 0));
    assert!(order[4..7].iter().all(|id| id.seq == 1));
    assert!(order[7..].iter().all(|id| id.seq == 2));
    let rank = |o: &[EntryId]| o[..4].iter().map(|id| id.writer).collect::<Vec<_>>();
    let mut ranks = std::collections::BTreeSet::new();
    let mut prev = cut0.clone();
    for n in 1..6u64 {
        let mut more = Vec::new();
        for w in [0usize, 2, 3, 4] {
            more.extend(feed(
                &ws[w],
                &mut logs[w],
                vec![format!("{w}-{n}-x").into_bytes()],
            ));
        }
        items.extend(more);
        let r = replica(&items);
        let next = Cut::of(&r, n);
        let order = added(&r, Some(&prev), &next).unwrap();
        assert_eq!(order.len(), 4);
        ranks.insert(rank(&order));
        prev = next;
    }
    assert!(ranks.len() > 1, "the rank changes from cut to cut");
    let _: Cid = cid(b"");
}
