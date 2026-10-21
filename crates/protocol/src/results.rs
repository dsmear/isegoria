//! An epoch's engine outputs as one event (`docs/04` §Events and replay, T73): the records
//! a node applies, and the Merkle root that binds them to the epoch's inputs.

use crate::appeal::{AuthorError, AuthorHistory};
use crate::author_time::{AuthorClock, AuthorConfig, AuthorPoint, ContextError, CutContext};
use crate::contested::{ContestedPool, PoolLimit, RecordError};
use crate::exposure::ExposureLedger;
use crate::probation::SkillTrack;
use identity::nym::Nym;
use network::cid::Cid;
use network::codec::{Reader, Writer};
use network::merkle::{leaf_hash, merkle_proof, merkle_root, verify_proof, MerkleProof};
use scoring::collusion::ResidualHistory;
use scoring::dtf::ClassCurves;
use scoring::latent::Ability;
use scoring::reputation::{AuthorPrior, CusumParams};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum ResultRecord {
    ReviewerScore {
        reviewer: Nym,
        score: f64,
        inclusion: f64,
    },
    ReviewerUnobserved {
        reviewer: Nym,
    },
    /// An unappealed item's pilot outcome; its origin is the applying context (`docs/15` C6).
    AuthorOutcome {
        author: Nym,
        item: Cid,
        passed: bool,
    },
    /// An appeal on `item` filed and started at once (`docs/15` C5–C6): its escrow, checked at this
    /// epoch after the event's earlier filings, and on a node its item's move to the pilot; refused
    /// unless `within_window`, the request's window reading.
    AppealStarted {
        author: Nym,
        item: Cid,
        within_window: bool,
    },
    /// The appealed `item`'s pilot outcome, settling its escrow.
    AppealSettled {
        author: Nym,
        item: Cid,
        passed: bool,
    },
    Exposure {
        item: Cid,
        times: u64,
    },
    Residual {
        reviewer: Nym,
        item: u64,
        residual: f64,
    },
    /// Per class `g`: `pi[g]`, `eta[g]`, `a[g][j]`, `b[g][j]`; per item its floor `c[j]`; the
    /// classes' ability histogram (`ClassCurves::with_ability`).
    ContestedFit {
        pi: Vec<f64>,
        eta: Vec<f64>,
        a: Vec<Vec<f64>>,
        b: Vec<Vec<f64>>,
        c: Vec<f64>,
        ability: Ability,
        members: Vec<(Cid, u64)>,
    },
    ContestedRemove {
        item: Cid,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct EpochResults {
    pub epoch: u64,
    pub inputs_root: [u8; 32],
    pub records: Vec<ResultRecord>,
}

/// Why an epoch's results were refused; `record` is the position of the offending record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResultsRejected {
    EpochRecorded,
    BadInclusion {
        record: usize,
    },
    NotFinite {
        record: usize,
    },
    AppealNotCovered {
        record: usize,
    },
    /// A filing whose request came after its item's appeal window.
    AppealWindowClosed {
        record: usize,
    },
    NoOpenEscrow {
        record: usize,
    },
    /// An author record for an item that already has an outcome or an escrow.
    DuplicateItem {
        record: usize,
    },
    MissingContext {
        record: usize,
    },
    FilingBatchRecorded,
    /// Refused by the node: the item's deposit proved another author, or none.
    NotTheProposer {
        record: usize,
    },
    /// Refused by the node: no pilot conclusion of this outcome and appeal on record.
    OutcomeMismatch {
        record: usize,
    },
    /// Refused by the node: a filing on an item that is not appeal-eligible.
    NotAppealable {
        record: usize,
    },
    BadClasses {
        record: usize,
    },
    BadMembers {
        record: usize,
    },
    /// A contested fit outside the pool's domain (`docs/15` D3), refused before its curves exist.
    ContestedOutOfDomain {
        record: usize,
        limit: PoolLimit,
    },
}

/// A rating's leaf: the judge's proven id, the item, the probability's bits.
pub fn rating_leaf(judge: Nym, item: Cid, prob: f64) -> Vec<u8> {
    Writer::new()
        .u8(1)
        .fixed(&judge.0)
        .fixed(&item.0)
        .u64(prob.to_bits())
        .finish()
}

/// An answer's leaf: the respondent's proven id, the batch, the item's index in it, the answer.
pub fn answer_leaf(respondent: Nym, batch: Cid, index: u64, answer: u8) -> Vec<u8> {
    Writer::new()
        .u8(2)
        .fixed(&respondent.0)
        .fixed(&batch.0)
        .u64(index)
        .u8(answer)
        .finish()
}

fn sorted_hashes(leaves: &[Vec<u8>]) -> Vec<[u8; 32]> {
    let mut hashes: Vec<[u8; 32]> = leaves.iter().map(|l| leaf_hash(l)).collect();
    hashes.sort_unstable();
    hashes.dedup();
    hashes
}

/// The RFC 6962 root over the sorted leaf hashes: a function of the set of inputs.
pub fn inputs_root(leaves: &[Vec<u8>]) -> [u8; 32] {
    merkle_root(&sorted_hashes(leaves))
}

/// The proof that `leaf` is one of `leaves`, against [`inputs_root`]; `None` if it is not.
pub fn inclusion_proof(leaves: &[Vec<u8>], leaf: &[u8]) -> Option<MerkleProof> {
    let hashes = sorted_hashes(leaves);
    let at = hashes.binary_search(&leaf_hash(leaf)).ok()?;
    merkle_proof(&hashes, at)
}

pub fn verify_inclusion(leaf: &[u8], proof: &MerkleProof, root: [u8; 32]) -> bool {
    verify_proof(leaf_hash(leaf), proof, root)
}

/// The engine's outputs a node holds, rebuilt from its results events.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResultsState {
    roots: BTreeMap<u64, [u8; 32]>,
    tracks: BTreeMap<Nym, SkillTrack>,
    authors: BTreeMap<Nym, AuthorHistory>,
    config: AuthorConfig,
    clock: AuthorClock,
    filing_epoch: Option<u64>,
    exposure: ExposureLedger,
    reviewer_index: BTreeMap<Nym, usize>,
    residuals: ResidualHistory,
    contested: ContestedPool,
}

