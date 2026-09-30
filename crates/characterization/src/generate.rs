//! The populations the studies draw (`docs/13` §3, §8.2): latent-DIF batches after the
//! paper's `dif_generate`, stage-1 pilots, the Level A mirror design and its probes, and the
//! `sim/` fixture.

use crate::grid::{draw_seed, Attack, DifDesign, ExtraDesign, Layout, ScreenDesign, SweepDesign};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use scoring::bridging::Ratings;

/// A standard normal by Box–Muller, with the engine's `libm` (`docs/13` §2).
pub fn normal(rng: &mut ChaCha8Rng) -> f64 {
    let u1: f64 = 1.0 - rng.gen::<f64>();
    let u2: f64 = rng.gen::<f64>();
    (-2.0 * libm::log(u1)).sqrt() * libm::cos(2.0 * std::f64::consts::PI * u2)
}

pub fn sigmoid(z: f64) -> f64 {
    1.0 / (1.0 + libm::exp(-z))
}

fn bit(rng: &mut ChaCha8Rng, p: f64) -> f64 {
    f64::from(rng.gen::<f64>() < p)
}

/// Per trial item, its sign on the first and on the second axis.
pub fn signs(layout: Layout, k: usize) -> (Vec<f64>, Vec<f64>) {
    let mut first = vec![0.0; k];
    let mut second = vec![0.0; k];
    match layout {
        Layout::Campaign(n) => first.iter_mut().take(n).for_each(|s| *s = 1.0),
        Layout::Mirror => {
            for (j, s) in first.iter_mut().take(4).enumerate() {
                *s = if j % 2 == 0 { 1.0 } else { -1.0 };
            }
        }
        Layout::TwoAxes(n) => {
            first.iter_mut().take(n).for_each(|s| *s = 1.0);
            second.iter_mut().skip(n).take(n).for_each(|s| *s = 1.0);
        }
    }
    (first, second)
}

/// A drawn batch; `roles` has one character per trial item (`docs/13` §3.1).
#[derive(Clone, Debug)]
pub struct DifBatch {
    pub anchors: Vec<Vec<f64>>,
    pub x: Vec<Vec<f64>>,
    /// Each respondent's class on the first axis, `±1`.
    pub z: Vec<f64>,
    pub roles: String,
    pub a: Vec<f64>,
    pub b: Vec<f64>,
    pub signs: Vec<f64>,
    /// Each trial item's guessing floor.
    pub floors: Vec<f64>,
}

/// Ability of unit variance and mean 0: a skew-normal of shape `shape`, or for shape 0 a
/// standard normal from one draw (`docs/13` §8.2).
pub fn ability(shape: f64, rng: &mut ChaCha8Rng) -> f64 {
    if shape == 0.0 {
        return normal(rng);
    }
    let delta = shape / (1.0 + shape * shape).sqrt();
    let (u0, u1) = (normal(rng), normal(rng));
    let x = delta * u0.abs() + (1.0 - delta * delta).sqrt() * u1;
    let mean = delta * (2.0 / std::f64::consts::PI).sqrt();
    (x - mean) / (1.0 - mean * mean).sqrt()
}

/// Per column, `guess`, or with a spread a draw from `U(guess − spread, guess + spread)`.
fn floors(d: &DifDesign, count: usize, rng: &mut ChaCha8Rng) -> Vec<f64> {
    if d.spread == 0.0 {
        return vec![d.guess; count];
    }
    (0..count)
        .map(|_| rng.gen_range(d.guess - d.spread..=d.guess + d.spread))
        .collect()
}

