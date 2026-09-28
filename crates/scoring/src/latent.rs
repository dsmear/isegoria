//! Latent DIF, the target model of `docs/01` D37: a latent-class IRT mixture with the
//! anchors inside the likelihood, θ integrated over a fixed grid and each item's guessing
//! floor set by its format (D25; `paper/` §7.3, `docs/02` §B.1, §B.3, `docs/08` DIF-010).

use crate::dif::MIN_CLASS_SHARE;
use crate::fmath::{cos, exp, ln, ln_1p};
use crate::optim::{lbfgs, Convergence};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::cell::RefCell;

/// The weight of a choice item's floor prior, in pseudo-observations (`docs/02` §B.1);
/// provisional (T25).
pub const FLOOR_PRIOR_WEIGHT: f64 = 20.0;

#[derive(Clone, Copy, Debug)]
pub struct LatentParams {
    /// Largest number of latent classes tried; the BIC picks among `1..=max_classes`.
    pub max_classes: usize,
    /// Seeded starts per candidate model.
    pub n_starts: usize,
    pub seed: u64,
    /// Quadrature nodes of the rectangular grid over `[−theta_max, theta_max]`.
    pub nodes: usize,
    pub theta_max: f64,
    /// The weight of every floor's prior ([`FLOOR_PRIOR_WEIGHT`]).
    pub floor_weight: f64,
}

impl Default for LatentParams {
    fn default() -> Self {
        LatentParams {
            max_classes: 4,
            n_starts: 4,
            seed: 0,
            nodes: 41,
            theta_max: 5.0,
            floor_weight: FLOOR_PRIOR_WEIGHT,
        }
    }
}

/// An item's declared format (`docs/02` §B.1), which sets its guessing floor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// The respondent gives the datum: nothing to guess, the floor fixed at 0.
    Open,
    /// A choice among `m ≥ 2` options, one keyed: the floor estimated, its prior's mode `1/m`.
    Choice(u8),
}

/// The formats of a batch's anchors and trial items, in their column order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Formats {
    pub anchors: Vec<Format>,
    pub items: Vec<Format>,
}

impl Formats {
    /// Every anchor and item an open answer: the 2PL target model.
    pub fn open(anchors: usize, items: usize) -> Formats {
        Formats::all(anchors, items, Format::Open)
    }

    /// Every anchor and item a choice among `m` options.
    pub fn choice(anchors: usize, items: usize, m: u8) -> Formats {
        Formats::all(anchors, items, Format::Choice(m))
    }

    fn all(anchors: usize, items: usize, format: Format) -> Formats {
        Formats {
            anchors: vec![format; anchors],
            items: vec![format; items],
        }
    }
}

/// Formats that do not describe the batch: not one per anchor and per trial item, or a
/// choice among fewer than two options.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BadFormats;

const MAX_ITERS: usize = 1000;

/// The target-model fit selected by BIC.
#[derive(Clone, Debug)]
pub struct LatentDif {
    /// Number of latent classes of the selected model; 1 means no mixture was found.
    pub classes: usize,
    /// Whether the selected model lets the trial items' discrimination differ by class.
    pub non_uniform: bool,
    /// Mixing proportion of each class.
    pub pi: Vec<f64>,
    /// Class ability means `η_g`, with `η_0 = 0`.
    pub eta: Vec<f64>,
    /// The anchors' class-invariant parameters.
    pub anchor_a: Vec<f64>,
    pub anchor_b: Vec<f64>,
    /// Per anchor, its guessing floor; 0 for an open answer.
    pub anchor_c: Vec<f64>,
    /// `item_a[g][j]`, `item_b[g][j]`: the trial items' parameters per class.
    pub item_a: Vec<Vec<f64>>,
    pub item_b: Vec<Vec<f64>>,
    /// Per trial item, its guessing floor, the same in every class; 0 for an open answer.
    pub item_c: Vec<f64>,
    /// Per item, `max_{g,h} |b_jg − b_jh|` over classes with share ≥ [`MIN_CLASS_SHARE`]
    /// (`docs/02` §B.3); 0 with one class.
    pub dif: Vec<f64>,
    /// Per-item `max_{g,h} |a_jg − a_jh|` (non-uniform DIF); 0 when `a_j` is shared.
    pub a_gap: Vec<f64>,
    /// `posterior[i][g]`: probability that respondent `i` belongs to class `g`.
    pub posterior: Vec<Vec<f64>>,
    /// `BIC(one class) − BIC(selected)`: > 0 when a mixture is preferred.
    pub bic_gain: f64,
    /// Every candidate tried: `(classes, non_uniform, BIC)`.
    pub candidates: Vec<(usize, bool, f64)>,
    pub status: Convergence,
}

impl LatentDif {
    /// Per item, `dif[j] > max_gap`, and only from a converged fit with two or more classes.
    pub fn flags(&self, max_gap: f64) -> Vec<bool> {
        let trustworthy = self.status == Convergence::Converged && self.classes >= 2;
        self.dif
            .iter()
            .map(|&d| trustworthy && d > max_gap)
            .collect()
    }
}

/// A candidate: `g` classes over `na` anchors and `k` trial items, `a` shared or per class,
/// and `floors` guessing floors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Model {
    g: usize,
    na: usize,
    k: usize,
    per_class_a: bool,
    floors: usize,
}

