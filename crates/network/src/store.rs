//! A node's durable log and object store (`docs/04` §A node's own disk): two append-only
//! files, synced before an append is acknowledged, checked and repaired when opened.

use crate::cid::{cid, Cid};
use crate::log::{entry_hash, Entry, TransparencyLog};
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// What opening a file cut from its tail: the bytes of a write a crash tore.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Recovery {
    pub torn_bytes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreError {
    Io(std::io::ErrorKind),
    /// Not this file or not this version.
    BadHeader,
    /// Record `record` (from 0) fails its check and is not the last: the file is refused.
    Corrupt {
        record: u64,
    },
    /// Record `record` declares, or `put` offers, more than [`MAX_OBJECT`] bytes.
    TooLarge {
        record: u64,
    },
}

pub const MAX_OBJECT: usize = 16 << 20;

const HEADER: u64 = 16;
const LOG_HEADER: &[u8; 16] = b"isegoria-log-v1\n";
const OBJECT_HEADER: &[u8; 16] = b"isegoria-obj-v1\n";
const RECORD: usize = 104;

fn io(e: std::io::Error) -> StoreError {
    StoreError::Io(e.kind())
}

/// Opens or creates `path` behind `header`; returns the file and the bytes of a torn header.
fn open_file(path: &Path, header: &[u8; 16]) -> Result<(File, u64), StoreError> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(io)?;
    let mut head = Vec::new();
    (&mut file)
        .take(HEADER)
        .read_to_end(&mut head)
        .map_err(io)?;
    if head.len() as u64 == HEADER {
        return if head == header {
            Ok((file, 0))
        } else {
            Err(StoreError::BadHeader)
        };
    }
    if !header.starts_with(&head) {
        return Err(StoreError::BadHeader);
    }
    file.set_len(0).map_err(io)?;
    write_at(&mut file, 0, header)?;
    sync_dir(path);
    Ok((file, head.len() as u64))
}

#[cfg(unix)]
fn sync_dir(path: &Path) {
    if let Some(dir) = path.parent().and_then(|d| File::open(d).ok()) {
        let _ = dir.sync_all();
    }
}

#[cfg(not(unix))]
fn sync_dir(_path: &Path) {}

/// Writes `bytes` at `at` and syncs; on failure cuts the file back to `at`.
fn write_at(file: &mut File, at: u64, bytes: &[u8]) -> Result<(), StoreError> {
    let written = file
        .seek(SeekFrom::Start(at))
        .and_then(|_| file.write_all(bytes))
        .and_then(|_| file.sync_data());
    written.map_err(|e| {
        let _ = file.set_len(at);
        io(e)
    })
}

/// Cuts a torn tail at `keep` bytes, on disk (a no-op when nothing is torn).
fn cut(file: &File, keep: u64) -> Result<(), StoreError> {
    file.set_len(keep).and_then(|_| file.sync_all()).map_err(io)
}

pub struct DurableLog {
    file: File,
    log: TransparencyLog,
}

impl DurableLog {
    /// Opens or creates the log file, rebuilding the log from its records (`docs/04` rule 2).
    pub fn open(path: &Path) -> Result<(Self, Recovery), StoreError> {
        let (mut file, torn_header) = open_file(path, LOG_HEADER)?;
        let mut body = Vec::new();
        file.read_to_end(&mut body).map_err(io)?;
        let records = body.len() / RECORD;
        let mut log = TransparencyLog::new();
        for (i, r) in body.chunks_exact(RECORD).enumerate() {
            let seq = u64::from_le_bytes(r[..8].try_into().expect("8 bytes"));
            let prev: [u8; 32] = r[8..40].try_into().expect("32 bytes");
            let payload = Cid(r[40..72].try_into().expect("32 bytes"));
            let chains = seq == i as u64
                && prev == log.head()
                && r[72..] == entry_hash(seq, &prev, &payload);
            if !chains {
                if i + 1 < records {
                    return Err(StoreError::Corrupt { record: i as u64 });
                }
                break;
            }
            log.append(payload);
        }
        let kept = (log.len() * RECORD) as u64;
        let torn = body.len() as u64 - kept;
        cut(&file, HEADER + kept)?;
        let recovery = Recovery {
            torn_bytes: torn_header + torn,
        };
        Ok((DurableLog { file, log }, recovery))
    }

