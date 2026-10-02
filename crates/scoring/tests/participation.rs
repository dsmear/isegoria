//! Who takes part in the collective bridging computation (`docs/02` §A.4, `docs/15` A7):
//! a row with `axis && weight > 0`. The others change no common parameter, side, score,
//! gap, coverage or bootstrap minimum, bit for bit; only their own projected position.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use scoring::bridging::{
    bridge_scores, coverage, fit, side_balanced, BridgeScores, BridgingParams, Fit, Ratings, Side,
    SideScores,
};

const M: usize = 10;
const BOOTSTRAP: usize = 6;
const KEEP: f64 = 0.8;

fn normal(r: &mut ChaCha8Rng) -> f64 {
    let u1: f64 = 1.0 - r.gen::<f64>();
    let u2: f64 = r.gen();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

/// Two camps of `n` reviewers on `M` items, four leaning each way; about one cell in six missing.
fn two_camps(n: usize, seed: u64) -> (Vec<Vec<f64>>, Vec<Vec<bool>>) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let lean = |j: usize| match j % 5 {
        0 | 1 => 0.4,
        2 | 3 => -0.4,
        _ => 0.0,
    };
    let mut r = Vec::new();
    let mut mask = Vec::new();
    for u in 0..n {
        let f = if u < n / 2 { -1.0 } else { 1.0 } + 0.2 * normal(&mut rng);
        r.push(
            (0..M)
                .map(|j| (0.75 + f * lean(j) + 0.08 * normal(&mut rng)).clamp(0.0, 1.0))
                .collect(),
        );
        mask.push((0..M).map(|_| rng.gen::<f64>() > 0.15).collect());
    }
    (r, mask)
}

/// A row that takes no part: its ratings, its axis flag and its weight.
#[derive(Clone)]
struct Inactive {
    at: usize,
    ratings: Vec<f64>,
    axis: bool,
    weight: f64,
}

/// One row of a test's ratings; `participant` is its index among the participants, if any.
struct Row {
    ratings: Vec<f64>,
    mask: Vec<bool>,
    axis: bool,
    weight: f64,
    participant: Option<usize>,
}

/// The participants' ratings with `inactive` rows inserted at their positions; returns the
/// ratings and, per participant, its row.
fn with_inactive(
    r: &[Vec<f64>],
    mask: &[Vec<bool>],
    inactive: &[Inactive],
) -> (Ratings, Vec<usize>) {
    let mut rows: Vec<Row> = r
        .iter()
        .zip(mask)
        .enumerate()
        .map(|(u, (row, m))| Row {
            ratings: row.clone(),
            mask: m.clone(),
            axis: true,
            weight: 1.0,
            participant: Some(u),
        })
        .collect();
    let mut sorted = inactive.to_vec();
    sorted.sort_by_key(|i| i.at);
    for i in sorted {
        let at = i.at.min(rows.len());
        rows.insert(
            at,
            Row {
                ratings: i.ratings,
                mask: vec![true; M],
                axis: i.axis,
                weight: i.weight,
                participant: None,
            },
        );
    }
    let dense: Vec<Vec<f64>> = rows.iter().map(|x| x.ratings.clone()).collect();
    let masks: Vec<Vec<bool>> = rows.iter().map(|x| x.mask.clone()).collect();
    let data = Ratings::from_dense(&dense, &masks)
        .with_axis(rows.iter().map(|x| x.axis).collect())
        .with_weights(rows.iter().map(|x| x.weight).collect());
    let mut of = vec![0; r.len()];
    for (row, x) in rows.iter().enumerate() {
        if let Some(u) = x.participant {
            of[u] = row;
        }
    }
    (data, of)
}

