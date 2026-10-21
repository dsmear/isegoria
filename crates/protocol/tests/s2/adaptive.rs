//! `17` §14.6 on finite constructions (`docs/20` §11): the scoring path, `cohort_scores` since
//! §12, against D1's expected values in exact rationals (A), and one term per channel of (B).

use super::fixture::{nym, U};
use super::records::Outcome;
use super::study::outcome;
use protocol::cohort_scores::{
    score, Assignment, Cohort, CohortScore, Item, Panelist, Report, Selection, Value,
};
use std::cmp::Ordering;
use std::ops::{Add, Div, Mul, Sub};

/// An exact rational in lowest terms, its denominator positive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Q {
    n: i128,
    d: i128,
}

fn gcd(a: i128, b: i128) -> i128 {
    if b == 0 {
        a.abs()
    } else {
        gcd(b, a % b)
    }
}

impl Q {
    pub fn new(n: i128, d: i128) -> Q {
        let g = gcd(n, d) * d.signum();
        Q { n: n / g, d: d / g }
    }

    pub fn f(self) -> f64 {
        self.n as f64 / self.d as f64
    }
}

pub const ZERO: Q = Q { n: 0, d: 1 };
pub const ONE: Q = Q { n: 1, d: 1 };
const HALF: Q = Q { n: 1, d: 2 };

impl Add for Q {
    type Output = Q;
    fn add(self, o: Q) -> Q {
        Q::new(self.n * o.d + o.n * self.d, self.d * o.d)
    }
}

impl Sub for Q {
    type Output = Q;
    fn sub(self, o: Q) -> Q {
        Q::new(self.n * o.d - o.n * self.d, self.d * o.d)
    }
}

impl Mul for Q {
    type Output = Q;
    fn mul(self, o: Q) -> Q {
        Q::new(self.n * o.n, self.d * o.d)
    }
}

impl Div for Q {
    type Output = Q;
    fn div(self, o: Q) -> Q {
        Q::new(self.n * o.d, self.d * o.n)
    }
}

impl Ord for Q {
    fn cmp(&self, o: &Q) -> Ordering {
        (self.n * o.d).cmp(&(o.n * self.d))
    }
}

impl PartialOrd for Q {
    fn partial_cmp(&self, o: &Q) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

fn sum(terms: impl Iterator<Item = Q>) -> Q {
    terms.fold(ZERO, |a, b| a + b)
}

fn r(n: i128, d: i128) -> Q {
    Q::new(n, d)
}

/// One assignment in a state: the item's outcome and the other first panelists' reports, each
/// with its frozen weight.
#[derive(Clone, Debug)]
pub struct Seat {
    pub outcome: Outcome,
    pub others: Vec<(Q, u64)>,
}

/// A state: its probability, `u`'s information cell, one seat per member of `u`'s cohort.
pub struct State {
    pub p: Q,
    pub cell: usize,
    pub seats: Vec<Seat>,
}

pub struct Construction {
    pub cells: usize,
    pub items: usize,
    pub grid: Vec<Q>,
    pub states: Vec<State>,
}

/// Per (cell, item), from the states alone: `P(cell)`, `c`, `q_c` (`None` at `c = 0`) and
/// `t = E[1{Y ≠ I}(b − o)² | cell]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Law {
    pub cell: Q,
    pub c: Q,
    pub q: Option<Q>,
    pub t: Q,
}

/// `seat` as the scorer's item under A: `u` first at weight 1, then the others, frozen, `π = 1`.
fn item(report: f64, seat: &Seat) -> Item {
    let own = (report, 1.0);
    let others = seat.others.iter().map(|(p, w)| (p.f(), *w as f64));
    let first = std::iter::once(own).chain(others).enumerate();
    Item {
        first: (first.map(|(k, (prob, weight))| Panelist {
            nym: nym(if k == 0 { U } else { 100 + k as u8 }),
            report: Report::Revealed(prob),
            weight,
        }))
        .collect(),
        extra: Vec::new(),
        frozen: true,
        selection: Selection::Selected {
            inclusion: 1.0,
            outcome: Some(outcome(seat.outcome)),
        },
    }
}

/// `u`'s prefixed cohort over `items`, all of them, through `cohort_scores::score`.
fn cohort(items: Vec<Item>) -> CohortScore {
    let k = Cohort {
        nym: nym(U),
        items: (0..items.len()).collect(),
    };
    let mut scores = score(&items, &[k]).expect("a coherent construction");
    scores.cohorts.remove(0)
}

/// `u`'s assignment on `seat` through the scorer.
pub fn scored(report: f64, seat: &Seat) -> Assignment {
    cohort(vec![item(report, seat)]).members.remove(0)
}

/// The scorer's baseline for `u` on `seat`: `first_panel_baselines` over the seat's others.
pub fn baseline(report: f64, seat: &Seat) -> Option<f64> {
    scored(report, seat).baseline
}

