//! The study records of S2's synthetic check (`docs/20` §§2–3, K6): one object per record, in
//! an encoding neither `NodeEvent::decode` nor `MemberObject::decode` accepts, and its reader.

use identity::nym::Nym;
use network::cid::{cid, Cid};
use network::codec::{Reader, Writer};

const TAG: u8 = 0x53;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Construction {
    T1,
    T2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    A,
    R,
    I,
}

/// An item's reading in an executed attempt (K4): stage 1 lets it survive or reads `R`,
/// stage 2 reads `A` or `R`; `NotConverged` is the named non-conclusion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reading {
    Survives,
    A,
    R,
    NotConverged,
}

impl Reading {
    pub fn verdict(self) -> Option<Outcome> {
        match self {
            Reading::A => Some(Outcome::A),
            Reading::R => Some(Outcome::R),
            Reading::Survives | Reading::NotConverged => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    Concluded,
    Exhausted,
}

/// `S0`: `Ω` (a cut number), K3's frozen weights and K2's cohorts, each a finite list of items.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Setup {
    pub omega: u64,
    pub weights: Vec<(Nym, u64)>,
    pub cohorts: Vec<(Nym, Vec<Cid>)>,
}

/// A group record: K4's budget and term (a cut), K5's reveal close per item (a log position),
/// and T2's production position (the cut after which it produces).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub group: u8,
    pub items: Vec<Cid>,
    pub construction: Construction,
    pub budget: u64,
    pub term: u64,
    pub reveal_close: Vec<u64>,
    pub production: Option<u64>,
}

/// An executed attempt: one reading per batch item, the evidence's reference, and its declared
/// admitted pseudonyms and optimizer runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attempt {
    pub group: u8,
    pub stage: u8,
    pub attempt: u8,
    pub batch: Vec<Cid>,
    pub readings: Vec<Reading>,
    pub evidence: [u8; 32],
    pub participations: u64,
    pub runs: u64,
}

/// A stage's administration short of its floor, as `pilot::screen` returned it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shortfall {
    pub group: u8,
    pub stage: u8,
    pub batch: Vec<Cid>,
    pub have: u64,
    pub need: u64,
}

/// A batch `admit_dif_batch` refused (`items` as it returned them), with its declared admitted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refused {
    pub group: u8,
    pub stage: u8,
    pub batch: Vec<Cid>,
    pub items: u64,
    pub participations: u64,
}

/// A terminal record (`17` §9.2): the rule by its group record's CID, the outcome, the
/// reason, and the attempt records it rests on by CID.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Terminal {
    pub item: Cid,
    pub rule: Cid,
    pub outcome: Outcome,
    pub reason: Reason,
    pub attempts: Vec<Cid>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Record {
    Setup(Setup),
    Group(Group),
    Start(u8),
    Attempt(Attempt),
    Shortfall(Shortfall),
    Refused(Refused),
    Terminal(Terminal),
}

fn write_cids(w: &mut Writer, cids: &[Cid]) {
    w.u64(cids.len() as u64);
    for c in cids {
        w.fixed(&c.0);
    }
}

fn read_cids(r: &mut Reader) -> Option<Vec<Cid>> {
    let n = r.u64().ok()?;
    (0..n).map(|_| r.fixed().ok().map(Cid)).collect()
}

fn outcome_byte(o: Outcome) -> u8 {
    match o {
        Outcome::A => 0,
        Outcome::R => 1,
        Outcome::I => 2,
    }
}

fn read_outcome(b: u8) -> Option<Outcome> {
    [Outcome::A, Outcome::R, Outcome::I]
        .get(usize::from(b))
        .copied()
}

const READINGS: [Reading; 4] = [
    Reading::Survives,
    Reading::A,
    Reading::R,
    Reading::NotConverged,
];

