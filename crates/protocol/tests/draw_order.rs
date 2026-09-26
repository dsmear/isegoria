//! The draws read their candidates as a set (`docs/08` PROTO-003, INV-10, T72): the same
//! candidates listed in any order draw the same panel or committee from the same seed.

mod common;

use identity::nym::Nym;
use protocol::governance::{stratified_sortition, Candidate};
use protocol::review::{
    assign_diverse, assign_extra_diverse_from_beacon, assign_extra_from_beacon, assign_reviewers,
    Reviewer,
};

/// 30 reviewers spread on the axis, 20 newcomers tied at the origin, and a tie at 0.5; each
/// with a cluster (pairs of neighbours share one).
fn population() -> Vec<(Reviewer, usize)> {
    let mut pool: Vec<(Reviewer, usize)> = (0..30u8)
        .map(|i| {
            let f_u = if i == 29 {
                0.5
            } else {
                -1.0 + 2.0 * f64::from(i) / 29.0 + 0.001
            };
            (
                Reviewer {
                    nym: Nym([i; 32]),
                    f_u,
                },
                usize::from(i / 2),
            )
        })
        .collect();
    pool.extend((30..50u8).map(|i| {
        (
            Reviewer {
                nym: Nym([i; 32]),
                f_u: 0.0,
            },
            usize::from(i),
        )
    }));
    pool.push((
        Reviewer {
            nym: Nym([50; 32]),
            f_u: 0.5,
        },
        50,
    ));
    pool
}

/// The same list in five orders: as built, reversed, rotated, interleaved, by nym descending.
fn orders<T: Clone>(base: &[T]) -> Vec<Vec<T>> {
    let mut reversed = base.to_vec();
    reversed.reverse();
    let mut rotated = base.to_vec();
    rotated.rotate_left(base.len() / 3);
    let interleaved: Vec<T> = base
        .iter()
        .step_by(2)
        .chain(base.iter().skip(1).step_by(2))
        .cloned()
        .collect();
    let mut shuffled = base.to_vec();
    shuffled.swap(0, base.len() - 1);
    shuffled.rotate_right(7);
    vec![reversed, rotated, interleaved, shuffled]
}

fn nyms(panel: &[Reviewer]) -> Vec<Nym> {
    panel.iter().map(|r| r.nym).collect()
}

fn split(list: &[(Reviewer, usize)]) -> (Vec<Reviewer>, Vec<usize>) {
    list.iter().copied().unzip()
}

/// AT-BR-12: the stratified panel is the same for the candidates in any order.
#[test]
fn at_br_12_the_panel_does_not_depend_on_the_order_of_the_candidates() {
    let (base, _) = split(&population());
    for seed in 0..100u64 {
        let reference = nyms(&assign_reviewers(&base, 9, seed));
        for order in orders(&base) {
            assert_eq!(
                nyms(&assign_reviewers(&order, 9, seed)),
                reference,
                "seed {seed}"
            );
        }
    }
}

/// AT-BR-12: the diversified panel (D40) is the same for the candidates in any order.
#[test]
fn at_br_12_the_diverse_panel_does_not_depend_on_the_order_of_the_candidates() {
    let pool = population();
    let (base, clusters) = split(&pool);
    let taken = [Nym([3; 32]), Nym([40; 32])];
    for seed in 0..100u64 {
        let reference = nyms(&assign_diverse(&base, &clusters, &taken, 9, seed));
        for order in orders(&pool) {
            let (r, c) = split(&order);
            assert_eq!(
                nyms(&assign_diverse(&r, &c, &taken, 9, seed)),
                reference,
                "seed {seed}"
            );
        }
    }
}

/// AT-BR-12: the band's extra round, plain and diversified, reads its candidates as a set.
#[test]
fn at_br_12_the_extra_round_does_not_depend_on_the_order_of_the_candidates() {
    let pool = population();
    let (base, clusters) = split(&pool);
    let first: Vec<Nym> = (0..9u8).map(|i| Nym([i * 5; 32])).collect();
    for epoch in 0..30u64 {
        let beacon = common::beacon(2, epoch);
        let plain = nyms(&assign_extra_from_beacon(&base, &first, 4, &beacon, 1));
        let diverse = nyms(&assign_extra_diverse_from_beacon(
            &base, &clusters, &first, 4, &beacon, 1,
        ));
        for order in orders(&pool) {
            let (r, c) = split(&order);
            assert_eq!(
                nyms(&assign_extra_from_beacon(&r, &first, 4, &beacon, 1)),
                plain
            );
            assert_eq!(
                nyms(&assign_extra_diverse_from_beacon(
                    &r, &c, &first, 4, &beacon, 1
                )),
                diverse
            );
        }
    }
}

/// AT-BR-12: sortition draws the same committee from the candidates in any order, with ties
/// on the axis and, without ties, when small strata leave seats to fill.
#[test]
fn at_br_12_sortition_does_not_depend_on_the_order_of_the_candidates() {
    let tied: Vec<Candidate<u32>> = (0..40u32)
        .map(|i| Candidate {
            id: i,
            f_u: if i < 25 { f64::from(i) } else { 0.0 },
        })
        .collect();
    let distinct: Vec<Candidate<u32>> = (0..8u32)
        .map(|i| Candidate {
            id: i * 7 % 8,
            f_u: f64::from(i),
        })
        .collect();
    for (candidates, seats, strata) in [(&tied, 9, 3), (&distinct, 7, 5)] {
        for seed in 0..100u64 {
            let reference = stratified_sortition(candidates, seats, strata, seed).unwrap();
            for order in orders(candidates) {
                assert_eq!(
                    stratified_sortition(&order, seats, strata, seed).unwrap(),
                    reference,
                    "seed {seed}, {seats} seats in {strata} strata"
                );
            }
        }
    }
}
