//! Declared offline context and immutable configuration (`docs/15` C6).

use crate::author_time::{AuthorConfig, CutContext};
use crate::events::NodeEvent;
use network::codec::{Reader, Writer};

const MAGIC: &[u8; 12] = b"ISEGORIA-C6\0";

pub enum OfflineRecord {
    Config(AuthorConfig),
    Begin(CutContext),
    Event(NodeEvent),
    End,
}

impl OfflineRecord {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.fixed(MAGIC);
        match self {
            Self::Config(config) => {
                w.u8(0).u64(config.half_life_closings());
            }
            Self::Begin(cut) => {
                w.u8(1)
                    .u64(cut.number)
                    .u64(cut.epoch)
                    .u8(u8::from(cut.closes));
            }
            Self::Event(event) => {
                w.u8(2).field(&event.encode());
            }
            Self::End => {
                w.u8(3);
            }
        }
        w.finish()
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader::new(bytes);
        if &r.fixed::<12>().ok()? != MAGIC {
            return None;
        }
        let record = match r.u8().ok()? {
            0 => Self::Config(AuthorConfig::new(r.u64().ok()?)?),
            1 => {
                let number = r.u64().ok()?;
                let epoch = r.u64().ok()?;
                let closes = match r.u8().ok()? {
                    0 => false,
                    1 => true,
                    _ => return None,
                };
                Self::Begin(CutContext {
                    number,
                    epoch,
                    closes,
                })
            }
            2 => Self::Event(NodeEvent::decode(r.field().ok()?)?),
            3 => Self::End,
            _ => return None,
        };
        r.finish().ok()?;
        Some(record)
    }
}
