//! The guessing floor of `docs/01` D25 in the latent re-check (`docs/02` §B.1, §B.3):
//! a batch that guesses is not read as a mixture, and its leaners keep their gap (AT-DIF-13).

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use scoring::dif::MIXTURE_DIF_MAX;
use scoring::latent::{latent_dif_with, Formats, LatentDif, LatentParams};
use std::time::Instant;

const K: usize = 8;

fn normal(r: &mut ChaCha8Rng) -> f64 {
    let u1: f64 = 1.0 - r.gen::<f64>();
    let u2: f64 = r.gen();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

fn sigmoid(z: f64) -> f64 {
    1.0 / (1.0 + (-z).exp())
}

/// `docs/13` §3.1's batch with a floor drawn per anchor and item from `floor`, the first
/// `n_leaning` items leaning `δ·z` on a hidden axis. Returns (anchors, responses).
fn batch(
    n: usize,
    n_anchor: usize,
    n_leaning: usize,
    delta: f64,
    floor: (f64, f64),
    seed: u64,
) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let draw_floor = |rng: &mut ChaCha8Rng| {
        if floor.0 < floor.1 {
            rng.gen_range(floor.0..floor.1)
        } else {
            floor.0
        }
    };
    let a_anchor: Vec<f64> = (0..n_anchor).map(|_| rng.gen_range(0.9..1.6)).collect();
    let b_anchor: Vec<f64> = (0..n_anchor).map(|_| normal(&mut rng)).collect();
    let c_anchor: Vec<f64> = (0..n_anchor).map(|_| draw_floor(&mut rng)).collect();
    let a: Vec<f64> = (0..K).map(|_| rng.gen_range(1.0..1.5)).collect();
    let b: Vec<f64> = (0..K).map(|_| 0.6 * normal(&mut rng)).collect();
    let c: Vec<f64> = (0..K).map(|_| draw_floor(&mut rng)).collect();
    let (mut anchors, mut x) = (Vec::with_capacity(n), Vec::with_capacity(n));
    for _ in 0..n {
        let z = if rng.gen::<bool>() { 1.0 } else { -1.0 };
        let theta = normal(&mut rng);
        let answer =
            |rng: &mut ChaCha8Rng, c: f64, p: f64| f64::from(rng.gen::<f64>() < c + (1.0 - c) * p);
        anchors.push(
            (0..n_anchor)
                .map(|j| {
                    let p = sigmoid(a_anchor[j] * (theta - b_anchor[j]));
                    answer(&mut rng, c_anchor[j], p)
                })
                .collect(),
        );
        x.push(
            (0..K)
                .map(|j| {
                    let lean = if j < n_leaning { delta * z } else { 0.0 };
                    answer(&mut rng, c[j], sigmoid(a[j] * (theta - b[j] - lean)))
                })
                .collect(),
        );
    }
    (anchors, x)
}

fn rounded(v: &[f64]) -> Vec<f64> {
    v.iter().map(|x| (x * 100.0).round() / 100.0).collect()
}

fn fit(anchors: &[Vec<f64>], x: &[Vec<f64>], formats: &Formats, what: &str) -> LatentDif {
    let t0 = Instant::now();
    let res = latent_dif_with(anchors, x, formats, &LatentParams::default()).unwrap();
    println!(
        "{what}: {} class(es), bic gain {:+.1}, gaps {:?}, floors {:?}, {:?}, {:.1}s",
        res.classes,
        res.bic_gain,
        rounded(&res.dif),
        rounded(&res.item_c),
        res.status,
        t0.elapsed().as_secs_f64()
    );
    res
}

fn assert_null(res: &LatentDif, what: &str) {
    assert_eq!(res.classes, 1, "{what}: a mixture, gaps {:?}", res.dif);
    assert!(res.flags(MIXTURE_DIF_MAX).iter().all(|&f| !f), "{what}");
}

/// AT-DIF-13: a null batch with a floor of 0.2 on every item, declared five options, is one class.
#[test]
fn at_dif_13_a_null_batch_that_guesses_is_not_a_mixture() {
    let (anchors, x) = batch(3000, 60, 0, 0.0, (0.2, 0.2), 2400);
    let res = fit(&anchors, &x, &Formats::choice(60, K, 5), "floor 0.2");
    assert_null(&res, "floor 0.2");
    for c in &res.item_c {
        assert!((0.1..0.3).contains(c), "floors {:?}", res.item_c);
    }
}

/// AT-DIF-13: three leaners at δ = 0.9 that guess are flagged alone, no gap under 1.2 (true 1.8).
#[test]
fn at_dif_13_leaners_that_guess_keep_their_gap() {
    let (anchors, x) = batch(3000, 60, 3, 0.9, (0.2, 0.2), 2410);
    let res = fit(&anchors, &x, &Formats::choice(60, K, 5), "three leaners");
    let expected: Vec<bool> = (0..K).map(|j| j < 3).collect();
    assert_eq!(res.flags(MIXTURE_DIF_MAX), expected, "gaps {:?}", res.dif);
    assert!(
        res.dif[..3].iter().all(|&gap| gap >= 1.2),
        "gaps {:?}",
        res.dif
    );
}

/// AT-DIF-13: two more null seeds, true/false items and floors that vary around the declared one.
#[cfg(feature = "calibration")]
#[test]
fn at_dif_13_null_batches_that_guess_are_not_mixtures() {
    for seed in [2401, 2402] {
        let (anchors, x) = batch(3000, 60, 0, 0.0, (0.2, 0.2), seed);
        let what = format!("floor 0.2, seed {seed}");
        assert_null(&fit(&anchors, &x, &Formats::choice(60, K, 5), &what), &what);
    }
    let (anchors, x) = batch(3000, 60, 0, 0.0, (0.5, 0.5), 2403);
    assert_null(
        &fit(&anchors, &x, &Formats::choice(60, K, 2), "true/false"),
        "true/false",
    );
    let (anchors, x) = batch(3000, 60, 0, 0.0, (0.1, 0.3), 2404);
    let varied = fit(&anchors, &x, &Formats::choice(60, K, 5), "floors 0.1–0.3");
    assert_null(&varied, "floors 0.1–0.3");
}

/// AT-DIF-13: the same guessing batch declared open is read as a mixture — the defect D25 fixes.
#[cfg(feature = "calibration")]
#[test]
fn at_dif_13_a_batch_that_guesses_declared_open_is_read_as_a_mixture() {
    let (anchors, x) = batch(3000, 60, 0, 0.0, (0.2, 0.2), 2400);
    let res = fit(&anchors, &x, &Formats::open(60, K), "declared open");
    assert!(res.classes >= 2, "gaps {:?}", res.dif);
    assert!(res.flags(MIXTURE_DIF_MAX).iter().any(|&f| f));
}
