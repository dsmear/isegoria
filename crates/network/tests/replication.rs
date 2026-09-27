//! Replication between nodes (`docs/04` §Replication between nodes, `docs/08` NET-010,
//! §10.3, T18): the replicated set, its convergence, the pull sync and its messages.

use network::cid::cid;
use network::log::TransparencyLog;
use network::replica::{
    Accepted, EntryId, Equivocation, FeedWriter, Message, Refused, Replica, SignedEntry, Summary,
    WriterSet, MAX_RESPONSE,
};
use network::store::MAX_OBJECT;
use proptest::prelude::*;

const NET: [u8; 32] = [7; 32];

type Item = (SignedEntry, Vec<u8>);

fn writers(n: u8) -> Vec<FeedWriter> {
    (0..n)
        .map(|i| FeedWriter::from_seed(NET, [i + 1; 32]))
        .collect()
}

fn set(ws: &[FeedWriter]) -> WriterSet {
    let keys: Vec<_> = ws.iter().map(|w| w.public()).collect();
    WriterSet::new(NET, &keys)
}

/// `n` chained entries by `w`, objects tagged by `tag`.
fn feed(w: &FeedWriter, n: usize, tag: &str) -> Vec<Item> {
    let mut log = TransparencyLog::new();
    (0..n)
        .map(|i| {
            let object = format!("{tag}-{i}").into_bytes();
            let entry = log.append(cid(&object)).clone();
            (w.sign(&entry), object)
        })
        .collect()
}

fn replica(ws: &[FeedWriter], items: &[Item]) -> Replica {
    let mut r = Replica::new(set(ws));
    for (e, o) in items {
        r.insert(e.clone(), o.clone()).expect("a valid entry");
    }
    r
}

/// One pull round: `to` asks `from` for what it lacks, entries within `cap` bytes.
fn pull(to: &mut Replica, from: &Replica, cap: usize) -> usize {
    let have = from.have_for(&to.summary());
    let want = to.want(&have);
    if want.is_empty() {
        return 0;
    }
    let got = from.entries_for(&want, cap);
    let n = got.len();
    for (e, o) in got {
        assert_eq!(to.insert(e, o).map(|_| ()), Ok(()));
    }
    n
}

/// Three writers' feeds, the third forked at `seq` 2 with a second branch.
fn corpus(ws: &[FeedWriter]) -> Vec<Item> {
    let mut items = feed(&ws[0], 6, "a");
    items.extend(feed(&ws[1], 3, "b"));
    items.extend(feed(&ws[2], 4, "c"));
    items.extend(feed(&ws[2], 4, "c'").into_iter().skip(2));
    items
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// AT-NET-13: the same set in any order, with duplicates, reads the same everywhere.
    #[test]
    fn at_net_13_the_set_not_the_order_decides(
        order in Just((0..15usize).collect::<Vec<_>>()).prop_shuffle(),
        dups in proptest::collection::vec(0..15usize, 0..6),
    ) {
        let ws = writers(3);
        let items = corpus(&ws);
        let reference = replica(&ws, &items);
        let mut other = Replica::new(set(&ws));
        for i in order.iter().chain(&dups) {
            let (e, o) = &items[*i];
            other.insert(e.clone(), o.clone()).unwrap();
        }
        prop_assert_eq!(other.digest(), reference.digest());
        prop_assert_eq!(other.equivocations(), reference.equivocations());
        prop_assert_eq!(other.summary(), reference.summary());
        for w in &ws {
            let k = w.public().to_bytes();
            prop_assert_eq!(other.feed(&k), reference.feed(&k));
        }
    }

    /// AT-NET-13: replicas holding any subsets converge on the union by pull rounds, each
    /// bringing at least one entry while the peer holds more.
    #[test]
    fn at_net_13_pull_rounds_converge_on_the_union(
        masks in proptest::collection::vec(0u8..8, 15),
        small in any::<bool>(),
    ) {
        let ws = writers(3);
        let items = corpus(&ws);
        let held: Vec<Item> = items
            .iter()
            .zip(&masks)
            .filter(|(_, m)| **m != 0)
            .map(|(i, _)| i.clone())
            .collect();
        let union = replica(&ws, &held);
        let mut nodes: Vec<Replica> = (0..3)
            .map(|n| {
                let held: Vec<Item> = items
                    .iter()
                    .zip(&masks)
                    .filter(|(_, m)| *m & (1 << n) != 0)
                    .map(|(i, _)| i.clone())
                    .collect();
                replica(&ws, &held)
            })
            .collect();
        let cap = if small { 1 } else { MAX_RESPONSE };
        for _ in 0..40 {
            let mut moved = 0;
            for (a, b) in [(0, 1), (1, 0), (1, 2), (2, 1), (0, 2), (2, 0)] {
                let from = nodes[b].clone();
                let before = nodes[a].len();
                let missing = from.ids().filter(|id| !nodes[a].contains(id)).count();
                let got = pull(&mut nodes[a], &from, cap);
                prop_assert!(missing == 0 || got >= 1, "a round with more brought nothing");
                prop_assert_eq!(nodes[a].len(), before + got);
                moved += got;
            }
            if moved == 0 {
                break;
            }
        }
        for n in &nodes {
            prop_assert_eq!(n.digest(), union.digest());
        }
    }
}

