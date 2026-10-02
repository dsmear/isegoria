//! Level A — bridging (`docs/02-scoring-engine.md` §A): matrix factorization `r̂_uj = μ +
//! b_u + b_j + ⟨f_u, f_j⟩` (`λ_b ≫ λ_f`). The bridge score is the side-balanced predicted
//! approval ([`side_balanced`], D32), never the majoritarian `b_j` (`docs/08` BRIDGE-008/009).

use crate::fmath::{cos, ln};
use crate::optim::{lbfgs, Convergence};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[derive(Clone, Copy, Debug)]
pub struct Obs {
    pub u: usize,
    pub j: usize,
    pub r: f64,
}

#[derive(Clone, Debug)]
pub struct Ratings {
    pub n: usize,
    pub m: usize,
    pub obs: Vec<Obs>,
    /// Per-reviewer weight `w_u`, length `n`; uniform (1.0) by default (docs/08 BRIDGE-007,
    /// G-03): the reviewer's `discount(cap(E_u))` (probation = 0) from the previous epoch.
    pub weights: Vec<f64>,
    /// Per-reviewer flag, length `n`: whether the reviewer may define the latent axis
    /// (`docs/02` §A.4, T39). Only a row with the flag and a positive weight takes part in the
    /// collective fit; the others are placed on it afterwards and enter nothing collective (A7).
    pub axis: Vec<bool>,
}

impl Ratings {
    /// Observations in canonical `(u, j)` order ([`Ratings::canonical`]), uniform weights.
    pub fn from_dense(r: &[Vec<f64>], mask: &[Vec<bool>]) -> Self {
        let n = r.len();
        let m = if n > 0 { r[0].len() } else { 0 };
        let mut obs = Vec::new();
        for u in 0..n {
            for j in 0..m {
                if mask[u][j] {
                    obs.push(Obs { u, j, r: r[u][j] });
                }
            }
        }
        Ratings {
            n,
            m,
            obs,
            weights: vec![1.0; n],
            axis: vec![true; n],
        }
    }

    /// Sets which reviewers define the latent axis (`docs/02` §A.4, T39): `false` keeps the
    /// reviewer out of the core fit, placing it on the fixed axis afterwards. A vector of
    /// another length is refused by [`Ratings::validate`] (`RatingsError::AxisCount`).
    pub fn with_axis(mut self, axis: Vec<bool>) -> Self {
        self.axis = axis;
        self
    }

    /// Sets the per-reviewer weights `w_u` (docs/08 BRIDGE-007); a vector of another length
    /// is refused by [`Ratings::validate`] at the fit (`RatingsError::WeightCount`), not here.
    pub fn with_weights(mut self, weights: Vec<f64>) -> Self {
        self.weights = weights;
        self
    }

    /// Checks the input the fit relies on (T62, `docs/12` §2.3): finite non-negative weights,
    /// observations inside `[0, n) × [0, m)` with finite ratings, no `(u, j)` pair observed
    /// twice. [`fit`] and [`bridge_scores`] run it first; the first problem found is reported.
    pub fn validate(&self) -> Result<(), RatingsError> {
        if self.weights.len() != self.n {
            return Err(RatingsError::WeightCount {
                expected: self.n,
                found: self.weights.len(),
            });
        }
        if let Some(u) = self.weights.iter().position(|w| !w.is_finite() || *w < 0.0) {
            return Err(RatingsError::BadWeight { u });
        }
        if self.axis.len() != self.n {
            return Err(RatingsError::AxisCount {
                expected: self.n,
                found: self.axis.len(),
            });
        }
        for o in &self.obs {
            if o.u >= self.n || o.j >= self.m {
                return Err(RatingsError::IndexOutOfRange {
                    u: o.u,
                    j: o.j,
                    n: self.n,
                    m: self.m,
                });
            }
            if !o.r.is_finite() {
                return Err(RatingsError::NonFiniteRating { u: o.u, j: o.j });
            }
        }
        let mut pairs: Vec<(usize, usize)> = self.obs.iter().map(|o| (o.u, o.j)).collect();
        pairs.sort_unstable();
        if let Some(w) = pairs.windows(2).find(|w| w[0] == w[1]) {
            return Err(RatingsError::DuplicateObservation {
                u: w[0].0,
                j: w[0].1,
            });
        }
        Ok(())
    }

