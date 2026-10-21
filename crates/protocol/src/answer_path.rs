//! The answer review's offline path (`docs/05` §The answer review, `docs/01` D49–D52): each
//! event is checked whole, then applied; the state is a function of the accepted events in order.

use crate::admission::{admit, Unproven};
use crate::answer::{commit_answer, Answer, AnswerCommit, Malformed, Question, ReviewerView};
use crate::answer_author::{contribution, AuthorRecord, Observation, QuotaBands};
use crate::answer_events::AnswerEvent;
use crate::answer_rule::{ConsensusRule, Tally, Verdict};
use crate::author_time::{AuthorConfig, AuthorPoint};
use crate::deposit::deposit_context;
use crate::lottery;
use crate::randomness::{Beacon, ANSWER_DRAW};
use identity::credential::IssuerPublic;
use identity::nullifier::NullifierProof;
use identity::nym::{Nym, Role};
use network::cid::Cid;
use std::collections::{BTreeMap, BTreeSet};

/// The path's parameters, pinned for a log's life: provisional, uncalibrated (`docs/14`, D52).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnswerConfig {
    pub rule: ConsensusRule,
    pub divided_weight: f64,
    pub quotas: QuotaBands,
    pub author: AuthorConfig,
}

impl Default for AnswerConfig {
    fn default() -> Self {
        AnswerConfig {
            rule: ConsensusRule::default(),
            divided_weight: 0.25,
            quotas: QuotaBands::default(),
            author: AuthorConfig::default(),
        }
    }
}

impl AnswerConfig {
    /// The weight of divided answers is in `(0, 1]`; the other parts check themselves.
    pub fn is_valid(&self) -> bool {
        self.divided_weight > 0.0 && self.divided_weight <= 1.0
    }
}

/// The context of a reviewer's proof to join the pool.
pub const POOL_CONTEXT: &[u8] = b"isegoria/answer-pool/v1";

/// The context of a commitment's proof: it binds the question and the commitment.
pub fn commit_context(question: u64, commitment: &AnswerCommit) -> Vec<u8> {
    let mut ctx = b"isegoria/answer-commit-proof/v1".to_vec();
    ctx.extend_from_slice(&question.to_le_bytes());
    ctx.extend_from_slice(&commitment.0);
    ctx
}

/// A question's round.
#[derive(Clone, Debug, PartialEq)]
pub enum Round {
    /// Deposited; its group not drawn yet.
    Pending,
    /// The pool its draw read held fewer than the group's size: no group (D49).
    Unavailable {
        available: u32,
    },
    Committing {
        group: Vec<Nym>,
        commits: BTreeMap<Nym, AnswerCommit>,
    },
    Revealing {
        group: Vec<Nym>,
        commits: BTreeMap<Nym, AnswerCommit>,
        answers: BTreeMap<Nym, Answer>,
    },
    Closed(Closure),
}

/// A closed round: its distribution, its outcome under `rule`, the point it closed at, and the
/// author's observation recorded with it (D50).
#[derive(Clone, Debug, PartialEq)]
pub struct Closure {
    pub tally: Tally,
    pub verdict: Verdict,
    pub rule: ConsensusRule,
    pub at: AuthorPoint,
    pub observation: Option<Observation>,
}

/// A deposited question, its author's proven id and the epoch it was deposited in.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub question: Question,
    pub content: Cid,
    pub author: Nym,
    pub epoch: u64,
    pub round: Round,
}

/// An author's quota for an epoch, fixed at its first deposit there, and the slots used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slots {
    pub quota: u32,
    pub used: u32,
}

/// A closed epoch: its beacon value and how many of the pool had joined by its closure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Seal {
    beacon: [u8; 32],
    pool: usize,
}

/// What an accepted event did.
#[derive(Clone, Debug, PartialEq)]
pub enum Applied {
    Joined(Nym),
    Deposited {
        question: u64,
        author: Nym,
    },
    Drawn {
        group: Vec<Nym>,
    },
    /// The explicit unavailability of D49: the round has no group.
    Unavailable {
        available: u32,
        required: u32,
    },
    Committed(Nym),
    CommitsClosed,
    Revealed,
    Closed(Closure),
    EpochClosed {
        epoch: u64,
    },
}

