//! The answer review's events (`docs/05` §The answer review): every change to its state, in our
//! encoding (`network::codec`), one encoding per event.

use crate::answer::{Answer, AnswerCommit, Question};
use identity::nullifier::NullifierProof;
use identity::nym::Nym;
use network::codec::{Reader, Writer};

pub enum AnswerEvent {
    /// A reviewer joins the pool, proven under `Judge` for `answer_path::POOL_CONTEXT`.
    JoinPool {
        proof: NullifierProof,
    },
    /// An author deposits in the open `epoch`, proven under `Propose` for the question's content.
    Deposit {
        epoch: u64,
        question: Question,
        proof: NullifierProof,
    },
    /// The draw of `question`'s group, from its deposit epoch's beacon and pool.
    Draw {
        question: u64,
    },
    /// A drawn reviewer commits, proven under `Judge` for `answer_path::commit_context`.
    Commit {
        question: u64,
        commitment: AnswerCommit,
        proof: NullifierProof,
    },
    CloseCommits {
        question: u64,
    },
    Reveal {
        question: u64,
        reviewer: Nym,
        answer: Answer,
        nonce: [u8; 32],
    },
    /// The round's explicit closure: its outcome and the author's observation (D49, D50).
    Close {
        question: u64,
    },
    /// The open epoch's explicit closure, recording its beacon value.
    CloseEpoch {
        epoch: u64,
        beacon: [u8; 32],
    },
}

const VERSION: u8 = 1;

impl AnswerEvent {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(VERSION);
        match self {
            AnswerEvent::JoinPool { proof } => {
                w.u8(1).field(&proof.encode());
            }
            AnswerEvent::Deposit {
                epoch,
                question,
                proof,
            } => {
                w.u8(2).u64(*epoch);
                question.write(&mut w);
                w.field(&proof.encode());
            }
            AnswerEvent::Draw { question } => {
                w.u8(3).u64(*question);
            }
            AnswerEvent::Commit {
                question,
                commitment,
                proof,
            } => {
                w.u8(4)
                    .u64(*question)
                    .fixed(&commitment.0)
                    .field(&proof.encode());
            }
            AnswerEvent::CloseCommits { question } => {
                w.u8(5).u64(*question);
            }
            AnswerEvent::Reveal {
                question,
                reviewer,
                answer,
                nonce,
            } => {
                w.u8(6)
                    .u64(*question)
                    .fixed(&reviewer.0)
                    .fixed(&answer.code())
                    .fixed(nonce);
            }
            AnswerEvent::Close { question } => {
                w.u8(7).u64(*question);
            }
            AnswerEvent::CloseEpoch { epoch, beacon } => {
                w.u8(8).u64(*epoch).fixed(beacon);
            }
        }
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
            1 => AnswerEvent::JoinPool {
                proof: proof(&mut r)?,
            },
            2 => AnswerEvent::Deposit {
                epoch: r.u64().ok()?,
                question: Question::read(&mut r)?,
                proof: proof(&mut r)?,
            },
            3 => AnswerEvent::Draw {
                question: r.u64().ok()?,
            },
            4 => AnswerEvent::Commit {
                question: r.u64().ok()?,
                commitment: AnswerCommit(r.fixed().ok()?),
                proof: proof(&mut r)?,
            },
            5 => AnswerEvent::CloseCommits {
                question: r.u64().ok()?,
            },
            6 => AnswerEvent::Reveal {
                question: r.u64().ok()?,
                reviewer: Nym(r.fixed().ok()?),
                answer: Answer::from_code(r.fixed().ok()?)?,
                nonce: r.fixed().ok()?,
            },
            7 => AnswerEvent::Close {
                question: r.u64().ok()?,
            },
            8 => AnswerEvent::CloseEpoch {
                epoch: r.u64().ok()?,
                beacon: r.fixed().ok()?,
            },
            _ => return None,
        };
        r.finish().ok()?;
        Some(event)
    }
}