// Parameter layout: [ ζ (G−1 class logits vs class 0) | η (G−1 class means, η_0 = 0) | anchor a
// (A) | anchor b (A) | item a (K, or K·G class-major) | item b (K·G) | floor logits ].
impl Model {
    fn n_item_a(&self) -> usize {
        if self.per_class_a {
            self.g * self.k
        } else {
            self.k
        }
    }
    fn len(&self) -> usize {
        self.floor_idx(self.floors)
    }
    fn floor_idx(&self, slot: usize) -> usize {
        2 * (self.g - 1) + 2 * self.na + self.n_item_a() + self.g * self.k + slot
    }
    fn zeta_idx(&self, g: usize) -> usize {
        g - 1
    }
    fn eta_idx(&self, g: usize) -> usize {
        (self.g - 1) + (g - 1)
    }
    fn anchor_a_idx(&self, a: usize) -> usize {
        2 * (self.g - 1) + a
    }
    fn anchor_b_idx(&self, a: usize) -> usize {
        2 * (self.g - 1) + self.na + a
    }
    fn item_a_idx(&self, g: usize, j: usize) -> usize {
        2 * (self.g - 1) + 2 * self.na + if self.per_class_a { g * self.k + j } else { j }
    }
    fn item_b_idx(&self, g: usize, j: usize) -> usize {
        2 * (self.g - 1) + 2 * self.na + self.n_item_a() + g * self.k + j
    }
    fn free_params(&self) -> usize {
        self.len()
    }
    fn pi(&self, p: &[f64]) -> Vec<f64> {
        let mut z = vec![0.0; self.g];
        for g in 1..self.g {
            z[g] = p[self.zeta_idx(g)];
        }
        let m = z.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let e: Vec<f64> = z.iter().map(|v| exp(v - m)).collect();
        let s: f64 = e.iter().sum();
        e.iter().map(|v| v / s).collect()
    }
    fn eta(&self, p: &[f64]) -> Vec<f64> {
        let mut eta = vec![0.0; self.g];
        for g in 1..self.g {
            eta[g] = p[self.eta_idx(g)];
        }
        eta
    }
}

struct Grid {
    theta: Vec<f64>,
}

impl Grid {
    fn new(nodes: usize, theta_max: f64) -> Grid {
        let q = nodes.max(2);
        let theta = (0..q)
            .map(|i| -theta_max + 2.0 * theta_max * i as f64 / (q - 1) as f64)
            .collect();
        Grid { theta }
    }

    /// Per node, `ln φ(θ_q − eta)` normalized over the grid, and the grid mean of θ under it.
    fn log_weights(&self, eta: f64) -> (Vec<f64>, f64) {
        let lw: Vec<f64> = self
            .theta
            .iter()
            .map(|t| -0.5 * (t - eta) * (t - eta))
            .collect();
        let m = lw.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let z: f64 = lw.iter().map(|v| exp(v - m)).sum();
        let ln_z = m + ln(z);
        let lw: Vec<f64> = lw.iter().map(|v| v - ln_z).collect();
        let mean = lw.iter().zip(&self.theta).map(|(lw, t)| exp(*lw) * t).sum();
        (lw, mean)
    }
}

/// The anchors and items with a guessing floor, each with its slot among the floor
/// parameters — the anchors' first — and its chance level `1/m`; `None` for an open answer.
struct Floors {
    anchor: Vec<Option<(usize, f64)>>,
    item: Vec<Option<(usize, f64)>>,
    /// `(anchor, chance)` and `(item, chance)` in slot order.
    anchor_list: Vec<(usize, f64)>,
    item_list: Vec<(usize, f64)>,
    count: usize,
    weight: f64,
}

impl Floors {
    fn new(formats: &Formats, na: usize, k: usize, weight: f64) -> Result<Floors, BadFormats> {
        let valid = formats.anchors.len() == na
            && formats.items.len() == k
            && formats
                .anchors
                .iter()
                .chain(&formats.items)
                .all(|f| !matches!(f, Format::Choice(m) if *m < 2));
        if !valid {
            return Err(BadFormats);
        }
        let slots = |formats: &[Format], first: usize| -> Vec<Option<(usize, f64)>> {
            let mut next = first;
            formats
                .iter()
                .map(|f| match *f {
                    Format::Open => None,
                    Format::Choice(m) => {
                        next += 1;
                        Some((next - 1, 1.0 / f64::from(m)))
                    }
                })
                .collect()
        };
        let listed = |slots: &[Option<(usize, f64)>]| -> Vec<(usize, f64)> {
            slots
                .iter()
                .enumerate()
                .filter_map(|(i, s)| s.map(|(_, chance)| (i, chance)))
                .collect()
        };
        let anchor = slots(&formats.anchors, 0);
        let anchor_list = listed(&anchor);
        let item = slots(&formats.items, anchor_list.len());
        let item_list = listed(&item);
        Ok(Floors {
            count: anchor_list.len() + item_list.len(),
            anchor,
            item,
            anchor_list,
            item_list,
            weight,
        })
    }
}

/// Per respondent, the open anchors and items answered correctly (`>= 0.5`) by index, those
/// with a floor by position in [`Floors`]' lists; per column, its correct answers.
struct Data {
    n: usize,
    correct_anchors: Vec<Vec<usize>>,
    correct_items: Vec<Vec<usize>>,
    floor_anchors: Vec<Vec<usize>>,
    floor_items: Vec<Vec<usize>>,
    anchor_count: Vec<f64>,
    item_count: Vec<f64>,
}

impl Data {
    fn new(anchors: &[Vec<f64>], x: &[Vec<f64>], na: usize, k: usize, floors: &Floors) -> Data {
        let split = |rows: &[Vec<f64>], len: usize, slots: &[Option<(usize, f64)>], first| {
            let mut count = vec![0.0; len];
            let (mut open, mut floor) = (Vec::new(), Vec::new());
            for row in rows {
                let (mut o, mut f) = (Vec::new(), Vec::new());
                for (i, &v) in row.iter().take(len).enumerate() {
                    if v >= 0.5 {
                        count[i] += 1.0;
                        match slots[i] {
                            None => o.push(i),
                            Some((s, _)) => f.push(s - first),
                        }
                    }
                }
                open.push(o);
                floor.push(f);
            }
            (open, floor, count)
        };
        let (correct_anchors, floor_anchors, anchor_count) = split(anchors, na, &floors.anchor, 0);
        let first_item = floors.anchor_list.len();
        let (correct_items, floor_items, item_count) = split(x, k, &floors.item, first_item);
        Data {
            n: x.len(),
            correct_anchors,
            correct_items,
            floor_anchors,
            floor_items,
            anchor_count,
            item_count,
        }
    }
}

/// A 2PL cell at a node: the softplus of its logit and its probability.
#[derive(Clone, Copy)]
struct Cell {
    softplus: f64,
    sigma: f64,
}

