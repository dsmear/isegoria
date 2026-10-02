//! The sides of the bridge score (D42, T71): the exact split of `f_u` with a floor on each
//! side, and predictions on the rating scale (`docs/02` §A.3, `docs/08` AT-BR-11).

use proptest::prelude::*;
use scoring::bridging::{
    coverage, side_balanced, side_floor, two_means, Fit, Ratings, Side, SideScores,
};
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

/// A one-item fit, `μ + b_j` = 0.75 and `f_j` = 0.125, its reviewers at `f_u`.
fn placed(f_u: &[f64], axis: &[bool]) -> Fit {
    Fit {
        mu: 0.5,
        b_u: vec![0.0; f_u.len()],
        b_j: vec![0.25],
        f_u: f_u.to_vec(),
        f_j: vec![0.125],
        participant: axis.to_vec(),
        status: Convergence::Converged,
    }
}

/// Of tied cuts, the one nearest the middle wins, then the lower (AT-BR-11).
#[test]
fn at_br_11_a_tie_goes_to_the_cut_nearest_the_middle_then_the_lower() {
    assert_eq!(sizes(&two_means(&[0.0, 1.0, 2.0])), (1, 2));
    let f: Vec<f64> = [(-7.5, 8), (-1.0, 5), (1.0, 5), (7.5, 8)]
        .iter()
        .flat_map(|&(x, k)| std::iter::repeat_n(x, k))
        .collect();
    assert_eq!(sizes(&two_means(&f)), (13, 13));
}

/// A reviewer off the axis takes the side whose centre is nearer (AT-BR-11).
#[test]
fn at_br_11_a_reviewer_off_the_axis_takes_the_nearer_centre() {
    let mut f = vec![-2.0; 3];
    f.extend([2.0; 7]);
    f.extend([-0.2, 0.2, 1.9, -1.9]);
    let axis: Vec<bool> = (0..14).map(|u| u < 10).collect();
    let side = side_balanced(&placed(&f, &axis)).side;
    assert_eq!(side[10..], [Side::A, Side::B, Side::B, Side::A]);
}

/// With the axis at one position there is one side, and the score is its mean (AT-BR-11).
#[test]
fn at_br_11_one_side_scores_its_own_mean() {
    let s = side_balanced(&placed(
        &[1.0, 1.0, 1.0, 1.0, 0.9],
        &[true, true, true, true, false],
    ));
    assert_eq!(s.side, vec![Side::A; 5]);
    assert_eq!(
        (s.side_a[0], s.side_b[0], s.score[0], s.gap[0]),
        (0.875, 0.875, 0.875, 0.0)
    );
}

/// With no reviewer on the axis the score is the item's level, `μ + b_j` (AT-BR-11).
#[test]
fn at_br_11_no_axis_scores_the_item_level() {
    let s = side_balanced(&placed(&[0.5, -0.5], &[false, false]));
    assert_eq!(
        (s.side_a[0], s.side_b[0], s.score[0], s.gap[0]),
        (0.75, 0.75, 0.75, 0.0)
    );
}

/// A reviewer of weight 0 neither counts in an item's coverage nor seats a side (AT-BR-11, A7).
#[test]
fn at_br_11_a_reviewer_of_weight_zero_covers_nothing() {
    let data = Ratings::from_dense(&[vec![0.5], vec![0.5]], &[vec![true], vec![true]])
        .with_weights(vec![1.0, 0.0]);
    let sides = SideScores {
        side: vec![Side::A, Side::B],
        side_a: vec![0.5],
        side_b: vec![0.5],
        score: vec![0.5],
        gap: vec![0.0],
    };
    assert_eq!(coverage(&data, &sides), vec![1]);
}

/// A side's floor is 5% of the reviewers rounded up, at least 1, at most half (AT-BR-11).
#[test]
fn at_br_11_the_side_floor_by_hand() {
    let table = [
        (0, 0),
        (1, 0),
        (2, 1),
        (3, 1),
        (20, 1),
        (21, 2),
        (40, 2),
        (41, 3),
        (800, 40),
    ];
    for (n, want) in table {
        assert_eq!(side_floor(n), want, "n = {n}");
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
        participant: vec![true; 4],
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
