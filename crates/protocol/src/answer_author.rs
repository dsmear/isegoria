//! The answer review's author record (`docs/02` §C.1, `docs/01` D50–D51): one weighted
//! observation per closed question, the regularized index over them, and the epoch's quota.

use crate::answer_rule::Verdict;
use crate::author_time::{AuthorConfig, AuthorPoint};
use scoring::reputation::{author_score_weighted, AuthorPrior};
use std::collections::BTreeMap;

/// A closed question's contribution: success or failure, its weight `ω`, and the point it was
/// recorded at, whose epoch it starts counting after.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Observation {
    pub success: bool,
    pub weight: f64,
    pub origin: AuthorPoint,
}

/// D50's input for `verdict`: `(success, ω)`, `ω` of divided answers `divided_weight`; `None`
/// for insufficient participation.
pub fn contribution(verdict: Verdict, divided_weight: f64) -> Option<(bool, f64)> {
    match verdict {
        Verdict::Accepted => Some((true, 1.0)),
        Verdict::Discarded => Some((false, 1.0)),
        Verdict::Divided => Some((false, divided_weight)),
        Verdict::InsufficientParticipation => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DuplicateObservation;

/// An author's observations, keyed by question number: one each, never revised.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AuthorRecord {
    observations: BTreeMap<u64, Observation>,
}

impl AuthorRecord {
    pub fn record(
        &mut self,
        question: u64,
        observation: Observation,
    ) -> Result<(), DuplicateObservation> {
        if self.observations.contains_key(&question) {
            return Err(DuplicateObservation);
        }
        self.observations.insert(question, observation);
        Ok(())
    }

    pub fn observation(&self, question: u64) -> Option<&Observation> {
        self.observations.get(&question)
    }

    /// The observations that count at `at`: those recorded in an earlier epoch.
    pub fn counted(&self, at: AuthorPoint) -> usize {
        self.counting(at).count()
    }

    /// `C_a` at `at`, its observations in question order, aged in closings since recorded.
    pub fn reputation(&self, at: AuthorPoint, config: AuthorConfig) -> f64 {
        let mut qualities = Vec::new();
        let mut weights = Vec::new();
        let mut ages = Vec::new();
        for o in self.counting(at) {
            qualities.push(if o.success { 1.0 } else { 0.0 });
            weights.push(o.weight);
            ages.push(at.closings.saturating_sub(o.origin.closings));
        }
        let prior = AuthorPrior::default();
        author_score_weighted(
            &qualities,
            &weights,
            &ages,
            config.half_life_closings(),
            &prior,
        )
    }

    fn counting(&self, at: AuthorPoint) -> impl Iterator<Item = &Observation> {
        self.observations
            .values()
            .filter(move |o| o.origin.epoch < at.epoch)
    }
}

/// D51's quota bands: `quotas[0]` below `low`, `quotas[1]` below `high`, `quotas[2]` from it;
/// `new_author` with no observation counted. Provisional (`docs/14`, D52).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuotaBands {
    low: f64,
    high: f64,
    quotas: [u32; 3],
    new_author: u32,
}

impl Default for QuotaBands {
    fn default() -> Self {
        QuotaBands {
            low: 0.30,
            high: 0.70,
            quotas: [1, 2, 3],
            new_author: 2,
        }
    }
}

impl QuotaBands {
    /// `None` unless `0 ≤ low ≤ high ≤ 1`.
    pub fn new(low: f64, high: f64, quotas: [u32; 3], new_author: u32) -> Option<Self> {
        (0.0 <= low && low <= high && high <= 1.0).then_some(QuotaBands {
            low,
            high,
            quotas,
            new_author,
        })
    }

    /// `(low, high, quotas, new_author)`.
    pub fn parts(self) -> (f64, f64, [u32; 3], u32) {
        (self.low, self.high, self.quotas, self.new_author)
    }

    /// The band of `reputation`, compared in binary64.
    pub fn quota(self, reputation: f64) -> u32 {
        if reputation < self.low {
            self.quotas[0]
        } else if reputation < self.high {
            self.quotas[1]
        } else {
            self.quotas[2]
        }
    }

    /// An author's quota at `at`: a new author's, or the band of its reputation (D51).
    pub fn quota_of(
        self,
        record: Option<&AuthorRecord>,
        at: AuthorPoint,
        config: AuthorConfig,
    ) -> u32 {
        match record {
            Some(r) if r.counted(at) > 0 => self.quota(r.reputation(at, config)),
            _ => self.new_author,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(epoch: u64, closings: u64) -> AuthorPoint {
        AuthorPoint { epoch, closings }
    }

    fn obs(success: bool, weight: f64, epoch: u64) -> Observation {
        Observation {
            success,
            weight,
            origin: at(epoch, epoch),
        }
    }

    // AT-ANS-10: the bands' edges in binary64: 0.30 and 0.70 open the upper band.
    #[test]
    fn the_bands_edges() {
        let bands = QuotaBands::default();
        assert_eq!(bands.quota(0.0), 1);
        assert_eq!(bands.quota(f64::from_bits(0.30f64.to_bits() - 1)), 1);
        assert_eq!(bands.quota(0.30), 2);
        assert_eq!(bands.quota(0.40), 2);
        assert_eq!(bands.quota(f64::from_bits(0.70f64.to_bits() - 1)), 2);
        assert_eq!(bands.quota(0.70), 3);
        assert_eq!(bands.quota(1.0), 3);
        assert!(QuotaBands::new(0.7, 0.3, [1, 2, 3], 2).is_none());
        assert!(QuotaBands::new(-0.1, 0.3, [1, 2, 3], 2).is_none());
        assert!(QuotaBands::new(0.3, 1.1, [1, 2, 3], 2).is_none());
    }

    // AT-ANS-10: a new author gets 2; observations recorded this epoch do not count yet.
    #[test]
    fn a_new_author_and_the_epoch_passage() {
        let (bands, config) = (QuotaBands::default(), AuthorConfig::default());
        assert_eq!(bands.quota_of(None, at(0, 0), config), 2);
        let mut record = AuthorRecord::default();
        for q in 0..2 {
            record.record(q, obs(false, 1.0, 3)).unwrap();
        }
        assert_eq!(record.counted(at(3, 3)), 0);
        assert_eq!(record.reputation(at(3, 3), config), 0.4);
        assert_eq!(bands.quota_of(Some(&record), at(3, 3), config), 2);
        assert_eq!(record.counted(at(4, 4)), 2);
        assert!(record.reputation(at(4, 4), config) < 0.30);
        assert_eq!(bands.quota_of(Some(&record), at(4, 4), config), 1);
    }

    // AT-ANS-06: one observation per question, never revised.
    #[test]
    fn one_observation_per_question() {
        let mut record = AuthorRecord::default();
        record.record(5, obs(true, 1.0, 0)).unwrap();
        assert_eq!(
            record.record(5, obs(false, 1.0, 0)),
            Err(DuplicateObservation)
        );
        assert!(record.observation(5).unwrap().success);
    }

    // AT-ANS-08: D50's inputs per verdict; insufficient participation gives none.
    #[test]
    fn the_inputs_per_verdict() {
        assert_eq!(contribution(Verdict::Accepted, 0.25), Some((true, 1.0)));
        assert_eq!(contribution(Verdict::Discarded, 0.25), Some((false, 1.0)));
        assert_eq!(contribution(Verdict::Divided, 0.25), Some((false, 0.25)));
        assert_eq!(contribution(Verdict::InsufficientParticipation, 0.25), None);
    }
}