fn cell(a: f64, theta: f64, b: f64) -> Cell {
    let lo = a * (theta - b);
    let e = exp(-lo.abs());
    let (softplus, sigma) = if lo > 0.0 {
        (lo + ln_1p(e), 1.0 / (1.0 + e))
    } else {
        (ln_1p(e), e / (1.0 + e))
    };
    Cell { softplus, sigma }
}

fn sigmoid(z: f64) -> f64 {
    if z >= 0.0 {
        1.0 / (1.0 + exp(-z))
    } else {
        let e = exp(z);
        e / (1.0 + e)
    }
}

fn softplus(z: f64) -> f64 {
    if z > 0.0 {
        z + ln_1p(exp(-z))
    } else {
        ln_1p(exp(z))
    }
}

/// `ln P(x = 1) − ln P(x = 0)` of a cell under the floor `σ(gamma)`.
fn log_odds(cell: &Cell, gamma: f64) -> f64 {
    let ln_wrong = -(cell.softplus + softplus(gamma));
    let wrong = exp(ln_wrong);
    let ln_right = if wrong < 0.5 {
        ln_1p(-wrong)
    } else {
        ln(sigmoid(gamma) + sigmoid(-gamma) * cell.sigma)
    };
    ln_right - ln_wrong
}

/// For an item with the floor `c`, per node its probability `sigma` without the floor, its
/// expected correct answers `right` and respondents `n`: the log-likelihood's derivatives in
/// `a`, in its logit's shift, and in the floor's logit.
fn floor_derivatives(
    c: f64,
    sigma: impl Iterator<Item = f64>,
    right: &[f64],
    n: &[f64],
    theta: &[f64],
    b: f64,
) -> (f64, f64, f64) {
    let (mut da, mut dz, mut dg) = (0.0, 0.0, 0.0);
    for (((s, &r), &n), &t) in sigma.zip(right).zip(n).zip(theta) {
        let p = (c + (1.0 - c) * s).max(f64::MIN_POSITIVE);
        let wrong = n - r;
        let d = r * (1.0 - c) * s * (1.0 - s) / p - wrong * s;
        da += (t - b) * d;
        dz += d;
        dg += r * c * (1.0 - c) * (1.0 - s) / p - wrong * c;
    }
    (da, dz, dg)
}

