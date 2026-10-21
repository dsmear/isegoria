//! The answer review's durable offline node (`docs/04` §A node's own disk, `docs/05` §The answer
//! review): its configuration pinned in the log's first record, then every accepted event.

use crate::answer_author::QuotaBands;
use crate::answer_events::AnswerEvent;
use crate::answer_path::{AnswerConfig, AnswerPath, Applied, Refused};
use crate::answer_rule::ConsensusRule;
use crate::author_time::AuthorConfig;
use identity::credential::IssuerPublic;
use network::codec::{Reader, Writer};
use network::store::{DurableLog, ObjectStore, StoreError, MAX_OBJECT};
use std::fs;
use std::path::Path;

const MAGIC: &[u8; 12] = b"ISEGORIA-ANS";

enum Record {
    Config(AnswerConfig),
    Event(AnswerEvent),
}

fn write_config(w: &mut Writer, config: &AnswerConfig) {
    let rule = config.rule;
    let (num, den) = rule.threshold();
    let (low, high, quotas, new_author) = config.quotas.parts();
    w.u64(u64::from(rule.group()))
        .u64(u64::from(rule.quorum()))
        .u64(u64::from(num))
        .u64(u64::from(den))
        .u64(config.divided_weight.to_bits())
        .u64(low.to_bits())
        .u64(high.to_bits());
    for q in quotas {
        w.u64(u64::from(q));
    }
    w.u64(u64::from(new_author))
        .u64(config.author.half_life_closings());
}

fn read_u32(r: &mut Reader) -> Option<u32> {
    u32::try_from(r.u64().ok()?).ok()
}

fn read_f64(r: &mut Reader) -> Option<f64> {
    Some(f64::from_bits(r.u64().ok()?))
}

fn read_config(r: &mut Reader) -> Option<AnswerConfig> {
    let (group, quorum) = (read_u32(r)?, read_u32(r)?);
    let rule = ConsensusRule::new(group, quorum, read_u32(r)?, read_u32(r)?)?;
    let divided_weight = read_f64(r)?;
    let (low, high) = (read_f64(r)?, read_f64(r)?);
    let quotas = [read_u32(r)?, read_u32(r)?, read_u32(r)?];
    let quotas = QuotaBands::new(low, high, quotas, read_u32(r)?)?;
    let author = AuthorConfig::new(r.u64().ok()?)?;
    Some(AnswerConfig {
        rule,
        divided_weight,
        quotas,
        author,
    })
}

fn config_record(config: &AnswerConfig) -> Vec<u8> {
    let mut w = Writer::new();
    w.fixed(MAGIC).u8(0);
    write_config(&mut w, config);
    w.finish()
}

fn event_record(event: &AnswerEvent) -> Vec<u8> {
    Writer::new()
        .fixed(MAGIC)
        .u8(1)
        .field(&event.encode())
        .finish()
}

impl Record {
    fn decode(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader::new(bytes);
        if &r.fixed::<12>().ok()? != MAGIC {
            return None;
        }
        let record = match r.u8().ok()? {
            0 => Record::Config(read_config(&mut r)?),
            1 => Record::Event(AnswerEvent::decode(r.field().ok()?)?),
            _ => return None,
        };
        r.finish().ok()?;
        Some(record)
    }
}

#[derive(Debug, PartialEq)]
pub enum AnswerNodeError {
    Store(StoreError),
    Refused(Refused),
    /// The event's record exceeds what the object store holds; nothing applied.
    TooLarge,
    /// The configuration is invalid, or is not the one the log pins.
    Configuration,
    /// Log entry `entry` names no stored object.
    MissingObject {
        entry: u64,
    },
    /// Log entry `entry` names an object that is not a record of this path.
    Undecodable {
        entry: u64,
    },
    /// Log entry `entry` names an event the path refuses on replay.
    ReplayRefused {
        entry: u64,
        refused: Refused,
    },
    /// A write failed after the state changed: reopen the node.
    Poisoned,
}

impl From<StoreError> for AnswerNodeError {
    fn from(e: StoreError) -> Self {
        AnswerNodeError::Store(e)
    }
}

pub struct AnswerNode {
    log: DurableLog,
    objects: ObjectStore,
    issuer: IssuerPublic,
    state: AnswerPath,
    poisoned: bool,
    events: usize,
}

impl AnswerNode {
    /// Opens or creates the node in `dir` (files `log` and `objects`) under `config`, and replays
    /// its log; a log pinning another configuration is refused.
    pub fn open(
        dir: &Path,
        issuer: IssuerPublic,
        config: AnswerConfig,
    ) -> Result<Self, AnswerNodeError> {
        let mut state = AnswerPath::new(config).ok_or(AnswerNodeError::Configuration)?;
        fs::create_dir_all(dir).map_err(|e| StoreError::Io(e.kind()))?;
        let (mut objects, _) = ObjectStore::open(&dir.join("objects"))?;
        let (mut log, _) = DurableLog::open(&dir.join("log"))?;
        if log.log().is_empty() {
            log.append(objects.put(&config_record(&config))?)?;
        }
        let mut events = 0;
        for (i, entry) in log.log().entries().iter().enumerate() {
            let entry_no = i as u64;
            let bytes = objects
                .get(&entry.payload)?
                .ok_or(AnswerNodeError::MissingObject { entry: entry_no })?;
            let record =
                Record::decode(&bytes).ok_or(AnswerNodeError::Undecodable { entry: entry_no })?;
            match (i, record) {
                (0, Record::Config(pinned)) if pinned == config => {}
                (_, Record::Config(_)) | (0, Record::Event(_)) => {
                    return Err(AnswerNodeError::Configuration)
                }
                (_, Record::Event(event)) => {
                    state.apply(&event, &issuer).map_err(|refused| {
                        AnswerNodeError::ReplayRefused {
                            entry: entry_no,
                            refused,
                        }
                    })?;
                    events += 1;
                }
            }
        }
        Ok(AnswerNode {
            log,
            objects,
            issuer,
            state,
            poisoned: false,
            events,
        })
    }

    /// Checks and applies `event`, then writes it; a refused event is not written.
    pub fn submit(&mut self, event: AnswerEvent) -> Result<Applied, AnswerNodeError> {
        if self.poisoned {
            return Err(AnswerNodeError::Poisoned);
        }
        let bytes = event_record(&event);
        if bytes.len() > MAX_OBJECT {
            return Err(AnswerNodeError::TooLarge);
        }
        let applied = self
            .state
            .apply(&event, &self.issuer)
            .map_err(AnswerNodeError::Refused)?;
        let written = self
            .objects
            .put(&bytes)
            .and_then(|object| self.log.append(object));
        if let Err(e) = written {
            self.poisoned = true;
            return Err(AnswerNodeError::Store(e));
        }
        self.events += 1;
        Ok(applied)
    }

    pub fn state(&self) -> &AnswerPath {
        &self.state
    }

    /// Accepted events, the configuration record excluded.
    pub fn events(&self) -> usize {
        self.events
    }
}
