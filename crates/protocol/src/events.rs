//! A node's events (`docs/04` §Events and replay): every change to its state, as an object
//! the log names, in our encoding (`network::codec`).

use crate::deposit::Draft;
use identity::nullifier::NullifierProof;
use network::cid::Cid;
use network::codec::{Reader, Writer};

pub enum NodeEvent {
    Deposit {
        epoch: u64,
        quota: u32,
        draft: Draft,
        proof: NullifierProof,
    },
    AdmitReviewer {
        item: Cid,
        epoch: u64,
        proof: NullifierProof,
    },
    AdmitRespondent {
        batch: Cid,
        epoch: u64,
        proof: NullifierProof,
    },
}

const VERSION: u8 = 1;
const DEPOSIT: u8 = 1;
const ADMIT_REVIEWER: u8 = 2;
const ADMIT_RESPONDENT: u8 = 3;

impl NodeEvent {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(VERSION);
        match self {
            NodeEvent::Deposit {
                epoch,
                quota,
                draft,
                proof,
            } => w
                .u8(DEPOSIT)
                .u64(*epoch)
                .u64(u64::from(*quota))
                .field(&draft.item)
                .field(&draft.primary_source)
                .field(&proof.encode()),
            NodeEvent::AdmitReviewer { item, epoch, proof } => w
                .u8(ADMIT_REVIEWER)
                .fixed(&item.0)
                .u64(*epoch)
                .field(&proof.encode()),
            NodeEvent::AdmitRespondent {
                batch,
                epoch,
                proof,
            } => w
                .u8(ADMIT_RESPONDENT)
                .fixed(&batch.0)
                .u64(*epoch)
                .field(&proof.encode()),
        };
        w.finish()
    }

    /// `None` unless `bytes` are exactly one well-formed event.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader::new(bytes);
        if r.u8().ok()? != VERSION {
            return None;
        }
        let proof = |r: &mut Reader| NullifierProof::decode(r.field().ok()?);
        let event = match r.u8().ok()? {
            DEPOSIT => {
                let epoch = r.u64().ok()?;
                let quota = u32::try_from(r.u64().ok()?).ok()?;
                let draft = Draft {
                    item: r.field().ok()?.to_vec(),
                    primary_source: r.field().ok()?.to_vec(),
                };
                NodeEvent::Deposit {
                    epoch,
                    quota,
                    draft,
                    proof: proof(&mut r)?,
                }
            }
            ADMIT_REVIEWER => NodeEvent::AdmitReviewer {
                item: Cid(r.fixed().ok()?),
                epoch: r.u64().ok()?,
                proof: proof(&mut r)?,
            },
            ADMIT_RESPONDENT => NodeEvent::AdmitRespondent {
                batch: Cid(r.fixed().ok()?),
                epoch: r.u64().ok()?,
                proof: proof(&mut r)?,
            },
            _ => return None,
        };
        r.finish().ok()?;
        Some(event)
    }
}