    /// Writes and syncs the entry, then appends it in memory.
    pub fn append(&mut self, payload: Cid) -> Result<Entry, StoreError> {
        let seq = self.log.len() as u64;
        let prev = self.log.head();
        let mut record = [0u8; RECORD];
        record[..8].copy_from_slice(&seq.to_le_bytes());
        record[8..40].copy_from_slice(&prev);
        record[40..72].copy_from_slice(&payload.0);
        record[72..].copy_from_slice(&entry_hash(seq, &prev, &payload));
        write_at(&mut self.file, HEADER + seq * RECORD as u64, &record)?;
        Ok(self.log.append(payload).clone())
    }

    pub fn log(&self) -> &TransparencyLog {
        &self.log
    }
}

pub struct ObjectStore {
    path: PathBuf,
    file: File,
    end: u64,
    records: u64,
    /// CID → (offset of the bytes, length, record).
    index: BTreeMap<Cid, (u64, u64, u64)>,
}

impl ObjectStore {
    /// Opens or creates the object file, indexing every record by its CID (`docs/04` rule 3).
    pub fn open(path: &Path) -> Result<(Self, Recovery), StoreError> {
        let (file, torn_header) = open_file(path, OBJECT_HEADER)?;
        let total = file.metadata().map_err(io)?.len();
        let mut reader = BufReader::new(&file);
        reader.seek(SeekFrom::Start(HEADER)).map_err(io)?;
        let (mut at, mut records) = (HEADER, 0u64);
        let mut index = BTreeMap::new();
        while total - at >= 8 {
            let mut len = [0u8; 8];
            reader.read_exact(&mut len).map_err(io)?;
            let len = u64::from_le_bytes(len);
            if len > MAX_OBJECT as u64 {
                return Err(StoreError::TooLarge { record: records });
            }
            if len > total - at - 8 {
                break;
            }
            let mut bytes = vec![0u8; len as usize];
            reader.read_exact(&mut bytes).map_err(io)?;
            index.entry(cid(&bytes)).or_insert((at + 8, len, records));
            at += 8 + len;
            records += 1;
        }
        drop(reader);
        let torn = total - at;
        cut(&file, at)?;
        let store = ObjectStore {
            path: path.to_path_buf(),
            file,
            end: at,
            records,
            index,
        };
        let recovery = Recovery {
            torn_bytes: torn_header + torn,
        };
        Ok((store, recovery))
    }

    /// Stores `bytes` once, synced, and returns their CID.
    pub fn put(&mut self, bytes: &[u8]) -> Result<Cid, StoreError> {
        if bytes.len() > MAX_OBJECT {
            return Err(StoreError::TooLarge {
                record: self.records,
            });
        }
        let id = cid(bytes);
        if self.index.contains_key(&id) {
            return Ok(id);
        }
        let len = bytes.len() as u64;
        let mut record = Vec::with_capacity(8 + bytes.len());
        record.extend_from_slice(&len.to_le_bytes());
        record.extend_from_slice(bytes);
        write_at(&mut self.file, self.end, &record)?;
        self.index.insert(id, (self.end + 8, len, self.records));
        self.end += 8 + len;
        self.records += 1;
        Ok(id)
    }

    /// The object stored under `cid`, re-hashed on the way out; `Corrupt` if the file no
    /// longer holds those bytes.
    pub fn get(&self, cid_: &Cid) -> Result<Option<Vec<u8>>, StoreError> {
        let Some(&(offset, len, record)) = self.index.get(cid_) else {
            return Ok(None);
        };
        let mut file = File::open(&self.path).map_err(io)?;
        file.seek(SeekFrom::Start(offset)).map_err(io)?;
        let mut bytes = vec![0u8; len as usize];
        file.read_exact(&mut bytes)
            .map_err(|_| StoreError::Corrupt { record })?;
        if cid(&bytes) != *cid_ {
            return Err(StoreError::Corrupt { record });
        }
        Ok(Some(bytes))
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }
}
