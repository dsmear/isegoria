//! D3's regression evidence (`docs/15` D3): draw outcomes recorded on the code before the resource
//! domain, over fixtures, sizes, tolerances and seeds; this source compiles against both versions.
//! `D3_RECORD=<path> cargo test -p protocol --test d3_draw_regression` rewrites the record.

mod common;

use network::cid::{cid, Cid};
use protocol::contested::ContestedPool;
use scoring::dtf::{ClassCurves, DTF_MAX};
use scoring::latent::Ability;
use std::fmt::Write;

const RECORD: &str = include_str!("fixtures/d3_draws.txt");
const TOLERANCES: [f64; 7] = [-1.0, 0.0, 0.05, DTF_MAX, 0.3, f64::NAN, f64::INFINITY];
const SEEDS: u64 = 64;

fn normal() -> Ability {
    let nodes: Vec<f64> = (0..41).map(|q| -5.0 + 0.25 * q as f64).collect();
    let mass: Vec<f64> = nodes.iter().map(|u| (-0.5 * u * u).exp()).collect();
    let total: f64 = mass.iter().sum();
    Ability {
        nodes,
        weights: mass.iter().map(|m| m / total).collect(),
    }
}

/// `items` `(a, b, lean)` in classes `pi` at `b + lean·[−1, 1, 0, 0][g]`, over a 41-node normal
/// ability when `ability`, else over the fixed grid of [`ClassCurves::new`].
fn fit(pi: &[f64], items: &[(f64, f64, f64)], ability: bool) -> ClassCurves {
    let shift = [-1.0, 1.0, 0.0, 0.0];
    let a: Vec<Vec<f64>> = pi
        .iter()
        .map(|_| items.iter().map(|i| i.0).collect())
        .collect();
    let b: Vec<Vec<f64>> = (0..pi.len())
        .map(|g| items.iter().map(|i| i.1 + i.2 * shift[g]).collect())
        .collect();
    let eta = vec![0.0; pi.len()];
    let c = vec![0.0; items.len()];
    if ability {
        ClassCurves::with_ability(pi, &eta, &normal(), &a, &b, &c).unwrap()
    } else {
        ClassCurves::new(pi, &eta, &a, &b).unwrap()
    }
}

fn record(pool: &mut ContestedPool, tag: &str, curves: ClassCurves) {
    let members: Vec<(Cid, usize)> = (0..curves.items())
        .map(|j| (cid(format!("{tag}{j}").as_bytes()), j))
        .collect();
    pool.record(curves, &members).unwrap();
}

/// Items alternating ±`lean`, difficulties spread over `[−1, 1]`.
fn spread(k: usize, lean: f64) -> Vec<(f64, f64, f64)> {
    (0..k)
        .map(|j| {
            let sign = if j % 2 == 0 { 1.0 } else { -1.0 };
            (
                1.1 + 0.05 * j as f64,
                -1.0 + 2.0 * j as f64 / k.max(2) as f64,
                sign * lean,
            )
        })
        .collect()
}

