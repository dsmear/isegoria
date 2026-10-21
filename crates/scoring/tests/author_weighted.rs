//! The answer review's author index (`docs/02` §C.1, `docs/01` D50): observation weights.

use scoring::reputation::{author_score_closings, author_score_weighted, AuthorPrior};

const H: u64 = 45;

fn time_weight(age: u64) -> f64 {
    (-(age as f64) * std::f64::consts::LN_2 / H as f64).exp()
}

// AT-ANS-08: unit weights give the closings index bit for bit.
#[test]
fn at_ans_08_unit_weights_are_the_closings_index() {
    let prior = AuthorPrior::default();
    let qualities = [1.0, 0.0, 1.0, 1.0, 0.0];
    let ages = [1, 3, 7, 45, 200];
    let unit = [1.0; 5];
    for n in 0..=qualities.len() {
        let weighted = author_score_weighted(&qualities[..n], &unit[..n], &ages[..n], H, &prior);
        let closings = author_score_closings(&qualities[..n], &ages[..n], H, &prior);
        assert_eq!(weighted.to_bits(), closings.to_bits());
    }
    assert_eq!(author_score_weighted(&[], &[], &[], H, &prior), 0.4);
}

// AT-ANS-08: a weight multiplies the time weight in both sums (hand-computed values).
#[test]
fn at_ans_08_the_weight_enters_numerator_and_denominator() {
    let prior = AuthorPrior::default();
    let w1 = time_weight(1);
    let divided = author_score_weighted(&[0.0], &[0.25], &[1], H, &prior);
    assert!((divided - 2.0 / (5.0 + 0.25 * w1)).abs() < 1e-15);
    let discarded = author_score_weighted(&[0.0], &[1.0], &[1], H, &prior);
    assert!((discarded - 2.0 / (5.0 + w1)).abs() < 1e-15);
    assert!(discarded < divided && divided < 0.4);
    let mixed = author_score_weighted(&[1.0, 0.0], &[1.0, 0.25], &[1, 2], H, &prior);
    let w2 = time_weight(2);
    assert!((mixed - (2.0 + w1) / (5.0 + w1 + 0.25 * w2)).abs() < 1e-15);
    let accepted = author_score_weighted(&[1.0], &[1.0], &[0], H, &prior);
    assert_eq!(accepted, 0.5);
}

// AT-ANS-08: observation lengths must agree.
#[test]
#[should_panic]
fn at_ans_08_mismatched_lengths_panic() {
    author_score_weighted(&[1.0], &[], &[0], H, &AuthorPrior::default());
}
