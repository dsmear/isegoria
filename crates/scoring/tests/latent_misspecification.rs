//! The ability histogram of `docs/01` D43 in the latent re-check (`docs/02` §B.1, §B.3): a
//! skewed ability is one class, its shape estimated, and its leaners keep their gap (AT-DIF-14).

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use scoring::dif::MIXTURE_DIF_MAX;
use scoring::latent::{latent_dif_with, Formats, LatentDif, LatentParams};
use std::time::Instant;

const K: usize = 8;
const ANCHORS: usize = 60;
const FLOOR: f64 = 0.2;

fn normal(r: &mut ChaCha8Rng) -> f64 {
    let u1: f64 = 1.0 - r.gen::<f64>();
    let u2: f64 = r.gen();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

fn sigmoid(z: f64) -> f64 {
    1.0 / (1.0 + (-z).exp())
}

/// A skew-normal of shape `shape`, standardized to mean 0 and variance 1.
fn skewed(shape: f64, r: &mut ChaCha8Rng) -> f64 {
    let delta = shape / (1.0 + shape * shape).sqrt();
    let (u0, u1) = (normal(r), normal(r));
    let x = delta * u0.abs() + (1.0 - delta * delta).sqrt() * u1;
    let mean = delta * (2.0 / std::f64::consts::PI).sqrt();
    (x - mean) / (1.0 - mean * mean).sqrt()
}

/// `docs/13` §3.1's batch at a floor of 0.2 on every column, the ability skewed by `shape`, the
/// first `n_leaning` items leaning `δ·z` on a hidden axis. Returns (anchors, responses).
fn batch(
    n: usize,
    n_leaning: usize,
    delta: f64,
    shape: f64,
    seed: u64,
) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let a_anchor: Vec<f64> = (0..ANCHORS).map(|_| rng.gen_range(0.9..1.6)).collect();
    let b_anchor: Vec<f64> = (0..ANCHORS).map(|_| normal(&mut rng)).collect();
    let a: Vec<f64> = (0..K).map(|_| rng.gen_range(1.0..1.5)).collect();
    let b: Vec<f64> = (0..K).map(|_| 0.6 * normal(&mut rng)).collect();
    let answer =
        |rng: &mut ChaCha8Rng, p: f64| f64::from(rng.gen::<f64>() < FLOOR + (1.0 - FLOOR) * p);
    let (mut anchors, mut x) = (Vec::with_capacity(n), Vec::with_capacity(n));
    for _ in 0..n {
        let z = if rng.gen::<bool>() { 1.0 } else { -1.0 };
        let theta = skewed(shape, &mut rng);
        anchors.push(
            (0..ANCHORS)
                .map(|j| answer(&mut rng, sigmoid(a_anchor[j] * (theta - b_anchor[j]))))
                .collect(),
        );
        x.push(
            (0..K)
                .map(|j| {
                    let lean = if j < n_leaning { delta * z } else { 0.0 };
                    answer(&mut rng, sigmoid(a[j] * (theta - b[j] - lean)))
                })
                .collect(),
        );
    }
    (anchors, x)
}

fn fit(anchors: &[Vec<f64>], x: &[Vec<f64>], what: &str) -> LatentDif {
    let t0 = Instant::now();
    let formats = Formats::choice(ANCHORS, K, 5);
    let res = latent_dif_with(anchors, x, &formats, &LatentParams::default()).unwrap();
    let (mean, var, skew) = moments(&res);
    println!(
        "{what}: {} class(es), bic gain {:+.1}, gaps {:?}, ability mean {mean:.3} var {var:.3} \
         skew {skew:.2}, {:?}, {:.1}s",
        res.classes,
        res.bic_gain,
        res.dif
            .iter()
            .map(|d| (d * 100.0).round() / 100.0)
            .collect::<Vec<_>>(),
        res.status,
        t0.elapsed().as_secs_f64()
    );
    res
}

/// The fitted ability's mean, variance and skewness.
fn moments(res: &LatentDif) -> (f64, f64, f64) {
    let pts = res.ability.nodes.iter().zip(&res.ability.weights);
    let mean: f64 = pts.clone().map(|(u, w)| u * w).sum();
    let var: f64 = pts.clone().map(|(u, w)| w * (u - mean).powi(2)).sum();
    let third: f64 = pts.map(|(u, w)| w * (u - mean).powi(3)).sum();
    (mean, var, third / var.powf(1.5))
}

fn assert_null(res: &LatentDif, what: &str) {
    assert_eq!(res.classes, 1, "{what}: a mixture, gaps {:?}", res.dif);
    assert!(res.flags(MIXTURE_DIF_MAX).iter().all(|&f| !f), "{what}");
}

/// The skewness of the standardized skew-normal of shape −4.
const SKEW_OF_SHAPE_MINUS_4: f64 = -0.784;

/// AT-DIF-14: a null batch whose ability is skewed (shape −4) is one class, its shape recovered.
#[test]
fn at_dif_14_a_skewed_null_batch_is_one_class() {
    let (anchors, x) = batch(3000, 0, 0.0, -4.0, 2450);
    let res = fit(&anchors, &x, "skew −4");
    assert_null(&res, "skew −4");
    let (mean, var, skew) = moments(&res);
    assert!(
        mean.abs() < 1e-9 && (var - 1.0).abs() < 1e-9,
        "mean {mean}, var {var}"
    );
    assert!(
        (skew - SKEW_OF_SHAPE_MINUS_4).abs() < 0.3,
        "skewness {skew}"
    );
}

/// AT-DIF-14: with its shape held, a skewed batch's ability is read as the normal.
#[test]
fn at_dif_14_a_held_shape_reads_the_normal() {
    let (anchors, x) = batch(1000, 0, 0.0, -4.0, 2450);
    let held = LatentParams {
        max_classes: 1,
        estimate_shape: false,
        ..LatentParams::default()
    };
    let res = latent_dif_with(&anchors, &x, &Formats::choice(ANCHORS, K, 5), &held).unwrap();
    let (mean, var, skew) = moments(&res);
    assert!(
        mean.abs() < 1e-9 && (var - 1.0).abs() < 1e-9 && skew.abs() < 1e-9,
        "mean {mean}, var {var}, skewness {skew}"
    );
}
