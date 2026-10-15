//! D3's isolated measurement (`docs/15` D3): one `ContestedPool::draw` on one synthetic fit of
//! `M` contested members leaning both ways, `G` equal classes, a 41-node normal ability.
//! Usage: `d3_draw_cost M n G`; prints the subsets the enumeration visits and the elapsed time.

use network::cid::cid;
use protocol::contested::ContestedPool;
use scoring::dtf::{ClassCurves, DTF_MAX};
use scoring::latent::Ability;
use std::time::Instant;

fn binomial(m: u128, r: u128) -> u128 {
    (0..r).fold(1, |acc, i| acc * (m - i) / (i + 1))
}

fn normal() -> Ability {
    let nodes: Vec<f64> = (0..41).map(|q| -5.0 + 0.25 * f64::from(q)).collect();
    let mass: Vec<f64> = nodes.iter().map(|u| (-0.5 * u * u).exp()).collect();
    let total: f64 = mass.iter().sum();
    Ability {
        nodes,
        weights: mass.iter().map(|m| m / total).collect(),
    }
}

/// `M` items of slope 1.25, difficulties evenly spaced on `[−1, 1]`, leaning ±0.9 in turn;
/// class `g` sits at `b + lean·[−1, 1, 0, 0][g]`.
fn curves(m: usize, g: usize) -> ClassCurves {
    let shift = [-1.0, 1.0, 0.0, 0.0];
    let b: Vec<f64> = (0..m)
        .map(|j| -1.0 + 2.0 * j as f64 / (m.max(2) - 1) as f64)
        .collect();
    let lean = |j: usize| if j % 2 == 0 { 0.9 } else { -0.9 };
    let a = vec![vec![1.25; m]; g];
    let bg = (0..g)
        .map(|c| (0..m).map(|j| b[j] + lean(j) * shift[c]).collect())
        .collect::<Vec<Vec<f64>>>();
    let pi = vec![1.0 / g as f64; g];
    ClassCurves::with_ability(&pi, &vec![0.0; g], &normal(), &a, &bg, &vec![0.0; m])
        .expect("well-formed classes")
}

fn main() {
    let args: Vec<usize> = std::env::args()
        .skip(1)
        .filter_map(|a| a.parse().ok())
        .collect();
    let [m, n, g] = args[..] else {
        eprintln!("usage: d3_draw_cost M n G  (G from 2 to 4)");
        std::process::exit(2);
    };
    assert!(
        (2..=4).contains(&g) && m >= 1,
        "G from 2 to 4, M at least 1"
    );
    let members: Vec<_> = (0..m)
        .map(|j| (cid(format!("d3-{j}").as_bytes()), j))
        .collect();
    let mut pool = ContestedPool::new();
    pool.record(curves(m, g), &members).expect("members fit");
    let top = m.min(n) as u128;
    let subsets: u128 = (0..=top).map(|r| binomial(m as u128, r)).sum();
    let evaluations: u128 = (0..=top).map(|r| r * binomial(m as u128, r)).sum();
    let start = Instant::now();
    let outcome = pool.draw(n, DTF_MAX, 0);
    let seconds = start.elapsed().as_secs_f64();
    let drawn = match &outcome {
        Ok(test) => format!("drew {}", test.len()),
        Err(e) => format!("no balanced draw; feasible sizes {:?}", e.feasible),
    };
    println!(
        "M={m} n={n} G={g} subsets={subsets} item_evaluations={evaluations} seconds={seconds:.3} \
         ns_per_subset={:.1} ns_per_item_evaluation={:.2} {drawn}",
        seconds * 1e9 / subsets as f64,
        seconds * 1e9 / evaluations.max(1) as f64,
    );
}