pub fn dif_batch(d: &DifDesign, seed: u64) -> DifBatch {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let (first, second) = signs(d.layout, d.k);
    let a_anchor: Vec<f64> = (0..d.anchors).map(|_| rng.gen_range(0.9..1.6)).collect();
    let b_anchor: Vec<f64> = (0..d.anchors).map(|_| normal(&mut rng)).collect();
    let a: Vec<f64> = (0..d.k).map(|_| rng.gen_range(1.0..1.5)).collect();
    let b: Vec<f64> = (0..d.k).map(|_| 0.6 * normal(&mut rng)).collect();
    let c_anchor = floors(d, d.anchors, &mut rng);
    let c = floors(d, d.k, &mut rng);
    let (fraction, targets, masked) = match d.attack {
        Attack::None => (0.0, 0, false),
        Attack::Inject { fraction, targets } => (fraction, targets.min(d.k), false),
        Attack::Mask { fraction } => (fraction, 0, true),
    };
    let attackers = ((fraction * d.n as f64).round() as usize).min(d.n);
    let targeted = |j: usize| j >= d.k - targets;
    let respond = |c: f64, p: f64| c + (1.0 - c) * p;
    let (mut anchors, mut x, mut z) = (
        Vec::with_capacity(d.n),
        Vec::with_capacity(d.n),
        Vec::with_capacity(d.n),
    );
    for i in 0..d.n {
        let z1 = if rng.gen::<f64>() < d.pi { 1.0 } else { -1.0 };
        let z2 = if rng.gen::<f64>() < 0.5 { 1.0 } else { -1.0 };
        let theta = ability(d.skew, &mut rng) + if z1 > 0.0 { d.impact } else { 0.0 };
        let bumps: Vec<f64> = if d.testlet == 0.0 {
            Vec::new()
        } else {
            (0..d.k.div_ceil(2))
                .map(|_| d.testlet * normal(&mut rng))
                .collect()
        };
        let attacker = i >= d.n - attackers;
        let row: Vec<f64> = (0..d.anchors)
            .map(|j| {
                let p = sigmoid(a_anchor[j] * (theta - b_anchor[j]));
                bit(&mut rng, respond(c_anchor[j], p))
            })
            .collect();
        anchors.push(row);
        let row: Vec<f64> = (0..d.k)
            .map(|j| {
                let lean = if attacker && masked {
                    0.0
                } else {
                    first[j] * z1 + second[j] * z2
                };
                let slope = a[j] + d.alpha / 2.0 * lean;
                let own = bumps.get(j / 2).map_or(theta, |bump| theta + bump);
                let drawn = bit(
                    &mut rng,
                    respond(c[j], sigmoid(slope * (own - b[j] - d.delta * lean))),
                );
                if attacker && targeted(j) {
                    0.0
                } else {
                    drawn
                }
            })
            .collect();
        x.push(row);
        z.push(z1);
    }
    let roles = (0..d.k)
        .map(|j| match (first[j], second[j]) {
            (s, _) if s > 0.0 => '+',
            (s, _) if s < 0.0 => '-',
            (_, s) if s != 0.0 => '2',
            _ if targeted(j) => 't',
            _ => 'c',
        })
        .collect();
    DifBatch {
        anchors,
        x,
        z,
        roles,
        a,
        b,
        signs: first,
        floors: c,
    }
}

/// The trial items of `floor-screen`, a role and `(a, b)` each: `g` good, `f` flat, `h` too
/// hard, `c` guessable, its floor [`SCREEN_GUESS`] over chance, `k` keyed backwards.
pub const SCREEN_ITEMS: [(char, f64, f64); 10] = [
    ('g', 0.8, 0.0),
    ('g', 1.2, -1.0),
    ('g', 1.2, 1.0),
    ('g', 1.6, 0.9),
    ('g', 1.2, 2.0),
    ('f', 0.3, 0.0),
    ('f', 0.45, 0.0),
    ('h', 1.2, 3.0),
    ('c', 1.2, 0.0),
    ('k', 1.2, 0.0),
];
pub const SCREEN_GUESS: f64 = 0.2;

/// A drawn stage-1 pilot: the anchors' and the trial items' answers, one role per item.
#[derive(Clone, Debug)]
pub struct ScreenBatch {
    pub anchors: Vec<Vec<f64>>,
    pub x: Vec<Vec<f64>>,
    pub roles: String,
}