    /// A copy with `obs` in canonical order (sorted by `(u, j)`, then rating bits), so the
    /// fit is invariant to input order (docs/08 INV-13, REPRO-002); weights are unaffected.
    pub fn canonical(&self) -> Ratings {
        let mut obs = self.obs.clone();
        obs.sort_by(|a, b| (a.u, a.j, a.r.to_bits()).cmp(&(b.u, b.j, b.r.to_bits())));
        Ratings {
            n: self.n,
            m: self.m,
            obs,
            weights: self.weights.clone(),
            axis: self.axis.clone(),
        }
    }
}

/// Why a [`Ratings`] cannot be fitted (T62): an error, never a panic inside the objective.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RatingsError {
    IndexOutOfRange {
        u: usize,
        j: usize,
        n: usize,
        m: usize,
    },
    /// `weights` does not hold one weight per reviewer.
    WeightCount { expected: usize, found: usize },
    /// `axis` does not hold one flag per reviewer (T39).
    AxisCount { expected: usize, found: usize },
    /// A rating that is not a finite number.
    NonFiniteRating { u: usize, j: usize },
    /// A weight that is not a finite, non-negative number.
    BadWeight { u: usize },
    /// The same `(u, j)` pair observed twice.
    DuplicateObservation { u: usize, j: usize },
    /// A requested item index outside `[0, m)`.
    ItemOutOfRange { j: usize, m: usize },
}

impl std::fmt::Display for RatingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RatingsError::IndexOutOfRange { u, j, n, m } => {
                write!(
                    f,
                    "observation ({u}, {j}) outside {n} reviewers × {m} items"
                )
            }
            RatingsError::WeightCount { expected, found } => {
                write!(f, "{found} weights for {expected} reviewers")
            }
            RatingsError::AxisCount { expected, found } => {
                write!(f, "{found} axis flags for {expected} reviewers")
            }
            RatingsError::NonFiniteRating { u, j } => write!(f, "rating ({u}, {j}) is not finite"),
            RatingsError::BadWeight { u } => {
                write!(
                    f,
                    "weight of reviewer {u} is not a finite, non-negative number"
                )
            }
            RatingsError::DuplicateObservation { u, j } => {
                write!(f, "observation ({u}, {j}) appears twice")
            }
            RatingsError::ItemOutOfRange { j, m } => write!(f, "item {j} outside {m} items"),
        }
    }
}

impl std::error::Error for RatingsError {}

#[derive(Clone, Copy, Debug)]
pub struct BridgingParams {
    pub lam_b: f64,
    pub lam_f: f64,
    pub m_hist: usize,
    pub max_iters: usize,
    pub g_tol: f64,
    pub seed: u64,
    /// Seeded starts of the non-convex fit (T48); the lowest objective wins (1 = a single fit).
    pub n_starts: usize,
}

impl Default for BridgingParams {
    fn default() -> Self {
        BridgingParams {
            lam_b: 0.15,
            lam_f: 0.03,
            m_hist: 10,
            max_iters: 4000,
            g_tol: 1e-7,
            seed: 0,
            n_starts: DEFAULT_STARTS,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Fit {
    pub mu: f64,
    pub b_u: Vec<f64>,
    pub b_j: Vec<f64>,
    pub f_u: Vec<f64>,
    pub f_j: Vec<f64>,
    /// Per row, whether it took part in the collective fit (`Ratings::axis`, positive weight).
    pub participant: Vec<bool>,
    /// Convergence of the L-BFGS fit (docs/08 OPT-001).
    pub status: Convergence,
}

// Parameter vector layout: [ μ | b_u(n) | b_j(m) | f_u(n) | f_j(m) ].
struct Layout {
    n: usize,
    m: usize,
}
impl Layout {
    #[inline]
    fn mu(&self, x: &[f64]) -> f64 {
        x[0]
    }
    #[inline]
    fn bu<'a>(&self, x: &'a [f64]) -> &'a [f64] {
        &x[1..1 + self.n]
    }
    #[inline]
    fn bj<'a>(&self, x: &'a [f64]) -> &'a [f64] {
        &x[1 + self.n..1 + self.n + self.m]
    }
    #[inline]
    fn fu<'a>(&self, x: &'a [f64]) -> &'a [f64] {
        &x[1 + self.n + self.m..1 + 2 * self.n + self.m]
    }
    #[inline]
    fn fj<'a>(&self, x: &'a [f64]) -> &'a [f64] {
        &x[1 + 2 * self.n + self.m..]
    }
}