/// Why an event was refused; a refused event leaves the state as it was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    WrongEpoch,
    Malformed(Malformed),
    Unproven(Unproven),
    AlreadyInPool,
    Duplicate {
        question: u64,
    },
    OverQuota {
        quota: u32,
    },
    UnknownQuestion,
    /// The question's deposit epoch is not closed, so its beacon is not recorded.
    BeaconPending,
    /// The event is not defined in the question's round.
    WrongRound,
    NotDrawn,
    AlreadyCommitted,
    NoCommit,
    AlreadyRevealed,
    RevealMismatch,
    AnswerOutOfRange,
    Overflow,
}

impl From<Unproven> for Refused {
    fn from(u: Unproven) -> Self {
        Refused::Unproven(u)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnswerPath {
    config: AnswerConfig,
    epoch: u64,
    closings: u64,
    pool: Vec<Nym>,
    in_pool: BTreeSet<Nym>,
    seals: BTreeMap<u64, Seal>,
    contents: BTreeMap<Cid, u64>,
    entries: Vec<Entry>,
    slots: BTreeMap<(u64, Nym), Slots>,
    authors: BTreeMap<Nym, AuthorRecord>,
}

impl AnswerPath {
    /// An empty path at epoch 0; `None` for an invalid configuration.
    pub fn new(config: AnswerConfig) -> Option<Self> {
        config.is_valid().then(|| AnswerPath {
            config,
            epoch: 0,
            closings: 0,
            pool: Vec::new(),
            in_pool: BTreeSet::new(),
            seals: BTreeMap::new(),
            contents: BTreeMap::new(),
            entries: Vec::new(),
            slots: BTreeMap::new(),
            authors: BTreeMap::new(),
        })
    }

    pub fn apply(
        &mut self,
        event: &AnswerEvent,
        issuer: &IssuerPublic,
    ) -> Result<Applied, Refused> {
        match event {
            AnswerEvent::JoinPool { proof } => self.join(proof, issuer),
            AnswerEvent::Deposit {
                epoch,
                question,
                proof,
            } => self.deposit(*epoch, question, proof, issuer),
            AnswerEvent::Draw { question } => self.draw(*question),
            AnswerEvent::Commit {
                question,
                commitment,
                proof,
            } => self.commit(*question, *commitment, proof, issuer),
            AnswerEvent::CloseCommits { question } => self.close_commits(*question),
            AnswerEvent::Reveal {
                question,
                reviewer,
                answer,
                nonce,
            } => self.reveal(*question, *reviewer, *answer, nonce),
            AnswerEvent::Close { question } => self.close(*question),
            AnswerEvent::CloseEpoch { epoch, beacon } => self.close_epoch(*epoch, *beacon),
        }
    }

    fn join(&mut self, proof: &NullifierProof, issuer: &IssuerPublic) -> Result<Applied, Refused> {
        let reviewer = admit(proof, issuer, Role::Judge, POOL_CONTEXT)?;
        if self.in_pool.contains(&reviewer) {
            return Err(Refused::AlreadyInPool);
        }
        self.in_pool.insert(reviewer);
        self.pool.push(reviewer);
        Ok(Applied::Joined(reviewer))
    }

    fn deposit(
        &mut self,
        epoch: u64,
        question: &Question,
        proof: &NullifierProof,
        issuer: &IssuerPublic,
    ) -> Result<Applied, Refused> {
        if epoch != self.epoch {
            return Err(Refused::WrongEpoch);
        }
        question.check().map_err(Refused::Malformed)?;
        let content = question.content_id();
        let author = admit(
            proof,
            issuer,
            Role::Propose,
            &deposit_context(content, epoch),
        )?;
        if let Some(&existing) = self.contents.get(&content) {
            return Err(Refused::Duplicate { question: existing });
        }
        let slots = self.slots.get(&(epoch, author)).copied().unwrap_or(Slots {
            quota: self.fresh_quota(&author),
            used: 0,
        });
        if slots.used >= slots.quota {
            return Err(Refused::OverQuota { quota: slots.quota });
        }
        let number = self.entries.len() as u64;
        self.entries.push(Entry {
            question: question.clone(),
            content,
            author,
            epoch,
            round: Round::Pending,
        });
        self.contents.insert(content, number);
        let used = slots.used + 1;
        self.slots.insert((epoch, author), Slots { used, ..slots });
        Ok(Applied::Deposited {
            question: number,
            author,
        })
    }

    fn draw(&mut self, question: u64) -> Result<Applied, Refused> {
        let entry = self.entry(question).ok_or(Refused::UnknownQuestion)?;
        if entry.round != Round::Pending {
            return Err(Refused::WrongRound);
        }
        let seal = *self.seals.get(&entry.epoch).ok_or(Refused::BeaconPending)?;
        let required = self.config.rule.group();
        if seal.pool < required as usize {
            let available = seal.pool as u32;
            self.entries[question as usize].round = Round::Unavailable { available };
            return Ok(Applied::Unavailable {
                available,
                required,
            });
        }
        let seed = Beacon::from_value(seal.beacon).seed(ANSWER_DRAW, question);
        let group = lottery::admit(
            &self.pool[..seal.pool],
            required as usize,
            seed,
            entry.epoch,
        );
        self.entries[question as usize].round = Round::Committing {
            group: group.clone(),
            commits: BTreeMap::new(),
        };
        Ok(Applied::Drawn { group })
    }

    fn commit(
        &mut self,
        question: u64,
        commitment: AnswerCommit,
        proof: &NullifierProof,
        issuer: &IssuerPublic,
    ) -> Result<Applied, Refused> {
        let entry = self.entry(question).ok_or(Refused::UnknownQuestion)?;
        let Round::Committing { group, commits } = &entry.round else {
            return Err(Refused::WrongRound);
        };
        let reviewer = admit(
            proof,
            issuer,
            Role::Judge,
            &commit_context(question, &commitment),
        )?;
        if !group.contains(&reviewer) {
            return Err(Refused::NotDrawn);
        }
        if commits.contains_key(&reviewer) {
            return Err(Refused::AlreadyCommitted);
        }
        if let Round::Committing { commits, .. } = &mut self.entries[question as usize].round {
            commits.insert(reviewer, commitment);
        }
        Ok(Applied::Committed(reviewer))
    }

    fn close_commits(&mut self, question: u64) -> Result<Applied, Refused> {
        let entry = self.entry(question).ok_or(Refused::UnknownQuestion)?;
        let Round::Committing { group, commits } = &entry.round else {
            return Err(Refused::WrongRound);
        };
        let round = Round::Revealing {
            group: group.clone(),
            commits: commits.clone(),
            answers: BTreeMap::new(),
        };
        self.entries[question as usize].round = round;
        Ok(Applied::CommitsClosed)
    }

    fn reveal(
        &mut self,
        question: u64,
        reviewer: Nym,
        answer: Answer,
        nonce: &[u8; 32],
    ) -> Result<Applied, Refused> {
        let entry = self.entry(question).ok_or(Refused::UnknownQuestion)?;
        let Round::Revealing {
            commits, answers, ..
        } = &entry.round
        else {
            return Err(Refused::WrongRound);
        };
        let commitment = commits.get(&reviewer).ok_or(Refused::NoCommit)?;
        if answers.contains_key(&reviewer) {
            return Err(Refused::AlreadyRevealed);
        }
        if commit_answer(answer, nonce, reviewer, question) != *commitment {
            return Err(Refused::RevealMismatch);
        }
        if let Answer::Alternative(i) = answer {
            if usize::from(i) >= entry.question.alternatives.len() {
                return Err(Refused::AnswerOutOfRange);
            }
        }
        if let Round::Revealing { answers, .. } = &mut self.entries[question as usize].round {
            answers.insert(reviewer, answer);
        }
        Ok(Applied::Revealed)
    }

    fn close(&mut self, question: u64) -> Result<Applied, Refused> {
        let entry = self.entry(question).ok_or(Refused::UnknownQuestion)?;
        let Round::Revealing { group, answers, .. } = &entry.round else {
            return Err(Refused::WrongRound);
        };
        let absences = (group.len() - answers.len()) as u32;
        let alternatives = entry.question.alternatives.len();
        let tally = Tally::of(alternatives, answers.values().copied(), absences);
        let rule = self.config.rule;
        let verdict = tally.verdict(entry.question.key, rule);
        let at = self.point();
        let observation =
            contribution(verdict, self.config.divided_weight).map(|(success, weight)| {
                Observation {
                    success,
                    weight,
                    origin: at,
                }
            });
        let author = entry.author;
        let closure = Closure {
            tally,
            verdict,
            rule,
            at,
            observation,
        };
        if let Some(o) = observation {
            self.authors
                .entry(author)
                .or_default()
                .record(question, o)
                .expect("a round closes once, and only a closure records its observation");
        }
        self.entries[question as usize].round = Round::Closed(closure.clone());
        Ok(Applied::Closed(closure))
    }

    fn close_epoch(&mut self, epoch: u64, beacon: [u8; 32]) -> Result<Applied, Refused> {
        if epoch != self.epoch {
            return Err(Refused::WrongEpoch);
        }
        let next = self.epoch.checked_add(1).ok_or(Refused::Overflow)?;
        let closings = self.closings.checked_add(1).ok_or(Refused::Overflow)?;
        let pool = self.pool.len();
        self.seals.insert(epoch, Seal { beacon, pool });
        self.epoch = next;
        self.closings = closings;
        Ok(Applied::EpochClosed { epoch })
    }

    fn point(&self) -> AuthorPoint {
        AuthorPoint {
            epoch: self.epoch,
            closings: self.closings,
        }
    }

    fn fresh_quota(&self, author: &Nym) -> u32 {
        let record = self.authors.get(author);
        self.config
            .quotas
            .quota_of(record, self.point(), self.config.author)
    }

    pub fn config(&self) -> &AnswerConfig {
        &self.config
    }

    /// The open epoch.
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Epochs closed so far: the age unit of the author's observations.
    pub fn closings(&self) -> u64 {
        self.closings
    }

    /// The reviewers who joined, in the order they joined.
    pub fn pool(&self) -> &[Nym] {
        &self.pool
    }

    /// The number of questions deposited.
    pub fn questions(&self) -> u64 {
        self.entries.len() as u64
    }

    pub fn entry(&self, question: u64) -> Option<&Entry> {
        usize::try_from(question)
            .ok()
            .and_then(|q| self.entries.get(q))
    }

    /// What `question`'s reviewers are shown: no key (D49).
    pub fn view(&self, question: u64) -> Option<ReviewerView<'_>> {
        self.entry(question).map(|e| e.question.view())
    }

    pub fn closure(&self, question: u64) -> Option<&Closure> {
        match &self.entry(question)?.round {
            Round::Closed(closure) => Some(closure),
            _ => None,
        }
    }

    /// The usable set: the accepted questions, by number (D49).
    pub fn usable(&self) -> impl Iterator<Item = (u64, &Question)> {
        self.entries.iter().enumerate().filter_map(|(q, e)| {
            matches!(
                &e.round,
                Round::Closed(Closure {
                    verdict: Verdict::Accepted,
                    ..
                })
            )
            .then_some((q as u64, &e.question))
        })
    }

    pub fn author(&self, author: &Nym) -> Option<&AuthorRecord> {
        self.authors.get(author)
    }

    /// `C_a` in the open epoch.
    pub fn reputation(&self, author: &Nym) -> f64 {
        let fresh = AuthorRecord::default();
        let record = self.authors.get(author).unwrap_or(&fresh);
        record.reputation(self.point(), self.config.author)
    }

    /// The author's quota in the open epoch: the one fixed at its first deposit, or the one it
    /// would be fixed at.
    pub fn quota(&self, author: &Nym) -> u32 {
        self.slots
            .get(&(self.epoch, *author))
            .map_or_else(|| self.fresh_quota(author), |s| s.quota)
    }

    pub fn slots(&self, author: &Nym, epoch: u64) -> Option<Slots> {
        self.slots.get(&(epoch, *author)).copied()
    }
}
