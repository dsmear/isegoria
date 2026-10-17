//! The author's history and the appeal stake (`docs/02` §C.1, `docs/01` D27, `docs/15` C5–C6):
//! one binary pilot outcome per item, its origin the applying cut; an appeal
//! escrows a zero at weight 1 that the item's outcome settles, never a ledger deduction.

use crate::author_time::{AuthorConfig, AuthorPoint};
use crate::lifecycle::{RejectReason, State};
use network::cid::Cid;
use scoring::reputation::{author_score_closings, AuthorPrior};
use std::collections::BTreeMap;

pub const STAKE_QUALITY: f64 = 0.0;

/// The stake an author's `C_a` must cover to file: the prior mean (`docs/08` §9.1; T25).
pub fn appeal_floor(prior: &AuthorPrior) -> f64 {
    prior.alpha0 / (prior.alpha0 + prior.beta0)
}

/// The pilot outcome a state concludes (`docs/02` §C.1): passed in either pool, failed on the
/// screen or DIF; `None` otherwise, whatever the state reached later (`docs/15` C5).
pub fn pilot_outcome(state: &State) -> Option<bool> {
    match state {
        State::ActivePool | State::Contested => Some(true),
        State::Rejected(RejectReason::Screen | RejectReason::Dif) => Some(false),
        State::Deposited
        | State::Admitted
        | State::InReview { .. }
        | State::Revealing { .. }
        | State::SupplementaryReview { .. }
        | State::AppealEligible
        | State::Pilot1 { .. }
        | State::Pilot2 { .. }
        | State::Rejected(
            RejectReason::Defect | RejectReason::Polarized | RejectReason::Borderline,
        )
        | State::Explored { .. }
        | State::Measured { .. }
        | State::Retired(_) => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Outcome {
    passed: bool,
    origin: AuthorPoint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Escrow {
    filed: u64,
    settled: Option<u64>,
}

/// An author's pilot outcomes and appeal escrows, keyed by item (`docs/15` C5–C6).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AuthorHistory {
    outcomes: BTreeMap<Cid, Outcome>,
    escrows: BTreeMap<Cid, Escrow>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InsufficientReputation {
    pub reputation: f64,
    pub floor: f64,
}

/// Why a history refused a change; a refused change leaves it as it was.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AuthorError {
    /// The item already has an outcome or an escrow: one observation per item.
    Duplicate,
    Uncovered(InsufficientReputation),
    NoOpenEscrow,
}

impl AuthorHistory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records an unappealed item's pilot outcome, its applying origin `at`.
    pub fn conclude(
        &mut self,
        item: Cid,
        passed: bool,
        at: AuthorPoint,
    ) -> Result<(), AuthorError> {
        self.fresh(&item)?;
        self.outcomes.insert(item, Outcome { passed, origin: at });
        Ok(())
    }

    /// Files an appeal on `item` if [`Self::reputation`] at `at` covers the stake, escrowing
    /// a zero that counts for every later check, those of `at.epoch` included; the escrow only, as
    /// the node starts the appeal with it (`docs/15` C5–C6).
    pub fn file_appeal(
        &mut self,
        item: Cid,
        at: AuthorPoint,
        config: AuthorConfig,
        prior: &AuthorPrior,
    ) -> Result<(), AuthorError> {
        self.fresh(&item)?;
        let reputation = self.reputation(at, config, prior);
        let floor = appeal_floor(prior);
        if reputation < floor {
            return Err(AuthorError::Uncovered(InsufficientReputation {
                reputation,
                floor,
            }));
        }
        let escrow = Escrow {
            filed: at.epoch,
            settled: None,
        };
        self.escrows.insert(item, escrow);
        Ok(())
    }

    /// Settles `item`'s open escrow with its pilot outcome: the escrow counts through `at.epoch`,
    /// the outcome, its origin `at`, from the next epoch.
    pub fn settle(&mut self, item: Cid, passed: bool, at: AuthorPoint) -> Result<(), AuthorError> {
        let filed = match self.escrows.get(&item) {
            Some(Escrow {
                filed,
                settled: None,
            }) => *filed,
            _ => return Err(AuthorError::NoOpenEscrow),
        };
        let settled = Some(at.epoch);
        self.escrows.insert(item, Escrow { filed, settled });
        self.outcomes.insert(item, Outcome { passed, origin: at });
        Ok(())
    }

    /// `C_a` for a check at `at`: outcomes of earlier epochs, aged in closings; escrows filed so
    /// far and not settled before `at.epoch`, at `q = 0` and weight 1.
    pub fn reputation(&self, at: AuthorPoint, config: AuthorConfig, prior: &AuthorPrior) -> f64 {
        self.score(at, |filed| filed <= at.epoch, config, prior)
    }

    /// `C_a` fixed for `at.epoch`'s quota: outcomes before it, escrows open when the prior ended.
    pub fn quota_reputation(
        &self,
        at: AuthorPoint,
        config: AuthorConfig,
        prior: &AuthorPrior,
    ) -> f64 {
        self.score(at, |filed| filed < at.epoch, config, prior)
    }

    pub fn covers_stake(&self, at: AuthorPoint, config: AuthorConfig, prior: &AuthorPrior) -> bool {
        self.reputation(at, config, prior) >= appeal_floor(prior)
    }

    /// `(passed, origin)` of `item`'s outcome, if recorded.
    pub fn outcome(&self, item: &Cid) -> Option<(bool, AuthorPoint)> {
        self.outcomes.get(item).map(|o| (o.passed, o.origin))
    }

    pub fn outcomes(&self) -> usize {
        self.outcomes.len()
    }

    pub fn is_open(&self, item: &Cid) -> bool {
        matches!(self.escrows.get(item), Some(Escrow { settled: None, .. }))
    }

    pub fn open_escrows(&self) -> Vec<Cid> {
        self.escrows
            .iter()
            .filter(|(_, e)| e.settled.is_none())
            .map(|(item, _)| *item)
            .collect()
    }

    fn fresh(&self, item: &Cid) -> Result<(), AuthorError> {
        if self.outcomes.contains_key(item) || self.escrows.contains_key(item) {
            return Err(AuthorError::Duplicate);
        }
        Ok(())
    }

    fn score(
        &self,
        at: AuthorPoint,
        counted: impl Fn(u64) -> bool,
        config: AuthorConfig,
        prior: &AuthorPrior,
    ) -> f64 {
        let mut qualities = Vec::new();
        let mut ages = Vec::new();
        for o in self.outcomes.values().filter(|o| o.origin.epoch < at.epoch) {
            qualities.push(if o.passed { 1.0 } else { 0.0 });
            ages.push(at.closings.saturating_sub(o.origin.closings));
        }
        let open = |e: &&Escrow| counted(e.filed) && e.settled.is_none_or(|s| s >= at.epoch);
        for _ in self.escrows.values().filter(open) {
            qualities.push(STAKE_QUALITY);
            ages.push(0);
        }
        author_score_closings(&qualities, &ages, config.half_life_closings(), prior)
    }
}