/// Default number of starts (T48): see `docs/08` BRIDGE-001 for the measurements.
pub const DEFAULT_STARTS: usize = 8;

/// The bridging fit: non-convex, with distinct local minima that a single seeded start can
/// land in on either side of `τ` (`docs/08` BRIDGE-001, T48). Runs `n_starts` seeded starts
/// and keeps the lowest objective (earliest on a tie). Malformed input errors (T62).
pub fn fit(data: &Ratings, p: &BridgingParams) -> Result<Fit, RatingsError> {
    data.validate()?;
    Ok(fit_validated(data, p))
}

/// [`fit`] on input [`Ratings::validate`] has accepted.
fn fit_validated(data: &Ratings, p: &BridgingParams) -> Fit {
    // Canonicalize: the init mean and cost/grad sums are order-dependent (INV-13).
    let data = data.canonical();
    let core = Core::of(&data);
    core.expand(&data, p, &fit_core(&core.data, p))
}

/// Whether row `u` takes part in the collective fit (`docs/15` A7): axis and a positive weight.
fn participates(data: &Ratings, u: usize) -> bool {
    data.axis.get(u).copied().unwrap_or(false) && data.weights.get(u).is_some_and(|w| *w > 0.0)
}

/// The collective fit's own problem: the participants renumbered in row order, with only their
/// observations; `rows[k]` is participant `k`'s row in the input. Nobody else enters it.
struct Core {
    rows: Vec<usize>,
    data: Ratings,
}

impl Core {
    fn of(data: &Ratings) -> Core {
        let rows: Vec<usize> = (0..data.n).filter(|&u| participates(data, u)).collect();
        let mut index = vec![None; data.n];
        for (k, &u) in rows.iter().enumerate() {
            index[u] = Some(k);
        }
        let obs = data
            .obs
            .iter()
            .filter_map(|o| index[o.u].map(|k| Obs { u: k, ..*o }))
            .collect();
        Core {
            data: Ratings {
                n: rows.len(),
                m: data.m,
                obs,
                weights: rows.iter().map(|&u| data.weights[u]).collect(),
                axis: vec![true; rows.len()],
            },
            rows,
        }
    }

    /// The core's fit on every row of `data`: participants at their own parameters, the others
    /// placed on the fixed axis by [`project`].
    fn expand(&self, data: &Ratings, p: &BridgingParams, core: &Fit) -> Fit {
        let mut f = Fit {
            mu: core.mu,
            b_u: vec![0.0; data.n],
            b_j: core.b_j.clone(),
            f_u: vec![0.0; data.n],
            f_j: core.f_j.clone(),
            participant: vec![false; data.n],
            status: core.status,
        };
        for (k, &u) in self.rows.iter().enumerate() {
            f.b_u[u] = core.b_u[k];
            f.f_u[u] = core.f_u[k];
            f.participant[u] = true;
        }
        for u in (0..data.n).filter(|&u| !f.participant[u]) {
            (f.b_u[u], f.f_u[u]) = project(data, p, core, u);
        }
        f
    }
}

/// The multi-start fit of a [`Core`]'s ratings (T48), in the canonical sign.
fn fit_core(core: &Ratings, p: &BridgingParams) -> Fit {
    let mut best: Option<(f64, Fit)> = None;
    for k in 0..p.n_starts.max(1) {
        let x0 = random_init(core, p.seed.wrapping_add(k as u64));
        let f = fit_with_init(core, p, x0);
        let obj = objective(core, p, &pack(&f));
        if best.as_ref().is_none_or(|(b, _)| obj < *b) {
            best = Some((obj, f));
        }
    }
    let mut f = best.expect("at least one start").1;
    canonical_sign(&mut f);
    f
}

