//! A tiered writer's identity record: each feed's state before its first signature, and a
//! durable stop (`docs/04` §Increments A and B, `docs/01` D58).

use network::cid::cid;
use network::tier::Tier;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

const HEADER: &[u8; 16] = b"isegoria-idr-v1\n";
const BODY: usize = 16 + 32 + 32 + 2 + 2;
/// The record's length: its body, then the body's CID.
pub const LEN: usize = BODY + 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Unused,
    Opening,
    Written,
}

/// Why a member writes nothing; [`Stop::durable`] ones are kept in the record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    Missing,
    Unreadable,
    /// A written feed whose files hold no entry.
    Lost(Tier),
    /// Files holding more entries than the feed's state allows.
    Inconsistent(Tier),
    /// The feed's damaged files were set aside.
    SetAside(Tier),
    /// A verified entry of the feed's key beyond its files or contradicting them.
    OwnEntry(Tier),
    /// A durable write failed: nothing is signed until a restart reads the disk again.
    Failed,
}

impl Stop {
    pub fn durable(&self) -> bool {
        matches!(
            self,
            Stop::Lost(_) | Stop::Inconsistent(_) | Stop::SetAside(_) | Stop::OwnEntry(_)
        )
    }

    /// The status file's word for it, with its feed's.
    pub fn name(&self) -> String {
        let feed = |t: &Tier| match t {
            Tier::Public => "public",
            Tier::Circle => "circle",
        };
        match self {
            Stop::Missing => "missing".into(),
            Stop::Unreadable => "unreadable".into(),
            Stop::Failed => "failed".into(),
            Stop::Lost(t) => format!("lost {}", feed(t)),
            Stop::Inconsistent(t) => format!("inconsistent {}", feed(t)),
            Stop::SetAside(t) => format!("set-aside {}", feed(t)),
            Stop::OwnEntry(t) => format!("own-entry {}", feed(t)),
        }
    }

    /// The stop [`Stop::name`] writes, `None` for other text.
    pub fn parse(text: &str) -> Option<Stop> {
        let tier = |t: &str| match t {
            "public" => Some(Tier::Public),
            "circle" => Some(Tier::Circle),
            _ => None,
        };
        let words: Vec<&str> = text.split(' ').collect();
        Some(match words[..] {
            ["missing"] => Stop::Missing,
            ["unreadable"] => Stop::Unreadable,
            ["failed"] => Stop::Failed,
            ["lost", t] => Stop::Lost(tier(t)?),
            ["inconsistent", t] => Stop::Inconsistent(tier(t)?),
            ["set-aside", t] => Stop::SetAside(tier(t)?),
            ["own-entry", t] => Stop::OwnEntry(tier(t)?),
            _ => return None,
        })
    }
}

fn index(tier: Tier) -> usize {
    match tier {
        Tier::Public => 0,
        Tier::Circle => 1,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub public: [u8; 32],
    pub circle: [u8; 32],
    /// The public feed's state, then the circle feed's.
    pub states: [State; 2],
    /// A durable stop only.
    pub stop: Option<Stop>,
}

impl Record {
    pub fn new(public: [u8; 32], circle: [u8; 32]) -> Record {
        Record {
            public,
            circle,
            states: [State::Unused; 2],
            stop: None,
        }
    }

    pub fn state(&self, tier: Tier) -> State {
        self.states[index(tier)]
    }

    pub fn encode(&self) -> Vec<u8> {
        let state = |s: State| s as u8;
        let (kind, tier) = match self.stop {
            None => (0, Tier::Public),
            Some(Stop::Lost(t)) => (1, t),
            Some(Stop::Inconsistent(t)) => (2, t),
            Some(Stop::SetAside(t)) => (3, t),
            Some(Stop::OwnEntry(t)) => (4, t),
            Some(_) => unreachable!("only durable stops are recorded"),
        };
        let mut out = HEADER.to_vec();
        out.extend_from_slice(&self.public);
        out.extend_from_slice(&self.circle);
        out.extend([state(self.states[0]), state(self.states[1])]);
        out.extend([kind, index(tier) as u8]);
        let sum = cid(&out);
        out.extend_from_slice(&sum.0);
        out
    }

    /// `None` unless `bytes` are exactly one encoding.
    pub fn decode(bytes: &[u8]) -> Option<Record> {
        if bytes.len() != LEN || &bytes[..16] != HEADER || cid(&bytes[..BODY]).0 != bytes[BODY..] {
            return None;
        }
        let state = |b: u8| match b {
            0 => Some(State::Unused),
            1 => Some(State::Opening),
            2 => Some(State::Written),
            _ => None,
        };
        let tier = match bytes[83] {
            0 => Tier::Public,
            1 => Tier::Circle,
            _ => return None,
        };
        let stop = match bytes[82] {
            0 if tier == Tier::Public => None,
            1 => Some(Stop::Lost(tier)),
            2 => Some(Stop::Inconsistent(tier)),
            3 => Some(Stop::SetAside(tier)),
            4 => Some(Stop::OwnEntry(tier)),
            _ => return None,
        };
        Some(Record {
            public: bytes[16..48].try_into().expect("32 bytes"),
            circle: bytes[48..80].try_into().expect("32 bytes"),
            states: [state(bytes[80])?, state(bytes[81])?],
            stop,
        })
    }
}

/// What is at a record's path.
#[derive(Debug, PartialEq, Eq)]
pub enum Found {
    Present(Record),
    Missing,
    Unreadable,
}

/// The record at `path`, reading at most one byte past its length.
pub fn read(path: &Path) -> Found {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Found::Missing,
        Err(_) => return Found::Unreadable,
    };
    let mut bytes = Vec::new();
    match file.take(LEN as u64 + 1).read_to_end(&mut bytes) {
        Ok(_) => Record::decode(&bytes).map_or(Found::Unreadable, Found::Present),
        Err(_) => Found::Unreadable,
    }
}

