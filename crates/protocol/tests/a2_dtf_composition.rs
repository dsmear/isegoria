//! A2 diagnostics (`docs/19`): the contested pool's cost sums per-fit DTFs, each over its own
//! batch's population; a mathematical construction run through the real pool, not a frequency.

use network::cid::cid;
use protocol::contested::ContestedPool;
use scoring::dtf::{ClassCurves, DTF_MAX};
use scoring::latent::Ability;

fn normal() -> Ability {
    let nodes: Vec<f64> = (0..41).map(|q| -5.0 + 0.25 * q as f64).collect();
    let mass: Vec<f64> = nodes.iter().map(|u| (-0.5 * u * u).exp()).collect();
    let total: f64 = mass.iter().sum();
    Ability {
        nodes,
        weights: mass.iter().map(|m| m / total).collect(),
    }
}

/// Two equal classes of normal ability; items of slope 1.25 at `b ∓ 0.9`, one `b` per item.
fn leaning(bs: &[f64]) -> ClassCurves {
    let a = vec![vec![1.25; bs.len()]; 2];
    let b = vec![
        bs.iter().map(|b| b - 0.9).collect(),
        bs.iter().map(|b| b + 0.9).collect(),
    ];
    ClassCurves::with_ability(
        &[0.5, 0.5],
        &[0.0, 0.0],
        &normal(),
        &a,
        &b,
        &vec![0.0; bs.len()],
    )
    .unwrap()
}

/// A2: two facts hard for their own batches pass the cost; an abler common population fails them.
#[test]
fn per_batch_dtfs_can_admit_a_test_its_population_would_refuse() {
    let (one, two) = (cid(b"fact one"), cid(b"fact two"));
    let mut pool = ContestedPool::new();
    pool.record(leaning(&[4.0]), &[(one, 0)]).unwrap();
    pool.record(leaning(&[4.0]), &[(two, 0)]).unwrap();
    let cost = pool.dtf(&[one, two]).unwrap();
    let drawn = pool.draw(2, DTF_MAX, 0).unwrap();
    let common = leaning(&[2.0, 2.0]).dtf(&[0, 1]).unwrap();
    println!("pool cost {cost:.5}; the same two facts on a population two units abler {common:.5}");
    assert!(cost <= DTF_MAX && drawn.len() == 2 && common > 4.0 * DTF_MAX);
}
