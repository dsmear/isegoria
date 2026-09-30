//! The generators draw the populations `docs/13` §3 defines: the paper's batches, the
//! design's class shares and attackers, and the bridging designs.

use characterization::generate::{
    ability, dif_batch, expected_clamped, extra_data, fixture, screen_batch, sweep_data, PROBES,
    PROBE_SPREAD, SCREEN_GUESS, SCREEN_ITEMS,
};
use characterization::grid::{Attack, DifDesign, ExtraDesign, Layout, ScreenDesign, SweepDesign};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use scoring::irt::kr20;
use sha2::Digest;

fn design(n: usize, anchors: usize, k: usize, layout: Layout) -> DifDesign {
    DifDesign {
        n,
        anchors,
        k,
        layout,
        ..DifDesign::default()
    }
}

fn share(rows: &[Vec<f64>], j: usize, keep: impl Fn(usize) -> bool) -> f64 {
    let kept: Vec<f64> = (0..rows.len())
        .filter(|&i| keep(i))
        .map(|i| rows[i][j])
        .collect();
    kept.iter().sum::<f64>() / kept.len() as f64
}

/// The anchors' KR-20 follows the paper's table for 10, 20, 30 and 60 anchors (T53).
#[test]
fn the_anchors_reach_the_paper_s_kr20() {
    for (anchors, paper) in [(10, 0.694), (20, 0.822), (30, 0.874), (60, 0.932)] {
        let mean = (0..4)
            .map(|s| kr20(&dif_batch(&design(6000, anchors, 8, Layout::Campaign(0)), s).anchors))
            .sum::<f64>()
            / 4.0;
        assert!(
            (mean - paper).abs() < 0.03,
            "{anchors} anchors: {mean:.3} vs {paper}"
        );
    }
}

/// The class share is π, a biased item is harder for class +1 by about 2δ, a clean one is not.
#[test]
fn a_biased_item_separates_the_classes_and_a_clean_one_does_not() {
    let d = DifDesign {
        delta: 0.9,
        pi: 0.3,
        ..design(6000, 20, 4, Layout::Campaign(1))
    };
    let batch = dif_batch(&d, 11);
    assert_eq!(batch.roles, "+ccc");
    let plus = batch.z.iter().filter(|&&z| z > 0.0).count() as f64 / 6000.0;
    assert!((plus - 0.3).abs() < 0.02, "share of class +1: {plus}");
    let gap = |j: usize| {
        share(&batch.x, j, |i| batch.z[i] < 0.0) - share(&batch.x, j, |i| batch.z[i] > 0.0)
    };
    assert!(gap(0) > 0.25, "biased item: {}", gap(0));
    assert!(gap(1).abs() < 0.05 && gap(2).abs() < 0.05, "clean items");
}

/// Impact shifts class +1's ability; guessing puts a floor under every item.
#[test]
fn impact_and_guessing_change_the_population_as_designed() {
    let d = DifDesign {
        impact: 1.0,
        guess: 0.2,
        ..design(6000, 20, 4, Layout::Campaign(0))
    };
    let batch = dif_batch(&d, 12);
    let total = |i: usize| batch.anchors[i].iter().sum::<f64>();
    let mean = |plus: bool| {
        let rows: Vec<f64> = (0..6000)
            .filter(|&i| (batch.z[i] > 0.0) == plus)
            .map(total)
            .collect();
        rows.iter().sum::<f64>() / rows.len() as f64
    };
    assert!(
        mean(true) > mean(false) + 2.0,
        "{} vs {}",
        mean(true),
        mean(false)
    );
    let lowest = (0..20)
        .map(|j| share(&batch.anchors, j, |_| true))
        .fold(1.0_f64, f64::min);
    assert!(lowest > 0.2, "an anchor below the guessing floor: {lowest}");
}