/// The negative marginal log-likelihood with its gradient and, on request, the class
/// posteriors. Transcendentals are evaluated per node and item, never per respondent: at a
/// node a respondent's log-likelihood is a constant plus the log-odds of its correct answers.
fn evaluate(
    model: &Model,
    p: &[f64],
    data: &Data,
    grid: &Grid,
    floors: &Floors,
    want_posterior: bool,
) -> (f64, Vec<f64>, Option<Vec<Vec<f64>>>) {
    let (gn, na, k, q) = (model.g, model.na, model.k, grid.theta.len());
    let pi = model.pi(p);
    let ln_pi: Vec<f64> = pi.iter().map(|v| ln(*v)).collect();
    let eta = model.eta(p);
    let mut ln_w = Vec::with_capacity(gn);
    let mut grid_mean = Vec::with_capacity(gn);
    for &e in &eta {
        let (lw, m) = grid.log_weights(e);
        ln_w.push(lw);
        grid_mean.push(m);
    }
    let anchor_a: Vec<f64> = (0..na).map(|a| p[model.anchor_a_idx(a)]).collect();
    let anchor_b: Vec<f64> = (0..na).map(|a| p[model.anchor_b_idx(a)]).collect();
    let anchor_ab: Vec<f64> = anchor_a.iter().zip(&anchor_b).map(|(a, b)| a * b).collect();
    let item_a: Vec<Vec<f64>> = (0..gn)
        .map(|g| (0..k).map(|j| p[model.item_a_idx(g, j)]).collect())
        .collect();
    let item_b: Vec<Vec<f64>> = (0..gn)
        .map(|g| (0..k).map(|j| p[model.item_b_idx(g, j)]).collect())
        .collect();
    let item_ab: Vec<Vec<f64>> = (0..gn)
        .map(|g| (0..k).map(|j| item_a[g][j] * item_b[g][j]).collect())
        .collect();
    let anchor_cells: Vec<Vec<Cell>> = (0..q)
        .map(|qi| {
            (0..na)
                .map(|a| cell(anchor_a[a], grid.theta[qi], anchor_b[a]))
                .collect()
        })
        .collect();
    let floor_logit = |f: &Option<(usize, f64)>| f.map(|(s, _)| p[model.floor_idx(s)]);
    let anchor_gamma: Vec<Option<f64>> = floors.anchor.iter().map(floor_logit).collect();
    let item_gamma: Vec<Option<f64>> = floors.item.iter().map(floor_logit).collect();
    // `−ln P(x = 0)` at a node: the softplus of the logit, plus `−ln(1 − c)` under a floor.
    let wrong = |cell: &Cell, gamma: &Option<f64>| match gamma {
        None => cell.softplus,
        Some(gm) => cell.softplus + softplus(*gm),
    };
    let anchor_softplus: Vec<f64> = anchor_cells
        .iter()
        .map(|cells| {
            cells
                .iter()
                .zip(&anchor_gamma)
                .map(|(c, gm)| wrong(c, gm))
                .sum()
        })
        .collect();
    let item_cells: Vec<Vec<Vec<Cell>>> = (0..gn)
        .map(|g| {
            (0..q)
                .map(|qi| {
                    (0..k)
                        .map(|j| cell(item_a[g][j], grid.theta[qi], item_b[g][j]))
                        .collect()
                })
                .collect()
        })
        .collect();
    let constant: Vec<Vec<f64>> = (0..gn)
        .map(|g| {
            (0..q)
                .map(|qi| {
                    ln_pi[g] + ln_w[g][qi]
                        - anchor_softplus[qi]
                        - item_cells[g][qi]
                            .iter()
                            .zip(&item_gamma)
                            .map(|(c, gm)| wrong(c, gm))
                            .sum::<f64>()
                })
                .collect()
        })
        .collect();
    let anchor_lodds: Vec<Vec<f64>> = floors
        .anchor_list
        .iter()
        .enumerate()
        .map(|(s, &(a, _))| {
            let gm = p[model.floor_idx(s)];
            (0..q)
                .map(|qi| log_odds(&anchor_cells[qi][a], gm))
                .collect()
        })
        .collect();
    let item_lodds: Vec<Vec<Vec<f64>>> = (0..gn)
        .map(|g| {
            floors
                .item_list
                .iter()
                .enumerate()
                .map(|(s, &(j, _))| {
                    let gm = p[model.floor_idx(floors.anchor_list.len() + s)];
                    (0..q)
                        .map(|qi| log_odds(&item_cells[g][qi][j], gm))
                        .collect()
                })
                .collect()
        })
        .collect();

    let mut n_gq = vec![vec![0.0; q]; gn];
    // Per anchor `Σ_i x_ia · E[θ | x_i]`; per class and item `Σ_i x_ij r_ig` and
    // `Σ_i x_ij Σ_q θ_q r_igq`.
    let mut anchor_theta = vec![0.0; na];
    let mut item_r0 = vec![vec![0.0; k]; gn];
    let mut item_r1 = vec![vec![0.0; k]; gn];
    let mut class_r0 = vec![0.0; gn];
    // Under a floor the log-odds are not linear in θ: per anchor the expected correct
    // answers at each node, per class and item the same within the class.
    let mut anchor_right = vec![vec![0.0; q]; floors.anchor_list.len()];
    let mut item_right = vec![vec![vec![0.0; q]; floors.item_list.len()]; gn];
    let has_floors = floors.count > 0;
    let mut extra = vec![0.0; if has_floors { gn * q } else { 0 }];
    let mut node = vec![0.0; if has_floors { q } else { 0 }];
    let mut posterior = if want_posterior {
        Some(Vec::with_capacity(data.n))
    } else {
        None
    };
    let mut terms = vec![0.0; gn * q];
    let mut s1 = vec![0.0; gn];
    let mut s2 = vec![0.0; gn];
    let mut r0 = vec![0.0; gn];
    let mut r1 = vec![0.0; gn];
    let mut total = 0.0;
    for i in 0..data.n {
        let ca = &data.correct_anchors[i];
        let ci = &data.correct_items[i];
        let s1a: f64 = ca.iter().map(|&a| anchor_a[a]).sum();
        let s2a: f64 = ca.iter().map(|&a| anchor_ab[a]).sum();
        if has_floors {
            node.fill(0.0);
            for &s in &data.floor_anchors[i] {
                node.iter_mut()
                    .zip(&anchor_lodds[s])
                    .for_each(|(v, l)| *v += l);
            }
            for g in 0..gn {
                let row = &mut extra[g * q..(g + 1) * q];
                row.copy_from_slice(&node);
                for &s in &data.floor_items[i] {
                    row.iter_mut()
                        .zip(&item_lodds[g][s])
                        .for_each(|(v, l)| *v += l);
                }
            }
        }
        let mut m = f64::NEG_INFINITY;
        for g in 0..gn {
            s1[g] = s1a + ci.iter().map(|&j| item_a[g][j]).sum::<f64>();
            s2[g] = s2a + ci.iter().map(|&j| item_ab[g][j]).sum::<f64>();
            let row = &constant[g];
            for qi in 0..q {
                let mut t = row[qi] + grid.theta[qi] * s1[g] - s2[g];
                if has_floors {
                    t += extra[g * q + qi];
                }
                terms[g * q + qi] = t;
                if t > m {
                    m = t;
                }
            }
        }
        let z: f64 = terms.iter().map(|t| exp(t - m)).sum();
        let lse = m + ln(z);
        total += lse;
        for g in 0..gn {
            let (mut a0, mut a1) = (0.0, 0.0);
            let n_g = &mut n_gq[g];
            for qi in 0..q {
                let r = exp(terms[g * q + qi] - lse);
                n_g[qi] += r;
                a0 += r;
                a1 += r * grid.theta[qi];
                terms[g * q + qi] = r;
            }
            r0[g] = a0;
            r1[g] = a1;
            class_r0[g] += a0;
            let (i0, i1) = (&mut item_r0[g], &mut item_r1[g]);
            for &j in ci {
                i0[j] += a0;
                i1[j] += a1;
            }
        }
        let theta_mean: f64 = r1.iter().sum();
        for &a in ca {
            anchor_theta[a] += theta_mean;
        }
        if has_floors {
            for (qi, v) in node.iter_mut().enumerate() {
                *v = (0..gn).map(|g| terms[g * q + qi]).sum();
            }
            for &s in &data.floor_anchors[i] {
                anchor_right[s]
                    .iter_mut()
                    .zip(&node)
                    .for_each(|(v, r)| *v += r);
            }
            for (g, right) in item_right.iter_mut().enumerate() {
                let r = &terms[g * q..(g + 1) * q];
                for &s in &data.floor_items[i] {
                    right[s].iter_mut().zip(r).for_each(|(v, r)| *v += r);
                }
            }
        }
        if let Some(post) = posterior.as_mut() {
            post.push(r0.clone());
        }
    }

    // The gradient from the EM "artificial data": the expected respondents per node, and
    // per respondent the class posterior `r_ig` and its first θ-moment `Σ_q θ_q r_igq`.
    let mut grad = vec![0.0; p.len()];
    let n_q: Vec<f64> = (0..q)
        .map(|qi| (0..gn).map(|g| n_gq[g][qi]).sum())
        .collect();
    for a in 0..na {
        if anchor_gamma[a].is_some() {
            continue;
        }
        let (ai, bi) = (model.anchor_a_idx(a), model.anchor_b_idx(a));
        let (mut expected_theta, mut expected) = (0.0, 0.0);
        for qi in 0..q {
            let s = anchor_cells[qi][a].sigma * n_q[qi];
            expected_theta += s * (grid.theta[qi] - anchor_b[a]);
            expected += s;
        }
        grad[ai] += expected_theta - (anchor_theta[a] - anchor_b[a] * data.anchor_count[a]);
        grad[bi] += anchor_a[a] * (data.anchor_count[a] - expected);
    }
    for g in 0..gn {
        for (j, gamma) in item_gamma.iter().enumerate() {
            if gamma.is_some() {
                continue;
            }
            let (ai, bi) = (model.item_a_idx(g, j), model.item_b_idx(g, j));
            let (mut expected_theta, mut expected) = (0.0, 0.0);
            for qi in 0..q {
                let s = item_cells[g][qi][j].sigma * n_gq[g][qi];
                expected_theta += s * (grid.theta[qi] - item_b[g][j]);
                expected += s;
            }
            grad[ai] += expected_theta - (item_r1[g][j] - item_b[g][j] * item_r0[g][j]);
            grad[bi] += item_a[g][j] * (item_r0[g][j] - expected);
        }
    }
    for (s, &(a, _)) in floors.anchor_list.iter().enumerate() {
        let c = sigmoid(p[model.floor_idx(s)]);
        let sigma = (0..q).map(|qi| anchor_cells[qi][a].sigma);
        let (da, dz, dg) =
            floor_derivatives(c, sigma, &anchor_right[s], &n_q, &grid.theta, anchor_b[a]);
        grad[model.anchor_a_idx(a)] -= da;
        grad[model.anchor_b_idx(a)] += anchor_a[a] * dz;
        grad[model.floor_idx(s)] -= dg;
    }
    for (s, &(j, _)) in floors.item_list.iter().enumerate() {
        let slot = floors.anchor_list.len() + s;
        let c = sigmoid(p[model.floor_idx(slot)]);
        for g in 0..gn {
            let sigma = (0..q).map(|qi| item_cells[g][qi][j].sigma);
            let (da, dz, dg) = floor_derivatives(
                c,
                sigma,
                &item_right[g][s],
                &n_gq[g],
                &grid.theta,
                item_b[g][j],
            );
            grad[model.item_a_idx(g, j)] -= da;
            grad[model.item_b_idx(g, j)] += item_a[g][j] * dz;
            grad[model.floor_idx(slot)] -= dg;
        }
    }
    let n = data.n as f64;
    for g in 1..gn {
        grad[model.zeta_idx(g)] -= class_r0[g] - n * pi[g];
        grad[model.eta_idx(g)] -= n_gq[g]
            .iter()
            .zip(&grid.theta)
            .map(|(n, t)| n * (t - grid_mean[g]))
            .sum::<f64>();
    }
    (-total, grad, posterior)
}