/// AT-NET-13: two nodes that pull in each direction hold the same set, fork included.
#[test]
fn at_net_13_one_round_each_way_equalises_two_nodes() {
    let ws = writers(3);
    let items = corpus(&ws);
    let half = |even: bool| -> Vec<Item> {
        items
            .iter()
            .enumerate()
            .filter(|(i, _)| (i % 2 == 0) == even)
            .map(|(_, x)| x.clone())
            .collect()
    };
    let (mut a, mut b) = (replica(&ws, &half(true)), replica(&ws, &half(false)));
    assert_ne!(a.digest(), b.digest());
    let b0 = b.clone();
    pull(&mut b, &a, MAX_RESPONSE);
    pull(&mut a, &b0, MAX_RESPONSE);
    assert_eq!(a.digest(), b.digest());
    assert_eq!(a.digest(), replica(&ws, &items).digest());
    assert_eq!(a.equivocations().len(), 2, "the fork at seq 2 and 3");
}

/// AT-NET-13: a peer whose feed is a prefix of ours is offered only what follows it.
#[test]
fn at_net_13_a_prefix_is_not_offered_again() {
    let ws = writers(2);
    let a_items = feed(&ws[0], 6, "a");
    let full = replica(&ws, &a_items);
    let behind = replica(&ws, &a_items[..4]);
    let offered = full.have_for(&behind.summary());
    let seqs: Vec<u64> = offered.iter().map(|id| id.seq).collect();
    assert_eq!(seqs, vec![4, 5]);
    assert!(full.have_for(&full.summary()).is_empty());
    let mut others = a_items[..4].to_vec();
    others.extend(feed(&ws[1], 2, "b"));
    let offered = replica(&ws, &others).have_for(&behind.summary());
    assert_eq!(offered.len(), 2, "only the writer the peer lacks");
    assert!(offered
        .iter()
        .all(|id| id.writer == ws[1].public().to_bytes()));
    let empty = Replica::new(set(&ws));
    assert!(empty.is_empty() && !full.is_empty());
    assert_eq!(full.have_for(&empty.summary()).len(), 6);
    let stranger = replica(&ws, &feed(&ws[0], 4, "x"));
    assert_eq!(
        full.have_for(&stranger.summary()).len(),
        6,
        "another head: all of it"
    );
}

