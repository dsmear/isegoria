//! The sides of the bridge score (D42, T71): the exact split of `f_u` with a floor on each
//! side, and predictions on the rating scale (`docs/02` §A.3, `docs/08` AT-BR-11).

use proptest::prelude::*;
use scoring::bridging::{side_balanced, two_means, Fit, Side};
use scoring::Convergence;

fn floor(n: usize) -> usize {
    (n * 50).div_ceil(1000).max(1).min(n / 2)
}

fn camps(outliers: &[f64]) -> Vec<f64> {
    let a = (0..40).map(|i| -1.2 + 0.4 * i as f64 / 39.0);
    let b = (0..60).map(|i| 0.8 + 0.4 * i as f64 / 59.0);
    a.chain(b).chain(outliers.iter().copied()).collect()
}

fn sizes(side: &[Side]) -> (usize, usize) {
    let a = side.iter().filter(|s| **s == Side::A).count();
    (a, side.len() - a)
}

/// AT-BR-11: two or five reviewers far out on the axis join their camp's side, not one of theirs.
#[test]
fn at_br_11_a_few_outlying_reviewers_do_not_form_a_side() {
    for outliers in [vec![4.0, 4.1], vec![4.0; 5]] {
        let f = camps(&outliers);
        let side = two_means(&f);
        assert!(
            side[..40].iter().all(|s| *s == Side::A),
            "{:?}",
            sizes(&side)
        );
        assert!(
            side[40..].iter().all(|s| *s == Side::B),
            "{:?}",
            sizes(&side)
        );
    }
}

/// AT-BR-11: predictions are clipped to [0, 1] before the side means: the score stays on the scale.
#[test]
fn at_br_11_predictions_are_clipped_to_the_rating_scale() {
    let fit = Fit {
        mu: 0.9,
        b_u: vec![0.2, 0.1, -0.1, -0.2],
        b_j: vec![0.3, -1.2, 0.0],
        f_u: vec![-1.0, -0.9, 0.9, 1.0],
        f_j: vec![0.0, 0.0, 0.5],
        axis: vec![true; 4],
        status: Convergence::Converged,
    };
    let s = side_balanced(&fit);
    assert_eq!((s.score[0], s.score[1]), (1.0, 0.0));
    let want_b = ((0.9_f64 - 0.1 + 0.45).min(1.0) + (0.9_f64 - 0.2 + 0.5).min(1.0)) / 2.0;
    assert!(
        (s.side_b[2] - want_b).abs() < 1e-12,
        "{} vs {want_b}",
        s.side_b[2]
    );
    for v in s.score.iter().chain(&s.side_a).chain(&s.side_b) {
        assert!((0.0..=1.0).contains(v), "{v}");
    }
}

fn positions() -> impl Strategy<Value = Vec<f64>> {
    prop::collection::vec(
        prop_oneof![8 => -2.0f64..2.0, 1 => 5.0f64..40.0, 1 => -40.0f64..-5.0],
        2..120,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// AT-BR-11: whenever there are two sides, each holds at least 5% of the reviewers.
    #[test]
    fn each_side_holds_at_least_the_floor(f in positions()) {
        let (a, b) = sizes(&two_means(&f));
        prop_assert!(a == 0 || b == 0 || (a >= floor(f.len()) && b >= floor(f.len())), "{a}/{b}");
    }

    /// AT-BR-11: the split ignores the axis' origin and scale, and a sign flip swaps the sides.
    #[test]
    fn the_split_is_invariant_under_the_axis_gauge(f in positions(), c in -3.0f64..3.0, k in 0.25f64..4.0) {
        let side = two_means(&f);
        let moved: Vec<f64> = f.iter().map(|x| x * k + c).collect();
        let flipped: Vec<f64> = f.iter().map(|x| -x).collect();
        prop_assert_eq!(&two_means(&moved), &side);
        let swapped: Vec<Side> = two_means(&flipped)
            .iter()
            .map(|s| if *s == Side::A { Side::B } else { Side::A })
            .collect();
        let (a, b) = sizes(&side);
        prop_assert!(a == 0 || b == 0 || swapped == side);
    }
}
