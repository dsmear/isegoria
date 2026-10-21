//! Level B — iterative purification. See `docs/02` §B.4.

use crate::dif::{logistic_dif, BETA2_MAX};
use crate::glm::LogisticFit;
use crate::irt::standardize;

/// One batch item's reading in a round; an inconclusive fit neither flags nor clears it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reading {
    Flagged,
    Clean,
    Inconclusive(LogisticFit),
}

/// A round's readings, every item read on `theta`.
#[derive(Clone, Debug, PartialEq)]
pub struct Round {
    pub index: usize,
    pub theta: Vec<f64>,
    pub readings: Vec<Reading>,
}

impl Round {
    pub fn inconclusive(&self) -> Vec<(usize, LogisticFit)> {
        let status = |(j, r): (usize, &Reading)| match r {
            Reading::Inconclusive(s) => Some((j, *s)),
            _ => None,
        };
        self.readings
            .iter()
            .enumerate()
            .filter_map(status)
            .collect()
    }

    fn flags(&self) -> Vec<bool> {
        self.readings
            .iter()
            .map(|r| *r == Reading::Flagged)
            .collect()
    }
}

/// A numerical fixed point of this procedure, its fits converged: `flagged` read on `theta`.
/// It certifies no statistical validity and no absence of DIF (`docs/02` §B.4).
#[derive(Clone, Debug, PartialEq)]
pub struct Purified {
    pub theta: Vec<f64>,
    pub flagged: Vec<bool>,
    pub iterations: usize,
}

/// How a purification ended; only `Completed` is a result, the rest diagnostics (`docs/02` §B.4).
#[derive(Clone, Debug, PartialEq)]
pub enum Purification {
    Completed(Purified),
    /// `round` read an inconclusive fit; θ not updated; `previous`, the last complete round.
    Interrupted {
        round: Round,
        previous: Option<Round>,
    },
    /// No verified fixed point in `rounds`; `theta` is what `last`'s flags gave (else the start).
    Exhausted {
        rounds: usize,
        last: Option<Round>,
        previous: Option<Round>,
        theta: Vec<f64>,
    },
}

/// `anchors` and `batch` are respondents × items.
pub fn purify_theta(
    anchors: &[Vec<f64>],
    batch: &[Vec<f64>],
    group: &[f64],
    max_rounds: usize,
) -> Purification {
    purify_with(anchors, batch, max_rounds, |j, theta| {
        let col: Vec<f64> = batch.iter().map(|r| r[j]).collect();
        let fit = logistic_dif(&col, theta, group);
        match fit.status {
            LogisticFit::Converged if fit.beta2.abs() > BETA2_MAX => Reading::Flagged,
            LogisticFit::Converged => Reading::Clean,
            status => Reading::Inconclusive(status),
        }
    })
}