/// The position of a row outside the collective fit (T39, A7): its `(b_u, f_u)` on the fixed axis
/// `(μ, b_j, f_j)` of the core fit, by ridge least squares over its own ratings (same
/// `λ_b`, `λ_f` as the fit); no influence on anyone else, the origin without ratings.
fn project(data: &Ratings, p: &BridgingParams, f: &Fit, u: usize) -> (f64, f64) {
    let (mut s1, mut sx, mut sxx, mut sy, mut sxy) = (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64);
    for o in data.obs.iter().filter(|o| o.u == u) {
        let x = f.f_j[o.j];
        let y = o.r - f.mu - f.b_j[o.j];
        s1 += 1.0;
        sx += x;
        sxx += x * x;
        sy += y;
        sxy += x * y;
    }
    let (a11, a12, a22) = (s1 + p.lam_b, sx, sxx + p.lam_f);
    let det = a11 * a22 - a12 * a12;
    if det.is_nan() || det <= 0.0 {
        return (0.0, 0.0);
    }
    ((a22 * sy - a12 * sxy) / det, (a11 * sxy - a12 * sy) / det)
}

/// `f` is identified only up to sign (the objective is unchanged by `(f_u, f_j) →
/// (−f_u, −f_j)`); the sign is fixed by making the largest `|f_j|` (earliest on a tie)
/// non-negative. Exact negation: `b_j` and the objective are untouched (T48).
fn canonical_sign(f: &mut Fit) {
    let lead = f.f_j.iter().copied().fold(
        0.0_f64,
        |best, v| if v.abs() > best.abs() { v } else { best },
    );
    if lead < 0.0 {
        for v in f.f_u.iter_mut().chain(f.f_j.iter_mut()) {
            *v = -*v;
        }
    }
}

// RNG consumption order is part of the reproducibility contract.
fn random_init(data: &Ratings, seed: u64) -> Vec<f64> {
    let (n, m) = (data.n, data.m);
    // Weighted like the objective, so a zero-weight (probation) reviewer cannot move the
    // start point either (T42); bit-identical to the plain mean when weights are uniform.
    let (sum_wr, sum_w) = data.obs.iter().fold((0.0, 0.0), |(swr, sw), o| {
        let w = data.weights[o.u];
        (swr + w * o.r, sw + w)
    });
    let mean_r = if sum_w > 0.0 { sum_wr / sum_w } else { 0.0 };
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut x0 = vec![0.0_f64; 1 + 2 * n + 2 * m];
    x0[0] = mean_r;
    for i in 0..n {
        x0[1 + i] = normal(&mut rng) * 0.1;
    }
    for j in 0..m {
        x0[1 + n + j] = normal(&mut rng) * 0.1;
    }
    for i in 0..n {
        x0[1 + n + m + i] = normal(&mut rng) * 0.3;
    }
    for j in 0..m {
        x0[1 + 2 * n + m + j] = normal(&mut rng) * 0.3;
    }
    x0
}

fn pack(f: &Fit) -> Vec<f64> {
    let mut x = Vec::with_capacity(1 + 2 * f.b_u.len() + 2 * f.b_j.len());
    x.push(f.mu);
    x.extend_from_slice(&f.b_u);
    x.extend_from_slice(&f.b_j);
    x.extend_from_slice(&f.f_u);
    x.extend_from_slice(&f.f_j);
    x
}

// The weighted objective `Σ w_u (r − r̂)² + λ_b‖b‖² + λ_f‖f‖²` (docs/08 BRIDGE-007).
fn objective(data: &Ratings, p: &BridgingParams, x: &[f64]) -> f64 {
    let lay = Layout {
        n: data.n,
        m: data.m,
    };
    let w = &data.weights;
    let mu = lay.mu(x);
    let (bu, bj, fu, fj) = (lay.bu(x), lay.bj(x), lay.fu(x), lay.fj(x));
    let mut se = 0.0;
    for o in &data.obs {
        let pred = mu + bu[o.u] + bj[o.j] + fu[o.u] * fj[o.j];
        let e = pred - o.r;
        se += w[o.u] * e * e;
    }
    let reg_b: f64 = bu.iter().map(|v| v * v).sum::<f64>() + bj.iter().map(|v| v * v).sum::<f64>();
    let reg_f: f64 = fu.iter().map(|v| v * v).sum::<f64>() + fj.iter().map(|v| v * v).sum::<f64>();
    se + p.lam_b * reg_b + p.lam_f * reg_f
}

