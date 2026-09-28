//! An item one side never rated is not decided by its score (D42, T71): the gate sends it
//! to the band's extra round and the re-decision cannot pass it (`docs/08` AT-BR-12).

use protocol::gate::{bridging_gate, supplementary_review, GateOutcome, APPEAL_GAP, EPS, TAU};
use scoring::bridging::{bridge_scores, BridgingParams, Obs, Ratings};
use std::fs;
use std::path::PathBuf;

fn read_matrix(name: &str) -> Vec<Vec<f64>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../scoring/tests/fixtures")
        .join(name);
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.split(',').map(|c| c.trim().parse().unwrap()).collect())
        .collect()
}

/// The fixture (camps of 80 and 120) plus an eleventh item only the camp of 120 rated,
/// at about 0.97; returns the ratings, the item and a reviewer of the other camp.
fn one_sided_item() -> (Ratings, usize, usize) {
    let r = read_matrix("R.csv");
    let mask: Vec<Vec<bool>> = read_matrix("mask.csv")
        .iter()
        .map(|row| row.iter().map(|&v| v != 0.0).collect())
        .collect();
    let true_f: Vec<f64> = read_matrix("true_f.csv").iter().map(|row| row[0]).collect();
    let base = Ratings::from_dense(&r, &mask);
    let positive = true_f.iter().filter(|f| **f > 0.0).count();
    let majority = |u: usize| (true_f[u] > 0.0) == (positive * 2 > true_f.len());
    let j = base.m;
    let mut obs = base.obs.clone();
    for u in (0..base.n).filter(|&u| majority(u)) {
        let wobble = (((u * 7) % 11) as f64 - 5.0) * 0.006;
        obs.push(Obs {
            u,
            j,
            r: 0.97 + wobble,
        });
    }
    let other = (0..base.n).find(|&u| !majority(u)).unwrap();
    let ratings = Ratings {
        m: base.m + 1,
        obs,
        ..base
    };
    (ratings, j, other)
}

/// AT-BR-12: an item one side never rated goes to supplementary review, whatever its score.
#[test]
fn at_br_12_an_item_one_side_never_rated_is_not_passed() {
    let (ratings, j, _) = one_sided_item();
    let bridge = bridge_scores(&ratings, &BridgingParams::default(), 10, 0.85).unwrap();
    let (s, g, c) = (bridge.robust[j], bridge.full.gap[j], bridge.coverage[j]);
    assert_eq!(c, 0);
    let outcome = bridging_gate(s, g, c, TAU, EPS, APPEAL_GAP);
    assert_eq!(
        outcome,
        GateOutcome::SupplementaryReview,
        "robust {:.3}",
        bridge.robust[j]
    );
    for k in 0..j {
        let (s, g, c) = (bridge.robust[k], bridge.full.gap[k], bridge.coverage[k]);
        let outcome = bridging_gate(s, g, c, TAU, EPS, APPEAL_GAP);
        assert_ne!(
            outcome,
            GateOutcome::SupplementaryReview,
            "fixture item {k}: {s:.3}"
        );
    }
}

/// AT-BR-12: the re-decision cannot pass it uncovered; one rating from the other side decides it.
#[test]
fn at_br_12_the_re_decision_needs_a_rating_from_each_side() {
    let (ratings, j, other) = one_sided_item();
    let p = BridgingParams::default();
    let uncovered = supplementary_review(&ratings, &p, j, TAU, APPEAL_GAP).unwrap();
    assert_ne!(uncovered, GateOutcome::Pass);
    for (r, want) in [(0.9, true), (0.1, false)] {
        let mut covered = ratings.clone();
        covered.obs.push(Obs { u: other, j, r });
        let outcome = supplementary_review(&covered, &p, j, TAU, APPEAL_GAP).unwrap();
        assert_eq!(
            outcome == GateOutcome::Pass,
            want,
            "the other side rates it {r}"
        );
    }
}
