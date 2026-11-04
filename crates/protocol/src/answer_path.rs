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
use network::codec::Writer;
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

/// The family tag before the network id in a network's proof contexts (`docs/05` T79-P1).
pub const NETWORK_FAMILY: &[u8] = b"isegoria/answer-review/network/v1";

/// `context`, an offline proof context, as network `network_id` binds it.
pub fn network_context(network_id: &[u8; 32], context: &[u8]) -> Vec<u8> {
    [NETWORK_FAMILY, network_id, context].concat()
}

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

/// A closed epoch: how many of the pool had joined by its closure, and its beacon value once
/// recorded (on a network, a later round's, `docs/05` T79-P3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Seal {
    beacon: Option<[u8; 32]>,
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
    /// This author already deposited this content: the request is not repeated. Only the
    /// author's own deposits answer it, never another author's; it is not a plagiarism check.
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
    /// A member's request to close an epoch: only a network counts it (`docs/05` T79-P5).
    NetworkOnly,
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
    contents: BTreeMap<(Nym, Cid), u64>,
    entries: Vec<Entry>,
    slots: BTreeMap<(u64, Nym), Slots>,
    authors: BTreeMap<Nym, AuthorRecord>,
    network: Option<[u8; 32]>,
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
            network: None,
        })
    }

    /// An empty path whose proofs bind `network_id` (`docs/05` T79-P1).
    pub(crate) fn for_network(config: AnswerConfig, network_id: [u8; 32]) -> Option<Self> {
        let mut path = Self::new(config)?;
        path.network = Some(network_id);
        Some(path)
    }

    fn context(&self, offline: &[u8]) -> Vec<u8> {
        self.network
            .map_or_else(|| offline.to_vec(), |id| network_context(&id, offline))
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
            AnswerEvent::RequestCloseEpoch { .. } => Err(Refused::NetworkOnly),
        }
    }

    fn join(&mut self, proof: &NullifierProof, issuer: &IssuerPublic) -> Result<Applied, Refused> {
        let reviewer = admit(proof, issuer, Role::Judge, &self.context(POOL_CONTEXT))?;
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
        let context = self.context(&deposit_context(content, epoch));
        let author = admit(proof, issuer, Role::Propose, &context)?;
        if let Some(&existing) = self.contents.get(&(author, content)) {
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
        self.contents.insert((author, content), number);
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
        let beacon = seal.beacon.ok_or(Refused::BeaconPending)?;
        let required = self.config.rule.group();
        if seal.pool < required as usize {
            let available = seal.pool as u32;
            self.entries[question as usize].round = Round::Unavailable { available };
            return Ok(Applied::Unavailable {
                available,
                required,
            });
        }
        let seed = Beacon::from_value(beacon).seed(ANSWER_DRAW, question);
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
        let context = self.context(&commit_context(question, &commitment));
        let reviewer = admit(proof, issuer, Role::Judge, &context)?;
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
        self.seal()?;
        self.seals
            .entry(epoch)
            .and_modify(|s| s.beacon = Some(beacon));
        Ok(Applied::EpochClosed { epoch })
    }

    /// Closes the open epoch without its beacon value: its pool, one closing more (T79-P3).
    pub(crate) fn seal(&mut self) -> Result<u64, Refused> {
        let epoch = self.epoch;
        let next = epoch.checked_add(1).ok_or(Refused::Overflow)?;
        let closings = self.closings.checked_add(1).ok_or(Refused::Overflow)?;
        let pool = self.pool.len();
        self.seals.insert(epoch, Seal { beacon: None, pool });
        self.epoch = next;
        self.closings = closings;
        Ok(epoch)
    }

    /// Gives `beacon`, round `round`'s value, to every sealed epoch up to it still lacking one,
    /// then draws their pending questions in question order (T79-P3, P4).
    pub(crate) fn assign(
        &mut self,
        round: u64,
        beacon: [u8; 32],
    ) -> (Vec<u64>, Vec<(u64, Applied)>) {
        let mut valued = Vec::new();
        for (epoch, seal) in self.seals.range_mut(..=round) {
            if seal.beacon.is_none() {
                seal.beacon = Some(beacon);
                valued.push(*epoch);
            }
        }
        let pending: Vec<u64> = (0..self.questions())
            .filter(|&q| {
                let e = &self.entries[q as usize];
                e.round == Round::Pending && valued.contains(&e.epoch)
            })
            .collect();
        let drawn = pending
            .into_iter()
            .map(|q| {
                (
                    q,
                    self.draw(q)
                        .expect("a valued epoch's pending question draws"),
                )
            })
            .collect();
        (valued, drawn)
    }

    /// Every field but the indexes rebuilt from them, in one canonical encoding (D60's digest).
    pub(crate) fn write_state(&self, w: &mut Writer) {
        w.field(&self.config.encode())
            .u64(self.epoch)
            .u64(self.closings)
            .u64(self.pool.len() as u64);
        for nym in &self.pool {
            w.fixed(&nym.0);
        }
        w.u64(self.seals.len() as u64);
        for (epoch, seal) in &self.seals {
            w.u64(*epoch).u64(seal.pool as u64);
            write_option(w, seal.beacon.as_ref());
        }
        w.u64(self.entries.len() as u64);
        for e in &self.entries {
            e.question.write(w);
            w.fixed(&e.content.0).fixed(&e.author.0).u64(e.epoch);
            write_round(w, &e.round);
        }
        w.u64(self.slots.len() as u64);
        for ((epoch, nym), s) in &self.slots {
            w.u64(*epoch).fixed(&nym.0);
            w.u64(u64::from(s.quota)).u64(u64::from(s.used));
        }
        w.u64(self.authors.len() as u64);
        for (nym, record) in &self.authors {
            w.fixed(&nym.0).u64(record.observations().count() as u64);
            for (q, o) in record.observations() {
                w.u64(q);
                write_observation(w, o);
            }
        }
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

fn write_option(w: &mut Writer, bytes: Option<&[u8; 32]>) {
    match bytes {
        None => w.u8(0),
        Some(b) => w.u8(1).fixed(b),
    };
}

fn write_nyms<'a>(w: &mut Writer, nyms: impl ExactSizeIterator<Item = &'a Nym>) {
    w.u64(nyms.len() as u64);
    for nym in nyms {
        w.fixed(&nym.0);
    }
}

fn write_commits(w: &mut Writer, commits: &BTreeMap<Nym, AnswerCommit>) {
    w.u64(commits.len() as u64);
    for (nym, c) in commits {
        w.fixed(&nym.0).fixed(&c.0);
    }
}

fn write_observation(w: &mut Writer, o: &Observation) {
    w.u8(u8::from(o.success)).u64(o.weight.to_bits());
    w.u64(o.origin.epoch).u64(o.origin.closings);
}

fn write_round(w: &mut Writer, round: &Round) {
    match round {
        Round::Pending => {
            w.u8(0);
        }
        Round::Unavailable { available } => {
            w.u8(1).u64(u64::from(*available));
        }
        Round::Committing { group, commits } => {
            w.u8(2);
            write_nyms(w, group.iter());
            write_commits(w, commits);
        }
        Round::Revealing {
            group,
            commits,
            answers,
        } => {
            w.u8(3);
            write_nyms(w, group.iter());
            write_commits(w, commits);
            w.u64(answers.len() as u64);
            for (nym, a) in answers {
                w.fixed(&nym.0).fixed(&a.code());
            }
        }
        Round::Closed(c) => {
            let (num, den) = c.rule.threshold();
            w.u8(4).u64(c.tally.alternatives.len() as u64);
            for n in c
                .tally
                .alternatives
                .iter()
                .chain([&c.tally.none, &c.tally.abstentions])
            {
                w.u64(u64::from(*n));
            }
            w.u64(u64::from(c.tally.absences)).u8(c.verdict as u8);
            for part in [c.rule.group(), c.rule.quorum(), num, den] {
                w.u64(u64::from(part));
            }
            w.u64(c.at.epoch).u64(c.at.closings);
            match &c.observation {
                None => {
                    w.u8(0);
                }
                Some(o) => {
                    w.u8(1);
                    write_observation(w, o);
                }
            }
        }
    }
}