impl ResultsState {
    pub fn with_author_config(config: AuthorConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn begin_cut(&mut self, cut: CutContext) -> Result<(), ContextError> {
        self.clock.begin(cut)
    }

    pub fn end_cut(&mut self) -> Result<(), ContextError> {
        self.clock.end()
    }

    pub fn author_point(&self) -> Option<AuthorPoint> {
        self.clock.point()
    }

    pub fn closing_count(&self) -> u64 {
        self.clock.closings()
    }

    pub fn filing_batch_at(&self, epoch: u64) -> bool {
        self.filing_epoch == Some(epoch)
    }

    pub fn author_config(&self) -> AuthorConfig {
        self.config
    }

    /// Applies every record or none (`docs/04` §Events and replay); a filing records its escrow
    /// only, and `NodeState::apply` moves its item in the same event.
    pub fn apply(&mut self, results: &EpochResults) -> Result<(), ResultsRejected> {
        if self.roots.contains_key(&results.epoch) {
            return Err(ResultsRejected::EpochRecorded);
        }
        let filings = results
            .records
            .iter()
            .any(|r| matches!(r, ResultRecord::AppealStarted { .. }));
        if filings
            && self
                .author_point()
                .is_some_and(|at| self.filing_epoch == Some(at.epoch))
        {
            return Err(ResultsRejected::FilingBatchRecorded);
        }
        let mut next = self.clone();
        for (i, record) in results.records.iter().enumerate() {
            next.record(i, record)?;
        }
        if filings {
            next.filing_epoch = next.author_point().map(|at| at.epoch);
        }
        next.roots.insert(results.epoch, results.inputs_root);
        *self = next;
        Ok(())
    }

    fn record(&mut self, i: usize, record: &ResultRecord) -> Result<(), ResultsRejected> {
        let finite = |v: f64| {
            v.is_finite()
                .then_some(())
                .ok_or(ResultsRejected::NotFinite { record: i })
        };
        let point = || {
            self.clock
                .point()
                .ok_or(ResultsRejected::MissingContext { record: i })
        };
        let at = match record {
            ResultRecord::AuthorOutcome { .. }
            | ResultRecord::AppealStarted { .. }
            | ResultRecord::AppealSettled { .. } => Some(point()?),
            _ => None,
        };
        match record {
            ResultRecord::ReviewerScore {
                reviewer,
                score,
                inclusion,
            } => {
                if !(*inclusion > 0.0 && *inclusion <= 1.0) {
                    return Err(ResultsRejected::BadInclusion { record: i });
                }
                finite(*score)?;
                let track = self.tracks.entry(*reviewer).or_default();
                track.record_observed(*score, *inclusion, &CusumParams::default());
            }
            ResultRecord::ReviewerUnobserved { reviewer } => {
                self.tracks
                    .entry(*reviewer)
                    .or_default()
                    .record_unobserved();
            }
            ResultRecord::AuthorOutcome {
                author,
                item,
                passed,
            } => {
                let history = self.authors.entry(*author).or_default();
                history
                    .conclude(*item, *passed, at.expect("author record"))
                    .map_err(|e| author_refusal(e, i))?;
            }
            ResultRecord::AppealStarted {
                author,
                item,
                within_window,
            } => {
                if !within_window {
                    return Err(ResultsRejected::AppealWindowClosed { record: i });
                }
                let history = self.authors.entry(*author).or_default();
                history
                    .file_appeal(
                        *item,
                        at.expect("author record"),
                        self.config,
                        &AuthorPrior::default(),
                    )
                    .map_err(|e| author_refusal(e, i))?;
            }
            ResultRecord::AppealSettled {
                author,
                item,
                passed,
            } => {
                let history = self
                    .authors
                    .get_mut(author)
                    .ok_or(ResultsRejected::NoOpenEscrow { record: i })?;
                history
                    .settle(*item, *passed, at.expect("author record"))
                    .map_err(|e| author_refusal(e, i))?;
            }
            ResultRecord::Exposure { item, times } => {
                let times = usize::try_from(*times).unwrap_or(usize::MAX);
                self.exposure.record_n(*item, times);
            }
            ResultRecord::Residual {
                reviewer,
                item,
                residual,
            } => {
                finite(*residual)?;
                let next = self.reviewer_index.len();
                let index = *self.reviewer_index.entry(*reviewer).or_insert(next);
                self.residuals.record(index, *item, *residual);
            }
            ResultRecord::ContestedFit {
                pi,
                eta,
                a,
                b,
                c,
                ability,
                members,
            } => {
                let refused = |e| match e {
                    RecordError::OutOfDomain(limit) => {
                        ResultsRejected::ContestedOutOfDomain { record: i, limit }
                    }
                    _ => ResultsRejected::BadMembers { record: i },
                };
                let points = pi
                    .len()
                    .checked_mul(ability.nodes.len())
                    .unwrap_or(usize::MAX);
                self.contested
                    .admits(pi.len(), points, c.len(), members)
                    .map_err(refused)?;
                let curves = ClassCurves::with_ability(pi, eta, ability, a, b, c)
                    .map_err(|_| ResultsRejected::BadClasses { record: i })?;
                let members: Option<Vec<(Cid, usize)>> = members
                    .iter()
                    .map(|(c, j)| usize::try_from(*j).ok().map(|j| (*c, j)))
                    .collect();
                let members = members.ok_or(ResultsRejected::BadMembers { record: i })?;
                self.contested.record(curves, &members).map_err(refused)?;
            }
            ResultRecord::ContestedRemove { item } => {
                self.contested.remove(item);
            }
        }
        Ok(())
    }

    pub fn epoch_root(&self, epoch: u64) -> Option<[u8; 32]> {
        self.roots.get(&epoch).copied()
    }

    pub fn track(&self, reviewer: &Nym) -> Option<&SkillTrack> {
        self.tracks.get(reviewer)
    }

    pub fn author(&self, author: &Nym) -> Option<&AuthorHistory> {
        self.authors.get(author)
    }

    pub fn open_escrows(&self, author: &Nym) -> Vec<Cid> {
        self.authors
            .get(author)
            .map_or_else(Vec::new, AuthorHistory::open_escrows)
    }

    /// The quota snapshot for the active cut's epoch (`docs/15` C6).
    pub fn quota_reputation(&self, author: &Nym) -> Option<f64> {
        let at = self.author_point()?;
        let fresh = AuthorHistory::new();
        let history = self.authors.get(author).unwrap_or(&fresh);
        Some(history.quota_reputation(at, self.config, &AuthorPrior::default()))
    }

    pub fn exposure(&self, item: &Cid) -> usize {
        self.exposure.count(item)
    }

    /// The residual history, reviewers indexed in the order they first appear.
    pub fn residuals(&self) -> (&ResidualHistory, &BTreeMap<Nym, usize>) {
        (&self.residuals, &self.reviewer_index)
    }

    pub fn contested(&self) -> &ContestedPool {
        &self.contested
    }
}

fn author_refusal(e: AuthorError, record: usize) -> ResultsRejected {
    match e {
        AuthorError::Duplicate => ResultsRejected::DuplicateItem { record },
        AuthorError::Uncovered(_) => ResultsRejected::AppealNotCovered { record },
        AuthorError::NoOpenEscrow => ResultsRejected::NoOpenEscrow { record },
    }
}

fn floats(values: &[f64]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|v| v.to_bits().to_le_bytes())
        .collect()
}

