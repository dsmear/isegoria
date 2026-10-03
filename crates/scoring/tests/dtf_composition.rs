//! A2 diagnostics (`docs/19`): what a fit's DTF measures, on hand-built curves and on one real
//! fit; each test pins a limit of the per-fit statistic, not a frequency.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use scoring::dif::MIXTURE_DIF_MAX;
use scoring::dtf::{ClassCurves, DTF_MAX};
use scoring::latent::{latent_dif_with, Ability, Formats, LatentParams};
use scoring::Convergence;

/// The standard normal on the latent grid, 41 nodes over `[−5, 5]`.
fn normal() -> Ability {
    let nodes: Vec<f64> = (0..41).map(|q| -5.0 + 0.25 * q as f64).collect();
    let mass: Vec<f64> = nodes.iter().map(|u| (-0.5 * u * u).exp()).collect();
    let total: f64 = mass.iter().sum();
    Ability {
        nodes,
        weights: mass.iter().map(|m| m / total).collect(),
    }
}

/// Two equal classes of normal ability; each item `(a, b, lean, c)` at `b ∓ lean`.
fn two_classes(items: &[(f64, f64, f64, f64)]) -> ClassCurves {
    let a = vec![items.iter().map(|it| it.0).collect::<Vec<f64>>(); 2];
    let b = vec![
        items.iter().map(|it| it.1 - it.2).collect(),
        items.iter().map(|it| it.1 + it.2).collect(),
    ];
    let c: Vec<f64> = items.iter().map(|it| it.3).collect();
    ClassCurves::with_ability(&[0.5, 0.5], &[0.0, 0.0], &normal(), &a, &b, &c).unwrap()
}

/// A2: one item's DTF is that of the population it is integrated over, not of the item alone.
#[test]
fn one_item_s_dtf_depends_on_the_population_it_is_integrated_over() {
    let weak = two_classes(&[(1.25, 3.0, 0.9, 0.0)]).dtf(&[0]).unwrap();
    let able = two_classes(&[(1.25, 1.0, 0.9, 0.0)]).dtf(&[0]).unwrap();
    println!("difficulty 3 (a weak batch): {weak:.5}; difficulty 1 (two units abler): {able:.5}");
    assert!(weak < DTF_MAX && able > 3.0 * DTF_MAX);
}

/// A2 (`docs/15`): an item whose class gap is under the flag cut can exceed the tolerance alone.
#[test]
fn an_item_under_the_flag_cut_can_exceed_the_tolerance() {
    let lean = 0.45;
    assert!(2.0 * lean < MIXTURE_DIF_MAX);
    let dtf = two_classes(&[(1.25, 0.0, lean, 0.2)]).dtf(&[0]).unwrap();
    println!("gap {:.2}, floor 0.2: DTF {dtf:.10}", 2.0 * lean);
    assert!((dtf - 0.1698619472).abs() < 1e-9 && dtf > DTF_MAX);
}

/// `golden.rs`'s open batch (seed 54): 1,500 respondents, 20 anchors, 8 items, the first two
/// at `b + δz`, `z = ±1`. Returns (anchors, responses, the items' `a`, `b`).
#[allow(clippy::type_complexity)]
fn golden_batch(delta: f64) -> (Vec<Vec<f64>>, Vec<Vec<f64>>, Vec<f64>, Vec<f64>) {
    let (n, na, k) = (1500, 20, 8);
    let mut rng = ChaCha8Rng::seed_from_u64(54);
    let normal = |rng: &mut ChaCha8Rng| {
        let u1: f64 = 1.0 - rng.gen::<f64>();
        let u2: f64 = rng.gen();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    };
    let sigmoid = |z: f64| 1.0 / (1.0 + (-z).exp());
    let theta: Vec<f64> = (0..n).map(|_| normal(&mut rng)).collect();
    let z: Vec<f64> = (0..n)
        .map(|_| if rng.gen::<bool>() { 1.0 } else { -1.0 })
        .collect();
    let a_anchor: Vec<f64> = (0..na).map(|_| rng.gen_range(0.9..1.6)).collect();
    let b_anchor: Vec<f64> = (0..na).map(|_| normal(&mut rng)).collect();
    let anchors = theta
        .iter()
        .map(|&t| {
            (0..na)
                .map(|j| f64::from(rng.gen::<f64>() < sigmoid(a_anchor[j] * (t - b_anchor[j]))))
                .collect()
        })
        .collect();
    let a: Vec<f64> = (0..k).map(|_| rng.gen_range(1.0..1.5)).collect();
    let b: Vec<f64> = (0..k).map(|_| 0.6 * normal(&mut rng)).collect();
    let responses = theta
        .iter()
        .zip(&z)
        .map(|(&t, &zi)| {
            (0..k)
                .map(|j| {
                    let d = if j < 2 { delta } else { 0.0 };
                    f64::from(rng.gen::<f64>() < sigmoid(a[j] * (t - b[j] - d * zi)))
                })
                .collect()
        })
        .collect();
    (anchors, responses, a, b)
}

/// A2: a converged fit that selects one class reads 0 for two items whose true curves lean.
#[test]
fn a_one_class_fit_reads_zero_where_two_items_lean() {
    let delta = 0.9;
    let (anchors, x, a, b) = golden_batch(delta);
    let lp = LatentParams {
        n_starts: 2,
        max_classes: 2,
        ..LatentParams::default()
    };
    let fit = latent_dif_with(&anchors, &x, &Formats::open(20, 8), &lp).unwrap();
    assert_eq!((fit.status, fit.classes), (Convergence::Converged, 1));
    let fitted = ClassCurves::of(&fit).unwrap().dtf(&[0, 1]).unwrap();
    let truth = two_classes(&[(a[0], b[0], delta, 0.0), (a[1], b[1], delta, 0.0)]);
    let true_dtf = truth.dtf(&[0, 1]).unwrap();
    println!("fitted DTF of the two leaning items {fitted}, of their true curves {true_dtf:.5}");
    assert!(fitted == 0.0 && true_dtf > DTF_MAX);
}