/// Injecting attackers answer the targets wrong; masking ones answer the biased items as if clean.
#[test]
fn the_attackers_are_the_last_rows_and_act_as_designed() {
    let inject = DifDesign {
        attack: Attack::Inject {
            fraction: 0.05,
            targets: 2,
        },
        ..design(2000, 10, 6, Layout::Campaign(0))
    };
    let batch = dif_batch(&inject, 13);
    assert_eq!(batch.roles, "cccctt");
    assert!(batch.x[1900..]
        .iter()
        .all(|row| row[4] == 0.0 && row[5] == 0.0));
    assert!(share(&batch.x, 4, |i| i < 1900) > 0.2);

    let mask = DifDesign {
        delta: 3.0,
        attack: Attack::Mask { fraction: 0.5 },
        ..design(4000, 10, 4, Layout::Campaign(1))
    };
    let batch = dif_batch(&mask, 14);
    let gap = |from: usize, to: usize| {
        share(&batch.x, 0, |i| (from..to).contains(&i) && batch.z[i] < 0.0)
            - share(&batch.x, 0, |i| (from..to).contains(&i) && batch.z[i] > 0.0)
    };
    assert!(gap(0, 2000) > 0.5, "honest rows: {}", gap(0, 2000));
    assert!(
        gap(2000, 4000).abs() < 0.06,
        "masking rows: {}",
        gap(2000, 4000)
    );
}

/// Mirror and two-axis layouts place their leaners as specified.
#[test]
fn the_layouts_place_their_leaners() {
    let mirror = dif_batch(&design(100, 5, 8, Layout::Mirror), 1);
    assert_eq!(mirror.roles, "+-+-cccc");
    assert_eq!(mirror.signs, vec![1.0, -1.0, 1.0, -1.0, 0.0, 0.0, 0.0, 0.0]);
    let two = dif_batch(&design(100, 5, 8, Layout::TwoAxes(2)), 1);
    assert_eq!(two.roles, "++22cccc");
}

/// The bridging design has its camps, its mirror pairs and its ratings per reviewer.
#[test]
fn the_bridging_design_has_its_camps_pairs_and_ratings() {
    let d = SweepDesign {
        n: 200,
        share: 0.6,
        per_reviewer: 9,
        noise: 0.07,
        lambda: None,
    };
    let data = sweep_data(&d, 5);
    assert_eq!((data.ratings.n, data.ratings.m), (200, 20));
    assert_eq!(data.ratings.obs.len(), 200 * 9);
    assert_eq!(data.true_f.iter().filter(|&&f| f > 0.0).count(), 120);
    assert!(data.lean[..10].iter().all(|&l| l == 0.0));
    assert_eq!(&data.lean[10..12], &[0.8, -0.8]);
    assert!(data.q[..10].iter().all(|&q| (0.70..0.95).contains(&q)));
    for j in 0..10 {
        let (q, t) = (data.q[j], data.truth[j]);
        assert!(
            t <= q + 0.01 && t > q - 0.06,
            "consensus item {j}: q {q}, truth {t}"
        );
    }
    assert!(data.truth[10..].iter().all(|&t| (0.45..0.65).contains(&t)));
    let fx = fixture();
    assert_eq!(
        (fx.r.len(), fx.mask.len(), fx.true_f.len()),
        (200, 200, 200)
    );
    assert_eq!(fx.true_f.iter().filter(|&&f| f < 0.0).count(), 80);
}

/// The expected clamped rating matches hand-computed values (a Monte Carlo agrees to 1e-4).
#[test]
fn the_expected_clamped_rating_matches_hand_computed_values() {
    let close = |a: f64, b: f64| (a - b).abs() < 1e-6;
    assert!(close(expected_clamped(0.95, 0.15), 0.911_865));
    assert!(close(expected_clamped(0.5, 0.15), 0.5));
    assert!(close(expected_clamped(0.02, 0.07), 0.039_058));
    assert_eq!(expected_clamped(1.3, 0.0), 1.0);
}