fn write_rows(w: &mut Writer, rows: &[Vec<f64>]) {
    w.u64(rows.len() as u64);
    for row in rows {
        w.field(&floats(row));
    }
}

/// The body of an epoch-results event (`docs/04` §Events and replay).
pub(crate) fn write_results(w: &mut Writer, results: &EpochResults) {
    w.u64(results.epoch)
        .fixed(&results.inputs_root)
        .u64(results.records.len() as u64);
    for record in &results.records {
        match record {
            ResultRecord::ReviewerScore {
                reviewer,
                score,
                inclusion,
            } => w
                .u8(1)
                .fixed(&reviewer.0)
                .u64(score.to_bits())
                .u64(inclusion.to_bits()),
            ResultRecord::ReviewerUnobserved { reviewer } => w.u8(2).fixed(&reviewer.0),
            ResultRecord::AuthorOutcome {
                author,
                item,
                passed,
            } => w
                .u8(10)
                .fixed(&author.0)
                .fixed(&item.0)
                .u8(u8::from(*passed)),
            ResultRecord::AppealStarted {
                author,
                item,
                within_window,
            } => w
                .u8(13)
                .fixed(&author.0)
                .fixed(&item.0)
                .u8(u8::from(*within_window)),
            ResultRecord::AppealSettled {
                author,
                item,
                passed,
            } => w
                .u8(12)
                .fixed(&author.0)
                .fixed(&item.0)
                .u8(u8::from(*passed)),
            ResultRecord::Exposure { item, times } => w.u8(6).fixed(&item.0).u64(*times),
            ResultRecord::Residual {
                reviewer,
                item,
                residual,
            } => w
                .u8(7)
                .fixed(&reviewer.0)
                .u64(*item)
                .u64(residual.to_bits()),
            ResultRecord::ContestedFit {
                pi,
                eta,
                a,
                b,
                c,
                ability,
                members,
            } => {
                w.u8(8).field(&floats(pi)).field(&floats(eta));
                write_rows(w, a);
                write_rows(w, b);
                w.field(&floats(c))
                    .field(&floats(&ability.nodes))
                    .field(&floats(&ability.weights));
                let members: Vec<u8> = members
                    .iter()
                    .flat_map(|(c, j)| c.0.into_iter().chain(j.to_le_bytes()))
                    .collect();
                w.field(&members)
            }
            ResultRecord::ContestedRemove { item } => w.u8(9).fixed(&item.0),
        };
    }
}

