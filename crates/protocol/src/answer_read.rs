//! Private reads (`docs/05` T79-P7, §Built for participants): a person's read request proved under
//! its role, and the reply a member computes from its ledger, one strict encoding each.

use crate::admission::admit;
use crate::answer::MAX_ALTERNATIVES;
use crate::answer_author::Observation;
use crate::answer_events::AnswerEvent;
use crate::answer_ledger::{read_refusal, write_refusal, AnswerLedger, Refusal};
use crate::answer_path::{network_context, Round};
use crate::answer_rule::Verdict;
use crate::answer_seal::seal;
use crate::author_time::AuthorPoint;
use identity::credential::{AnonymousCredential, IssuerPublic};
use identity::nullifier::{prove, NullifierProof};
use identity::nym::{Nym, Role};
use network::cid::Cid;
use network::codec::{Reader, Writer};
use network::replica::{EntryId, Replica};
use network::tier::Tier;

const VERSION: u8 = 1;
const CONTEXT: &[u8] = b"isegoria/answer-read/v1";

/// What a read asks; the request's proof is under its nym's role.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Read {
    /// Where the event whose bytes have content id `event` was logged, and what the cuts did.
    Status { event: Cid },
    /// The first question numbered from `from` whose group holds the `Judge` nym.
    Reviewer { from: u64 },
    /// The `Propose` nym's questions and records.
    Author,
}

impl Read {
    fn kind(self) -> u8 {
        match self {
            Read::Status { .. } => 1,
            Read::Reviewer { .. } => 2,
            Read::Author => 3,
        }
    }
}

/// A read request: `response_key`, the X25519 key the reply is sealed to, fresh for each request.
pub struct ReadRequest {
    pub read: Read,
    pub response_key: [u8; 32],
    pub proof: NullifierProof,
}

/// The context a read's proof binds on network `network_id`: the read's kind and response key.
pub fn read_context(network_id: &[u8; 32], read: Read, response_key: &[u8; 32]) -> Vec<u8> {
    let offline = [CONTEXT, &[read.kind()], response_key].concat();
    network_context(network_id, &offline)
}

impl ReadRequest {
    /// `read` proved with `cred` under `role` for network `network_id`.
    pub fn new(
        read: Read,
        response_key: [u8; 32],
        cred: &AnonymousCredential,
        role: Role,
        issuer: &IssuerPublic,
        network_id: &[u8; 32],
    ) -> Self {
        let context = read_context(network_id, read, &response_key);
        ReadRequest {
            read,
            response_key,
            proof: prove(cred, issuer, role, &context),
        }
    }

    /// Version 1, the kind, the response key, the kind's body, the proof as a field.
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(VERSION).u8(self.read.kind()).fixed(&self.response_key);
        match self.read {
            Read::Status { event } => w.fixed(&event.0),
            Read::Reviewer { from } => w.u64(from),
            Read::Author => &mut w,
        };
        w.field(&self.proof.encode()).finish()
    }

    /// `None` unless `bytes` are exactly one request.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader::new(bytes);
        if r.u8().ok()? != VERSION {
            return None;
        }
        let kind = r.u8().ok()?;
        let response_key = r.fixed().ok()?;
        let read = match kind {
            1 => Read::Status {
                event: Cid(r.fixed().ok()?),
            },
            2 => Read::Reviewer {
                from: r.u64().ok()?,
            },
            3 => Read::Author,
            _ => return None,
        };
        let proof = NullifierProof::decode(r.field().ok()?)?;
        r.finish().ok()?;
        Some(ReadRequest {
            read,
            response_key,
            proof,
        })
    }
}

/// What the cuts did with one circle entry holding the event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventStatus {
    /// No cut applied so far counts the entry.
    Pending,
    /// Applied; for a deposit, its question's number.
    Applied {
        question: Option<u64>,
    },
    Refused(Refusal),
}

/// A circle entry `(writer, seq)` holding the event, and its status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Logged {
    pub writer: [u8; 32],
    pub seq: u64,
    pub status: EventStatus,
}