/// The same seed draws the same batch; another seed another one.
#[test]
fn a_batch_is_a_function_of_its_seed() {
    let d = DifDesign {
        delta: 0.5,
        ..design(300, 8, 4, Layout::Campaign(2))
    };
    let (a, b, c) = (dif_batch(&d, 3), dif_batch(&d, 3), dif_batch(&d, 4));
    assert_eq!((&a.anchors, &a.x), (&b.anchors, &b.x));
    assert_ne!(a.x, c.x);
}

/// A floor per column around the design's, the fitted curves' `floors` the items' (`docs/13` §8.2).
#[test]
fn a_spread_draws_each_column_s_floor_around_the_design_s() {
    let d = DifDesign {
        guess: 0.2,
        spread: 0.1,
        ..design(6000, 20, 8, Layout::Campaign(0))
    };
    let batch = dif_batch(&d, 21);
    assert!(
        batch.floors.iter().all(|c| (0.1..=0.3).contains(c)),
        "{:?}",
        batch.floors
    );
    let distinct: std::collections::BTreeSet<u64> =
        batch.floors.iter().map(|c| c.to_bits()).collect();
    assert_eq!(distinct.len(), 8);
    let fixed = dif_batch(&DifDesign { spread: 0.0, ..d }, 21);
    assert!(fixed.floors.iter().all(|&c| c == 0.2));
    let lowest = |b: &characterization::generate::DifBatch| {
        (0..20)
            .map(|j| share(&b.anchors, j, |_| true))
            .fold(1.0_f64, f64::min)
    };
    assert!(lowest(&batch) > 0.1 && lowest(&fixed) > 0.2);
}

/// The skewed ability has mean 0, variance 1 and the skew of its shape's sign (`docs/13` §8.2).
#[test]
fn the_skewed_ability_is_standardized_and_skewed_as_its_shape() {
    for shape in [-4.0, 0.0, 4.0] {
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        let x: Vec<f64> = (0..200_000).map(|_| ability(shape, &mut rng)).collect();
        let n = x.len() as f64;
        let mean = x.iter().sum::<f64>() / n;
        let var = x.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        let skew = x.iter().map(|v| (v - mean).powi(3)).sum::<f64>() / n / var.powf(1.5);
        assert!(
            mean.abs() < 0.01 && (var - 1.0).abs() < 0.01,
            "{shape}: {mean} {var}"
        );
        let expected = if shape == 0.0 {
            0.0
        } else {
            0.7845 * shape.signum()
        };
        assert!((skew - expected).abs() < 0.03, "{shape}: skewness {skew}");
    }
}

/// Items of one template, a pair, agree more than items of two (`docs/13` §8.2).
#[test]
fn items_of_one_template_are_answered_alike() {
    let corr = |testlet: f64, i: usize, j: usize| {
        let d = DifDesign {
            testlet,
            ..design(8000, 10, 4, Layout::Campaign(0))
        };
        let b = dif_batch(&d, 31);
        let (mi, mj) = (share(&b.x, i, |_| true), share(&b.x, j, |_| true));
        let cov = b.x.iter().map(|r| (r[i] - mi) * (r[j] - mj)).sum::<f64>() / 8000.0;
        cov / (mi * (1.0 - mi) * mj * (1.0 - mj)).sqrt()
    };
    let (pair, apart) = (corr(1.0, 0, 1), corr(1.0, 1, 2));
    assert!(
        pair > apart + 0.08,
        "same template {pair}, two templates {apart}"
    );
    assert!((corr(0.0, 0, 1) - corr(0.0, 1, 2)).abs() < 0.05);
}

