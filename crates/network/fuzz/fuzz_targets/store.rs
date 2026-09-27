//! The durable log and object store (T13): opening arbitrary file bytes never panics; an
//! opened log verifies, and every indexed object reads back under its own CID.
#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use network::cid::cid;
use network::store::{DurableLog, ObjectStore};
use std::path::PathBuf;

#[derive(Debug, Arbitrary)]
struct Input {
    log_header: bool,
    object_header: bool,
    log: Vec<u8>,
    objects: Vec<u8>,
    append: Vec<u8>,
}

fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("isegoria-fuzz-store-{}-{name}", std::process::id()))
}

fuzz_target!(|input: Input| {
    let log_path = scratch("log");
    let mut bytes = if input.log_header { b"isegoria-log-v1\n".to_vec() } else { Vec::new() };
    bytes.extend_from_slice(&input.log);
    std::fs::write(&log_path, &bytes).unwrap();
    if let Ok((mut log, recovery)) = DurableLog::open(&log_path) {
        assert!(log.log().verify());
        assert!(recovery.torn_bytes <= bytes.len() as u64);
        let before = log.log().len();
        log.append(cid(&input.append)).unwrap();
        drop(log);
        let (again, _) = DurableLog::open(&log_path).unwrap();
        assert_eq!(again.log().len(), before + 1);
    }

    let object_path = scratch("objects");
    let mut bytes = if input.object_header { b"isegoria-obj-v1\n".to_vec() } else { Vec::new() };
    bytes.extend_from_slice(&input.objects);
    std::fs::write(&object_path, &bytes).unwrap();
    if let Ok((mut store, _)) = ObjectStore::open(&object_path) {
        let id = store.put(&input.append).unwrap();
        assert_eq!(store.get(&id).unwrap().as_deref(), Some(&input.append[..]));
        drop(store);
        let (again, _) = ObjectStore::open(&object_path).unwrap();
        assert_eq!(again.get(&id).unwrap().as_deref(), Some(&input.append[..]));
    }
});