fn sync_dir(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    if let Some(dir) = path.parent() {
        File::open(dir)?.sync_all()?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

/// Writes a new identity's record at `path`, both feeds unused; never over an existing file.
pub fn create(path: &Path, public: [u8; 32], circle: [u8; 32]) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(&Record::new(public, circle).encode())?;
    file.sync_all()?;
    sync_dir(path)
}

/// Replaces the record at `path`: beside it, synced, renamed over it, the directory synced.
fn store(path: &Path, record: &Record) -> io::Result<()> {
    let next = path.with_extension("next");
    let mut file = File::create(&next)?;
    file.write_all(&record.encode())?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(&next, path)?;
    sync_dir(path)
}

/// A running node's view of its record: what it last stored, and whether it may sign.
pub(crate) struct Guard {
    path: PathBuf,
    record: Option<Record>,
    stop: Option<Stop>,
    /// How the record is replaced: [`store`].
    write: fn(&Path, &Record) -> io::Result<()>,
}

/// A record naming another key for a configured feed.
#[derive(Debug)]
pub(crate) struct OtherIdentity;

impl Guard {
    /// The record at `path` for the configured feeds' keys; no record stops the member.
    pub(crate) fn open(
        path: Option<&Path>,
        public: [u8; 32],
        circle: Option<[u8; 32]>,
    ) -> Result<Guard, OtherIdentity> {
        let found = path.map_or(Found::Missing, read);
        let (record, stop) = match found {
            Found::Missing => (None, Some(Stop::Missing)),
            Found::Unreadable => (None, Some(Stop::Unreadable)),
            Found::Present(r) => {
                if r.public != public || circle.is_some_and(|c| c != r.circle) {
                    return Err(OtherIdentity);
                }
                let stop = r.stop;
                (Some(r), stop)
            }
        };
        Ok(Guard {
            path: path.map(Path::to_path_buf).unwrap_or_default(),
            record,
            stop,
            write: store,
        })
    }

    pub(crate) fn stop(&self) -> Option<Stop> {
        self.stop
    }

    /// `Unused` without a record, which stops the member anyway.
    pub(crate) fn state(&self, tier: Tier) -> State {
        self.record
            .as_ref()
            .map_or(State::Unused, |r| r.state(tier))
    }

    /// Stops the member and returns the first reason; a durable stop is written into a
    /// readable record, and the member stays stopped if that write fails.
    pub(crate) fn halt(&mut self, stop: Stop) -> Stop {
        let first = *self.stop.get_or_insert(stop);
        let Some(record) = self
            .record
            .as_ref()
            .filter(|r| stop.durable() && r.stop.is_none())
        else {
            return first;
        };
        let stopped = Record {
            stop: Some(stop),
            ..record.clone()
        };
        if (self.write)(&self.path, &stopped).is_ok() {
            self.record = Some(stopped);
        }
        first
    }

    /// Moves `tier`'s feed to `to`; a failed write stops the member as `Failed`.
    pub(crate) fn advance(&mut self, tier: Tier, to: State) -> Result<(), Stop> {
        if let Some(stop) = self.stop {
            return Err(stop);
        }
        let record = self.record.as_ref().ok_or(Stop::Missing)?;
        let mut next = record.clone();
        next.states[index(tier)] = to;
        if (self.write)(&self.path, &next).is_err() {
            self.halt(Stop::Failed);
            return Err(Stop::Failed);
        }
        self.record = Some(next);
        Ok(())
    }
}

#[cfg(test)]
#[path = "record_tests.rs"]
mod tests;