fn defined(a: &Assignment) -> f64 {
    match a.term {
        Ok(Some(g)) => g,
        t => panic!("a defined term: {t:?}"),
    }
}

fn indicator(x: bool) -> Q {
    if x {
        ONE
    } else {
        ZERO
    }
}

impl Construction {
    /// `Σ P(state) × u's cohort value` through the path; `reports[cell × items + j]`.
    pub fn expectation(&self, reports: &[Q]) -> f64 {
        let value = |s: &State| {
            let items = s.seats.iter().enumerate().map(|(j, seat)| {
                let report = reports[s.cell * self.items + j];
                item(report.f(), seat)
            });
            match cohort(items.collect()).value {
                Value::Final(x) => x,
                v => panic!("H-d: {v:?}"),
            }
        };
        self.states.iter().map(|s| s.p.f() * value(s)).sum()
    }

    /// The laws, indexed as the reports; `b` the others' weighted mean, in rationals.
    pub fn laws(&self) -> Vec<Law> {
        let mut laws = Vec::new();
        for cell in 0..self.cells {
            let of: Vec<&State> = self.states.iter().filter(|s| s.cell == cell).collect();
            let pc = sum(of.iter().map(|s| s.p));
            for j in 0..self.items {
                let given =
                    |f: &dyn Fn(&Seat) -> Q| sum(of.iter().map(|s| s.p * f(&s.seats[j]))) / pc;
                let a = given(&|s| indicator(s.outcome == Outcome::A));
                let c = given(&|s| indicator(s.outcome != Outcome::I));
                let t = given(&|s| {
                    let w = sum(s.others.iter().map(|(_, w)| r(*w as i128, 1)));
                    let b = sum(s.others.iter().map(|(p, w)| *p * r(*w as i128, 1))) / w;
                    let gap = b - indicator(s.outcome == Outcome::A);
                    indicator(s.outcome != Outcome::I) * gap * gap
                });
                let q = (c != ZERO).then(|| a / c);
                laws.push(Law { cell: pc, c, q, t });
            }
        }
        laws
    }

    /// D1's `E[Ŝ_K]` for `reports` (`docs/20` §11), exact; 0 where `c = 0`.
    pub fn predicted(&self, laws: &[Law], reports: &[Q]) -> Q {
        let term = |(l, p): (&Law, &Q)| match l.q {
            None => ZERO,
            Some(q) => l.cell * (l.t - l.c * ((*p - q) * (*p - q) + q * (ONE - q))),
        };
        sum(laws.iter().zip(reports).map(term)) / r(self.items as i128, 1)
    }

    /// Every map from (cell, item) to the grid.
    pub fn strategies(&self) -> impl Iterator<Item = Vec<Q>> + '_ {
        let (n, g) = (self.cells * self.items, self.grid.len());
        (0..g.pow(n as u32)).map(move |mut i| {
            let pick = |_| {
                let x = self.grid[i % g];
                i /= g;
                x
            };
            (0..n).map(pick).collect()
        })
    }

    /// C5 pathwise: every seat's baseline the same whatever `u` reports on the grid.
    pub fn baselines_invariant(&self) -> bool {
        let mut seats = self.states.iter().flat_map(|s| &s.seats);
        seats.all(|seat| {
            let b: Vec<Option<f64>> = self.grid.iter().map(|x| baseline(x.f(), seat)).collect();
            b.windows(2).all(|w| w[0] == w[1])
        })
    }
}

/// `q_c` where `c > 0`, ½ where `c = 0`.
pub fn truthful(laws: &[Law]) -> Vec<Q> {
    laws.iter().map(|l| l.q.unwrap_or(HALF)).collect()
}

fn accuracy(signal: u8, z: u8) -> Q {
    if signal == z {
        r(3, 4)
    } else {
        r(1, 4)
    }
}

fn verdict(z: u8) -> Outcome {
    if z == 1 {
        Outcome::A
    } else {
        Outcome::R
    }
}

/// `docs/20` §11.1's P1; cell `2s + d`.
pub fn p1() -> Construction {
    let mut states = Vec::new();
    for (z, s, t, d) in (0..16u8).map(|i| (i >> 3, (i >> 2) & 1, (i >> 1) & 1, i & 1)) {
        let p = HALF * accuracy(s, z) * accuracy(t, z) * HALF;
        let coins = if d == 0 {
            vec![(true, ONE)]
        } else {
            vec![(true, r(2, 3)), (false, r(1, 3))]
        };
        for (concludes, coin) in coins {
            let r_t = if t == 1 { r(3, 4) } else { r(1, 4) };
            let j1 = Seat {
                outcome: if concludes { verdict(z) } else { Outcome::I },
                others: vec![(r_t, 1), (HALF, 1)],
            };
            let j2 = Seat {
                outcome: if d == 0 { verdict(z) } else { Outcome::I },
                others: vec![(HALF, 1)],
            };
            let cell = usize::from(2 * s + d);
            let seats = vec![j1, j2];
            states.push(State {
                p: p * coin,
                cell,
                seats,
            });
        }
    }
    Construction {
        cells: 4,
        items: 2,
        grid: vec![r(1, 4), HALF, r(3, 4)],
        states,
    }
}

