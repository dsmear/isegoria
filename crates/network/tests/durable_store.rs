//! A node's durable log and object store (`docs/04` §A node's own disk, `docs/08` NET-011,
//! T13): a reopened log is the acknowledged log, a torn tail is cut, corruption is refused.

use network::cid::{cid, Cid};
use network::store::{DurableLog, ObjectStore, Recovery, StoreError, MAX_OBJECT};
use proptest::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const HEADER: usize = 16;
const RECORD: usize = 104;

fn scratch(name: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("durable_store");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!(
        "{name}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_file(&path);
    path
}

fn payloads(n: u8) -> Vec<Cid> {
    (0..n).map(|i| cid(&[i])).collect()
}

/// A log file holding `n` acknowledged entries; returns its path and bytes.
fn written_log(name: &str, n: u8) -> (PathBuf, Vec<u8>) {
    let path = scratch(name);
    let (mut log, _) = DurableLog::open(&path).unwrap();
    for p in payloads(n) {
        log.append(p).unwrap();
    }
    drop(log);
    let bytes = fs::read(&path).unwrap();
    (path, bytes)
}

/// AT-NET-11: after a restart the log is entry for entry the log that was acknowledged.
#[test]
fn at_net_11_a_reopened_log_is_the_acknowledged_log() {
    let path = scratch("reopen");
    let (mut log, recovery) = DurableLog::open(&path).unwrap();
    assert_eq!(recovery, Recovery::default());
    for p in payloads(20) {
        log.append(p).unwrap();
    }
    let entries = log.log().entries().to_vec();
    let cp = log.log().checkpoint([1; 32], [2; 32]);
    drop(log);

    let (mut again, recovery) = DurableLog::open(&path).unwrap();
    assert_eq!(recovery, Recovery::default());
    assert_eq!(again.log().entries(), &entries[..]);
    assert_eq!(again.log().head(), cp.head);
    assert!(again.log().verify());
    assert_eq!(again.log().verify_extends(&cp), Ok(()));
    assert!(payloads(20).iter().all(|p| again.log().contains(p)));
    let next = again.append(cid(b"after the restart")).unwrap();
    assert_eq!(next.seq, 20);
    drop(again);
    let (third, _) = DurableLog::open(&path).unwrap();
    assert_eq!(third.log().len(), 21);
    assert_eq!(fs::read(&path).unwrap().len(), HEADER + 21 * RECORD);
}

/// AT-NET-11: a crash at any byte keeps every complete entry and cuts only the torn tail.
#[test]
fn at_net_11_a_crash_at_any_byte_keeps_every_complete_entry() {
    let (_, bytes) = written_log("full", 5);
    let (reference, _) = {
        let path = scratch("reference");
        fs::write(&path, &bytes).unwrap();
        DurableLog::open(&path).unwrap()
    };
    for cut in 0..=bytes.len() {
        let path = scratch("cut");
        fs::write(&path, &bytes[..cut]).unwrap();
        let (mut log, recovery) = DurableLog::open(&path).unwrap();
        let complete = cut.saturating_sub(HEADER) / RECORD;
        let torn = if cut < HEADER {
            cut
        } else {
            (cut - HEADER) % RECORD
        };
        assert_eq!(log.log().len(), complete, "cut at {cut}");
        assert_eq!(recovery.torn_bytes, torn as u64, "cut at {cut}");
        assert_eq!(log.log().entries(), &reference.log().entries()[..complete]);
        log.append(cid(b"next")).unwrap();
        drop(log);
        let (again, recovery) = DurableLog::open(&path).unwrap();
        assert_eq!(again.log().len(), complete + 1, "cut at {cut}");
        assert_eq!(recovery, Recovery::default());
    }
}

/// AT-NET-11: a final record that fails its check is a torn write: cut and reported.
#[test]
fn at_net_11_a_torn_final_record_is_cut() {
    let (path, mut bytes) = written_log("torn-final", 4);
    let last = HEADER + 3 * RECORD;
    bytes[last + 8..last + RECORD].fill(0);
    fs::write(&path, &bytes).unwrap();
    let (log, recovery) = DurableLog::open(&path).unwrap();
    assert_eq!(log.log().len(), 3);
    assert_eq!(recovery.torn_bytes, RECORD as u64);
    assert_eq!(
        fs::read(&path).unwrap().len(),
        HEADER + 3 * RECORD,
        "the tail is cut on disk"
    );
}

/// AT-NET-11: a damaged record before the last refuses the file, whichever field is hit.
#[test]
fn at_net_11_a_corrupt_earlier_record_refuses_the_file() {
    let (path, bytes) = written_log("corrupt", 5);
    for record in 0..4usize {
        for offset in [0, 8, 40, 72, 103] {
            let mut damaged = bytes.clone();
            damaged[HEADER + record * RECORD + offset] ^= 0x01;
            fs::write(&path, &damaged).unwrap();
            assert_eq!(
                DurableLog::open(&path).err(),
                Some(StoreError::Corrupt {
                    record: record as u64
                }),
                "record {record}, byte {offset}"
            );
        }
    }
}

/// AT-NET-11: another file, or another version, is refused by its header.
#[test]
fn at_net_11_a_foreign_header_refuses_the_file() {
    let (path, mut bytes) = written_log("header", 2);
    bytes[HEADER - 2] = b'2';
    fs::write(&path, &bytes).unwrap();
    assert_eq!(DurableLog::open(&path).err(), Some(StoreError::BadHeader));
    let objects = scratch("objects-as-log");
    ObjectStore::open(&objects).unwrap();
    assert_eq!(
        DurableLog::open(&objects).err(),
        Some(StoreError::BadHeader)
    );
    fs::write(&path, b"not a log").unwrap();
    assert_eq!(DurableLog::open(&path).err(), Some(StoreError::BadHeader));
    assert_eq!(ObjectStore::open(&path).err(), Some(StoreError::BadHeader));
}

/// AT-NET-11: objects come back by CID after a restart, once each, and only as stored.
#[test]
fn at_net_11_objects_survive_a_restart_by_content_id() {
    let path = scratch("objects");
    let (mut store, _) = ObjectStore::open(&path).unwrap();
    assert!(store.is_empty());
    let blobs: Vec<Vec<u8>> = (0..10u8).map(|i| vec![i; usize::from(i) * 37]).collect();
    let ids: Vec<Cid> = blobs.iter().map(|b| store.put(b).unwrap()).collect();
    assert!(!store.is_empty());
    for (id, blob) in ids.iter().zip(&blobs) {
        assert_eq!(
            store.get(id).unwrap().as_deref(),
            Some(&blob[..]),
            "before the restart"
        );
    }
    assert_eq!(ids, blobs.iter().map(|b| cid(b)).collect::<Vec<_>>());
    assert_eq!(store.put(&blobs[3]).unwrap(), ids[3]);
    let size = fs::metadata(&path).unwrap().len();
    drop(store);

    let (store, recovery) = ObjectStore::open(&path).unwrap();
    assert_eq!(recovery, Recovery::default());
    assert_eq!(store.len(), 10);
    assert_eq!(
        fs::metadata(&path).unwrap().len(),
        size,
        "a repeated put writes nothing"
    );
    for (id, blob) in ids.iter().zip(&blobs) {
        assert_eq!(store.get(id).unwrap().as_deref(), Some(&blob[..]));
    }
    assert_eq!(store.get(&cid(b"never stored")).unwrap(), None);
}

/// AT-NET-11: a torn object is cut; bytes changed under an open store are refused on read.
#[test]
fn at_net_11_a_torn_object_is_cut_and_changed_bytes_are_refused() {
    let path = scratch("torn-object");
    let (mut store, _) = ObjectStore::open(&path).unwrap();
    let a = store.put(b"first object").unwrap();
    let b = store.put(b"second object").unwrap();
    drop(store);
    let bytes = fs::read(&path).unwrap();
    for cut in 1..(8 + b"second object".len()) {
        fs::write(&path, &bytes[..bytes.len() - cut]).unwrap();
        let (store, recovery) = ObjectStore::open(&path).unwrap();
        assert_eq!(store.len(), 1, "cut {cut}");
        assert_eq!(
            recovery.torn_bytes as usize,
            8 + b"second object".len() - cut
        );
        assert_eq!(store.get(&b).unwrap(), None);
        let on_disk = fs::metadata(&path).unwrap().len() as usize;
        assert_eq!(
            on_disk,
            bytes.len() - (8 + b"second object".len()),
            "cut on disk"
        );
    }
    fs::write(&path, &bytes).unwrap();
    let (store, _) = ObjectStore::open(&path).unwrap();
    let mut changed = bytes.clone();
    let at = changed.len() - 1;
    changed[at] ^= 1;
    fs::write(&path, &changed).unwrap();
    assert_eq!(
        store.get(&a).unwrap().as_deref(),
        Some(&b"first object"[..])
    );
    assert_eq!(store.get(&b), Err(StoreError::Corrupt { record: 1 }));
}

/// AT-NET-11: no length above the bound is sized from the file or accepted by `put`.
#[test]
fn at_net_11_an_oversized_object_is_refused() {
    let path = scratch("oversized");
    let (mut store, _) = ObjectStore::open(&path).unwrap();
    store.put(b"small").unwrap();
    assert_eq!(
        store.put(&vec![0; MAX_OBJECT + 1]),
        Err(StoreError::TooLarge { record: 1 })
    );
    let largest = store.put(&vec![7; MAX_OBJECT]).unwrap();
    assert_eq!(
        store.get(&largest).unwrap().map(|b| b.len()),
        Some(MAX_OBJECT)
    );
    drop(store);
    let (store, _) = ObjectStore::open(&path).unwrap();
    assert_eq!(
        store.len(),
        2,
        "an object of exactly the bound is stored and reopened"
    );
    drop(store);
    let mut bytes = fs::read(&path).unwrap();
    bytes.truncate(bytes.len() - 8 - MAX_OBJECT);
    bytes.extend_from_slice(&u64::MAX.to_le_bytes());
    bytes.extend_from_slice(&[0; 32]);
    fs::write(&path, &bytes).unwrap();
    assert_eq!(
        ObjectStore::open(&path).err(),
        Some(StoreError::TooLarge { record: 1 })
    );
    let (store, _) = {
        bytes.truncate(bytes.len() - 40);
        bytes.extend_from_slice(&(MAX_OBJECT as u64).to_le_bytes());
        fs::write(&path, &bytes).unwrap();
        ObjectStore::open(&path).unwrap()
    };
    assert_eq!(
        store.len(),
        1,
        "a bounded length past the end is a torn tail"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// AT-NET-11: arbitrary bytes after a genuine prefix never panic; an opened log verifies.
    #[test]
    fn at_net_11_hostile_log_bytes_never_panic(keep in 0usize..600, tail in prop::collection::vec(any::<u8>(), 0..400)) {
        let (path, bytes) = written_log("hostile-log", 5);
        let mut file = bytes[..keep.min(bytes.len())].to_vec();
        file.extend_from_slice(&tail);
        fs::write(&path, &file).unwrap();
        if let Ok((log, recovery)) = DurableLog::open(&path) {
            prop_assert!(log.log().verify());
            if file.len() < HEADER {
                prop_assert_eq!((log.log().len(), recovery.torn_bytes), (0, file.len() as u64));
            } else {
                let kept = HEADER + log.log().len() * RECORD;
                prop_assert_eq!(kept as u64 + recovery.torn_bytes, file.len() as u64);
            }
        }
    }

    /// AT-NET-11: arbitrary object files never panic, and every indexed object reads back.
    #[test]
    fn at_net_11_hostile_object_bytes_never_panic(tail in prop::collection::vec(any::<u8>(), 0..400)) {
        let path = scratch("hostile-objects");
        ObjectStore::open(&path).unwrap();
        let mut file = fs::read(&path).unwrap();
        file.extend_from_slice(&tail);
        fs::write(&path, &file).unwrap();
        if let Ok((store, _)) = ObjectStore::open(&path) {
            let mut seen = 0;
            let mut at = HEADER;
            while at + 8 <= file.len() {
                let len = u64::from_le_bytes(file[at..at + 8].try_into().unwrap()) as usize;
                if at + 8 + len > file.len() { break; }
                let id = cid(&file[at + 8..at + 8 + len]);
                prop_assert_eq!(store.get(&id).unwrap(), Some(file[at + 8..at + 8 + len].to_vec()));
                seen += 1;
                at += 8 + len;
            }
            prop_assert!(store.len() <= seen);
        }
    }
}