impl Record {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(TAG);
        match self {
            Record::Setup(s) => {
                w.u8(0).u64(s.omega).u64(s.weights.len() as u64);
                for (nym, weight) in &s.weights {
                    w.fixed(&nym.0).u64(*weight);
                }
                w.u64(s.cohorts.len() as u64);
                for (nym, items) in &s.cohorts {
                    w.fixed(&nym.0);
                    write_cids(&mut w, items);
                }
            }
            Record::Group(g) => {
                w.u8(1).u8(g.group);
                write_cids(&mut w, &g.items);
                w.u8(u8::from(g.construction == Construction::T2))
                    .u64(g.budget)
                    .u64(g.term)
                    .u64(g.reveal_close.len() as u64);
                for p in &g.reveal_close {
                    w.u64(*p);
                }
                w.u8(u8::from(g.production.is_some()))
                    .u64(g.production.unwrap_or(0));
            }
            Record::Start(group) => {
                w.u8(2).u8(*group);
            }
            Record::Attempt(a) => {
                w.u8(3).u8(a.group).u8(a.stage).u8(a.attempt);
                write_cids(&mut w, &a.batch);
                w.u64(a.readings.len() as u64);
                for r in &a.readings {
                    w.u8(READINGS.iter().position(|x| x == r).unwrap() as u8);
                }
                w.fixed(&a.evidence).u64(a.participations).u64(a.runs);
            }
            Record::Shortfall(s) => {
                w.u8(4).u8(s.group).u8(s.stage);
                write_cids(&mut w, &s.batch);
                w.u64(s.have).u64(s.need);
            }
            Record::Refused(b) => {
                w.u8(5).u8(b.group).u8(b.stage);
                write_cids(&mut w, &b.batch);
                w.u64(b.items).u64(b.participations);
            }
            Record::Terminal(t) => {
                w.u8(6)
                    .fixed(&t.item.0)
                    .fixed(&t.rule.0)
                    .u8(outcome_byte(t.outcome))
                    .u8(u8::from(t.reason == Reason::Exhausted));
                write_cids(&mut w, &t.attempts);
            }
        }
        w.finish()
    }

    /// `None` unless `bytes` are exactly one well-formed study record.
    pub fn decode(bytes: &[u8]) -> Option<Record> {
        let mut r = Reader::new(bytes);
        if r.u8().ok()? != TAG {
            return None;
        }
        let record = match r.u8().ok()? {
            0 => {
                let omega = r.u64().ok()?;
                let n = r.u64().ok()?;
                let weights = (0..n)
                    .map(|_| Some((Nym(r.fixed().ok()?), r.u64().ok()?)))
                    .collect::<Option<_>>()?;
                let n = r.u64().ok()?;
                let cohorts = (0..n)
                    .map(|_| Some((Nym(r.fixed().ok()?), read_cids(&mut r)?)))
                    .collect::<Option<_>>()?;
                Record::Setup(Setup {
                    omega,
                    weights,
                    cohorts,
                })
            }
            1 => {
                let group = r.u8().ok()?;
                let items = read_cids(&mut r)?;
                let construction = match r.u8().ok()? {
                    0 => Construction::T1,
                    1 => Construction::T2,
                    _ => return None,
                };
                let budget = r.u64().ok()?;
                let term = r.u64().ok()?;
                let n = r.u64().ok()?;
                let reveal_close = (0..n).map(|_| r.u64().ok()).collect::<Option<_>>()?;
                let produces = r.u8().ok()?;
                let at = r.u64().ok()?;
                Record::Group(Group {
                    group,
                    items,
                    construction,
                    budget,
                    term,
                    reveal_close,
                    production: match produces {
                        0 => None,
                        1 => Some(at),
                        _ => return None,
                    },
                })
            }
            2 => Record::Start(r.u8().ok()?),
            3 => {
                let (group, stage, attempt) = (r.u8().ok()?, r.u8().ok()?, r.u8().ok()?);
                let batch = read_cids(&mut r)?;
                let n = r.u64().ok()?;
                let readings = (0..n)
                    .map(|_| READINGS.get(usize::from(r.u8().ok()?)).copied())
                    .collect::<Option<_>>()?;
                Record::Attempt(Attempt {
                    group,
                    stage,
                    attempt,
                    batch,
                    readings,
                    evidence: r.fixed().ok()?,
                    participations: r.u64().ok()?,
                    runs: r.u64().ok()?,
                })
            }
            4 => Record::Shortfall(Shortfall {
                group: r.u8().ok()?,
                stage: r.u8().ok()?,
                batch: read_cids(&mut r)?,
                have: r.u64().ok()?,
                need: r.u64().ok()?,
            }),
            5 => Record::Refused(Refused {
                group: r.u8().ok()?,
                stage: r.u8().ok()?,
                batch: read_cids(&mut r)?,
                items: r.u64().ok()?,
                participations: r.u64().ok()?,
            }),
            6 => Record::Terminal(Terminal {
                item: Cid(r.fixed().ok()?),
                rule: Cid(r.fixed().ok()?),
                outcome: read_outcome(r.u8().ok()?)?,
                reason: match r.u8().ok()? {
                    0 => Reason::Concluded,
                    1 => Reason::Exhausted,
                    _ => return None,
                },
                attempts: read_cids(&mut r)?,
            }),
            _ => return None,
        };
        r.finish().ok()?;
        Some(record)
    }

    pub fn cid(&self) -> Cid {
        cid(&self.encode())
    }
}