/// A reviewer's question by number and view, whether its commit and its reveal counted, and its
/// phase.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assigned {
    pub question: u64,
    pub context: Vec<u8>,
    pub stem: Vec<u8>,
    pub alternatives: Vec<Vec<u8>>,
    pub committed: bool,
    pub revealed: bool,
    pub phase: ReviewPhase,
}

/// A drawn question's round as its reviewers read it (`docs/01` D66): no verdict, count or key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewPhase {
    CommitsOpen,
    RevealsOpen,
    Closed,
}

const REVIEW_PHASES: [ReviewPhase; 3] = [
    ReviewPhase::CommitsOpen,
    ReviewPhase::RevealsOpen,
    ReviewPhase::Closed,
];

/// A question's round as its author reads it: no group, commit or answer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Pending,
    Unavailable,
    Committing,
    Revealing,
    Closed {
        verdict: Verdict,
        observation: Option<Observation>,
    },
}

/// An author's records: `C_a` and the quota in the open epoch, and each question by number.
#[derive(Clone, Debug, PartialEq)]
pub struct Authored {
    pub reputation: f64,
    pub quota: u32,
    pub questions: Vec<(u64, Phase)>,
}

/// A read's reply; a status names no entry for an event that is not the requester's.
#[derive(Clone, Debug, PartialEq)]
pub enum Reply {
    Status(Vec<Logged>),
    Reviewer(Option<Assigned>),
    Author(Authored),
}

const VERDICTS: [Verdict; 4] = [
    Verdict::InsufficientParticipation,
    Verdict::Accepted,
    Verdict::Discarded,
    Verdict::Divided,
];

fn read_bool(r: &mut Reader) -> Option<bool> {
    match r.u8().ok()? {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    }
}

impl Reply {
    /// Version 1, the kind of the read it answers, then its body.
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(VERSION);
        match self {
            Reply::Status(logged) => {
                w.u8(1).u64(logged.len() as u64);
                for l in logged {
                    w.fixed(&l.writer).u64(l.seq);
                    match l.status {
                        EventStatus::Pending => w.u8(0),
                        EventStatus::Applied { question: None } => w.u8(1).u8(0),
                        EventStatus::Applied { question: Some(q) } => w.u8(1).u8(1).u64(q),
                        EventStatus::Refused(refusal) => {
                            write_refusal(w.u8(2), &refusal);
                            &mut w
                        }
                    };
                }
            }
            Reply::Reviewer(None) => {
                w.u8(2).u8(0);
            }
            Reply::Reviewer(Some(a)) => {
                w.u8(2).u8(1).u64(a.question);
                w.field(&a.context).field(&a.stem);
                w.u64(a.alternatives.len() as u64);
                for alternative in &a.alternatives {
                    w.field(alternative);
                }
                w.u8(a.committed.into()).u8(a.revealed.into());
                w.u8(a.phase as u8);
            }
            Reply::Author(a) => {
                w.u8(3).u64(a.reputation.to_bits()).u64(u64::from(a.quota));
                w.u64(a.questions.len() as u64);
                for (q, phase) in &a.questions {
                    w.u64(*q);
                    write_phase(&mut w, phase);
                }
            }
        }
        w.finish()
    }

    /// `None` unless `bytes` are exactly one reply.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader::new(bytes);
        if r.u8().ok()? != VERSION {
            return None;
        }
        let reply = match r.u8().ok()? {
            1 => {
                let mut logged = Vec::new();
                for _ in 0..r.u64().ok()? {
                    let (writer, seq) = (r.fixed().ok()?, r.u64().ok()?);
                    let status = match r.u8().ok()? {
                        0 => EventStatus::Pending,
                        1 => EventStatus::Applied {
                            question: match read_bool(&mut r)? {
                                false => None,
                                true => Some(r.u64().ok()?),
                            },
                        },
                        2 => EventStatus::Refused(read_refusal(&mut r)?),
                        _ => return None,
                    };
                    logged.push(Logged {
                        writer,
                        seq,
                        status,
                    });
                }
                Reply::Status(logged)
            }
            2 => Reply::Reviewer(match read_bool(&mut r)? {
                false => None,
                true => Some(read_assigned(&mut r)?),
            }),
            3 => {
                let reputation = f64::from_bits(r.u64().ok()?);
                let quota = u32::try_from(r.u64().ok()?).ok()?;
                let mut questions = Vec::new();
                for _ in 0..r.u64().ok()? {
                    questions.push((r.u64().ok()?, read_phase(&mut r)?));
                }
                Reply::Author(Authored {
                    reputation,
                    quota,
                    questions,
                })
            }
            _ => return None,
        };
        r.finish().ok()?;
        Some(reply)
    }
}