/// A parameter vector with its NLL and gradient.
type Evaluated = (Vec<f64>, f64, Vec<f64>);

/// Gradient tolerance of a fit, on the NLL per respondent.
const G_TOL: f64 = 1e-5;

/// The floors' priors (`docs/02` §B.1) as a penalty on the negative log-likelihood; its
/// gradient is added into `grad`.
fn penalty(model: &Model, floors: &Floors, p: &[f64], grad: &mut [f64]) -> f64 {
    let mut total = 0.0;
    let listed = floors.anchor_list.iter().chain(&floors.item_list);
    for (s, &(_, chance)) in listed.enumerate() {
        let i = model.floor_idx(s);
        let gm = p[i];
        total += floors.weight * (chance * softplus(-gm) + (1.0 - chance) * softplus(gm));
        grad[i] += floors.weight * (sigmoid(gm) - chance);
    }
    total
}

/// Minimizes the NLL of `model` from `p0`, plus the floors' priors; the optimizer sees the
/// objective per respondent, the NLL returned is the likelihood's total, without the priors.
fn fit_from(
    model: &Model,
    p0: Vec<f64>,
    data: &Data,
    grid: &Grid,
    floors: &Floors,
) -> (f64, Vec<f64>, Convergence) {
    let scale = 1.0 / data.n.max(1) as f64;
    let cache: RefCell<Option<Evaluated>> = RefCell::new(None);
    let eval = |p: &[f64]| -> (f64, Vec<f64>) {
        if let Some((cp, f, g)) = cache.borrow().as_ref() {
            if cp.as_slice() == p {
                return (*f, g.clone());
            }
        }
        let (f, mut g, _) = evaluate(model, p, data, grid, floors, false);
        let f = f + penalty(model, floors, p, &mut g);
        for v in g.iter_mut() {
            *v *= scale;
        }
        let f = f * scale;
        *cache.borrow_mut() = Some((p.to_vec(), f, g.clone()));
        (f, g)
    };
    let m = lbfgs(p0, |p| eval(p).0, |p| eval(p).1, 10, MAX_ITERS, G_TOL);
    let nll = if floors.count > 0 {
        evaluate(model, &m.x, data, grid, floors, false).0
    } else {
        eval(&m.x).0 / scale
    };
    (nll, m.x, m.status)
}

fn start_difficulty(correct: usize, n: usize) -> f64 {
    let p = ((correct as f64 + 0.5) / (n as f64 + 1.0)).clamp(0.02, 0.98);
    (-(ln(p) - ln(1.0 - p))).clamp(-3.0, 3.0)
}

/// [`start_difficulty`] of the share answered correctly above the floor `chance`.
fn start_difficulty_above(correct: usize, n: usize, chance: f64) -> f64 {
    let p = (correct as f64 + 0.5) / (n as f64 + 1.0);
    let above = ((p - chance) / (1.0 - chance)).clamp(0.02, 0.98);
    (-(ln(above) - ln(1.0 - above))).clamp(-3.0, 3.0)
}

/// The D37 target model on a batch: `anchors` (respondents × anchors, 0/1) and `x`
/// (respondents × trial items, 0/1), each column's format in `formats` (D25). Classes
/// (`1..=4`) and uniform vs non-uniform DIF are chosen by BIC (`docs/02` §B.3).
pub fn latent_dif(
    anchors: &[Vec<f64>],
    x: &[Vec<f64>],
    formats: &Formats,
    seed: u64,
) -> Result<LatentDif, BadFormats> {
    latent_dif_with(
        anchors,
        x,
        formats,
        &LatentParams {
            seed,
            ..LatentParams::default()
        },
    )
}