fn read_f64(r: &mut Reader) -> Option<f64> {
    Some(f64::from_bits(r.u64().ok()?))
}

fn read_floats(r: &mut Reader) -> Option<Vec<f64>> {
    let bytes = r.field().ok()?;
    if bytes.len() % 8 != 0 {
        return None;
    }
    Some(
        bytes
            .chunks_exact(8)
            .map(|c| f64::from_bits(u64::from_le_bytes(c.try_into().expect("8 bytes"))))
            .collect(),
    )
}

fn read_rows(r: &mut Reader) -> Option<Vec<Vec<f64>>> {
    let n = r.u64().ok()?;
    let mut rows = Vec::new();
    for _ in 0..n {
        rows.push(read_floats(r)?);
    }
    Some(rows)
}

pub(crate) fn read_results(r: &mut Reader) -> Option<EpochResults> {
    let epoch = r.u64().ok()?;
    let inputs_root = r.fixed().ok()?;
    let n = r.u64().ok()?;
    let mut records = Vec::new();
    for _ in 0..n {
        records.push(read_record(r)?);
    }
    Some(EpochResults {
        epoch,
        inputs_root,
        records,
    })
}

fn read_record(r: &mut Reader) -> Option<ResultRecord> {
    let nym = |r: &mut Reader| r.fixed().ok().map(Nym);
    let cid = |r: &mut Reader| r.fixed().ok().map(Cid);
    let flag = |r: &mut Reader| match r.u8().ok()? {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    };
    Some(match r.u8().ok()? {
        1 => ResultRecord::ReviewerScore {
            reviewer: nym(r)?,
            score: read_f64(r)?,
            inclusion: read_f64(r)?,
        },
        2 => ResultRecord::ReviewerUnobserved { reviewer: nym(r)? },
        6 => ResultRecord::Exposure {
            item: cid(r)?,
            times: r.u64().ok()?,
        },
        7 => ResultRecord::Residual {
            reviewer: nym(r)?,
            item: r.u64().ok()?,
            residual: read_f64(r)?,
        },
        8 => {
            let (pi, eta) = (read_floats(r)?, read_floats(r)?);
            let (a, b) = (read_rows(r)?, read_rows(r)?);
            let c = read_floats(r)?;
            let ability = Ability {
                nodes: read_floats(r)?,
                weights: read_floats(r)?,
            };
            let bytes = r.field().ok()?;
            if bytes.len() % 40 != 0 {
                return None;
            }
            let members = bytes
                .chunks_exact(40)
                .map(|c| {
                    let item = Cid(c[..32].try_into().expect("32 bytes"));
                    (
                        item,
                        u64::from_le_bytes(c[32..].try_into().expect("8 bytes")),
                    )
                })
                .collect();
            ResultRecord::ContestedFit {
                pi,
                eta,
                a,
                b,
                c,
                ability,
                members,
            }
        }
        9 => ResultRecord::ContestedRemove { item: cid(r)? },
        10 => ResultRecord::AuthorOutcome {
            author: nym(r)?,
            item: cid(r)?,
            passed: flag(r)?,
        },
        12 => ResultRecord::AppealSettled {
            author: nym(r)?,
            item: cid(r)?,
            passed: flag(r)?,
        },
        13 => ResultRecord::AppealStarted {
            author: nym(r)?,
            item: cid(r)?,
            within_window: flag(r)?,
        },
        _ => return None,
    })
}