/// AT-NET-13: entries from outside the writer set, badly signed, or with the wrong or an
/// oversized object are refused and leave the replica as it was.
#[test]
fn at_net_13_bad_entries_are_refused() {
    let ws = writers(2);
    let (e, o) = feed(&ws[0], 1, "a").remove(0);
    let mut r = Replica::new(set(&ws[1..]));
    assert!(!r.writers().contains(&ws[0].public()) && r.writers().contains(&ws[1].public()));
    assert_eq!(r.writers().network_id(), NET);
    assert_eq!(r.insert(e.clone(), o.clone()), Err(Refused::UnknownWriter));
    let mut r = Replica::new(set(&ws));
    let mut forged = e.clone();
    forged.signature[5] ^= 1;
    assert_eq!(r.insert(forged, o.clone()), Err(Refused::BadSignature));
    let elsewhere = FeedWriter::from_seed([8; 32], [1; 32]).sign(&e.entry);
    assert_eq!(r.insert(elsewhere, o.clone()), Err(Refused::BadSignature));
    assert_eq!(
        r.insert(e.clone(), b"other".to_vec()),
        Err(Refused::ObjectMismatch)
    );
    let big = vec![0u8; MAX_OBJECT + 1];
    let mut log = TransparencyLog::new();
    let entry = log.append(cid(&big)).clone();
    assert_eq!(
        r.insert(ws[0].sign(&entry), big),
        Err(Refused::ObjectTooLarge)
    );
    let fits = vec![0u8; MAX_OBJECT];
    let mut log = TransparencyLog::new();
    let entry = log.append(cid(&fits)).clone();
    assert_eq!(r.insert(ws[1].sign(&entry), fits), Ok(Accepted::New));
    assert_eq!(r.len(), 1);
    assert_eq!(r.insert(e.clone(), o.clone()), Ok(Accepted::New));
    assert_eq!(r.insert(e, o), Ok(Accepted::Duplicate));
    assert_eq!(r.len(), 2);
}

/// AT-NET-13: a fork is evidence that verifies, the feed stops before it, and the pair
/// reported is the one with the smallest hashes.
#[test]
fn at_net_13_equivocation_is_evidence_and_stops_the_feed() {
    let ws = writers(2);
    let k = ws[0].public().to_bytes();
    let branches: Vec<Vec<Item>> = ["x", "y", "z"].iter().map(|t| feed(&ws[0], 4, t)).collect();
    let mut r = Replica::new(set(&ws));
    for (e, o) in &branches[0] {
        assert_eq!(r.insert(e.clone(), o.clone()), Ok(Accepted::New));
    }
    assert_eq!(r.feed(&k).len(), 4);
    let (e, o) = branches[1][1].clone();
    assert_eq!(r.insert(e, o), Ok(Accepted::Equivocation));
    let (e, o) = branches[2][1].clone();
    assert_eq!(r.insert(e, o), Ok(Accepted::Equivocation));
    assert_eq!(r.feed(&k).len(), 1);
    let found = r.equivocations();
    assert_eq!(found.len(), 1);
    let mut at1: Vec<SignedEntry> = branches.iter().map(|b| b[1].0.clone()).collect();
    at1.sort_by_key(|e| e.entry.hash);
    assert_eq!((&found[0].a, &found[0].b), (&at1[0], &at1[1]));
    assert!(found[0].verify(&set(&ws)));
    assert!(!found[0].verify(&set(&ws[1..])));
    let same = Equivocation {
        a: at1[0].clone(),
        b: at1[0].clone(),
    };
    assert!(!same.verify(&set(&ws)));
    let other_seq = Equivocation {
        a: branches[0][0].0.clone(),
        b: at1[1].clone(),
    };
    assert!(!other_seq.verify(&set(&ws)));
    let other_writer = Equivocation {
        a: at1[0].clone(),
        b: ws[1].sign(&at1[1].entry),
    };
    assert!(!other_writer.verify(&set(&ws)));
    let mut forged = found[0].clone();
    forged.b.signature[0] ^= 1;
    assert!(!forged.verify(&set(&ws)));
}

/// AT-NET-13: a feed stops at a gap and at a link that does not chain.
#[test]
fn at_net_13_a_feed_stops_at_a_gap_or_a_break() {
    let ws = writers(1);
    let k = ws[0].public().to_bytes();
    let items = feed(&ws[0], 5, "a");
    let gap: Vec<Item> = items
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != 2)
        .map(|(_, x)| x.clone())
        .collect();
    assert_eq!(replica(&ws, &gap).feed(&k).len(), 2);
    let mut broken = items[..2].to_vec();
    broken.extend(feed(&ws[0], 4, "b").into_iter().skip(2));
    assert_eq!(replica(&ws, &broken).feed(&k).len(), 2);
    assert!(replica(&ws, &items[1..]).feed(&k).is_empty());
    assert_eq!(replica(&ws, &items).feed(&k).len(), 5);
    let s = replica(&ws, &gap).summary();
    assert_eq!((s.0[0].count, s.0[0].feed_len), (4, 2));
    assert_eq!(s.0[0].head, items[1].0.entry.hash);
}

