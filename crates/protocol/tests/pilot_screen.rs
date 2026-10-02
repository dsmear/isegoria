//! The pilot's stage-1 screen on the target model (`docs/02` §B.2, `docs/01` D25, T25 step 2):
//! each threshold at its boundary, and an item the fit cannot model left out of it.

use protocol::pilot::{
    stage1_fit, stage1_screen, stage1_verdicts, PilotError, Screening, Stage1Fit,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use scoring::irt::{A_MIN, B_ABS_MAX, C_EXCESS_MAX, R_PBIS_MIN};
use scoring::latent::{Format, Formats};
use scoring::Convergence;

/// A stage-1 fit whose items have the `(a, b, c, r_pbis)` given.
fn fit(items: &[(f64, f64, f64, f64)], status: Convergence) -> Stage1Fit {
    Stage1Fit {
        status,
        a: items.iter().map(|it| it.0).collect(),
        b: items.iter().map(|it| it.1).collect(),
        c: items.iter().map(|it| it.2).collect(),
        rpb: items.iter().map(|it| it.3).collect(),
    }
}

/// AT-PRO-15: `a ≥ A_MIN`, `|b| ≤ B_ABS_MAX`, the floor over chance and `r_pbis`, at the edges.
#[test]
fn at_pro_15_each_threshold_holds_at_its_boundary() {
    let ceiling = 1.0 / 4.0 + C_EXCESS_MAX;
    let items = [
        (A_MIN, 0.0, 0.25, 0.5),
        (A_MIN - 1e-9, 0.0, 0.25, 0.5),
        (1.2, B_ABS_MAX, 0.25, 0.5),
        (1.2, -B_ABS_MAX - 1e-9, 0.25, 0.5),
        (1.2, 0.0, ceiling, 0.5),
        (1.2, 0.0, ceiling + 1e-9, 0.5),
        (1.2, 0.0, 0.0, 0.5),
        (1.2, 0.0, 1e-9, 0.5),
        (1.2, 0.0, 0.25, R_PBIS_MIN),
        (1.2, 0.0, 0.25, R_PBIS_MIN - 1e-9),
    ];
    let mut formats = Formats::choice(2, items.len(), 4);
    formats.items[6] = Format::Open;
    formats.items[7] = Format::Open;
    let got = stage1_verdicts(&fit(&items, Convergence::Converged), &formats);
    let edges = [Screening::Pass, Screening::Fail];
    assert_eq!(got, edges.repeat(5));
}

/// AT-PRO-15, A11: a fit that did not converge decides no item, however good its parameters.
#[test]
fn at_pro_15_a_fit_that_did_not_converge_decides_nothing() {
    let items = [(1.2, 0.0, 0.25, 0.5), (0.1, 0.0, 0.25, 0.1)];
    let formats = Formats::choice(2, 2, 4);
    for (status, reading) in [
        (
            Convergence::Converged,
            vec![Screening::Pass, Screening::Fail],
        ),
        (Convergence::MaxIters, vec![Screening::Indeterminate; 2]),
        (
            Convergence::LineSearchFailed,
            vec![Screening::Indeterminate; 2],
        ),
    ] {
        let got = stage1_verdicts(&fit(&items, status), &formats);
        assert_eq!(got, reading, "{status:?}");
    }
}

/// Eight respondents' two anchors, their totals 0, 0, 1, 1, 2, 2, 1, 1.
fn anchors() -> Vec<Vec<f64>> {
    [
        [0, 0],
        [0, 0],
        [1, 0],
        [0, 1],
        [1, 1],
        [1, 1],
        [1, 0],
        [0, 1],
    ]
    .iter()
    .map(|row| row.iter().map(|&v| f64::from(v)).collect())
    .collect()
}

/// An item right when the total is at least 1, and one uncorrelated with the total.
const FOLLOWS: [f64; 8] = [0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
const FLAT: [f64; 8] = [1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0];

fn rows(columns: &[[f64; 8]]) -> Vec<Vec<f64>> {
    (0..8)
        .map(|i| columns.iter().map(|c| c[i]).collect())
        .collect()
}

/// Formats that do not describe the pilot are refused, an item left out of the fit included.
#[test]
fn formats_that_do_not_describe_the_pilot_are_refused() {
    let data = rows(&[FOLLOWS, FLAT]);
    let mut one_option = Formats::choice(2, 2, 4);
    one_option.items[1] = Format::Choice(1);
    for formats in [
        Formats::choice(1, 2, 4),
        Formats::choice(2, 3, 4),
        one_option,
    ] {
        assert_eq!(
            stage1_screen(&anchors(), &data, &formats),
            Err(PilotError::BadFormats)
        );
    }
}

fn normal(r: &mut ChaCha8Rng) -> f64 {
    let u1: f64 = 1.0 - r.gen::<f64>();
    let u2: f64 = r.gen();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

/// `n` respondents of normal ability answer `na` anchors and the `items`' `(a, b)`, every column
/// at the floor of five options, the last item keyed backwards. Returns (anchors, responses).
fn pilot(n: usize, na: usize, items: &[(f64, f64)], seed: u64) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let a: Vec<f64> = (0..na).map(|_| rng.gen_range(0.9..1.6)).collect();
    let b: Vec<f64> = (0..na).map(|_| normal(&mut rng)).collect();
    let answer = |rng: &mut ChaCha8Rng, a: f64, b: f64, theta: f64| {
        f64::from(rng.gen::<f64>() < 0.2 + 0.8 / (1.0 + (-a * (theta - b)).exp()))
    };
    let (mut anchors, mut x) = (Vec::with_capacity(n), Vec::with_capacity(n));
    for _ in 0..n {
        let theta = normal(&mut rng);
        anchors.push(
            (0..na)
                .map(|j| answer(&mut rng, a[j], b[j], theta))
                .collect(),
        );
        let mut row: Vec<f64> = items
            .iter()
            .map(|&(a, b)| answer(&mut rng, a, b, theta))
            .collect();
        if let Some(last) = row.last_mut() {
            *last = 1.0 - *last;
        }
        x.push(row);
    }
    (anchors, x)
}

/// AT-PRO-15: an item keyed backwards is left out of the fit, and the others are screened.
#[test]
fn at_pro_15_an_item_keyed_backwards_does_not_stop_the_screen() {
    let items = [(1.2, -0.5), (1.2, 0.0), (1.2, 0.5), (1.2, 0.0)];
    let (anchors, x) = pilot(300, 20, &items, 0);
    let formats = Formats::choice(20, 4, 5);
    let got = stage1_screen(&anchors, &x, &formats);
    let pass = Screening::Pass;
    assert_eq!(got, Ok(vec![pass, pass, pass, Screening::Fail]));
    let fit = stage1_fit(&anchors, &x, &formats).unwrap();
    assert!(fit.rpb[3] < 0.0 && fit.rpb[..3].iter().all(|&r| r >= R_PBIS_MIN));
    assert!(fit.a[3].is_nan() && fit.b[3].is_nan() && fit.c[3].is_nan());
}

/// AT-PRO-15: at 300 respondents stage 1 holds the ability normal and keeps every good item.
#[test]
fn at_pro_15_stage_1_holds_the_ability_normal() {
    let items = [(1.2, -0.5), (1.2, 0.0), (1.2, 0.5), (1.6, 0.9), (1.2, 0.0)];
    let (anchors, x) = pilot(300, 20, &items, 7);
    let got = stage1_screen(&anchors, &x, &Formats::choice(20, 5, 5));
    let pass = Screening::Pass;
    assert_eq!(got, Ok(vec![pass, pass, pass, pass, Screening::Fail]));
}
