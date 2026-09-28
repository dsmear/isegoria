//! Cuts (T74): arbitrary cut and signed-entry bytes decoded, and arbitrary cuts checked
//! against a replica with forked and partial feeds. No panic; one encoding; `added` names
//! only held entries, each once.
#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use network::cid::cid;
use network::cut::{added, Cut, Mark, MemberObject};
use network::log::TransparencyLog;
use network::replica::{FeedWriter, Replica, SignedEntry, WriterSet};
use std::collections::BTreeSet;

const NET: [u8; 32] = [4; 32];

#[derive(Debug, Arbitrary)]
struct Input {
    cut_bytes: Vec<u8>,
    entry_bytes: Vec<u8>,
    /// Per writer, how many entries of each of two branches the replica holds.
    held: [(u8, u8); 3],
    marks: Vec<(u8, u8, bool, u8)>,
    number: u8,
    prev_marks: Vec<(u8, u8, bool, u8)>,
    epochs: (u8, bool, u8, bool),
}

fuzz_target!(|input: Input| {
    if let Some(cut) = Cut::decode(&input.cut_bytes) {
        assert_eq!(cut.encode(), input.cut_bytes);
    }
    if let Some(e) = SignedEntry::decode(&input.entry_bytes) {
        assert_eq!(e.encode(), input.entry_bytes);
    }
    let writers: Vec<FeedWriter> = (1..=3).map(|i| FeedWriter::from_seed(NET, [i; 32])).collect();
    let keys: Vec<_> = writers.iter().map(|w| w.public()).collect();
    let mut replica = Replica::new(WriterSet::new(NET, &keys));
    let mut heads = Vec::new();
    for (w, (a, b)) in writers.iter().zip(input.held) {
        let mut branch = Vec::new();
        for (tag, n) in [("a", a % 6), ("b", b % 6)] {
            let mut log = TransparencyLog::new();
            let mut hs = Vec::new();
            for i in 0..n {
                let o = format!("{tag}-{i}").into_bytes();
                let e = log.append(cid(&o)).clone();
                hs.push(e.hash);
                let _ = replica.insert(w.sign(&e), o);
            }
            branch.push(hs);
        }
        heads.push((w.public().to_bytes(), branch));
    }
    let mark = |(w, len, b, h): (u8, u8, bool, u8)| {
        let (writer, branch) = &heads[usize::from(w % 3)];
        let hs = &branch[usize::from(b)];
        let head = if h == 0 {
            [h; 32]
        } else {
            hs.get(usize::from(len).wrapping_sub(1)).copied().unwrap_or([h; 32])
        };
        Mark {
            writer: *writer,
            len: u64::from(len % 8),
            head,
        }
    };
    let prev = Cut {
        number: u64::from(input.number),
        epoch: u64::from(input.epochs.0 % 4),
        closes: input.epochs.1,
        marks: input.prev_marks.into_iter().map(mark).collect(),
    };
    let next = Cut {
        number: u64::from(input.number) + 1,
        epoch: u64::from(input.epochs.2 % 4),
        closes: input.epochs.3,
        marks: input.marks.into_iter().map(mark).collect(),
    };
    if let Ok(ids) = added(&replica, Some(&prev), &next) {
        assert!(next.epoch >= prev.epoch && !(prev.closes && next.epoch == prev.epoch));
        let _ = ids;
    }
    let proposal = Cut::next(&replica, Some(&prev), next.epoch, next.closes);
    if prev.is_well_formed() && added(&replica, None, &prev).is_ok() && next.epoch > prev.epoch {
        assert!(added(&replica, Some(&prev), &proposal).is_ok(), "a proposal extends");
    }
    if let Some(o) = MemberObject::decode(&input.cut_bytes) {
        assert_eq!(o.encode(), input.cut_bytes);
    }
    for (p, n) in [(None, &prev), (Some(&prev), &next)] {
        if let Ok(ids) = added(&replica, p, n) {
            let distinct: BTreeSet<_> = ids.iter().collect();
            assert_eq!(distinct.len(), ids.len(), "each entry once");
            assert!(ids.iter().all(|id| replica.contains(id)));
        }
    }
});