fn read_assigned(r: &mut Reader) -> Option<Assigned> {
    let question = r.u64().ok()?;
    let context = r.field().ok()?.to_vec();
    let stem = r.field().ok()?.to_vec();
    let n = r.u64().ok()?;
    if n > MAX_ALTERNATIVES as u64 {
        return None;
    }
    let mut alternatives = Vec::new();
    for _ in 0..n {
        alternatives.push(r.field().ok()?.to_vec());
    }
    Some(Assigned {
        question,
        context,
        stem,
        alternatives,
        committed: read_bool(r)?,
        revealed: read_bool(r)?,
        phase: *REVIEW_PHASES.get(usize::from(r.u8().ok()?))?,
    })
}

fn write_phase(w: &mut Writer, phase: &Phase) {
    match phase {
        Phase::Pending => w.u8(0),
        Phase::Unavailable => w.u8(1),
        Phase::Committing => w.u8(2),
        Phase::Revealing => w.u8(3),
        Phase::Closed {
            verdict,
            observation,
        } => {
            w.u8(4).u8(*verdict as u8);
            match observation {
                None => w.u8(0),
                Some(o) => {
                    w.u8(1).u8(o.success.into()).u64(o.weight.to_bits());
                    w.u64(o.origin.epoch).u64(o.origin.closings)
                }
            }
        }
    };
}

fn read_phase(r: &mut Reader) -> Option<Phase> {
    Some(match r.u8().ok()? {
        0 => Phase::Pending,
        1 => Phase::Unavailable,
        2 => Phase::Committing,
        3 => Phase::Revealing,
        4 => Phase::Closed {
            verdict: *VERDICTS.get(usize::from(r.u8().ok()?))?,
            observation: match read_bool(r)? {
                false => None,
                true => Some(Observation {
                    success: read_bool(r)?,
                    weight: f64::from_bits(r.u64().ok()?),
                    origin: AuthorPoint {
                        epoch: r.u64().ok()?,
                        closings: r.u64().ok()?,
                    },
                }),
            },
        },
        _ => return None,
    })
}

/// The reply `ledger` gives `request` after the cuts it applied on `replica`; `None` unless the
/// proof holds in the read's context under its role: `Judge`, `Propose`, either for a status.
pub fn reply(ledger: &AnswerLedger, replica: &Replica, request: &ReadRequest) -> Option<Reply> {
    let role = match request.read {
        Read::Status { .. } => request.proof.role(),
        Read::Reviewer { .. } => Role::Judge,
        Read::Author => Role::Propose,
    };
    if role == Role::Respond {
        return None;
    }
    let network = ledger.tiers().network_id();
    let context = read_context(&network, request.read, &request.response_key);
    let nym = admit(&request.proof, ledger.issuer(), role, &context).ok()?;
    Some(match request.read {
        Read::Status { event } => Reply::Status(status(ledger, replica, event, (role, nym))),
        Read::Reviewer { from } => Reply::Reviewer(assigned(ledger, &nym, from)),
        Read::Author => Reply::Author(authored(ledger, &nym)),
    })
}

/// The encoded `request`'s reply sealed to its response key; `None` where it does not decode or
/// [`reply`] gives none.
pub fn answer(ledger: &AnswerLedger, replica: &Replica, request: &[u8]) -> Option<Vec<u8>> {
    let request = ReadRequest::decode(request)?;
    let reply = reply(ledger, replica, &request)?;
    seal(&request.response_key, &reply.encode())
}

