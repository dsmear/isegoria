//! A node's events (`docs/04` §Events and replay): every change to its state, as an object
//! the log names, in our encoding (`network::codec`).

use crate::deposit::Draft;
use crate::gate::GateOutcome;
use crate::lifecycle::Event;
use crate::results::{read_results, write_results, EpochResults};
use crate::review::Commit as Commitment;
use identity::nullifier::NullifierProof;
use identity::nym::Nym;
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
    Step {
        item: Cid,
        event: Event,
    },
    Results(EpochResults),
}

const VERSION: u8 = 1;
const DEPOSIT: u8 = 1;
const ADMIT_REVIEWER: u8 = 2;
const ADMIT_RESPONDENT: u8 = 3;
const STEP: u8 = 4;
const RESULTS: u8 = 5;

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
            NodeEvent::Results(results) => {
                w.u8(RESULTS);
                write_results(&mut w, results);
                &mut w
            }
            NodeEvent::Step { item, event } => {
                w.u8(STEP).fixed(&item.0);
                write_step(&mut w, event);
                &mut w
            }
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
            STEP => NodeEvent::Step {
                item: Cid(r.fixed().ok()?),
                event: read_step(&mut r)?,
            },
            RESULTS => NodeEvent::Results(read_results(&mut r)?),
            _ => return None,
        };
        r.finish().ok()?;
        Some(event)
    }
}

fn outcome_byte(outcome: GateOutcome) -> u8 {
    match outcome {
        GateOutcome::Pass => 0,
        GateOutcome::SupplementaryReview => 1,
        GateOutcome::AppealEligible => 2,
        GateOutcome::Reject => 3,
    }
}

fn nym_field(nyms: &[Nym]) -> Vec<u8> {
    nyms.iter().flat_map(|n| n.0).collect()
}

/// One lifecycle event: its number in the order of [`Event`], then its fields.
fn write_step(w: &mut Writer, event: &Event) {
    match event {
        Event::Admit { seed_from_beacon } => w.u8(1).u8(u8::from(*seed_from_beacon)),
        Event::AssignReviewers { panel, item } => w.u8(2).field(&nym_field(panel)).fixed(&item.0),
        Event::Commit { nym, commitment } => w.u8(3).fixed(&nym.0).fixed(&commitment.0),
        Event::CloseCommits => w.u8(4),
        Event::Reveal { nym, prob, nonce } => {
            w.u8(5).fixed(&nym.0).u64(prob.to_bits()).fixed(nonce)
        }
        Event::Score { outcome } => w.u8(6).u8(outcome_byte(*outcome)),
        Event::AssignExtraReviewers { panel } => w.u8(7).field(&nym_field(panel)),
        Event::Resolve { outcome } => w.u8(8).u8(outcome_byte(*outcome)),
        Event::Appeal {
            within_window,
            reputation_covers_stake,
        } => w
            .u8(9)
            .u8(u8::from(*within_window))
            .u8(u8::from(*reputation_covers_stake)),
        Event::AppealExpires => w.u8(10),
        Event::Pilot1Batch {
            enough_respondents,
            passed,
        } => w
            .u8(11)
            .u8(u8::from(*enough_respondents))
            .u8(u8::from(*passed)),
        Event::Pilot2Batch {
            batch_size,
            passed,
            source_verified,
        } => w
            .u8(12)
            .u64(*batch_size as u64)
            .u8(u8::from(*passed))
            .u8(u8::from(*source_verified)),
        Event::Explore { seed_from_beacon } => w.u8(13).u8(u8::from(*seed_from_beacon)),
        Event::Administer => w.u8(14),
        Event::Revalidate {
            emerging_dif,
            source_verified,
        } => w
            .u8(15)
            .u8(u8::from(*emerging_dif))
            .u8(u8::from(*source_verified)),
        Event::ExposureLimit => w.u8(16),
    };
}

fn read_bool(r: &mut Reader) -> Option<bool> {
    match r.u8().ok()? {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    }
}

fn read_outcome(r: &mut Reader) -> Option<GateOutcome> {
    Some(match r.u8().ok()? {
        0 => GateOutcome::Pass,
        1 => GateOutcome::SupplementaryReview,
        2 => GateOutcome::AppealEligible,
        3 => GateOutcome::Reject,
        _ => return None,
    })
}

fn read_nyms(r: &mut Reader) -> Option<Vec<Nym>> {
    let bytes = r.field().ok()?;
    if bytes.len() % 32 != 0 {
        return None;
    }
    Some(
        bytes
            .chunks_exact(32)
            .map(|c| Nym(c.try_into().expect("32 bytes")))
            .collect(),
    )
}

fn read_step(r: &mut Reader) -> Option<Event> {
    Some(match r.u8().ok()? {
        1 => Event::Admit {
            seed_from_beacon: read_bool(r)?,
        },
        2 => Event::AssignReviewers {
            panel: read_nyms(r)?,
            item: Cid(r.fixed().ok()?),
        },
        3 => Event::Commit {
            nym: Nym(r.fixed().ok()?),
            commitment: Commitment(r.fixed().ok()?),
        },
        4 => Event::CloseCommits,
        5 => Event::Reveal {
            nym: Nym(r.fixed().ok()?),
            prob: f64::from_bits(r.u64().ok()?),
            nonce: r.fixed().ok()?,
        },
        6 => Event::Score {
            outcome: read_outcome(r)?,
        },
        7 => Event::AssignExtraReviewers {
            panel: read_nyms(r)?,
        },
        8 => Event::Resolve {
            outcome: read_outcome(r)?,
        },
        9 => Event::Appeal {
            within_window: read_bool(r)?,
            reputation_covers_stake: read_bool(r)?,
        },
        10 => Event::AppealExpires,
        11 => Event::Pilot1Batch {
            enough_respondents: read_bool(r)?,
            passed: read_bool(r)?,
        },
        12 => Event::Pilot2Batch {
            batch_size: usize::try_from(r.u64().ok()?).ok()?,
            passed: read_bool(r)?,
            source_verified: read_bool(r)?,
        },
        13 => Event::Explore {
            seed_from_beacon: read_bool(r)?,
        },
        14 => Event::Administer,
        15 => Event::Revalidate {
            emerging_dif: read_bool(r)?,
            source_verified: read_bool(r)?,
        },
        16 => Event::ExposureLimit,
        _ => return None,
    })
}
