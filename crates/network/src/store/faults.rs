//! Injected failures of chosen writes to a store's files, built only with `fault-injection`
//! for tests (`docs/04` §Increments A and B, AT-NET-36).

use super::{cid, Cid, StoreError, RECORD};
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::sync::Mutex;

/// How a write fails, and what it leaves in the file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Half the bytes are written, then the write fails and the file is cut back.
    Write,
    /// The bytes are written and synced, the sync reports a failure, the cut fails: they stay.
    Sync,
}

/// Which record of an object a fault hits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum On {
    Object,
    /// The log record whose payload is the object's CID.
    Log,
}

static ARMED: Mutex<Vec<(On, Cid, Fault)>> = Mutex::new(Vec::new());

/// Fails once, by `fault`, the next write in any store of `on`'s record for the object `object`.
pub fn arm(on: On, object: Cid, fault: Fault) {
    ARMED.lock().unwrap().push((on, object, fault));
}

/// Carries out the fault armed for writing `bytes` at `at`, if any, and returns its error.
pub(super) fn strike(file: &mut File, at: u64, bytes: &[u8]) -> Option<StoreError> {
    let hits = |on: On, object: &Cid| match on {
        On::Object => bytes.len() >= 8 && cid(&bytes[8..]) == *object,
        On::Log => bytes.len() == RECORD && bytes[40..72] == object.0,
    };
    let fault = {
        let mut armed = ARMED.lock().unwrap();
        let k = armed.iter().position(|(on, object, _)| hits(*on, object))?;
        armed.remove(k).2
    };
    let reached = match fault {
        Fault::Write => &bytes[..bytes.len() / 2],
        Fault::Sync => bytes,
    };
    let _ = file
        .seek(SeekFrom::Start(at))
        .and_then(|_| file.write_all(reached))
        .and_then(|_| file.sync_data());
    if fault == Fault::Write {
        let _ = file.set_len(at);
    }
    Some(StoreError::Io(std::io::ErrorKind::Other))
}
