//! The author's history and the appeal stake (`docs/02` §C.1, `docs/01` D27, `docs/15` C5–C6):
//! one binary pilot outcome per item, its origin the results epoch that records it; an appeal
//! escrows a zero at weight 1 that the item's outcome settles, never a ledger deduction.

use crate::lifecycle::{RejectReason, State};
use network::cid::Cid;
use scoring::reputation::{author_score, AuthorPrior};
use std::collections::BTreeMap;

pub const STAKE_QUALITY: f64 = 0.0;

/// The stake an author's `C_a` must cover to file: the prior mean (`docs/08` §9.1; T25).
pub fn appeal_floor(prior: &AuthorPrior) -> f64 {
    prior.alpha0 / (prior.alpha0 + prior.beta0)
}

/// `E_m`, epochs per month: at epoch `e` an outcome of origin `e_j` is `(e − e_j)/E_m` months old.
/// No default: the value is undecided (`docs/15` C6, `docs/14`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EpochScale(f64);

impl EpochScale {
    /// `None` unless `epochs_per_month` is finite and positive.
    pub fn new(epochs_per_month: f64) -> Option<Self> {
        (epochs_per_month.is_finite() && epochs_per_month > 0.0)
            .then_some(EpochScale(epochs_per_month))
    }

    pub fn epochs_per_month(self) -> f64 {
        self.0
    }

    pub fn age_months(self, origin: u64, epoch: u64) -> f64 {
        epoch.saturating_sub(origin) as f64 / self.0
    }
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
    origin: u64,
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
    last: u64,
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
    /// An epoch before the latest one the history recorded.
    EpochBehind,
}

impl AuthorHistory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records an unappealed item's pilot outcome, its origin `epoch`.
    pub fn conclude(&mut self, item: Cid, passed: bool, epoch: u64) -> Result<(), AuthorError> {
        self.fresh(&item, epoch)?;
        self.outcomes.insert(
            item,
            Outcome {
                passed,
                origin: epoch,
            },
        );
        self.last = epoch;
        Ok(())
    }

    /// Files an appeal on `item` if [`Self::reputation`] at `epoch` covers the stake, escrowing
    /// a zero that counts for every later check, those of `epoch` included; the escrow only, as
    /// the node starts the appeal with it (`docs/15` C5–C6).
    pub fn file_appeal(
        &mut self,
        item: Cid,
        epoch: u64,
        scale: EpochScale,
        prior: &AuthorPrior,
    ) -> Result<(), AuthorError> {
        self.fresh(&item, epoch)?;
        let reputation = self.reputation(epoch, scale, prior);
        let floor = appeal_floor(prior);
        if reputation < floor {
            return Err(AuthorError::Uncovered(InsufficientReputation {
                reputation,
                floor,
            }));
        }
        let escrow = Escrow {
            filed: epoch,
            settled: None,
        };
        self.escrows.insert(item, escrow);
        self.last = epoch;
        Ok(())
    }

    /// Settles `item`'s open escrow with its pilot outcome: the escrow counts through `epoch`,
    /// the outcome, its origin `epoch`, from the next.
    pub fn settle(&mut self, item: Cid, passed: bool, epoch: u64) -> Result<(), AuthorError> {
        let filed = match self.escrows.get(&item) {
            Some(Escrow {
                filed,
                settled: None,
            }) => *filed,
            _ => return Err(AuthorError::NoOpenEscrow),
        };
        if epoch < self.last {
            return Err(AuthorError::EpochBehind);
        }
        let settled = Some(epoch);
        self.escrows.insert(item, Escrow { filed, settled });
        self.outcomes.insert(
            item,
            Outcome {
                passed,
                origin: epoch,
            },
        );
        self.last = epoch;
        Ok(())
    }

    /// `C_a` for a check in `epoch`: outcomes of earlier epochs, aged; escrows filed so far and
    /// not settled before `epoch`, at `q = 0` and weight 1.
    pub fn reputation(&self, epoch: u64, scale: EpochScale, prior: &AuthorPrior) -> f64 {
        self.score(epoch, |filed| filed <= epoch, scale, prior)
    }

    /// `C_a` fixed for epoch `e`'s quota: outcomes before `e`, escrows open at the end of `e − 1`.
    pub fn quota_reputation(&self, epoch: u64, scale: EpochScale, prior: &AuthorPrior) -> f64 {
        self.score(epoch, |filed| filed < epoch, scale, prior)
    }

    pub fn covers_stake(&self, epoch: u64, scale: EpochScale, prior: &AuthorPrior) -> bool {
        self.reputation(epoch, scale, prior) >= appeal_floor(prior)
    }

    /// `(passed, origin)` of `item`'s outcome, if recorded.
    pub fn outcome(&self, item: &Cid) -> Option<(bool, u64)> {
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

    fn fresh(&self, item: &Cid, epoch: u64) -> Result<(), AuthorError> {
        if self.outcomes.contains_key(item) || self.escrows.contains_key(item) {
            return Err(AuthorError::Duplicate);
        }
        if epoch < self.last {
            return Err(AuthorError::EpochBehind);
        }
        Ok(())
    }

    fn score(
        &self,
        epoch: u64,
        counted: impl Fn(u64) -> bool,
        scale: EpochScale,
        prior: &AuthorPrior,
    ) -> f64 {
        let mut qualities = Vec::new();
        let mut ages = Vec::new();
        for o in self.outcomes.values().filter(|o| o.origin < epoch) {
            qualities.push(if o.passed { 1.0 } else { 0.0 });
            ages.push(scale.age_months(o.origin, epoch));
        }
        let open = |e: &&Escrow| counted(e.filed) && e.settled.is_none_or(|s| s >= epoch);
        for _ in self.escrows.values().filter(open) {
            qualities.push(STAKE_QUALITY);
            ages.push(0.0);
        }
        author_score(&qualities, &ages, prior)
    }
}