fn bits(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

/// The collective results of a fit and its bridge scores, participants read at their rows.
fn collective(f: &Fit, s: &SideScores, b: &BridgeScores, rows: &[usize]) -> Vec<Vec<u64>> {
    let at = |v: &[f64]| rows.iter().map(|&u| v[u].to_bits()).collect::<Vec<_>>();
    let sides: Vec<u64> = rows.iter().map(|&u| s.side[u] as u64).collect();
    vec![
        vec![f.mu.to_bits()],
        bits(&f.b_j),
        bits(&f.f_j),
        at(&f.b_u),
        at(&f.f_u),
        sides,
        bits(&s.score),
        bits(&s.gap),
        bits(&b.robust),
        b.coverage.iter().map(|&c| c as u64).collect(),
    ]
}

fn run(data: &Ratings, rows: &[usize]) -> Vec<Vec<u64>> {
    let p = BridgingParams::default();
    let f = fit(data, &p).unwrap();
    let s = side_balanced(&f);
    let b = bridge_scores(data, &p, BOOTSTRAP, KEEP).unwrap();
    assert_eq!(
        bits(&b.full.score),
        bits(&s.score),
        "the fit and the bridge scores agree"
    );
    collective(&f, &s, &b, rows)
}

fn extreme(at: usize, value: f64, axis: bool, weight: f64) -> Inactive {
    Inactive {
        at,
        ratings: vec![value; M],
        axis,
        weight,
    }
}

/// A7: an axis row of weight 0 changes nothing collective, added, removed or voting otherwise.
#[test]
fn an_axis_row_of_weight_zero_takes_no_part() {
    let (r, mask) = two_camps(40, 39);
    let reference = run(
        &with_inactive(&r, &mask, &[]).0,
        &(0..40).collect::<Vec<_>>(),
    );
    for value in [0.0, 1.0] {
        let (data, rows) = with_inactive(&r, &mask, &[extreme(25, value, true, 0.0)]);
        assert_eq!(run(&data, &rows), reference, "rating {value}");
    }
}

/// A7: an off-axis row of positive weight takes no part, in the full fit or the bootstrap.
#[test]
fn an_off_axis_row_of_positive_weight_takes_no_part() {
    let (r, mask) = two_camps(40, 39);
    let reference = run(
        &with_inactive(&r, &mask, &[]).0,
        &(0..40).collect::<Vec<_>>(),
    );
    for value in [0.0, 1.0] {
        let (data, rows) = with_inactive(&r, &mask, &[extreme(25, value, false, 2.0)]);
        assert_eq!(run(&data, &rows), reference, "rating {value}");
    }
}

/// A7: both kinds together, first, among and after the participants, leave the results bit for bit.
#[test]
fn inactive_rows_anywhere_leave_the_collective_results() {
    let (r, mask) = two_camps(40, 39);
    let reference = run(
        &with_inactive(&r, &mask, &[]).0,
        &(0..40).collect::<Vec<_>>(),
    );
    let inactive = [
        extreme(0, 0.0, true, 0.0),
        extreme(7, 1.0, false, 1.5),
        extreme(20, 0.0, false, 0.4),
        extreme(33, 1.0, true, 0.0),
        extreme(99, 0.0, true, 0.0),
    ];
    let (data, rows) = with_inactive(&r, &mask, &inactive);
    assert_eq!(data.n, 45);
    assert_eq!(run(&data, &rows), reference);
}

/// A7: the bootstrap is active here, and participants' votes still move the scores.
#[test]
fn participants_still_decide() {
    let (r, mask) = two_camps(40, 39);
    let p = BridgingParams::default();
    let b = bridge_scores(&with_inactive(&r, &mask, &[]).0, &p, BOOTSTRAP, KEEP).unwrap();
    assert!(
        b.robust.iter().zip(&b.full.score).any(|(r, s)| r < s),
        "a bootstrap minimum"
    );
    let mut harsh = r.clone();
    for row in harsh.iter_mut().take(10) {
        row[4] = 0.0;
    }
    let moved = bridge_scores(&with_inactive(&harsh, &mask, &[]).0, &p, BOOTSTRAP, KEEP).unwrap();
    assert!(
        moved.full.score[4] < b.full.score[4] - 0.01,
        "{} vs {}",
        moved.full.score[4],
        b.full.score[4]
    );
}

fn sides(side: Vec<Side>) -> SideScores {
    SideScores {
        side,
        side_a: vec![0.5],
        side_b: vec![0.5],
        score: vec![0.5],
        gap: vec![0.0],
    }
}

/// A7, D42: a side held only by inactive rows is no side; a participants' side with no rating is.
#[test]
fn coverage_seats_only_participants() {
    let one_item = |w: Vec<f64>, axis: Vec<bool>, rated: Vec<bool>| {
        let r: Vec<Vec<f64>> = rated.iter().map(|_| vec![0.5]).collect();
        let mask: Vec<Vec<bool>> = rated.iter().map(|&k| vec![k]).collect();
        Ratings::from_dense(&r, &mask)
            .with_weights(w)
            .with_axis(axis)
    };
    let ab = sides(vec![Side::A, Side::B]);
    let zero_weight_b = one_item(vec![1.0, 0.0], vec![true, true], vec![true, true]);
    assert_eq!(coverage(&zero_weight_b, &ab), vec![1]);
    let off_axis_b = one_item(vec![1.0, 2.0], vec![true, false], vec![true, true]);
    assert_eq!(coverage(&off_axis_b, &ab), vec![1]);
    let silent_b = one_item(vec![1.0, 1.0], vec![true, true], vec![true, false]);
    assert_eq!(coverage(&silent_b, &ab), vec![0]);
    let abb = sides(vec![Side::A, Side::B, Side::B]);
    let silent_b_but_inactive =
        one_item(vec![1.0, 1.0, 0.0], vec![true; 3], vec![true, false, true]);
    assert_eq!(coverage(&silent_b_but_inactive, &abb), vec![0]);
}

/// A7: no participant, one, coincident positions and empty replicas: finite, and no coverage
/// without a participant.
#[test]
fn the_edge_cases_are_finite() {
    let (r, mask) = two_camps(6, 3);
    let p = BridgingParams::default();
    let finite = |b: &BridgeScores| {
        b.robust
            .iter()
            .chain(&b.full.score)
            .chain(&b.full.gap)
            .all(|v| v.is_finite())
    };
    let base = Ratings::from_dense(&r, &mask);
    for (w, axis) in [
        (vec![0.0; 6], vec![true; 6]),
        (vec![1.0; 6], vec![false; 6]),
    ] {
        let none = base.clone().with_weights(w).with_axis(axis);
        let b = bridge_scores(&none, &p, 3, KEEP).unwrap();
        assert!(finite(&b));
        assert_eq!(b.coverage, vec![0; M]);
    }
    let mut w = vec![0.0; 6];
    w[2] = 1.0;
    let one = base.clone().with_weights(w);
    let b = bridge_scores(&one, &p, 3, KEEP).unwrap();
    assert!(finite(&b));
    let alike = Ratings::from_dense(&vec![vec![0.6; M]; 6], &vec![vec![true; M]; 6]);
    assert!(finite(&bridge_scores(&alike, &p, 3, KEEP).unwrap()));
    let empty_replicas = bridge_scores(&base, &p, 3, 0.0).unwrap();
    assert!(finite(&empty_replicas));
    assert_eq!(
        bits(&empty_replicas.robust),
        bits(&bridge_scores(&base, &p, 3, 0.0).unwrap().robust)
    );
}

/// A7: an extra row's own ratings may move only its projected position.
#[test]
fn an_inactive_row_keeps_a_position_of_its_own() {
    let (r, mask) = two_camps(40, 39);
    let p = BridgingParams::default();
    let (low, _) = with_inactive(&r, &mask, &[extreme(40, 0.0, true, 0.0)]);
    let (high, _) = with_inactive(&r, &mask, &[extreme(40, 1.0, true, 0.0)]);
    let (a, b) = (fit(&low, &p).unwrap(), fit(&high, &p).unwrap());
    assert!(a.b_u[40].is_finite() && b.b_u[40].is_finite());
    assert_ne!(a.b_u[40], b.b_u[40]);
}
