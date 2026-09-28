//! The first pass's evidence against the side-balanced score (D42, T71), re-run from its
//! seeds: a partisan pass, scores off the rating scale, a robust score split off one reviewer.

use characterization::grid::{seed, Cell, Study};
use characterization::run::{capture, sweep};
use protocol::gate::{EPS, TAU};

fn sweep_run(key: &str, replicate: u32) -> characterization::run::SweepOutcome {
    let Some(Cell::Sweep(d)) = Cell::parse(Study::BridgingSweep, key) else {
        panic!("{key}")
    };
    sweep(&d, seed(Study::BridgingSweep, key, replicate))
}

/// AT-BR-12: the partisan item no minority reviewer rated (n = 100, replicate 15) does not pass.
#[test]
fn the_partisan_item_without_minority_ratings_does_not_pass() {
    let o = sweep_run("n=100 s=0.8 r=5 e=0.15", 15);
    let passed: Vec<usize> = (0..o.lean.len())
        .filter(|&j| o.lean[j] != 0.0 && o.gate.as_bytes()[j] == b'P')
        .collect();
    assert!(
        passed.is_empty(),
        "partisan items passed: {passed:?}, gates {}",
        o.gate
    );
}

/// AT-BR-11: the n = 800 replicates whose split isolated 2 and 5 reviewers score within [0, 1].
#[test]
fn the_scores_stay_on_the_rating_scale() {
    for replicate in [8, 12] {
        let o = sweep_run("n=800 s=0.8 r=5 e=0.15", replicate);
        for (j, s) in o.full.iter().enumerate() {
            assert!(
                (0.0..=1.0).contains(s),
                "replicate {replicate}, item {j}: {s:.3}"
            );
        }
    }
}

/// AT-BR-11: item 09 with 40 own and 60 opposing boosters (replicate 0) passes robustly as in full.
#[test]
fn no_subsample_split_drops_the_robust_score() {
    let key = "item=8 own=40 step=5";
    let Some(Cell::Capture(d)) = Cell::parse(Study::BridgingCapture, key) else {
        panic!("{key}")
    };
    let o = capture(&d, seed(Study::BridgingCapture, key, 0));
    let at = o.opposing.iter().position(|&c| c == 60).unwrap();
    assert!(o.full[at] >= TAU + EPS, "full {:.3}", o.full[at]);
    assert!(
        o.robust[at] >= TAU + EPS,
        "robust {:.3}, full {:.3}",
        o.robust[at],
        o.full[at]
    );
}
