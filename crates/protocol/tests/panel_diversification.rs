//! Panel diversification (`docs/01` D40, T57): a detected coordination cluster constrains
//! the assignment, not the weights (AT-BR-10) — no panel holds two of its members, and no
//! honest reviewer's weight changes.

mod common;

use identity::nym::Nym;
use protocol::orchestrator::{bridging_weights, ReviewerStanding};
use protocol::review::{
    assign_diverse, assign_diverse_from_beacon, assign_extra_diverse_from_beacon,
    assign_extra_from_beacon, assign_reviewers, Reviewer, K_EXTRA,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::HashSet;

const N: usize = 1000;
const CLUSTER: usize = 50;
const K: usize = 9;
const DRAWS: u64 = 2000;

fn nym(i: usize) -> Nym {
    let mut id = [0u8; 32];
    id[..8].copy_from_slice(&(i as u64).to_le_bytes());
    Nym(id)
}

fn normal(r: &mut ChaCha8Rng) -> f64 {
    let u1: f64 = 1.0 - r.gen::<f64>();
    let u2: f64 = r.gen();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

/// Reviewers spread on the axis; the first `CLUSTER` form one flagged cluster.
fn population() -> (Vec<Reviewer>, Vec<usize>) {
    let mut r = ChaCha8Rng::seed_from_u64(40);
    let reviewers: Vec<Reviewer> = (0..N)
        .map(|i| Reviewer {
            nym: nym(i),
            f_u: normal(&mut r),
        })
        .collect();
    let clusters: Vec<usize> = (0..N).map(|i| if i < CLUSTER { 0 } else { i }).collect();
    (reviewers, clusters)
}

fn in_cluster(panel: &[Reviewer]) -> usize {
    panel
        .iter()
        .filter(|r| (0..CLUSTER).any(|i| nym(i) == r.nym))
        .count()
}

/// AT-BR-10: no diversified panel holds two members of the flagged cluster, and every
/// panel still spans the axis.
#[test]
fn at_br_10_no_panel_holds_two_members_of_a_flagged_cluster() {
    let (reviewers, clusters) = population();
    let mut uniform_two_or_more = 0usize;
    for seed in 0..DRAWS {
        let panel = assign_diverse(&reviewers, &clusters, &[], K, seed);
        assert_eq!(panel.len(), K);
        let distinct: HashSet<Nym> = panel.iter().map(|r| r.nym).collect();
        assert_eq!(distinct.len(), K, "seed {seed}: a repeated reviewer");
        assert!(in_cluster(&panel) <= 1, "seed {seed}: two cluster members");
        assert!(
            panel.iter().any(|r| r.f_u < -0.5) && panel.iter().any(|r| r.f_u > 0.5),
            "seed {seed}: the panel does not span the axis"
        );
        if in_cluster(&assign_reviewers(&reviewers, K, seed)) >= 2 {
            uniform_two_or_more += 1;
        }
    }
    let share = uniform_two_or_more as f64 / DRAWS as f64;
    println!(
        "uniform draw: two or more cluster members on {:.1}% of panels (paper 7.0%)",
        share * 100.0
    );
    assert!((0.04..0.11).contains(&share), "uniform share {share:.3}");
}

/// The extra round respects the constraint too: outside the first panel and outside the
/// clusters of its members (T60, T57).
#[test]
fn the_extra_panel_avoids_the_first_panel_and_its_clusters() {
    let (reviewers, clusters) = population();
    let beacon = common::beacon(3, 5);
    for slot in 0..200u64 {
        let first = assign_diverse_from_beacon(&reviewers, &clusters, K, &beacon, slot);
        assert!(in_cluster(&first) <= 1);
        let first_nyms: Vec<Nym> = first.iter().map(|r| r.nym).collect();
        let extra = assign_extra_diverse_from_beacon(
            &reviewers,
            &clusters,
            &first_nyms,
            K_EXTRA,
            &beacon,
            slot,
        );
        assert_eq!(extra.len(), K_EXTRA);
        for r in &extra {
            assert!(
                !first_nyms.contains(&r.nym),
                "slot {slot}: an extra reviewer from the first panel"
            );
        }
        assert_eq!(
            in_cluster(&first) + in_cluster(&extra),
            in_cluster(&first).min(1).max(in_cluster(&extra)),
            "slot {slot}: the cluster is on both rounds"
        );
        assert!(in_cluster(&first) + in_cluster(&extra) <= 1);
    }
}

/// A stratum the constraint empties is filled from the nearest eligible reviewers; with
/// singleton clusters the draw is the plain stratified one.
#[test]
fn an_emptied_stratum_is_filled_from_the_axis_and_singletons_change_nothing() {
    let reviewers: Vec<Reviewer> = (0..12)
        .map(|i| Reviewer {
            nym: nym(i),
            f_u: i as f64 - 6.0,
        })
        .collect();
    let clusters: Vec<usize> = (0..12).map(|i| if i < 4 { 0 } else { i }).collect();
    let panel = assign_diverse(&reviewers, &clusters, &[], 9, 3);
    assert_eq!(panel.len(), 9);
    let from_cluster = panel
        .iter()
        .filter(|r| (0..4).any(|i| nym(i) == r.nym))
        .count();
    assert!(
        from_cluster <= 1,
        "{from_cluster} members of the four-reviewer cluster"
    );
    let distinct: HashSet<Nym> = panel.iter().map(|r| r.nym).collect();
    assert_eq!(distinct.len(), 9);
    let singletons: Vec<usize> = (0..12).collect();
    let plain: Vec<Nym> = assign_reviewers(&reviewers, 9, 3)
        .iter()
        .map(|r| r.nym)
        .collect();
    let diverse: Vec<Nym> = assign_diverse(&reviewers, &singletons, &[], 9, 3)
        .iter()
        .map(|r| r.nym)
        .collect();
    assert_eq!(plain, diverse);
    // Mismatched inputs: no panel.
    assert!(assign_diverse(&reviewers, &clusters[..5], &[], 9, 3).is_empty());
}

/// AT-BR-10, the other half: weights take no cluster input, so equal standings weigh
/// the same and no sublinear discount applies (D40).
#[test]
fn no_reviewer_s_weight_changes_with_the_clusters() {
    let standings = vec![ReviewerStanding::established(0.02); 4];
    let w = bridging_weights(&standings, 3.0);
    assert!(w.iter().all(|&x| (x - w[0]).abs() < 1e-12 && x > 1.0));
}

/// The beacon's panels are full: the diverse first panel holds `K`, and the band's plain extra
/// panel `K_EXTRA` reviewers, none of them on the first panel (D26, T57).
#[test]
fn the_beacon_panels_are_full_and_the_extra_one_comes_from_outside_the_first() {
    let (reviewers, clusters) = population();
    let beacon = common::beacon(3, 5);
    for slot in 0..50u64 {
        let first = assign_diverse_from_beacon(&reviewers, &clusters, K, &beacon, slot);
        assert_eq!(first.len(), K);
        let first_nyms: Vec<Nym> = first.iter().map(|r| r.nym).collect();
        let extra = assign_extra_from_beacon(&reviewers, &first_nyms, K_EXTRA, &beacon, slot);
        assert_eq!(extra.len(), K_EXTRA);
        assert!(
            extra.iter().all(|r| !first_nyms.contains(&r.nym)),
            "slot {slot}"
        );
    }
}

/// A reviewer `nym(i)` at each position `f[i]` on the axis.
fn line(f: &[f64]) -> Vec<Reviewer> {
    f.iter()
        .enumerate()
        .map(|(i, &f_u)| Reviewer { nym: nym(i), f_u })
        .collect()
}

/// AT-BR-10: with no cluster in the way the draw takes one reviewer per stratum, and over
/// seeds every member of every stratum.
#[test]
fn at_br_10_the_draw_takes_one_per_stratum_and_each_member_in_turn() {
    let f: Vec<f64> = (0..6).map(f64::from).collect();
    let reviewers = line(&f);
    let singletons: Vec<usize> = (0..6).collect();
    let mut drawn = HashSet::new();
    for seed in 0..64 {
        let panel: Vec<usize> = assign_diverse(&reviewers, &singletons, &[], 3, seed)
            .iter()
            .map(|r| r.f_u as usize)
            .collect();
        assert_eq!(panel.iter().map(|p| p / 2).collect::<Vec<_>>(), [0, 1, 2]);
        drawn.extend(panel);
    }
    assert_eq!(drawn.len(), 6);
}

/// AT-BR-10: a stratum with no eligible member takes the eligible reviewer nearest its
/// centre, `f = 5` here: the one at 3.2, or at 7.9 when the first stratum drew it.
#[test]
fn at_br_10_an_emptied_stratum_takes_the_reviewer_nearest_its_centre() {
    let f = [0.0, 1.0, 2.0, 3.2, 4.0, 5.0, 6.0, 7.0, 7.9, 9.0, 10.0, 11.0];
    let reviewers = line(&f);
    let clusters: Vec<usize> = (0..12)
        .map(|i| if (4..8).contains(&i) { 5 } else { i })
        .collect();
    let at = |r: &Reviewer| f.iter().position(|&x| x == r.f_u).unwrap();
    let mut first = HashSet::new();
    for seed in 0..32 {
        let panel = assign_diverse(&reviewers, &clusters, &[nym(5)], 3, seed);
        let (p0, p1) = (at(&panel[0]), at(&panel[1]));
        assert_eq!(p1, if p0 == 3 { 8 } else { 3 }, "seed {seed}");
        first.insert(p0);
    }
    assert!(first.len() > 1);
}
