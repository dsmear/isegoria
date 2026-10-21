//! The baselines and difference scores of one item's reviewers (`docs/02` §C.2, `docs/01` D33,
//! D26; `docs/16` C5): a first panelist against the other first panelists, an extra reviewer
//! against the first panel. No report of the extra round enters any baseline.

use scoring::reputation::difference_score;

/// A first panelist's revealed forecast and its review weight, frozen for the epoch (finite,
/// non-negative: `orchestrator::bridging_weights`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Forecast {
    pub prob: f64,
    pub weight: f64,
}

/// The weighted mean of `forecasts`; `None` when they carry no weight. The weights are scaled by
/// the largest one's power of two, exactly in `f64`'s normal range (`docs/17` §14.9).
fn weighted_mean(forecasts: impl Iterator<Item = Forecast> + Clone) -> Option<f64> {
    let top = forecasts.clone().fold(0.0, |top: f64, f| top.max(f.weight));
    (top > 0.0).then(|| {
        let k = -libm::ilogb(top);
        let (sum, total) = forecasts.fold((0.0, 0.0), |(sum, total), f| {
            let weight = libm::scalbn(f.weight, k);
            (sum + weight * f.prob, total + weight)
        });
        sum / total
    })
}

/// Per first panelist, the weighted mean of the other first panelists' forecasts; `None` where
/// they carry no weight: no baseline, never the panelist's own forecast and never 0.
pub fn first_panel_baselines(first: &[Forecast]) -> Vec<Option<f64>> {
    (0..first.len())
        .map(|u| {
            weighted_mean(
                first
                    .iter()
                    .enumerate()
                    .filter(|&(v, _)| v != u)
                    .map(|(_, &f)| f),
            )
        })
        .collect()
}

/// Every extra reviewer's baseline: the first panel's weighted mean, fixed before the extra
/// round reports; `None` when the first panel carries no weight.
pub fn extra_round_baseline(first: &[Forecast]) -> Option<f64> {
    weighted_mean(first.iter().copied())
}

/// An item's difference scores on its outcome, in input order; `None` without a baseline.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemScores {
    pub first: Vec<Option<f64>>,
    pub extra: Vec<Option<f64>>,
}

/// `extra` holds the extra round's forecasts, empty for an item decided without it; their
/// weights enter no baseline, so none is taken.
pub fn item_scores(first: &[Forecast], extra: &[f64], outcome: f64) -> ItemScores {
    let score = |p: f64, b: Option<f64>| b.map(|b| difference_score(p, b, outcome));
    let extra_baseline = extra_round_baseline(first);
    ItemScores {
        first: first
            .iter()
            .zip(first_panel_baselines(first))
            .map(|(f, b)| score(f.prob, b))
            .collect(),
        extra: extra.iter().map(|&p| score(p, extra_baseline)).collect(),
    }
}
