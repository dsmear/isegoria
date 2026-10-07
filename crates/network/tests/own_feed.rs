//! A writer's feed taken back from a replica (`docs/04` §A writer's own feed, `docs/08`
//! G-23, T80): what `Replica::feed_beyond` adopts and where it refuses.

use network::cid::cid;
use network::log::{Entry, TransparencyLog};
use network::replica::{FeedError, FeedWriter, Replica, SignedEntry, WriterSet};
use proptest::prelude::*;

const NET: [u8; 32] = [6; 32];

type Item = (SignedEntry, Vec<u8>);

fn w(seed: u8) -> FeedWriter {
    FeedWriter::from_seed(NET, [seed; 32])
}

fn me() -> [u8; 32] {
    w(1).public().to_bytes()
}

/// `n` chained entries of writer `seed`, objects tagged by `tag`.
fn chain(seed: u8, n: usize, tag: &str) -> Vec<Item> {
    let mut log = TransparencyLog::new();
    (0..n)
        .map(|i| {
            let object = format!("{tag}-{i}").into_bytes();
            let entry = log.append(cid(&object)).clone();
            (w(seed).sign(&entry), object)
        })
        .collect()
}

fn replica<'a>(items: impl IntoIterator<Item = &'a Item>) -> Replica {
    let mut r = Replica::new(WriterSet::new(NET, &[w(1).public(), w(2).public()]));
    for (e, o) in items {
        r.insert(e.clone(), o.clone()).expect("a valid entry");
    }
    r
}

fn log(items: &[Item]) -> Vec<Entry> {
    items.iter().map(|(e, _)| e.entry.clone()).collect()
}

/// The `seq`s `feed_beyond` adopts.
fn adopted(r: &Replica, log: &[Entry]) -> Result<Vec<u64>, FeedError> {
    r.feed_beyond(&me(), log)
        .map(|v| v.iter().map(|(e, _)| e.entry.seq).collect())
}

/// AT-NET-18: an empty log takes the whole feed with its objects; a prefix the rest; a
/// whole log nothing; another writer's entries are ignored.
#[test]
fn at_net_18_the_feed_past_the_log_is_adopted() {
    let feed = chain(1, 4, "a");
    let other = chain(2, 3, "b");
    let r = replica(feed.iter().chain(&other));
    let all = r.feed_beyond(&me(), &[]).unwrap();
    assert_eq!(all.len(), 4);
    for ((e, o), (want_e, want_o)) in all.iter().zip(&feed) {
        assert_eq!((*e, *o), (want_e, want_o.as_slice()));
    }
    assert_eq!(adopted(&r, &log(&feed[..1])), Ok(vec![1, 2, 3]));
    assert_eq!(adopted(&r, &log(&feed)), Ok(vec![]));
    assert_eq!(adopted(&replica(&other), &log(&feed)), Ok(vec![]));
}

/// AT-NET-18: a log longer than the replica's part of the feed adopts what chains after it.
#[test]
fn at_net_18_a_log_ahead_of_a_partial_replica() {
    let feed = chain(1, 6, "a");
    let r = replica([&feed[0], &feed[4], &feed[5]]);
    assert_eq!(adopted(&r, &log(&feed[..4])), Ok(vec![4, 5]));
    assert_eq!(adopted(&r, &log(&feed[..5])), Ok(vec![5]));
}

/// AT-NET-18: an entry the log contradicts, two entries at the next `seq`, or one that does
/// not chain on the log is a fork; an entry past a missing `seq` is a gap.
#[test]
fn at_net_18_forks_and_gaps_are_refused() {
    let feed = chain(1, 3, "a");
    let other = chain(1, 3, "z");
    assert_eq!(
        adopted(&replica([&feed[0], &other[1]]), &log(&feed[..2])),
        Err(FeedError::Forked { seq: 1 })
    );
    assert_eq!(
        adopted(&replica([&feed[0], &feed[1], &other[1]]), &log(&feed[..1])),
        Err(FeedError::Forked { seq: 1 })
    );
    assert_eq!(
        adopted(&replica([&other[0], &other[1]]), &log(&feed[..1])),
        Err(FeedError::Forked { seq: 0 })
    );
    assert_eq!(
        adopted(&replica([&feed[0], &other[1]]), &log(&feed[..1])),
        Err(FeedError::Forked { seq: 1 })
    );
    assert_eq!(
        adopted(&replica([&feed[0], &feed[2]]), &[]),
        Err(FeedError::Gap { seq: 1 })
    );
    assert_eq!(
        adopted(&replica([&feed[2]]), &log(&feed[..1])),
        Err(FeedError::Gap { seq: 1 })
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// AT-NET-18: over any held subset of a feed and any log prefix, the log plus what is
    /// adopted is a prefix of the feed, and the replica holds no entry past it.
    #[test]
    fn at_net_18_the_rebuilt_feed_leaves_no_seq_behind(
        held in proptest::collection::vec(any::<bool>(), 8),
        prefix in 0..=8usize,
    ) {
        let feed = chain(1, 8, "p");
        let r = replica(feed.iter().zip(&held).filter(|(_, h)| **h).map(|(i, _)| i));
        match r.feed_beyond(&me(), &log(&feed[..prefix])) {
            Ok(beyond) => {
                let len = prefix + beyond.len();
                for (i, (e, _)) in beyond.iter().enumerate() {
                    prop_assert_eq!(&e.entry, &feed[prefix + i].0.entry);
                }
                prop_assert!(held[len.min(8)..].iter().all(|h| !h));
            }
            Err(FeedError::Gap { seq }) => {
                let seq = seq as usize;
                prop_assert!(seq >= prefix && !held[seq]);
                prop_assert!(held[seq..].iter().any(|h| *h));
            }
            Err(e) => prop_assert!(false, "no fork in one chain: {:?}", e),
        }
    }
}