/// `docs/20` §11.1's P2; cell `D`, 1 a pass.
pub fn p2() -> Construction {
    let mut states = Vec::new();
    for (concludes, z, d) in (0..8u8).map(|i| (i >> 2 == 1, (i >> 1) & 1, i & 1)) {
        let p = HALF * HALF * accuracy(d, z);
        let r1 = if d == 1 { r(5, 8) } else { r(3, 8) };
        let seat = |outcome| Seat {
            outcome,
            others: vec![(r1, 1), (HALF, 1)],
        };
        let cell = usize::from(d);
        if !concludes {
            let seats = vec![seat(Outcome::I); 3];
            states.push(State { p, cell, seats });
            continue;
        }
        let a = if z == 1 { r(3, 4) } else { r(1, 4) };
        for ys in 0..8u8 {
            let draws: Vec<bool> = (0..3).map(|j| (ys >> j) & 1 == 1).collect();
            let pr = draws
                .iter()
                .fold(ONE, |x, y| x * if *y { a } else { ONE - a });
            let seats = draws.iter().map(|y| seat(verdict(u8::from(*y)))).collect();
            states.push(State {
                p: p * pr,
                cell,
                seats,
            });
        }
    }
    Construction {
        cells: 2,
        items: 3,
        grid: vec![r(3, 8), HALF, r(5, 8)],
        states,
    }
}

/// One assignment's expected term over (probability, report, seat), through [`scored`].
pub fn expected(states: &[(Q, f64, Seat)]) -> f64 {
    let term = |(p, report, seat): &(Q, f64, Seat)| p.f() * defined(&scored(*report, seat));
    states.iter().map(term).sum()
}

fn seat(outcome: Outcome, other: Q) -> Seat {
    Seat {
        outcome,
        others: vec![(other, 1)],
    }
}

/// `17` §7.3 for B-b: `j`'s law (3/5, 2/5, 0) while `k` stays out, `I` once it enters; the other
/// first panelist reports the conclusive outcome `j` would have; `u` reports 3/5 on `j`.
pub fn load(entered: bool) -> Vec<(Q, f64, Seat)> {
    let y = |o| if entered { Outcome::I } else { o };
    vec![
        (r(3, 5), r(3, 5).f(), seat(y(Outcome::A), ONE)),
        (r(2, 5), r(3, 5).f(), seat(y(Outcome::R), ZERO)),
    ]
}

/// `17` §9.4: the expected term from the effective baseline `b(p)`, supplied as the only other
/// first report at weight 1, which the composition returns; the outcome Bernoulli(½).
pub fn replacement(p: f64) -> f64 {
    let b = (2.0 * p - 0.5).clamp(0.0, 1.0);
    let g = |o| {
        let mut i = item(p, &seat(o, ZERO));
        i.first[1].report = Report::Revealed(b);
        defined(&cohort(vec![i]).members[0])
    };
    0.5 * g(Outcome::A) + 0.5 * g(Outcome::R)
}

/// `17` §11.3, timing: `p ≤ ½` sends `j` to the band and `A`'s probability from 3/5 to 2/5.
pub fn timing(p: f64) -> Vec<(Q, f64, Seat)> {
    let a = if p <= 0.5 { r(2, 5) } else { r(3, 5) };
    vec![
        (a, p, seat(Outcome::A, r(3, 5))),
        (ONE - a, p, seat(Outcome::R, r(3, 5))),
    ]
}

/// `17` §11.3, disclosure: `Y = Z`, `p = ½`; the other report ½, or `Z` once the signal arrives.
pub fn disclosure(signal: bool) -> Vec<(Q, f64, Seat)> {
    let other = |z: Q| if signal { z } else { HALF };
    vec![
        (HALF, 0.5, seat(Outcome::A, other(ONE))),
        (HALF, 0.5, seat(Outcome::R, other(ZERO))),
    ]
}

/// `17` §12.4: `Y_k` equiprobable, `p = ½`; with `δ`, a ± signal moves `P(A)` and the other
/// report to `½ ± δ`.
pub fn across(delta: Option<Q>) -> Vec<(Q, f64, Seat)> {
    let posteriors = match delta {
        None => vec![(ONE, HALF)],
        Some(d) => vec![(HALF, HALF + d), (HALF, HALF - d)],
    };
    let mut states = Vec::new();
    for (ps, post) in posteriors {
        states.push((ps * post, 0.5, seat(Outcome::A, post)));
        states.push((ps * (ONE - post), 0.5, seat(Outcome::R, post)));
    }
    states
}
