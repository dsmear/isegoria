//! The candidates' diagnostic path (`docs/19` §13.14) on small batches: the returned fit is the
//! ordinary one bit for bit, and each candidate agrees with the search and its selection rule.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use scoring::latent::{
    latent_dif_candidates, latent_dif_with, Formats, LatentDif, LatentParams, Verdict,
};
use scoring::Convergence;

/// `n` respondents, `na` anchors, and items `(a, b, lean)` at `b − lean·z`, `z = ±1`.
fn batch(
    seed: u64,
    n: usize,
    na: usize,
    items: &[(f64, f64, f64)],
) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let normal = |rng: &mut ChaCha8Rng| {
        let u1: f64 = 1.0 - rng.gen::<f64>();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * rng.gen::<f64>()).cos()
    };
    let aa: Vec<f64> = (0..na).map(|_| rng.gen_range(0.9..1.6)).collect();
    let ba: Vec<f64> = (0..na).map(|_| normal(&mut rng)).collect();
    let sig = |z: f64| 1.0 / (1.0 + (-z).exp());
    let (mut anchors, mut x) = (Vec::new(), Vec::new());
    for _ in 0..n {
        let (t, z) = (normal(&mut rng), if rng.gen::<bool>() { 1.0 } else { -1.0 });
        let mut bit = |p: f64| f64::from(rng.gen::<f64>() < p);
        anchors.push((0..na).map(|j| bit(sig(aa[j] * (t - ba[j])))).collect());
        x.push(
            items
                .iter()
                .map(|&(a, b, l)| bit(sig(a * (t - b + l * z))))
                .collect(),
        );
    }
    (anchors, x)
}

fn bits(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

fn rows(m: &[Vec<f64>]) -> Vec<Vec<u64>> {
    m.iter().map(|r| bits(r)).collect()
}

/// Every field of two fits, the floats compared by their bits.
fn same(a: &LatentDif, b: &LatentDif) -> bool {
    let cand = |f: &LatentDif| -> Vec<(usize, bool, u64)> {
        f.candidates
            .iter()
            .map(|&(g, s, c)| (g, s, c.to_bits()))
            .collect()
    };
    (a.classes, a.non_uniform, a.status) == (b.classes, b.non_uniform, b.status)
        && bits(&a.pi) == bits(&b.pi)
        && bits(&a.eta) == bits(&b.eta)
        && bits(&a.ability.nodes) == bits(&b.ability.nodes)
        && bits(&a.ability.weights) == bits(&b.ability.weights)
        && bits(&a.anchor_a) == bits(&b.anchor_a)
        && bits(&a.anchor_b) == bits(&b.anchor_b)
        && bits(&a.anchor_c) == bits(&b.anchor_c)
        && rows(&a.item_a) == rows(&b.item_a)
        && rows(&a.item_b) == rows(&b.item_b)
        && bits(&a.item_c) == bits(&b.item_c)
        && bits(&a.dif) == bits(&b.dif)
        && bits(&a.a_gap) == bits(&b.a_gap)
        && rows(&a.posterior) == rows(&b.posterior)
        && a.bic_gain.to_bits() == b.bic_gain.to_bits()
        && cand(a) == cand(b)
}

type Case = (&'static str, Vec<Vec<f64>>, Vec<Vec<f64>>, Formats);

fn cases() -> Vec<Case> {
    let clean = [
        (1.2, -0.5, 0.0),
        (1.3, 0.0, 0.0),
        (1.1, 0.4, 0.0),
        (1.25, 0.8, 0.0),
    ];
    let leaning = [
        (1.2, -0.5, 1.2),
        (1.3, 0.0, 1.2),
        (1.1, 0.4, 0.0),
        (1.25, 0.8, 0.0),
    ];
    let (a0, x0) = batch(3, 400, 10, &clean);
    let (a1, x1) = batch(4, 600, 10, &leaning);
    let (a2, x2) = batch(5, 400, 10, &leaning);
    vec![
        ("null, open", a0.clone(), x0.clone(), Formats::open(10, 4)),
        ("leaning, open", a1, x1, Formats::open(10, 4)),
        ("leaning, open, smaller", a2, x2, Formats::open(10, 4)),
        ("null, four options", a0, x0, Formats::choice(10, 4, 4)),
    ]
}

/// The diagnostic path returns the ordinary path's fit bit for bit, on four small batches.
#[test]
fn the_diagnostic_path_returns_the_ordinary_fit_bit_for_bit() {
    for (name, anchors, x, formats) in cases() {
        let lp = LatentParams {
            seed: 11,
            ..LatentParams::default()
        };
        let ordinary = latent_dif_with(&anchors, &x, &formats, &lp).unwrap();
        let (diagnosed, seen) = latent_dif_candidates(&anchors, &x, &formats, &lp).unwrap();
        assert!(same(&ordinary, &diagnosed), "{name}");
        assert_eq!(seen.len(), ordinary.candidates.len(), "{name}");
    }
}

/// Each candidate is the search's, in its order, with its BIC; one is selected, the returned fit;
/// a mixture is excluded only as unconverged or not lowering the provisional best.
#[test]
fn each_candidate_agrees_with_the_search_and_its_rule() {
    let mut mixtures_selected = 0;
    for (name, anchors, x, formats) in cases() {
        let lp = LatentParams {
            seed: 11,
            ..LatentParams::default()
        };
        let (fit, seen) = latent_dif_candidates(&anchors, &x, &formats, &lp).unwrap();
        let ln_n = (x.len() as f64).ln();
        let mut best = f64::INFINITY;
        for (c, &(g, s, b)) in seen.iter().zip(&fit.candidates) {
            assert_eq!(
                (c.classes, c.non_uniform, c.bic.to_bits()),
                (g, s, b.to_bits()),
                "{name}"
            );
            let recomputed = -2.0 * c.log_lik + c.params as f64 * ln_n;
            assert!(
                (recomputed - c.bic).abs() < 1e-6 * c.bic.abs().max(1.0),
                "{name}"
            );
            assert_eq!(c.fit.classes, c.classes, "{name}");
            assert!(
                c.fit.posterior.is_empty() && c.fit.candidates.is_empty(),
                "{name}"
            );
            let provisional =
                c.classes == 1 || (c.status == Convergence::Converged && c.bic < best);
            match c.verdict {
                Verdict::Selected | Verdict::Superseded => assert!(provisional, "{name}"),
                Verdict::NotConverged => assert_ne!(c.status, Convergence::Converged, "{name}"),
                Verdict::BicNotLower => {
                    assert!(
                        c.status == Convergence::Converged && c.bic >= best,
                        "{name}"
                    )
                }
            }
            if provisional {
                best = c.bic;
            }
        }
        assert_eq!(seen[0].classes, 1, "{name}");
        let selected: Vec<_> = seen
            .iter()
            .filter(|c| c.verdict == Verdict::Selected)
            .collect();
        assert_eq!(selected.len(), 1, "{name}");
        let chosen = selected[0];
        assert_eq!(
            (chosen.classes, chosen.non_uniform, chosen.status),
            (fit.classes, fit.non_uniform, fit.status)
        );
        assert_eq!(bits(&chosen.fit.pi), bits(&fit.pi), "{name}");
        assert_eq!(rows(&chosen.fit.item_b), rows(&fit.item_b), "{name}");
        assert_eq!(
            chosen.fit.bic_gain.to_bits(),
            fit.bic_gain.to_bits(),
            "{name}"
        );
        mixtures_selected += usize::from(fit.classes >= 2);
    }
    assert!(mixtures_selected >= 1, "no small batch selected a mixture");
}