fn purify_with(
    anchors: &[Vec<f64>],
    batch: &[Vec<f64>],
    max_rounds: usize,
    mut read: impl FnMut(usize, &[f64]) -> Reading,
) -> Purification {
    let k = batch.first().map_or(0, Vec::len);
    let anchor_totals: Vec<f64> = anchors.iter().map(|r| r.iter().sum()).collect();
    let update = |flags: &[bool]| {
        let totals: Vec<f64> = (0..anchor_totals.len())
            .map(|i| {
                let clean = (0..k).filter(|&j| !flags[j]).map(|j| batch[i][j]);
                anchor_totals[i] + clean.sum::<f64>()
            })
            .collect();
        standardize(&totals)
    };

    let mut theta = standardize(&anchor_totals);
    let mut last: Option<Round> = None;
    let mut previous: Option<Round> = None;
    for index in 1..=max_rounds {
        let readings = (0..k).map(|j| read(j, &theta)).collect();
        let round = Round {
            index,
            theta: theta.clone(),
            readings,
        };
        if !round.inconclusive().is_empty() {
            return Purification::Interrupted {
                round,
                previous: last,
            };
        }
        let flags = round.flags();
        if last.as_ref().is_some_and(|l| l.flags() == flags) {
            return Purification::Completed(Purified {
                theta: round.theta,
                flagged: flags,
                iterations: index,
            });
        }
        theta = update(&flags);
        previous = last.replace(round);
    }
    Purification::Exhausted {
        rounds: max_rounds,
        last,
        previous,
        theta,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn data() -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
        let anchors = (0..6)
            .map(|i| vec![(i % 3) as f64, (i % 2) as f64])
            .collect();
        let batch = (0..6)
            .map(|i| vec![(i % 2) as f64, (i / 3) as f64])
            .collect();
        (anchors, batch)
    }

    /// Readings by round from `script(round, item)`, counting every read.
    fn scripted<'a>(
        script: impl Fn(usize, usize) -> Reading + 'a,
        k: usize,
        calls: &'a Cell<usize>,
    ) -> impl FnMut(usize, &[f64]) -> Reading + 'a {
        move |j, _| {
            let round = calls.get() / k + 1;
            calls.set(calls.get() + 1);
            script(round, j)
        }
    }

    // Constructed: an all-clean first round is no verified fixed point; round 2 confirms it.
    #[test]
    fn an_all_clean_first_round_needs_a_second_round() {
        let (a, b) = data();
        let calls = Cell::new(0);
        let read = scripted(|_, _| Reading::Clean, 2, &calls);
        match purify_with(&a, &b, 5, read) {
            Purification::Completed(p) => {
                assert_eq!((p.iterations, p.flagged), (2, vec![false, false]));
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(calls.get(), 4);
    }

    // Constructed: a fixed point verified at the last allowed round is Completed.
    #[test]
    fn a_fixed_point_at_the_last_round_completes() {
        let (a, b) = data();
        let calls = Cell::new(0);
        let script = |_, j| {
            if j == 0 {
                Reading::Flagged
            } else {
                Reading::Clean
            }
        };
        let res = purify_with(&a, &b, 2, scripted(script, 2, &calls));
        assert!(matches!(res, Purification::Completed(ref p) if p.iterations == 2));
    }

    // Constructed: an inconclusive read in round 2 ends it after every item, θ not updated.
    #[test]
    fn an_inconclusive_read_interrupts_after_the_whole_round() {
        let (a, b) = data();
        let calls = Cell::new(0);
        let script = |round, j| match (round, j) {
            (2, 1) => Reading::Inconclusive(LogisticFit::NotConverged),
            (_, 0) => Reading::Flagged,
            _ => Reading::Clean,
        };
        let res = purify_with(&a, &b, 5, scripted(script, 2, &calls));
        let Purification::Interrupted { round, previous } = res else {
            panic!("{res:?}");
        };
        let previous = previous.expect("round 1 completed");
        assert_eq!(calls.get(), 4);
        assert_eq!(round.index, 2);
        assert_eq!(round.inconclusive(), vec![(1, LogisticFit::NotConverged)]);
        assert_eq!(round.readings[0], Reading::Flagged);
        assert_eq!(previous.index, 1);
        assert_ne!(round.theta, previous.theta);
    }

    // Constructed: readings that keep changing exhaust the budget; no cycle is claimed.
    #[test]
    fn changing_readings_exhaust_the_budget() {
        let (a, b) = data();
        let calls = Cell::new(0);
        let script = |round: usize, j| {
            if j == 0 && round % 2 == 1 {
                Reading::Flagged
            } else {
                Reading::Clean
            }
        };
        let res = purify_with(&a, &b, 4, scripted(script, 2, &calls));
        let Purification::Exhausted {
            rounds,
            last,
            previous,
            theta,
        } = res
        else {
            panic!("{res:?}");
        };
        let (last, previous) = (last.unwrap(), previous.unwrap());
        assert_eq!((rounds, last.index, previous.index), (4, 4, 3));
        assert_ne!(last.flags(), previous.flags());
        assert_ne!(theta, last.theta);
    }

    // Constructed: no round, no read, no flag; the start θ is returned as a start.
    #[test]
    fn a_zero_budget_reads_nothing() {
        let (a, b) = data();
        let calls = Cell::new(0);
        let res = purify_with(&a, &b, 0, scripted(|_, _| Reading::Clean, 2, &calls));
        let totals: Vec<f64> = a.iter().map(|r| r.iter().sum()).collect();
        let want = Purification::Exhausted {
            rounds: 0,
            last: None,
            previous: None,
            theta: standardize(&totals),
        };
        assert_eq!((res, calls.get()), (want, 0));
    }
}