fn messages() -> Vec<Message> {
    let ws = writers(3);
    let items = corpus(&ws);
    let r = replica(&ws, &items);
    let ids: Vec<EntryId> = r.ids().collect();
    vec![
        Message::Summary(r.summary()),
        Message::Summary(Summary::default()),
        Message::Have(ids.clone()),
        Message::Want(ids[..3].to_vec()),
        Message::Want(vec![]),
        Message::Entries(r.entries_for(&ids, MAX_RESPONSE)),
    ]
}

/// AT-NET-13: every message round-trips; cuts, trailing bytes, unknown kinds and
/// out-of-order records are refused.
#[test]
fn at_net_13_messages_have_one_encoding() {
    for m in messages() {
        let bytes = m.encode();
        assert_eq!(Message::decode(&bytes), Some(m.clone()));
        for cut in 0..bytes.len() {
            assert!(Message::decode(&bytes[..cut]).is_none(), "cut at {cut}");
        }
        let mut longer = bytes.clone();
        longer.push(0);
        assert!(Message::decode(&longer).is_none());
        let mut version = bytes.clone();
        version[0] = 2;
        assert!(Message::decode(&version).is_none());
    }
    for kind in [0u8, 5] {
        assert!(Message::decode(&[1, kind, 0, 0, 0, 0, 0, 0, 0, 0]).is_none());
    }
    let ws = writers(3);
    let r = replica(&ws, &corpus(&ws));
    let mut ids: Vec<EntryId> = r.ids().collect();
    let mut summary = r.summary();
    summary.0.swap(0, 1);
    assert!(Message::decode(&Message::Summary(summary.clone()).encode()).is_none());
    summary.0[0] = summary.0[1];
    assert!(Message::decode(&Message::Summary(summary).encode()).is_none());
    ids.swap(0, 1);
    assert!(Message::decode(&Message::Have(ids.clone()).encode()).is_none());
    let dup = vec![ids[0], ids[0]];
    assert!(Message::decode(&Message::Want(dup).encode()).is_none());
    let mut entries = r.entries_for(&ids, MAX_RESPONSE);
    entries.swap(0, 1);
    assert!(Message::decode(&Message::Entries(entries.clone()).encode()).is_none());
    entries[0] = entries[1].clone();
    assert!(Message::decode(&Message::Entries(entries).encode()).is_none());
    let mut ragged = Message::Have(vec![ids[0]]).encode();
    ragged.pop();
    ragged[2..10].copy_from_slice(&71u64.to_le_bytes());
    assert!(
        Message::decode(&ragged).is_none(),
        "an identifier field of 71 bytes"
    );
    let mut ragged = Message::Summary(r.summary()).encode();
    ragged.pop();
    let len = (ragged.len() - 10) as u64;
    ragged[2..10].copy_from_slice(&len.to_le_bytes());
    assert!(
        Message::decode(&ragged).is_none(),
        "a summary field not a multiple of 112"
    );
}

/// AT-NET-13: an Entries message stops at the cap but always carries one entry, and
/// never an entry it was not asked for or does not hold.
#[test]
fn at_net_13_entries_respect_the_cap_and_the_request() {
    let ws = writers(3);
    let items = corpus(&ws);
    let r = replica(&ws, &items);
    let ids: Vec<EntryId> = r.ids().collect();
    let per = items[0].1.len() + 168;
    assert_eq!(r.entries_for(&ids, 0).len(), 1);
    assert_eq!(r.entries_for(&ids, per).len(), 1);
    assert_eq!(r.entries_for(&ids, 2 * per).len(), 2);
    let mut asked = vec![ids[3], ids[1], ids[3]];
    let stranger = EntryId {
        writer: [9; 32],
        seq: 0,
        hash: [0; 32],
    };
    asked.push(stranger);
    let got: Vec<EntryId> = r
        .entries_for(&asked, MAX_RESPONSE)
        .iter()
        .map(|(e, _)| e.id())
        .collect();
    assert_eq!(got, vec![ids[1], ids[3]]);
    assert_eq!(r.want(&[ids[0], stranger]), vec![stranger]);
    let (e, o) = r.get(&ids[0]).unwrap();
    assert_eq!((e.id(), cid(o)), (ids[0], e.entry.payload));
    assert!(r.get(&stranger).is_none());
}