/// The role and nym an event a person sends names.
fn named(event: &AnswerEvent) -> Option<(Role, Nym)> {
    match event {
        AnswerEvent::JoinPool { proof } | AnswerEvent::Commit { proof, .. } => {
            Some((Role::Judge, proof.id()))
        }
        AnswerEvent::Deposit { proof, .. } => Some((Role::Propose, proof.id())),
        AnswerEvent::Reveal { reviewer, .. } => Some((Role::Judge, *reviewer)),
        _ => None,
    }
}

fn status(ledger: &AnswerLedger, replica: &Replica, event: Cid, by: (Role, Nym)) -> Vec<Logged> {
    let tiers = ledger.tiers();
    let held: Vec<EntryId> = (replica.ids())
        .filter(|id| tiers.tier(&id.writer) == Some(Tier::Circle))
        .filter(|id| {
            replica
                .get(id)
                .is_some_and(|(e, _)| e.entry.payload == event)
        })
        .collect();
    let Some((_, object)) = held.first().and_then(|id| replica.get(id)) else {
        return Vec::new();
    };
    let Some(decoded) = AnswerEvent::decode(object).filter(|e| named(e) == Some(by)) else {
        return Vec::new();
    };
    (held.iter())
        .map(|id| Logged {
            writer: id.writer,
            seq: id.seq,
            status: entry_status(ledger, replica, id, &decoded),
        })
        .collect()
}

fn entry_status(
    ledger: &AnswerLedger,
    replica: &Replica,
    id: &EntryId,
    event: &AnswerEvent,
) -> EventStatus {
    if !counted(ledger, replica, id) {
        return EventStatus::Pending;
    }
    if let Some((_, refusal)) = ledger.refused().iter().find(|(i, _)| i == id) {
        return EventStatus::Refused(*refusal);
    }
    let question = match event {
        AnswerEvent::Deposit {
            question, proof, ..
        } => ledger
            .state()
            .deposited(&proof.id(), &question.content_id()),
        _ => None,
    };
    EventStatus::Applied { question }
}

/// Whether the last cut applied counts entry `id`: on its writer's marked chain.
fn counted(ledger: &AnswerLedger, replica: &Replica, id: &EntryId) -> bool {
    let Some(mark) = (ledger.last().into_iter())
        .flat_map(|cut| &cut.marks)
        .find(|m| m.writer == id.writer && id.seq < m.len)
    else {
        return false;
    };
    let chain = replica.chain(&mark.writer, mark.len, mark.head);
    let at = usize::try_from(id.seq).ok();
    chain.and_then(|c| Some(c.get(at?)?.id())) == Some(*id)
}

fn assigned(ledger: &AnswerLedger, reviewer: &Nym, from: u64) -> Option<Assigned> {
    let (question, review) = ledger.review(reviewer, from)?;
    let path = ledger.state();
    let view = path.view(question)?;
    let phase = match path.entry(question)?.round {
        Round::Committing { .. } => ReviewPhase::CommitsOpen,
        Round::Revealing { .. } => ReviewPhase::RevealsOpen,
        Round::Closed(_) => ReviewPhase::Closed,
        Round::Pending | Round::Unavailable { .. } => return None,
    };
    Some(Assigned {
        question,
        context: view.context.to_vec(),
        stem: view.stem.to_vec(),
        alternatives: view.alternatives.to_vec(),
        committed: review.committed,
        revealed: review.revealed,
        phase,
    })
}

fn authored(ledger: &AnswerLedger, author: &Nym) -> Authored {
    let path = ledger.state();
    let questions = (0..path.questions())
        .filter_map(|q| Some((q, path.entry(q).filter(|e| e.author == *author)?)))
        .map(|(q, e)| (q, phase(&e.round)))
        .collect();
    Authored {
        reputation: path.reputation(author),
        quota: path.quota(author),
        questions,
    }
}

fn phase(round: &Round) -> Phase {
    match round {
        Round::Pending => Phase::Pending,
        Round::Unavailable { .. } => Phase::Unavailable,
        Round::Committing { .. } => Phase::Committing,
        Round::Revealing { .. } => Phase::Revealing,
        Round::Closed(c) => Phase::Closed {
            verdict: c.verdict,
            observation: c.observation,
        },
    }
}