pub fn screen_batch(d: &ScreenDesign, seed: u64) -> ScreenBatch {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let chance = 1.0 / f64::from(d.options);
    let a_anchor: Vec<f64> = (0..d.anchors).map(|_| rng.gen_range(0.9..1.6)).collect();
    let b_anchor: Vec<f64> = (0..d.anchors).map(|_| normal(&mut rng)).collect();
    let respond = |c: f64, p: f64| c + (1.0 - c) * p;
    let (mut anchors, mut x) = (Vec::with_capacity(d.n), Vec::with_capacity(d.n));
    for _ in 0..d.n {
        let theta = normal(&mut rng);
        let row: Vec<f64> = (0..d.anchors)
            .map(|j| {
                let p = sigmoid(a_anchor[j] * (theta - b_anchor[j]));
                bit(&mut rng, respond(chance, p))
            })
            .collect();
        anchors.push(row);
        let row: Vec<f64> = SCREEN_ITEMS
            .iter()
            .map(|&(role, a, b)| {
                let floor = if role == 'c' {
                    chance + SCREEN_GUESS
                } else {
                    chance
                };
                let right = bit(&mut rng, respond(floor, sigmoid(a * (theta - b))));
                if role == 'k' {
                    1.0 - right
                } else {
                    right
                }
            })
            .collect();
        x.push(row);
    }
    ScreenBatch {
        anchors,
        x,
        roles: SCREEN_ITEMS.iter().map(|item| item.0).collect(),
    }
}

/// A drawn mirror design: the ratings, and per item its quality `q`, its lean and its truth,
/// the mean over the two camps of the camp's expected clamped rating (`docs/13` §3.2).
#[derive(Clone, Debug)]
pub struct SweepData {
    pub ratings: Ratings,
    pub q: Vec<f64>,
    pub lean: Vec<f64>,
    pub truth: Vec<f64>,
    pub true_f: Vec<f64>,
    pub severity: Vec<f64>,
}

fn phi(x: f64) -> f64 {
    libm::exp(-0.5 * x * x) / (2.0 * std::f64::consts::PI).sqrt()
}

fn cdf(x: f64) -> f64 {
    0.5 * libm::erfc(-x / std::f64::consts::SQRT_2)
}

/// `E[clamp(X, 0, 1)]` for `X ~ N(mu, sd²)`.
pub fn expected_clamped(mu: f64, sd: f64) -> f64 {
    if sd <= 0.0 {
        return mu.clamp(0.0, 1.0);
    }
    let (a, b) = (-mu / sd, (1.0 - mu) / sd);
    mu * (cdf(b) - cdf(a)) + sd * (phi(a) - phi(b)) + (1.0 - cdf(b))
}

pub const CONSENSUS_ITEMS: usize = 10;
pub const PARTISAN_ITEMS: usize = 10;

/// The truth of an item of quality `q` and lean `lean`: the mean over the camps, the first
/// `camp_a` reviewers and the rest, of the camp's mean expected clamped rating.
fn truth_of(q: f64, lean: f64, true_f: &[f64], severity: &[f64], camp_a: usize, noise: f64) -> f64 {
    let camp = |range: std::ops::Range<usize>| -> Option<f64> {
        let expected: Vec<f64> = range
            .map(|u| expected_clamped(q + 0.45 * true_f[u] * lean + severity[u], noise))
            .collect();
        (!expected.is_empty()).then(|| expected.iter().sum::<f64>() / expected.len() as f64)
    };
    let sides: Vec<f64> = [camp(0..camp_a), camp(camp_a..true_f.len())]
        .into_iter()
        .flatten()
        .collect();
    sides.iter().sum::<f64>() / sides.len() as f64
}

pub fn sweep_data(d: &SweepDesign, seed: u64) -> SweepData {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let n_b = (d.n as f64 * d.share).round() as usize;
    let true_f: Vec<f64> = (0..d.n)
        .map(|u| if u < d.n - n_b { -1.0 } else { 1.0 } + 0.25 * normal(&mut rng))
        .collect();
    let severity: Vec<f64> = (0..d.n).map(|_| 0.06 * normal(&mut rng)).collect();
    let m = CONSENSUS_ITEMS + PARTISAN_ITEMS;
    let q: Vec<f64> = (0..m)
        .map(|j| {
            if j < CONSENSUS_ITEMS {
                rng.gen_range(0.70..0.95)
            } else {
                0.55
            }
        })
        .collect();
    let lean: Vec<f64> = (0..m)
        .map(|j| match j.checked_sub(CONSENSUS_ITEMS) {
            None => 0.0,
            Some(p) if p % 2 == 0 => 0.8,
            Some(_) => -0.8,
        })
        .collect();
    let per = d.per_reviewer.min(m);
    let mut r = vec![vec![0.0; m]; d.n];
    let mut mask = vec![vec![false; m]; d.n];
    for u in 0..d.n {
        let mut items: Vec<usize> = (0..m).collect();
        for slot in 0..per {
            let pick = slot + rng.gen_range(0..m - slot);
            items.swap(slot, pick);
            mask[u][items[slot]] = true;
        }
        for j in 0..m {
            let rating =
                q[j] + 0.45 * true_f[u] * lean[j] + severity[u] + d.noise * normal(&mut rng);
            r[u][j] = rating.clamp(0.0, 1.0);
        }
    }
    let camp_a = d.n - n_b;
    let truth = (0..m)
        .map(|j| truth_of(q[j], lean[j], &true_f, &severity, camp_a, d.noise))
        .collect();
    SweepData {
        ratings: Ratings::from_dense(&r, &mask),
        q,
        lean,
        truth,
        true_f,
        severity,
    }
}