// Analytic gradient of [`objective`]; pinned against central differences in the tests.
fn gradient(data: &Ratings, p: &BridgingParams, x: &[f64]) -> Vec<f64> {
    let (n, m) = (data.n, data.m);
    let lay = Layout { n, m };
    let w = &data.weights;
    let mu = lay.mu(x);
    let (bu, bj, fu, fj) = (lay.bu(x), lay.bj(x), lay.fu(x), lay.fj(x));
    let mut g = vec![0.0_f64; x.len()];
    for o in &data.obs {
        let pred = mu + bu[o.u] + bj[o.j] + fu[o.u] * fj[o.j];
        let e = 2.0 * w[o.u] * (pred - o.r);
        g[0] += e;
        g[1 + o.u] += e;
        g[1 + n + o.j] += e;
        g[1 + n + m + o.u] += e * fj[o.j];
        g[1 + 2 * n + m + o.j] += e * fu[o.u];
    }
    for i in 0..n {
        g[1 + i] += 2.0 * p.lam_b * bu[i];
        g[1 + n + m + i] += 2.0 * p.lam_f * fu[i];
    }
    for j in 0..m {
        g[1 + n + j] += 2.0 * p.lam_b * bj[j];
        g[1 + 2 * n + m + j] += 2.0 * p.lam_f * fj[j];
    }
    g
}

fn fit_with_init(data: &Ratings, p: &BridgingParams, x0: Vec<f64>) -> Fit {
    let lay = Layout {
        n: data.n,
        m: data.m,
    };
    let cost = |x: &[f64]| objective(data, p, x);
    let grad = |x: &[f64]| gradient(data, p, x);

    let m = lbfgs(x0, cost, grad, p.m_hist, p.max_iters, p.g_tol);
    let x = m.x;

    Fit {
        mu: lay.mu(&x),
        b_u: lay.bu(&x).to_vec(),
        b_j: lay.bj(&x).to_vec(),
        f_u: lay.fu(&x).to_vec(),
        f_j: lay.fj(&x).to_vec(),
        participant: vec![true; data.n],
        status: m.status,
    }
}

/// Which of the two sides a reviewer falls on (D32, D42): the exact 1-D 2-means of `f_u`,
/// side A the lower positions under the canonical sign of `f` (T48); the score is symmetric.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    A,
    B,
}

/// The side-balanced bridge score (`docs/02` §A.3, D32, T49, D42): the participants split by
/// [`two_means`] on `f_u`, predicted ratings clipped to `[0, 1]` and averaged within each
/// side, the score the mean of the two — each side counts once whatever its size.
#[derive(Clone, Debug, PartialEq)]
pub struct SideScores {
    pub side: Vec<Side>,
    /// Mean clipped prediction over side A, per item.
    pub side_a: Vec<f64>,
    /// Mean clipped prediction over side B, per item.
    pub side_b: Vec<f64>,
    pub score: Vec<f64>,
    pub gap: Vec<f64>,
}

/// Each side's floor, in thousandths of the reviewers split (D42; provisional, T25).
pub const MIN_SIDE_PER_MILLE: usize = 50;

/// The fewest of `n` reviewers a side of [`two_means`] holds: rounded up, at least 1, at most n/2.
pub fn side_floor(n: usize) -> usize {
    (n * MIN_SIDE_PER_MILLE).div_ceil(1000).max(1).min(n / 2)
}