/// At `b = 0` a pilot's item is right at its floor plus half the rest: chance, or chance plus
/// `SCREEN_GUESS` for the guessable item; the item keyed backwards the complement.
#[test]
fn a_screen_pilot_draws_its_items_as_designed() {
    let d = ScreenDesign {
        n: 20_000,
        anchors: 10,
        options: 4,
    };
    let batch = screen_batch(&d, 3);
    assert_eq!(batch.roles, "gggggffhck");
    assert_eq!((batch.x.len(), batch.anchors.len()), (20_000, 20_000));
    assert!(batch.x.iter().all(|row| row.len() == SCREEN_ITEMS.len()));
    assert!(batch.anchors.iter().all(|row| row.len() == 10));
    let at_chance = 0.25 + 0.75 * 0.5;
    let guessable = 0.25 + SCREEN_GUESS + (0.75 - SCREEN_GUESS) * 0.5;
    for (j, expected) in [
        (0, at_chance),
        (5, at_chance),
        (6, at_chance),
        (8, guessable),
        (9, 1.0 - at_chance),
    ] {
        let right = share(&batch.x, j, |_| true);
        assert!(
            (right - expected).abs() < 0.015,
            "item {j}: {right} vs {expected}"
        );
    }
    let hard = share(&batch.x, 7, |_| true);
    assert!(hard > 0.25 && hard < 0.3, "the item too hard: {hard}");
}

/// A pilot's draws are pinned: the stream of `docs/13` §8.2 that recorded runs reproduce.
#[test]
fn a_screen_pilot_s_draws_are_pinned() {
    let d = ScreenDesign {
        n: 50,
        anchors: 6,
        options: 5,
    };
    let batch = screen_batch(&d, 17);
    let mut h = sha2::Sha256::new();
    for row in batch.anchors.iter().chain(&batch.x) {
        for v in row {
            h.update(v.to_bits().to_le_bytes());
        }
    }
    let digest: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(
        digest,
        "438ebd6ec97c2926ebb2dbc753d18832b51270def76e9c0321dbc6add2cc161f"
    );
}

/// The probes lie within `PROBE_SPREAD` of `τ`, their truth follows their quality, and every
/// reviewer has a rating of each (`docs/13` §8.2).
#[test]
fn the_extra_round_s_probes_lie_around_the_threshold() {
    let d = ExtraDesign {
        n: 400,
        share: 0.8,
        per_reviewer: 5,
        noise: 0.15,
        panel: 7,
    };
    let data = extra_data(&d, 0.8, 9);
    assert_eq!(data.probe_q.len(), PROBES);
    assert!(data
        .probe_q
        .iter()
        .all(|q| (0.8 - PROBE_SPREAD..0.8 + PROBE_SPREAD).contains(q)));
    for (q, t) in data.probe_q.iter().zip(&data.probe_truth) {
        assert!((q - t).abs() < 0.02, "quality {q}, truth {t}");
    }
    assert!(data.probe_ratings.iter().all(|r| r.len() == 400));
    let plain = sweep_data(
        &SweepDesign {
            n: 400,
            share: 0.8,
            per_reviewer: 5,
            noise: 0.15,
            lambda: None,
        },
        9,
    );
    let triples = |obs: &[scoring::bridging::Obs]| -> Vec<(usize, usize, u64)> {
        obs.iter().map(|o| (o.u, o.j, o.r.to_bits())).collect()
    };
    assert_eq!(
        triples(&data.sweep.ratings.obs),
        triples(&plain.ratings.obs)
    );
}

/// The draws of a batch are pinned: the stream of `docs/13` §3.1 that recorded runs reproduce.
#[test]
fn a_batch_s_draws_are_pinned() {
    let d = DifDesign {
        delta: 0.9,
        alpha: 0.4,
        pi: 0.3,
        impact: 0.5,
        ..design(300, 6, 6, Layout::TwoAxes(2))
    };
    let batch = dif_batch(&d, 41);
    let mut h = sha2::Sha256::new();
    for row in batch.anchors.iter().chain(&batch.x) {
        for v in row {
            h.update(v.to_bits().to_le_bytes());
        }
    }
    for v in batch
        .z
        .iter()
        .chain(&batch.a)
        .chain(&batch.b)
        .chain(&batch.floors)
    {
        h.update(v.to_bits().to_le_bytes());
    }
    let digest: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
    let pin = "dcb0aad53a8927d09b07aa39f7f831ef4e94184b2e7c351bd4d0df19959b560a";
    assert_eq!(digest, pin);
}
