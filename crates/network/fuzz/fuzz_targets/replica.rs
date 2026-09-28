//! Replication (NET-010, T18): arbitrary messages decoded, and replicas fed honest, forked
//! and raw entries then pulled from each other. No panic; one encoding; convergence.
#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use network::cid::{cid, Cid};
use network::log::Entry;
use network::replica::{FeedWriter, Message, Replica, SignedEntry, WriterSet};

const NET: [u8; 32] = [7; 32];

#[derive(Debug, Arbitrary)]
enum Op {
    /// Writer `w % 3` (the third is outside the set) signs an entry for replica `to % 2`.
    Sign {
        w: u8,
        to: u8,
        seq: u8,
        prev: u8,
        object: Vec<u8>,
    },
    /// Raw bytes decoded as a message; an Entries message is fed to replica `to % 2`.
    Raw { to: u8, bytes: Vec<u8> },
    /// Replica `to % 2` pulls from the other within `cap` bytes.
    Pull { to: u8, cap: u16 },
}

fn pull(to: &mut Replica, from: &Replica, cap: usize) {
    let have = Message::decode(&Message::Have(from.have_for(&to.summary())).encode());
    let Some(Message::Have(have)) = have else {
        panic!("a Have message round-trips")
    };
    let want = to.want(&have);
    if want.is_empty() {
        return;
    }
    let got = from.entries_for(&want, cap);
    assert!(!got.is_empty(), "a wanted entry held by the peer is sent");
    for (e, o) in got {
        to.insert(e, o).expect("an entry a replica holds is valid");
    }
}

fuzz_target!(|ops: Vec<Op>| {
    let writers: Vec<FeedWriter> = (1..=3).map(|i| FeedWriter::from_seed(NET, [i; 32])).collect();
    let keys: Vec<_> = writers[..2].iter().map(|w| w.public()).collect();
    let set = WriterSet::new(NET, &keys);
    let mut replicas = [Replica::new(set.clone()), Replica::new(set.clone())];
    for op in ops {
        match op {
            Op::Sign {
                w,
                to,
                seq,
                prev,
                object,
            } => {
                let r = &mut replicas[usize::from(to % 2)];
                let payload = cid(&object);
                let prev = if prev == 0 {
                    [0; 32]
                } else {
                    r.ids().nth(usize::from(prev)).map_or([prev; 32], |id| id.hash)
                };
                let entry = chained(u64::from(seq), prev, payload);
                let _ = r.insert(writers[usize::from(w % 3)].sign(&entry), object);
            }
            Op::Raw { to, bytes } => {
                if let Some(m) = Message::decode(&bytes) {
                    assert_eq!(m.encode(), bytes, "one encoding per message");
                    if let Message::Entries(list) = m {
                        for (e, o) in list {
                            let _ = replicas[usize::from(to % 2)].insert(e, o);
                        }
                    }
                }
            }
            Op::Pull { to, cap } => {
                let to = usize::from(to % 2);
                let from = replicas[1 - to].clone();
                pull(&mut replicas[to], &from, usize::from(cap));
            }
        }
    }
    for r in &replicas {
        for e in r.equivocations() {
            assert!(e.verify(&set));
        }
        let summary = Message::Summary(r.summary());
        assert_eq!(Message::decode(&summary.encode()), Some(summary));
    }
    let [mut a, mut b] = replicas;
    for _ in 0..(a.len() + b.len() + 1) {
        let b0 = b.clone();
        pull(&mut b, &a, usize::MAX);
        pull(&mut a, &b0, usize::MAX);
    }
    assert_eq!(a.digest(), b.digest(), "two pulls each way converge");
});

/// An entry at `seq` after `prev`, its hash recomputed as decoding does (`network::log`).
fn chained(seq: u64, prev: [u8; 32], payload: Cid) -> Entry {
    let raw = SignedEntry {
        writer: [0; 32],
        entry: Entry {
            seq,
            prev,
            payload,
            hash: [0; 32],
        },
        signature: [0; 64],
    };
    match Message::decode(&Message::Entries(vec![(raw, vec![])]).encode()) {
        Some(Message::Entries(mut list)) => list.remove(0).0.entry,
        _ => panic!("an Entries message of one entry decodes"),
    }
}