/// The exact 1-D 2-means of `f_u` (D42): of the cuts of the sorted positions that split no run
/// of equal values and leave [`side_floor`] reviewers on each side, the one with the largest
/// between-side sum of squares (a tie: nearer the middle, then lower). No such cut: all A.
pub fn two_means(f_u: &[f64]) -> Vec<Side> {
    let n = f_u.len();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| f_u[a].total_cmp(&f_u[b]).then(a.cmp(&b)));
    let sorted: Vec<f64> = order.iter().map(|&u| f_u[u]).collect();
    // Summed from each end, so negating `f_u` maps a cut's side sums onto the mirror cut's.
    let mut low = vec![0.0_f64; n + 1];
    for k in 0..n {
        low[k + 1] = low[k] + sorted[k];
    }
    let mut high = vec![0.0_f64; n + 1];
    for k in (0..n).rev() {
        high[k] = high[k + 1] + sorted[k];
    }
    let floor = side_floor(n).max(1);
    let mut best: Option<(f64, usize)> = None;
    for k in floor..=n.saturating_sub(floor) {
        if sorted[k - 1].partial_cmp(&sorted[k]) != Some(std::cmp::Ordering::Less) {
            continue;
        }
        let (na, nb) = (k as f64, (n - k) as f64);
        let d = nb * low[k] - na * high[k];
        let between = d * d / (na * nb);
        let better = best.is_none_or(|(b, c)| {
            between > b || (between == b && k.abs_diff(n - k) < c.abs_diff(n - c))
        });
        if better {
            best = Some((between, k));
        }
    }
    let cut = best.map_or(n, |(_, k)| k);
    let mut side = vec![Side::B; n];
    for &u in &order[..cut] {
        side[u] = Side::A;
    }
    side
}

/// The side-balanced score of every item of a fit (see [`SideScores`]).
pub fn side_balanced(fit: &Fit) -> SideScores {
    let (n, m) = (fit.b_u.len(), fit.b_j.len());
    let on_axis: Vec<usize> = (0..n)
        .filter(|&u| fit.participant.get(u).copied().unwrap_or(true))
        .collect();
    let side = if on_axis.len() == n {
        two_means(&fit.f_u)
    } else {
        // Other rows take the nearer side's label (a tie to A) and count in no average.
        let f_axis: Vec<f64> = on_axis.iter().map(|&u| fit.f_u[u]).collect();
        let axis_sides = two_means(&f_axis);
        let (mut sum, mut count) = ([0.0_f64; 2], [0usize; 2]);
        for (s, &f) in axis_sides.iter().zip(&f_axis) {
            sum[*s as usize] += f;
            count[*s as usize] += 1;
        }
        let centre = |k: usize| {
            if count[k] > 0 {
                sum[k] / count[k] as f64
            } else {
                0.0
            }
        };
        let (ca, cb) = (centre(0), centre(1));
        let mut axis_side = axis_sides.iter();
        (0..n)
            .map(|u| {
                if fit.participant[u] {
                    *axis_side.next().expect("one side per participant")
                } else if (fit.f_u[u] - ca).abs() <= (fit.f_u[u] - cb).abs() {
                    Side::A
                } else {
                    Side::B
                }
            })
            .collect()
    };
    let n_a = on_axis.iter().filter(|&&u| side[u] == Side::A).count();
    let n_b = on_axis.len() - n_a;
    let mut out = SideScores {
        side,
        side_a: Vec::with_capacity(m),
        side_b: Vec::with_capacity(m),
        score: Vec::with_capacity(m),
        gap: Vec::with_capacity(m),
    };
    for j in 0..m {
        let (mut sum_a, mut sum_b) = (0.0_f64, 0.0_f64);
        for &u in &on_axis {
            let pred = (fit.mu + fit.b_u[u] + fit.b_j[j] + fit.f_u[u] * fit.f_j[j]).clamp(0.0, 1.0);
            match out.side[u] {
                Side::A => sum_a += pred,
                Side::B => sum_b += pred,
            }
        }
        let (a, b) = match (n_a, n_b) {
            (0, 0) => {
                let neutral = (fit.mu + fit.b_j[j]).clamp(0.0, 1.0);
                (neutral, neutral)
            }
            (0, _) => {
                let all = sum_b / n_b as f64;
                (all, all)
            }
            (_, 0) => {
                let all = sum_a / n_a as f64;
                (all, all)
            }
            _ => (sum_a / n_a as f64, sum_b / n_b as f64),
        };
        out.side_a.push(a);
        out.side_b.push(b);
        out.score.push((a + b) / 2.0);
        out.gap.push((a - b).abs());
    }
    out
}