fn fixtures() -> Vec<(&'static str, ContestedPool)> {
    let mut four = ContestedPool::new();
    let a = [
        (1.25, 0.0, 0.9),
        (1.25, 0.2, -0.9),
        (1.1, -0.6, 0.8),
        (1.3, -0.5, -0.8),
    ];
    record(&mut four, "A", fit(&[0.5, 0.5], &a, false));
    let b = [(1.2, 0.5, 0.9), (1.2, 0.6, -0.9), (1.0, -1.2, 0.7)];
    record(&mut four, "B", fit(&[0.6, 0.4], &b, false));
    let c = [(1.3, 0.3, 0.8), (1.3, 0.4, -0.8), (1.2, -0.2, 0.6)];
    record(&mut four, "C", fit(&[0.4, 0.35, 0.25], &c, false));
    record(&mut four, "L", fit(&[0.5, 0.5], &[(1.25, 0.0, 0.9)], false));

    let mut estimated = ContestedPool::new();
    record(&mut estimated, "E", fit(&[0.5, 0.5], &spread(7, 0.9), true));
    record(
        &mut estimated,
        "F",
        fit(&[0.4, 0.3, 0.3], &spread(6, 0.6), true),
    );

    let mut one_class = ContestedPool::new();
    record(&mut one_class, "O", fit(&[1.0], &spread(6, 0.9), false));

    let mut small = ContestedPool::new();
    for f in 0..10 {
        let lean = if f % 2 == 0 { 0.4 } else { -0.4 };
        let items = spread(1 + f % 2, lean);
        record(
            &mut small,
            &format!("S{f}-"),
            fit(&[0.5, 0.5], &items, f % 3 == 0),
        );
    }

    let mut mirror = ContestedPool::new();
    record(
        &mut mirror,
        "X",
        fit(&[0.5, 0.5], &[(1.25, 0.0, 0.9)], false),
    );
    record(
        &mut mirror,
        "Y",
        fit(&[0.5, 0.5], &[(1.25, 0.0, -0.9)], false),
    );

    vec![
        ("four fits", four),
        ("estimated ability", estimated),
        ("one class", one_class),
        ("ten small fits", small),
        ("mirror leaners", mirror),
        ("empty", ContestedPool::new()),
    ]
}

/// One draw as text: the drawn ids in order, or the refusal's `requested` and `feasible`.
fn outcome<E: std::fmt::Debug>(drawn: Result<Vec<Cid>, E>) -> String {
    match drawn {
        Ok(test) => test
            .iter()
            .map(|c| format!("{:02x}{:02x} ", c.0[0], c.0[1]))
            .collect(),
        Err(e) => {
            let text = format!("{e:?}");
            let at = text.find("requested").expect("a no-balanced-draw refusal");
            text[at..].trim_end_matches([')', '}', ' ']).to_string()
        }
    }
}

/// Per fixture, size and tolerance: the successes over the seeds and a digest of every outcome.
fn outcomes() -> String {
    let mut text = String::new();
    let beacon = common::beacon(5, 9);
    for (name, pool) in fixtures() {
        for n in 0..=9 {
            for tolerance in TOLERANCES {
                let (mut all, mut drawn) = (String::new(), 0);
                for seed in 0..SEEDS {
                    let result = pool.draw(n, tolerance, seed);
                    drawn += usize::from(result.is_ok());
                    writeln!(all, "{}", outcome(result)).unwrap();
                }
                let beaconed = outcome(pool.draw_from_beacon(n, tolerance, &beacon, 3));
                writeln!(all, "beacon {beaconed}").unwrap();
                let digest: String = cid(all.as_bytes()).0[..8]
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect();
                let first = all
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .trim_end()
                    .to_string();
                writeln!(
                    text,
                    "{name} n={n} tol={tolerance}: drawn {drawn}/{SEEDS} digest {digest} seed 0 [{first}]"
                )
                .unwrap();
            }
        }
    }
    text
}

/// D3: inside the domain every draw and every refusal is the one recorded before the domain.
#[test]
fn draws_inside_the_domain_are_the_recorded_ones() {
    let text = outcomes();
    if let Ok(path) = std::env::var("D3_RECORD") {
        std::fs::write(path, &text).unwrap();
    }
    assert_eq!(text, RECORD);
}

/// D3: a request whose table would not fit is refused, not attempted (a panic before the domain).
#[test]
fn a_request_past_the_table_is_refused_not_attempted() {
    let (_, pool) = fixtures().swap_remove(0);
    let result = std::panic::catch_unwind(|| pool.draw(usize::MAX, DTF_MAX, 0));
    match result {
        Ok(Err(e)) => assert!(format!("{e:?}").contains("TableCells"), "{e:?}"),
        Ok(Ok(test)) => panic!("drew {} facts", test.len()),
        Err(_) => panic!("the draw panicked"),
    }
}