/// The probes of `bridging-extra` and the half-width of their qualities' range around `τ`.
pub const PROBES: usize = 10;
pub const PROBE_SPREAD: f64 = 0.06;

/// A drawn extra-round design: the mirror design, and per probe, on a stream of its own, its
/// quality, its truth and every reviewer's rating of it, rated or not (`docs/13` §8.2).
#[derive(Clone, Debug)]
pub struct ExtraData {
    pub sweep: SweepData,
    pub probe_q: Vec<f64>,
    pub probe_truth: Vec<f64>,
    pub probe_ratings: Vec<Vec<f64>>,
}

pub fn extra_data(d: &ExtraDesign, tau: f64, seed: u64) -> ExtraData {
    let design = SweepDesign {
        n: d.n,
        share: d.share,
        per_reviewer: d.per_reviewer,
        noise: d.noise,
        lambda: None,
    };
    let sweep = sweep_data(&design, seed);
    let mut rng = ChaCha8Rng::seed_from_u64(draw_seed("probes", seed, 0));
    let probe_q: Vec<f64> = (0..PROBES)
        .map(|_| rng.gen_range(tau - PROBE_SPREAD..tau + PROBE_SPREAD))
        .collect();
    let probe_ratings = probe_q
        .iter()
        .map(|q| {
            (0..d.n)
                .map(|u| (q + sweep.severity[u] + d.noise * normal(&mut rng)).clamp(0.0, 1.0))
                .collect()
        })
        .collect();
    let camp_a = d.n - (d.n as f64 * d.share).round() as usize;
    let probe_truth = probe_q
        .iter()
        .map(|&q| truth_of(q, 0.0, &sweep.true_f, &sweep.severity, camp_a, d.noise))
        .collect();
    ExtraData {
        sweep,
        probe_q,
        probe_truth,
        probe_ratings,
    }
}

/// The reference simulation's Level A dataset (`sim/export_fixtures.py`).
#[derive(Clone, Debug)]
pub struct Fixture {
    pub r: Vec<Vec<f64>>,
    pub mask: Vec<Vec<bool>>,
    pub true_f: Vec<f64>,
}

/// The fixture items' leans (`paper/scripts/common.py::sim_levelA_dataset`).
pub const FIXTURE_LEAN: [f64; 10] = [0.0, 0.0, 0.75, 0.05, 0.0, 0.0, 0.0, 0.80, -0.80, 0.30];

const R_CSV: &str = include_str!("../../scoring/tests/fixtures/R.csv");
const MASK_CSV: &str = include_str!("../../scoring/tests/fixtures/mask.csv");
const TRUE_F_CSV: &str = include_str!("../../scoring/tests/fixtures/true_f.csv");

fn matrix(text: &str) -> Vec<Vec<f64>> {
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            l.split(',')
                .map(|c| c.trim().parse().expect("a number in a committed fixture"))
                .collect()
        })
        .collect()
}

pub fn fixture() -> Fixture {
    Fixture {
        r: matrix(R_CSV),
        mask: matrix(MASK_CSV)
            .iter()
            .map(|row| row.iter().map(|&v| v != 0.0).collect())
            .collect(),
        true_f: matrix(TRUE_F_CSV).iter().map(|row| row[0]).collect(),
    }
}