/// Per item, the ratings from its less-rated side (D42): only participants count and seat a side
/// (`axis`, positive weight, A7); a side none sits on is left out, indices outside are ignored.
pub fn coverage(data: &Ratings, sides: &SideScores) -> Vec<usize> {
    let mut seated = [false; 2];
    for (u, s) in sides.side.iter().enumerate() {
        if participates(data, u) {
            seated[*s as usize] = true;
        }
    }
    let mut rated = vec![[0usize; 2]; data.m];
    for o in &data.obs {
        if let (true, Some(s), Some(r)) = (
            participates(data, o.u),
            sides.side.get(o.u),
            rated.get_mut(o.j),
        ) {
            r[*s as usize] += 1;
        }
    }
    rated
        .iter()
        .map(|r| match seated {
            [true, true] => r[0].min(r[1]),
            [true, false] => r[0],
            [false, true] => r[1],
            [false, false] => 0,
        })
        .collect()
}

/// The gate's inputs per item (`docs/02` §A.3–A.4, D32, D42): the bootstrap-min `robust` read
/// against `τ`, the `full` fit's side scores (the appeal reads its `gap`) and its [`coverage`].
#[derive(Clone, Debug, PartialEq)]
pub struct BridgeScores {
    pub robust: Vec<f64>,
    pub full: SideScores,
    pub coverage: Vec<usize>,
}

/// Robust bridge score (`docs/02` §A.4): the bootstrap-min of the side-balanced score over
/// `n_bootstrap` subsamples (each observation kept with probability `keep_frac`), with the
/// full fit's side scores (see [`BridgeScores`]). Malformed input errors (T62).
pub fn bridge_scores(
    data: &Ratings,
    p: &BridgingParams,
    n_bootstrap: usize,
    keep_frac: f64,
) -> Result<BridgeScores, RatingsError> {
    data.validate()?;
    // Canonicalize: the bootstrap subsampling walks `obs` in order (INV-13, REPRO-002).
    let data = data.canonical();

    // Warm-start each subsample from the full fit: the bilinear term makes the objective
    // non-convex, so independent random inits could land in a different minimum.
    let core = Core::of(&data);
    let fitted = fit_core(&core.data, p);
    let anchor = pack(&fitted);
    let full_sides = side_balanced(&core.expand(&data, p, &fitted));

    let mut robust = full_sides.score.clone();
    for s in 0..n_bootstrap {
        let mut rng = ChaCha8Rng::seed_from_u64(p.seed.wrapping_add(100 + s as u64));
        // Only the participants' observations and parameters draw from the stream (A7).
        let sub_obs: Vec<Obs> = core
            .data
            .obs
            .iter()
            .copied()
            .filter(|_| rng.gen::<f64>() < keep_frac)
            .collect();
        let sub = Ratings {
            obs: sub_obs,
            ..core.data.clone()
        };
        let mut x0 = anchor.clone();
        for v in x0.iter_mut() {
            *v += normal(&mut rng) * 0.02;
        }
        let fit_s = fit_with_init(&sub, p, x0);
        for (b, &sj) in robust.iter_mut().zip(side_balanced(&fit_s).score.iter()) {
            if sj < *b {
                *b = sj;
            }
        }
    }
    Ok(BridgeScores {
        robust,
        coverage: coverage(&data, &full_sides),
        full: full_sides,
    })
}