/// [`latent_dif`] with explicit settings; [`BadFormats`] unless `formats` holds one format
/// per anchor and per trial item (of the shortest row).
pub fn latent_dif_with(
    anchors: &[Vec<f64>],
    x: &[Vec<f64>],
    formats: &Formats,
    lp: &LatentParams,
) -> Result<LatentDif, BadFormats> {
    let n = x.len();
    let na = anchors.iter().map(Vec::len).min().unwrap_or(0);
    let k = x.iter().map(Vec::len).min().unwrap_or(0);
    let floors = Floors::new(formats, na, k, lp.floor_weight)?;
    let data = Data::new(anchors, x, na, k, &floors);
    let grid = Grid::new(lp.nodes, lp.theta_max);
    let ln_n = ln(n.max(1) as f64);
    let bic = |model: &Model, nll: f64| 2.0 * nll + model.free_params() as f64 * ln_n;

    // The one-class solution seeds every mixture start.
    let one = Model {
        g: 1,
        na,
        k,
        per_class_a: false,
        floors: floors.count,
    };
    let mut p1 = vec![0.0; one.len()];
    let mut start = |slot: Option<(usize, f64)>, correct: f64| match slot {
        None => start_difficulty(correct as usize, n),
        Some((s, chance)) => {
            p1[one.floor_idx(s)] = ln(chance) - ln(1.0 - chance);
            start_difficulty_above(correct as usize, n, chance)
        }
    };
    let anchor_b: Vec<f64> = (0..na)
        .map(|a| start(floors.anchor[a], data.anchor_count[a]))
        .collect();
    let item_b: Vec<f64> = (0..k)
        .map(|j| start(floors.item[j], data.item_count[j]))
        .collect();
    for a in 0..na {
        p1[one.anchor_a_idx(a)] = 1.0;
        p1[one.anchor_b_idx(a)] = anchor_b[a];
    }
    for j in 0..k {
        p1[one.item_a_idx(0, j)] = 1.0;
        p1[one.item_b_idx(0, j)] = item_b[j];
    }
    let (nll1, p1, st1) = fit_from(&one, p1, &data, &grid, &floors);
    let bic1 = bic(&one, nll1);

    let mut candidates = vec![(1, false, bic1)];
    let mut best = (bic1, one, p1.clone(), st1);
    let mut rng = ChaCha8Rng::seed_from_u64(lp.seed);
    for g in 2..=lp.max_classes.max(1) {
        // Once adding a class no longer lowers the BIC, larger mixtures are not tried.
        let best_before = best.0;
        for per_class_a in [false, true] {
            let model = Model {
                g,
                na,
                k,
                per_class_a,
                floors: floors.count,
            };
            let mut chosen: Option<(f64, Vec<f64>, Convergence)> = None;
            for _ in 0..lp.n_starts.max(1) {
                let mut p0 = vec![0.0; model.len()];
                for c in 1..g {
                    p0[model.zeta_idx(c)] = normal(&mut rng) * 0.3;
                    p0[model.eta_idx(c)] = normal(&mut rng) * 0.3;
                }
                for a in 0..na {
                    p0[model.anchor_a_idx(a)] = p1[one.anchor_a_idx(a)];
                    p0[model.anchor_b_idx(a)] = p1[one.anchor_b_idx(a)];
                }
                for c in 0..g {
                    for j in 0..k {
                        p0[model.item_a_idx(c, j)] = p1[one.item_a_idx(0, j)];
                        p0[model.item_b_idx(c, j)] =
                            p1[one.item_b_idx(0, j)] + normal(&mut rng) * 0.3;
                    }
                }
                for s in 0..floors.count {
                    p0[model.floor_idx(s)] = p1[one.floor_idx(s)];
                }
                let fit = fit_from(&model, p0, &data, &grid, &floors);
                let better = match &chosen {
                    None => true,
                    Some((f, _, s)) => {
                        let (conv, prev_conv) = (
                            fit.2 == Convergence::Converged,
                            *s == Convergence::Converged,
                        );
                        (conv && !prev_conv) || (conv == prev_conv && fit.0 < *f)
                    }
                };
                if better {
                    chosen = Some(fit);
                }
            }
            let (f, p, st) = chosen.expect("at least one start");
            let b = bic(&model, f);
            candidates.push((g, per_class_a, b));
            if st == Convergence::Converged && b < best.0 {
                best = (b, model, p, st);
            }
        }
        if best.0 >= best_before {
            break;
        }
    }

    let (best_bic, model, p, status) = best;
    let pi = model.pi(&p);
    let eta = model.eta(&p);
    let counted: Vec<usize> = (0..model.g).filter(|&g| pi[g] >= MIN_CLASS_SHARE).collect();
    let gap = |idx: &dyn Fn(usize, usize) -> usize, j: usize| -> f64 {
        let vals: Vec<f64> = counted.iter().map(|&g| p[idx(g, j)]).collect();
        let hi = vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let lo = vals.iter().copied().fold(f64::INFINITY, f64::min);
        if vals.len() < 2 {
            0.0
        } else {
            hi - lo
        }
    };
    let dif: Vec<f64> = (0..k)
        .map(|j| gap(&|g, j| model.item_b_idx(g, j), j))
        .collect();
    let a_gap: Vec<f64> = (0..k)
        .map(|j| gap(&|g, j| model.item_a_idx(g, j), j))
        .collect();
    let (_, _, posterior) = evaluate(&model, &p, &data, &grid, &floors, true);
    let floor_of =
        |f: &Option<(usize, f64)>| f.map_or(0.0, |(s, _)| sigmoid(p[model.floor_idx(s)]));

    Ok(LatentDif {
        classes: model.g,
        non_uniform: model.per_class_a,
        pi,
        eta,
        anchor_a: (0..na).map(|a| p[model.anchor_a_idx(a)]).collect(),
        anchor_b: (0..na).map(|a| p[model.anchor_b_idx(a)]).collect(),
        anchor_c: floors.anchor.iter().map(floor_of).collect(),
        item_a: (0..model.g)
            .map(|g| (0..k).map(|j| p[model.item_a_idx(g, j)]).collect())
            .collect(),
        item_b: (0..model.g)
            .map(|g| (0..k).map(|j| p[model.item_b_idx(g, j)]).collect())
            .collect(),
        item_c: floors.item.iter().map(floor_of).collect(),
        dif,
        a_gap,
        posterior: posterior.expect("requested"),
        bic_gain: bic1 - best_bic,
        candidates,
        status,
    })
}

