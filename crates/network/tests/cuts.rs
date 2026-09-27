//! Cuts and the replica on disk (`docs/04` §Cuts, §A replica on disk, `docs/08` §10.3, T74).

use network::cid::cid;
use network::consortium::{Checkpoint, Consortium, Member};
use network::cut::{added, Cut, CutError, Mark};
use network::log::TransparencyLog;
use network::replica::{
    Accepted, DiskError, DurableReplica, FeedWriter, Refused, Replica, SignedEntry, WriterSet,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const NET: [u8; 32] = [4; 32];

type Item = (SignedEntry, Vec<u8>);

fn scratch(name: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("cuts")
        .join(format!("{name}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn writers() -> Vec<FeedWriter> {
    (1..=3)
        .map(|i| FeedWriter::from_seed(NET, [i; 32]))
        .collect()
}

fn set() -> WriterSet {
    let keys: Vec<_> = writers().iter().map(|w| w.public()).collect();
    WriterSet::new(NET, &keys)
}

fn feed(w: &FeedWriter, n: usize, tag: &str) -> Vec<Item> {
    let mut log = TransparencyLog::new();
    (0..n)
        .map(|i| {
            let o = format!("{tag}-{i}").into_bytes();
            (w.sign(&log.append(cid(&o)).clone()), o)
        })
        .collect()
}

fn replica(items: &[Item]) -> Replica {
    let mut r = Replica::new(set());
    for (e, o) in items {
        r.insert(e.clone(), o.clone()).unwrap();
    }
    r
}

fn corpus() -> Vec<Item> {
    let ws = writers();
    let mut items = feed(&ws[0], 3, "a");
    items.extend(feed(&ws[1], 2, "b"));
    items
}

/// AT-NET-15: a cut marks each feed, round-trips, and its digest and checkpoint bind every
/// field; malformed encodings are refused.
#[test]
fn at_net_15_a_cut_binds_its_marks() {
    let r = replica(&corpus());
    let cut = Cut::of(&r, 3);
    assert_eq!(cut.marks.len(), 2);
    assert!(cut
        .marks
        .iter()
        .all(|m| m.len == r.feed(&m.writer).len() as u64));
    assert!(Cut::of(&Replica::new(set()), 0).marks.is_empty());
    let bytes = cut.encode();
    assert_eq!(Cut::decode(&bytes), Some(cut.clone()));
    for at in 0..bytes.len() {
        assert!(Cut::decode(&bytes[..at]).is_none(), "cut at {at}");
    }
    let mut longer = bytes.clone();
    longer.push(0);
    assert!(Cut::decode(&longer).is_none());
    let mut version = bytes.clone();
    version[0] = 2;
    assert!(Cut::decode(&version).is_none());
    let mut changed = vec![Cut {
        number: 4,
        ..cut.clone()
    }];
    for i in 0..2 {
        let mut c = cut.clone();
        c.marks[i].len += 1;
        changed.push(c.clone());
        c.marks[i].head[0] ^= 1;
        changed.push(c);
    }
    for c in &changed {
        assert_ne!(c.digest(), cut.digest());
    }
    let cp = cut.checkpoint(NET, [9; 32]);
    assert_eq!(cp, Checkpoint::new(NET, [9; 32], 3, cut.digest()));
    let mut unsorted = cut.clone();
    unsorted.marks.reverse();
    assert!(!unsorted.is_well_formed());
    assert!(Cut::decode(&unsorted.encode()).is_none());
    let mut twice = cut.clone();
    twice.marks[1] = twice.marks[0];
    assert!(Cut::decode(&twice.encode()).is_none());
    let mut empty = cut.clone();
    empty.marks[0].len = 0;
    assert!(Cut::decode(&empty.encode()).is_none());
    let mut ragged = cut.encode();
    ragged.pop();
    let len = (ragged.len() - 17) as u64;
    ragged[9..17].copy_from_slice(&len.to_le_bytes());
    assert!(Cut::decode(&ragged).is_none());
}

/// AT-NET-15: a chain is found only whole, by its head, including a forked writer's branch.
#[test]
fn at_net_15_a_chain_is_found_by_its_head() {
    let ws = writers();
    let a = feed(&ws[0], 3, "a");
    let b = feed(&ws[0], 3, "x");
    let mut items = a.clone();
    items.extend(b.clone());
    let r = replica(&items);
    let k = ws[0].public().to_bytes();
    let ids = |c: Vec<&SignedEntry>| c.iter().map(|e| e.id()).collect::<Vec<_>>();
    let want = |f: &[Item]| f.iter().map(|(e, _)| e.id()).collect::<Vec<_>>();
    assert_eq!(ids(r.chain(&k, 3, a[2].0.entry.hash).unwrap()), want(&a));
    assert_eq!(ids(r.chain(&k, 3, b[2].0.entry.hash).unwrap()), want(&b));
    assert_eq!(
        ids(r.chain(&k, 2, a[1].0.entry.hash).unwrap()),
        want(&a[..2])
    );
    assert!(r.chain(&k, 3, a[1].0.entry.hash).is_none());
    assert!(r.chain(&k, 2, a[2].0.entry.hash).is_none());
    assert_eq!(r.chain(&k, 0, [0; 32]).unwrap().len(), 0);
    assert!(r.chain(&k, 0, [1; 32]).is_none());
    assert!(replica(&a[1..]).chain(&k, 3, a[2].0.entry.hash).is_none());
}

/// AT-NET-15: `added` refuses cuts out of turn, malformed, retracting or naming what the
/// replica lacks, and names each writer's new entries once.
#[test]
fn at_net_15_added_checks_the_cut() {
    let items = corpus();
    let r = replica(&items);
    let cut0 = Cut::of(&replica(&items[..4]), 0);
    let cut1 = Cut::of(&r, 1);
    assert_eq!(added(&r, None, &cut0).unwrap().len(), 4);
    assert_eq!(
        added(&r, Some(&cut0), &cut1).unwrap(),
        vec![items[4].0.id()]
    );
    assert_eq!(added(&r, Some(&cut1), &Cut::of(&r, 2)).unwrap(), vec![]);
    assert_eq!(
        added(&r, None, &cut1),
        Err(CutError::NotNext { expected: 0 })
    );
    assert_eq!(
        added(&r, Some(&cut0), &cut0),
        Err(CutError::NotNext { expected: 1 })
    );
    let lacking = replica(&items[1..]);
    let writer = items[0].0.writer;
    assert_eq!(
        added(&lacking, None, &cut0),
        Err(CutError::Missing { writer })
    );
    let mut bad = cut0.clone();
    bad.marks.swap(0, 1);
    assert_eq!(added(&r, None, &bad), Err(CutError::Malformed));
    let gone = Cut {
        number: 1,
        marks: vec![cut1.marks[0]],
    };
    let dropped = cut0.marks[1].writer;
    assert_eq!(
        added(&r, Some(&cut0), &gone),
        Err(CutError::Retracts { writer: dropped })
    );
    let mut shorter = cut1.clone();
    let m = shorter.marks.iter_mut().find(|m| m.len == 3).unwrap();
    *m = Mark {
        len: 1,
        head: r.feed(&m.writer)[0].entry.hash,
        ..*m
    };
    let w = m.writer;
    assert_eq!(
        added(
            &r,
            Some(&cut1),
            &Cut {
                number: 2,
                ..shorter
            }
        ),
        Err(CutError::Retracts { writer: w })
    );
}

/// AT-NET-15: a replica on disk reopens to the set it held; damage and entries its writer
/// set refuses stop it from opening.
#[test]
fn at_net_15_a_replica_on_disk_survives_a_restart() {
    let dir = scratch("reopen");
    let items = corpus();
    let (e, o) = &items[0];
    {
        let mut d = DurableReplica::open(&dir, set()).unwrap();
        for (e, o) in &items {
            assert_eq!(d.insert(e.clone(), o.clone()), Ok(Accepted::New));
        }
        assert_eq!(d.insert(e.clone(), o.clone()), Ok(Accepted::Duplicate));
        assert_eq!(
            d.insert(e.clone(), b"other".to_vec()),
            Err(DiskError::Refused(Refused::ObjectMismatch))
        );
    }
    let d = DurableReplica::open(&dir, set()).unwrap();
    assert_eq!(d.replica().digest(), replica(&items).digest());
    let ws = writers();
    let others = WriterSet::new(NET, &[ws[1].public()]);
    assert_eq!(
        DurableReplica::open(&dir, others).err(),
        Some(DiskError::Refused(Refused::UnknownWriter))
    );
    let entries = dir.join("entries");
    let mut bytes = std::fs::read(&entries).unwrap();
    let at = 16 + 8 + 20;
    bytes[at] ^= 1;
    std::fs::write(&entries, &bytes).unwrap();
    assert!(DurableReplica::open(&dir, set()).is_err());
    let lost = scratch("lost");
    {
        let mut d = DurableReplica::open(&lost, set()).unwrap();
        d.insert(e.clone(), o.clone()).unwrap();
    }
    std::fs::write(lost.join("objects"), b"isegoria-obj-v1\n").unwrap();
    assert_eq!(
        DurableReplica::open(&lost, set()).err(),
        Some(DiskError::Damaged)
    );
}

/// AT-NET-15: a signed entry's encoding is 168 bytes and one per entry.
#[test]
fn at_net_15_signed_entries_round_trip() {
    for (e, _) in corpus() {
        let bytes = e.encode();
        assert_eq!(bytes.len(), 168);
        assert_eq!(SignedEntry::decode(&bytes), Some(e));
        assert!(SignedEntry::decode(&bytes[..167]).is_none());
        let mut longer = bytes;
        longer.push(0);
        assert!(SignedEntry::decode(&longer).is_none());
    }
}

/// AT-NET-15: a previous cut with a mark of length 0 is refused, not a panic (F11, found by
/// `fuzz/cut`).
#[test]
fn at_net_15_a_malformed_previous_cut_is_refused() {
    let items = corpus();
    let r = replica(&items);
    let mut prev = Cut::of(&r, 0);
    prev.marks[0].len = 0;
    let writer = prev.marks[0].writer;
    assert_eq!(
        added(&r, Some(&prev), &Cut::of(&r, 1)),
        Err(CutError::Retracts { writer })
    );
}

/// AT-NET-15: a writer whose feed is empty (its first entry missing) gets no mark, and the
/// consortium answers which writer keys are its members'.
#[test]
fn at_net_15_empty_feeds_get_no_mark_and_members_are_known() {
    let ws = writers();
    let mut items = corpus();
    items.extend(feed(&ws[2], 2, "c").into_iter().skip(1));
    let cut = Cut::of(&replica(&items), 0);
    assert_eq!(cut.marks.len(), 2);
    assert!(cut.is_well_formed());
    let members: Vec<Member> = (1..=2).map(|i| Member::from_seed([i; 32])).collect();
    let c = Consortium::new(members.iter().map(|m| m.public()).collect(), 1);
    assert!(c.is_member(&ws[0].public().to_bytes()));
    assert!(c.is_member(&ws[1].public().to_bytes()));
    assert!(!c.is_member(&ws[2].public().to_bytes()));
}