// Standard normal via Box–Muller, to control exact RNG consumption order.
fn normal(rng: &mut ChaCha8Rng) -> f64 {
    let u1: f64 = 1.0 - rng.gen::<f64>();
    let u2: f64 = rng.gen::<f64>();
    (-2.0 * ln(u1)).sqrt() * cos(2.0 * std::f64::consts::PI * u2)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> (Ratings, BridgingParams) {
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        let (n, m) = (6, 5);
        let r: Vec<Vec<f64>> = (0..n)
            .map(|_| (0..m).map(|_| rng.gen::<f64>()).collect())
            .collect();
        let mask: Vec<Vec<bool>> = (0..n)
            .map(|u| (0..m).map(|j| (u + 2 * j) % 4 != 0).collect())
            .collect();
        let data = Ratings::from_dense(&r, &mask).with_weights(vec![1.0, 0.5, 2.0, 0.0, 1.5, 0.8]);
        (data, BridgingParams::default())
    }

    /// The analytic gradient matches central differences in every component (T41).
    #[test]
    fn gradient_matches_central_differences() {
        let (data, p) = sample();
        let dim = 1 + 2 * data.n + 2 * data.m;
        let mut rng = ChaCha8Rng::seed_from_u64(11);
        for _ in 0..5 {
            let x: Vec<f64> = (0..dim).map(|_| normal(&mut rng)).collect();
            let g = gradient(&data, &p, &x);
            let h = 1e-6;
            for k in 0..dim {
                let (mut xp, mut xm) = (x.clone(), x.clone());
                xp[k] += h;
                xm[k] -= h;
                let num = (objective(&data, &p, &xp) - objective(&data, &p, &xm)) / (2.0 * h);
                assert!(
                    (g[k] - num).abs() <= 1e-6 * (1.0 + num.abs()),
                    "component {k}: analytic {} vs numerical {num}",
                    g[k]
                );
            }
        }
    }

    use proptest::prelude::*;

    fn data_and_point() -> impl Strategy<Value = (Ratings, Vec<f64>)> {
        (2usize..7, 2usize..5).prop_flat_map(|(n, m)| {
            (
                prop::collection::vec(prop::collection::vec(0.0f64..=1.0, m), n),
                prop::collection::vec(prop::collection::vec(prop::bool::weighted(0.8), m), n),
                prop::collection::vec(0.0f64..2.0, n),
                prop::collection::vec(-2.0f64..2.0, 1 + 2 * n + 2 * m),
            )
                .prop_map(|(r, mask, w, x)| (Ratings::from_dense(&r, &mask).with_weights(w), x))
        })
    }

    proptest! {
        /// `(f_u, f_j) → (−f_u, −f_j)` leaves the objective unchanged bit for bit (T42).
        #[test]
        fn the_objective_is_symmetric_in_the_sign_of_f((data, x) in data_and_point()) {
            let p = BridgingParams::default();
            let mut flipped = x.clone();
            for v in &mut flipped[1 + data.n + data.m..] {
                *v = -*v;
            }
            prop_assert_eq!(
                objective(&data, &p, &x).to_bits(),
                objective(&data, &p, &flipped).to_bits()
            );
        }

        #[test]
        fn the_gradient_matches_central_differences_anywhere((data, x) in data_and_point()) {
            let p = BridgingParams::default();
            let g = gradient(&data, &p, &x);
            let h = 1e-6;
            for k in 0..x.len() {
                let (mut xp, mut xm) = (x.clone(), x.clone());
                xp[k] += h;
                xm[k] -= h;
                let num = (objective(&data, &p, &xp) - objective(&data, &p, &xm)) / (2.0 * h);
                prop_assert!((g[k] - num).abs() <= 1e-6 * (1.0 + num.abs()), "component {}", k);
            }
        }
    }

    /// The largest `|f_j|`, the earliest on a tie, is made non-negative; a zero axis stays.
    #[test]
    fn the_sign_follows_the_earliest_largest_item() {
        let signed = |f_j: Vec<f64>| {
            let mut f = Fit {
                mu: 0.0,
                b_u: vec![0.0],
                b_j: vec![0.0; f_j.len()],
                f_u: vec![1.0],
                f_j,
                participant: vec![true],
                status: Convergence::Converged,
            };
            canonical_sign(&mut f);
            (f.f_u[0], f.f_j)
        };
        assert_eq!(signed(vec![0.5, -0.5]), (1.0, vec![0.5, -0.5]));
        assert_eq!(signed(vec![0.25, -0.5]), (-1.0, vec![-0.25, 0.5]));
        assert_eq!(signed(vec![0.0, 0.0]), (1.0, vec![0.0, 0.0]));
    }

    /// The objective is the weighted squared error plus both penalties, checked by hand.
    #[test]
    fn objective_is_weighted_error_plus_penalties() {
        // One reviewer, one item, weight 2: x = [μ, b_u, b_j, f_u, f_j].
        let data = Ratings::from_dense(&[vec![1.0]], &[vec![true]]).with_weights(vec![2.0]);
        let p = BridgingParams::default();
        let x = [0.5, 0.1, 0.2, 0.3, 0.4];
        let pred = 0.5 + 0.1 + 0.2 + 0.3 * 0.4;
        let want = 2.0 * (pred - 1.0_f64).powi(2)
            + p.lam_b * (0.1_f64.powi(2) + 0.2_f64.powi(2))
            + p.lam_f * (0.3_f64.powi(2) + 0.4_f64.powi(2));
        assert!((objective(&data, &p, &x) - want).abs() < 1e-15);
    }
}