fn normal(rng: &mut ChaCha8Rng) -> f64 {
    let u1: f64 = 1.0 - rng.gen::<f64>();
    let u2: f64 = rng.gen::<f64>();
    (-2.0 * ln(u1)).sqrt() * cos(2.0 * std::f64::consts::PI * u2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optim::numerical_gradient;

    /// The marginal likelihood computed the straightforward way.
    fn reference_nll(
        model: &Model,
        p: &[f64],
        anchors: &[Vec<f64>],
        x: &[Vec<f64>],
        grid: &Grid,
        floors: &Floors,
    ) -> f64 {
        let pi = model.pi(p);
        let eta = model.eta(p);
        let floor =
            |f: Option<(usize, f64)>| f.map_or(0.0, |(s, _)| sigmoid(p[model.floor_idx(s)]));
        let term = |x: f64, c: f64, s: f64| {
            let right = c + (1.0 - c) * s;
            if x >= 0.5 {
                ln(right)
            } else {
                ln(1.0 - right)
            }
        };
        let mut total = 0.0;
        for (i, row) in x.iter().enumerate() {
            let mut lik = 0.0;
            for g in 0..model.g {
                let (lw, _) = grid.log_weights(eta[g]);
                for (qi, &t) in grid.theta.iter().enumerate() {
                    let mut l = ln(pi[g]) + lw[qi];
                    for (a, &xa) in anchors[i].iter().enumerate() {
                        let c = cell(p[model.anchor_a_idx(a)], t, p[model.anchor_b_idx(a)]);
                        l += term(xa, floor(floors.anchor[a]), c.sigma);
                    }
                    for (j, &xj) in row.iter().enumerate() {
                        let c = cell(p[model.item_a_idx(g, j)], t, p[model.item_b_idx(g, j)]);
                        l += term(xj, floor(floors.item[j]), c.sigma);
                    }
                    lik += exp(l);
                }
            }
            total += ln(lik);
        }
        -total
    }

    /// Timing of one evaluation and of single fits on a paper-sized batch (`--ignored`).
    #[test]
    #[ignore]
    fn timing_on_a_paper_batch() {
        use std::time::Instant;
        let mut rng = ChaCha8Rng::seed_from_u64(700);
        let (n, na, k) = (6000, 30, 8);
        let sigmoid = |z: f64| 1.0 / (1.0 + exp(-z));
        let theta: Vec<f64> = (0..n).map(|_| normal(&mut rng)).collect();
        let z: Vec<f64> = (0..n)
            .map(|_| if rng.gen::<bool>() { 1.0 } else { -1.0 })
            .collect();
        let aa: Vec<f64> = (0..na).map(|_| rng.gen_range(0.9..1.6)).collect();
        let ba: Vec<f64> = (0..na).map(|_| normal(&mut rng)).collect();
        let anchors: Vec<Vec<f64>> = theta
            .iter()
            .map(|&t| {
                (0..na)
                    .map(|j| f64::from(rng.gen::<f64>() < sigmoid(aa[j] * (t - ba[j]))))
                    .collect()
            })
            .collect();
        let a: Vec<f64> = (0..k).map(|_| rng.gen_range(1.0..1.5)).collect();
        let b: Vec<f64> = (0..k).map(|_| 0.6 * normal(&mut rng)).collect();
        let x: Vec<Vec<f64>> = theta
            .iter()
            .zip(&z)
            .map(|(&t, &zi)| {
                (0..k)
                    .map(|j| {
                        let d = if j < 2 { 0.9 } else { 0.0 };
                        f64::from(rng.gen::<f64>() < sigmoid(a[j] * (t - b[j] - d * zi)))
                    })
                    .collect()
            })
            .collect();
        let floors =
            Floors::new(&Formats::open(na, k), na, k, FLOOR_PRIOR_WEIGHT).expect("formats");
        let data = Data::new(&anchors, &x, na, k, &floors);
        let grid = Grid::new(41, 5.0);
        let one = Model {
            g: 1,
            na,
            k,
            per_class_a: false,
            floors: 0,
        };
        let p: Vec<f64> = (0..one.len())
            .map(|i| if i < na { 1.0 } else { 0.0 })
            .collect();
        let t0 = Instant::now();
        for _ in 0..10 {
            evaluate(&one, &p, &data, &grid, &floors, false);
        }
        println!(
            "one evaluation (G = 1): {:.1} ms",
            t0.elapsed().as_secs_f64() * 100.0
        );
        let mut p1 = vec![0.0; one.len()];
        for a in 0..na {
            p1[one.anchor_a_idx(a)] = 1.0;
        }
        for j in 0..k {
            p1[one.item_a_idx(0, j)] = 1.0;
        }
        let t0 = Instant::now();
        let (nll1, p1, st) = fit_from(&one, p1, &data, &grid, &floors);
        println!(
            "G = 1 fit: {:.2}s, nll {nll1:.1}, {st:?}",
            t0.elapsed().as_secs_f64()
        );
        let two = Model {
            g: 2,
            na,
            k,
            per_class_a: false,
            floors: 0,
        };
        let mut p0 = vec![0.0; two.len()];
        p0[two.eta_idx(1)] = 0.1;
        for a in 0..na {
            p0[two.anchor_a_idx(a)] = p1[one.anchor_a_idx(a)];
            p0[two.anchor_b_idx(a)] = p1[one.anchor_b_idx(a)];
        }
        for c in 0..2 {
            for j in 0..k {
                p0[two.item_a_idx(c, j)] = p1[one.item_a_idx(0, j)];
                p0[two.item_b_idx(c, j)] =
                    p1[one.item_b_idx(0, j)] + if c == 0 { 0.3 } else { -0.3 };
            }
        }
        let t0 = Instant::now();
        let (nll2, p2, st) = fit_from(&two, p0, &data, &grid, &floors);
        let gaps: Vec<f64> = (0..k)
            .map(|j| (p2[two.item_b_idx(1, j)] - p2[two.item_b_idx(0, j)]).abs())
            .collect();
        println!(
            "G = 2 fit: {:.2}s, nll {nll2:.1}, {st:?}, gaps {gaps:?}",
            t0.elapsed().as_secs_f64()
        );
    }

    /// The BIC penalty counts the free parameters of each model shape, floors included.
    #[test]
    fn free_parameters_are_counted_as_specified() {
        let (na, k) = (5, 8);
        for g in 1..=4 {
            for floors in [0, 3] {
                let shared = Model {
                    g,
                    na,
                    k,
                    per_class_a: false,
                    floors,
                };
                let per_class = Model {
                    g,
                    na,
                    k,
                    per_class_a: true,
                    floors,
                };
                assert_eq!(
                    shared.free_params(),
                    2 * (g - 1) + 2 * na + k + k * g + floors,
                    "G={g} shared, {floors} floors"
                );
                assert_eq!(
                    per_class.free_params(),
                    2 * (g - 1) + 2 * na + 2 * k * g + floors,
                    "G={g} per-class, {floors} floors"
                );
            }
        }
    }

    /// The fused NLL and analytic gradient match the reference likelihood and central differences.
    #[test]
    fn nll_and_gradient_match_the_reference() {
        let mut rng = ChaCha8Rng::seed_from_u64(54);
        let (n, na, k) = (30, 3, 4);
        let bit = |rng: &mut ChaCha8Rng| f64::from(rng.gen::<f64>() < 0.5);
        let anchors: Vec<Vec<f64>> = (0..n)
            .map(|_| (0..na).map(|_| bit(&mut rng)).collect())
            .collect();
        let x: Vec<Vec<f64>> = (0..n)
            .map(|_| (0..k).map(|_| bit(&mut rng)).collect())
            .collect();
        let grid = Grid::new(11, 4.0);
        let (open, choice) = (Format::Open, Format::Choice(4));
        let mixed = Formats {
            anchors: vec![open, choice, open],
            items: vec![Format::Choice(2), open, choice, open],
        };
        for formats in [Formats::open(na, k), mixed, Formats::choice(na, k, 5)] {
            let floors = Floors::new(&formats, na, k, FLOOR_PRIOR_WEIGHT).expect("formats");
            let data = Data::new(&anchors, &x, na, k, &floors);
            for g in 1..=3 {
                for per_class_a in [false, true] {
                    let model = Model {
                        g,
                        na,
                        k,
                        per_class_a,
                        floors: floors.count,
                    };
                    let p: Vec<f64> = (0..model.len()).map(|_| normal(&mut rng) * 0.7).collect();
                    let (fused, analytic, _) = evaluate(&model, &p, &data, &grid, &floors, false);
                    let reference = reference_nll(&model, &p, &anchors, &x, &grid, &floors);
                    assert!(
                        (fused - reference).abs() <= 1e-9 * reference.abs(),
                        "g={g}, {} floors: NLL {fused} vs {reference}",
                        floors.count
                    );
                    let objective = |q: &[f64]| {
                        let (f, mut grad, _) = evaluate(&model, q, &data, &grid, &floors, false);
                        (f + penalty(&model, &floors, q, &mut grad), grad)
                    };
                    let numeric = numerical_gradient(&|q: &[f64]| objective(q).0, &p, 1e-6);
                    for (i, (a, b)) in objective(&p).1.iter().zip(&numeric).enumerate() {
                        assert!(
                            (a - b).abs() <= 1e-5 * (1.0 + b.abs()),
                            "g={g} per_class_a={per_class_a} {} floors param {i}: {a} vs {b}",
                            floors.count
                        );
                    }
                    assert_eq!(analytic.len(), model.len());
                }
            }
        }
    }

    /// The floor's helpers stay finite and exact at extreme arguments, where one branch overflows.
    #[test]
    fn the_floor_helpers_are_stable_at_extreme_arguments() {
        assert_eq!(softplus(800.0), 800.0);
        assert_eq!(softplus(-800.0), 0.0);
        assert_eq!(softplus(0.0), ln(2.0));
        assert!((softplus(1.5) - ln_1p(exp(1.5))).abs() < 1e-15);
        assert!((softplus(-1.5) - ln_1p(exp(-1.5))).abs() < 1e-15);
        assert_eq!(sigmoid(800.0), 1.0);
        assert_eq!(sigmoid(-800.0), 0.0);
        assert_eq!(sigmoid(0.0), 0.5);
        let mid = cell(1.2, 0.5, 0.0);
        let (floor, s) = (sigmoid(-1.4), mid.sigma);
        let want = ln(floor + (1.0 - floor) * s) - ln((1.0 - floor) * (1.0 - s));
        assert!((log_odds(&mid, -1.4) - want).abs() < 1e-12);
        let limits = [
            (40.0, -1.4, 48.0 + ln_1p(exp(-1.4))),
            (-40.0, -1.4, -1.4),
            (0.0, 30.0, 30.0 + ln(2.0)),
        ];
        for (theta, gamma, want) in limits {
            let got = log_odds(&cell(1.2, theta, 0.0), gamma);
            assert!(
                (got - want).abs() < 1e-9,
                "θ {theta}, γ {gamma}: {got} vs {want}"
            );
        }
    }

    /// Formats are refused unless one per anchor and item, each open or with two options or more.
    #[test]
    fn formats_that_do_not_describe_the_batch_are_refused() {
        let ok = |f: &Formats| Floors::new(f, 2, 3, FLOOR_PRIOR_WEIGHT).is_ok();
        assert!(ok(&Formats::open(2, 3)));
        assert!(ok(&Formats::choice(2, 3, 2)));
        assert!(!ok(&Formats::open(1, 3)));
        assert!(!ok(&Formats::open(2, 4)));
        assert!(!ok(&Formats::choice(2, 3, 1)));
        assert!(!ok(&Formats::choice(2, 3, 0)));
        let floors = Floors::new(&Formats::choice(2, 3, 4), 2, 3, FLOOR_PRIOR_WEIGHT).expect("ok");
        assert_eq!(floors.count, 5);
        assert_eq!(floors.anchor[1], Some((1, 0.25)));
        assert_eq!(floors.item[0], Some((2, 0.25)));
    }
}
