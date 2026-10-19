//! The first bounded A2 probe (`docs/19` §13.11): six synthetic cases with known truth through
//! the production gate, re-check, routing and contested-facts draw, one JSON record per run.

use crate::generate::{dif_batch, DifBatch};
use crate::grid::{draw_seed, engine_seed, DifDesign, Layout};
use crate::run::true_curves;
use identity::nym::Nym;
use network::cid::{cid, Cid};
use protocol::admission::NullifierSet;
use protocol::contested::{ContestedPool, DrawError};
use protocol::lifecycle::{step, Event, State};
use protocol::pilot::{admit_anchors, admit_dif_batch, admit_templates, PilotError, Templates};
use protocol::revalidation::{latent_batch, target_rechecks, Recheck, N_LATENT_MIN};
use scoring::dif::MIXTURE_DIF_MAX;
use scoring::dtf::{ClassCurves, DTF_MAX};
use scoring::irt::kr20;
use scoring::latent::{
    latent_dif_candidates, Candidate, Formats, LatentDif, LatentParams, Verdict,
};
use scoring::Convergence;
use sha2::{Digest, Sha256};

pub const SCHEMA: &str = "a2-probe/v1";

/// The θ span on each side of a class mean and the Simpson panels of [`generating_dtf`].
pub const SPAN: f64 = 10.0;
pub const PANELS: usize = 4000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Case {
    C0,
    C1,
    C2,
    C3,
    C4,
    C5,
}

pub const CASES: [Case; 6] = [Case::C0, Case::C1, Case::C2, Case::C3, Case::C4, Case::C5];

impl Case {
    pub fn name(self) -> &'static str {
        match self {
            Case::C0 => "C0",
            Case::C1 => "C1",
            Case::C2 => "C2",
            Case::C3 => "C3",
            Case::C4 => "C4",
            Case::C5 => "C5",
        }
    }

    pub fn parse(name: &str) -> Option<Case> {
        CASES.into_iter().find(|c| c.name() == name)
    }

    /// The case's batch at `phase`'s scale: `N = 3,000` and 60 anchors, or 400 and 10 (smoke).
    pub fn design(self, phase: Phase) -> DifDesign {
        let (layout, delta, impact) = match self {
            Case::C0 => (Layout::Campaign(0), 0.0, 0.0),
            Case::C1 => (Layout::Campaign(0), 0.0, 1.0),
            Case::C2 => (Layout::Campaign(2), 0.9, 0.0),
            Case::C3 => (Layout::Campaign(8), 0.17, 0.0),
            Case::C4 => (Layout::Mirror, 0.9, 0.0),
            Case::C5 => (Layout::Campaign(8), 0.04, 0.0),
        };
        let (n, anchors) = match phase {
            Phase::Smoke => (400, 10),
            Phase::Explore
            | Phase::Confirm
            | Phase::Candidates
            | Phase::DtfCheck
            | Phase::IndCal
            | Phase::IndVal
            | Phase::Size => (3000, 60),
        };
        DifDesign {
            n,
            anchors,
            k: 8,
            layout,
            delta,
            impact,
            pi: 0.5,
            ..DifDesign::default()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Phase {
    Smoke,
    Explore,
    Confirm,
    Candidates,
    DtfCheck,
    IndCal,
    IndVal,
    Size,
}

/// The two sizes of `size` (`docs/19` §13.26), each run at both on one seed.
pub const SIZES: [u32; 2] = [3000, 6000];

impl Phase {
    pub fn name(self) -> &'static str {
        match self {
            Phase::Smoke => "smoke",
            Phase::Explore => "explore",
            Phase::Confirm => "confirm",
            Phase::Candidates => "candidates",
            Phase::DtfCheck => "dtf-check",
            Phase::IndCal => "ind-cal",
            Phase::IndVal => "ind-val",
            Phase::Size => "size",
        }
    }

    pub fn parse(name: &str) -> Option<Phase> {
        [
            Phase::Smoke,
            Phase::Explore,
            Phase::Confirm,
            Phase::Candidates,
            Phase::DtfCheck,
            Phase::IndCal,
            Phase::IndVal,
            Phase::Size,
        ]
        .into_iter()
        .find(|p| p.name() == name)
    }

    /// The cases a phase runs: `candidates` (`docs/19` §13.14) C0, C3 and C5, `dtf-check`
    /// (§13.18) and `size` (§13.26) C0 and C3, `ind-cal` (§13.21) C0 and C1, `ind-val` C0, C1,
    /// C3 and C5.
    pub fn cases(self) -> &'static [Case] {
        match self {
            Phase::Candidates => &[Case::C0, Case::C3, Case::C5],
            Phase::DtfCheck | Phase::Size => &[Case::C0, Case::C3],
            Phase::IndCal => &[Case::C0, Case::C1],
            Phase::IndVal => &[Case::C0, Case::C1, Case::C3, Case::C5],
            _ => &CASES,
        }
    }

    /// Whether the phase runs the candidates' diagnostic path, [`gate_and_diagnose`].
    pub fn diagnostic(self) -> bool {
        matches!(
            self,
            Phase::Candidates | Phase::DtfCheck | Phase::IndCal | Phase::IndVal | Phase::Size
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Task {
    pub phase: Phase,
    pub case: Case,
    pub replicate: u32,
    /// The respondents of a `size` run, one of [`SIZES`]; `None` elsewhere, the phase's scale.
    pub n: Option<u32>,
}

impl Task {
    pub fn key(&self) -> String {
        let (phase, case, r) = (self.phase.name(), self.case.name(), self.replicate);
        match self.n {
            Some(n) => format!("{phase}-n{n}-{case}-{r}"),
            None => format!("{phase}-{case}-{r}"),
        }
    }

    /// The case's design at the task's phase, with its `n` where it has one.
    pub fn design(&self) -> DifDesign {
        let mut d = self.case.design(self.phase);
        if let Some(n) = self.n {
            d.n = n as usize;
        }
        d
    }

    /// The population seed: `candidates` reuses confirm's on purpose, the same batches refitted;
    /// `ind-cal`, `ind-val` and `size` give each case its own ([`case_seed`]), `size` at both `n`.
    pub fn seed(&self) -> u64 {
        match self.phase {
            Phase::Candidates => probe_seed(Phase::Confirm, self.replicate),
            Phase::IndCal | Phase::IndVal | Phase::Size => {
                case_seed(self.phase, self.case, self.replicate)
            }
            phase => probe_seed(phase, self.replicate),
        }
    }
}

/// The runs of `phase`, replicate by replicate, every case at each; `size` at each of [`SIZES`].
pub fn tasks(phase: Phase, replicates: u32) -> Vec<Task> {
    let sizes: &[Option<u32>] = match phase {
        Phase::Size => &[Some(SIZES[0]), Some(SIZES[1])],
        _ => &[None],
    };
    (0..replicates)
        .flat_map(|replicate| {
            sizes.iter().flat_map(move |&n| {
                phase.cases().iter().map(move |&case| Task {
                    phase,
                    case,
                    replicate,
                    n,
                })
            })
        })
        .collect()
}

/// The plan's configuration lines: the record schema, each case's design at `phase`'s scale,
/// and the engine's constants the probe reads.
pub fn config_lines(phase: Phase) -> String {
    let mut out = format!("schema\t{SCHEMA}\n");
    for case in CASES {
        let d = case.design(phase);
        out.push_str(&format!(
            "design\t{}\t{}\t{}\t{}\t{}\t{:?}\t{:?}\t{:?}\n",
            case.name(),
            d.n,
            d.anchors,
            d.k,
            layout_name(d.layout),
            d.delta,
            d.impact,
            d.pi
        ));
    }
    if phase == Phase::Size {
        out.push_str(&format!("sizes\t{}\t{}\n", SIZES[0], SIZES[1]));
    }
    out.push_str(&format!(
        "constants\t{DTF_MAX:?}\t{MIXTURE_DIF_MAX:?}\t{SPAN:?}\t{PANELS}\n"
    ));
    out
}

/// The population seed of replicate `r`, the same in every case: SHA-256, length-prefixed.
pub fn probe_seed(phase: Phase, r: u32) -> u64 {
    let mut h = Sha256::new();
    h.update(b"isegoria/a2-probe/v1");
    h.update((phase.name().len() as u64).to_le_bytes());
    h.update(phase.name().as_bytes());
    h.update(r.to_le_bytes());
    let mut first = [0u8; 8];
    first.copy_from_slice(&h.finalize()[..8]);
    u64::from_le_bytes(first)
}

/// The seed of replicate `r` of `case` at `phase` (`docs/19` §13.21): SHA-256, length-prefixed.
pub fn case_seed(phase: Phase, case: Case, r: u32) -> u64 {
    let mut h = Sha256::new();
    h.update(b"isegoria/a2-probe/v1");
    for name in [phase.name(), case.name()] {
        h.update((name.len() as u64).to_le_bytes());
        h.update(name.as_bytes());
    }
    h.update(r.to_le_bytes());
    let mut first = [0u8; 8];
    first.copy_from_slice(&h.finalize()[..8]);
    u64::from_le_bytes(first)
}

/// The generating law's items: class `z = ±1` answers item `j` at `σ(a_j (θ − b_j − δ s_j z))`,
/// its ability `N(0, 1)` for `z = −1` and `N(impact, 1)` for `z = +1`, at share `pi`.
#[derive(Clone, Debug, PartialEq)]
pub struct Items {
    pub a: Vec<f64>,
    pub b: Vec<f64>,
    pub signs: Vec<f64>,
    pub delta: f64,
    pub impact: f64,
    pub pi: f64,
}

impl Items {
    pub fn of(d: &DifDesign, batch: &DifBatch) -> Items {
        Items {
            a: batch.a.clone(),
            b: batch.b.clone(),
            signs: batch.signs.clone(),
            delta: d.delta,
            impact: d.impact,
            pi: d.pi,
        }
    }

    fn p(&self, j: usize, z: f64, t: f64) -> f64 {
        sigmoid(self.a[j] * (t - self.b[j] - self.delta * self.signs[j] * z))
    }

    fn gap(&self, set: &[usize], t: f64) -> f64 {
        set.iter()
            .map(|&j| self.p(j, 1.0, t) - self.p(j, -1.0, t))
            .sum()
    }
}

fn sigmoid(z: f64) -> f64 {
    if z >= 0.0 {
        1.0 / (1.0 + libm::exp(-z))
    } else {
        let e = libm::exp(z);
        e / (1.0 + e)
    }
}

/// `E f(θ)` for `θ ~ N(mean, 1)` by composite Simpson over `mean ± SPAN`, [`PANELS`] panels.
fn normal_expectation(mean: f64, f: impl Fn(f64) -> f64) -> f64 {
    let h = 2.0 * SPAN / PANELS as f64;
    let mut sum = 0.0;
    for i in 0..=PANELS {
        let z = -SPAN + h * i as f64;
        let w = match i {
            0 => 1.0,
            _ if i == PANELS => 1.0,
            _ if i % 2 == 1 => 4.0,
            _ => 2.0,
        };
        sum += w * f(mean + z) * libm::exp(-0.5 * z * z);
    }
    sum * h / 3.0 / (2.0 * std::f64::consts::PI).sqrt()
}

/// The generating law's unsigned DTF of `set` between `z = +1` and `z = −1`, over the population
/// mixture, by Simpson's rule (method A of `docs/19` §13.11; `sim/a2_probe.py` checks it).
pub fn generating_dtf(items: &Items, set: &[usize]) -> f64 {
    let abs_gap = |t: f64| items.gap(set, t).abs();
    (1.0 - items.pi) * normal_expectation(0.0, abs_gap)
        + items.pi * normal_expectation(items.impact, abs_gap)
}

/// `E[S | z = +1] − E[S | z = −1]` on `set`, each class over its own ability law.
pub fn raw_gap(items: &Items, set: &[usize]) -> f64 {
    let score = |z: f64| move |t: f64| set.iter().map(|&j| items.p(j, z, t)).sum::<f64>();
    normal_expectation(items.impact, score(1.0)) - normal_expectation(0.0, score(-1.0))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    Active,
    Contested,
    Rejected,
    Pilot,
    Invalid,
}

impl Route {
    fn code(self) -> char {
        match self {
            Route::Active => 'A',
            Route::Contested => 'C',
            Route::Rejected => 'R',
            Route::Pilot => 'P',
            Route::Invalid => 'X',
        }
    }
}

/// Item `j`'s stage-2 route for its re-check reading and source input (`lifecycle::step`).
pub fn route(k: usize, dif: Recheck, source_verified: bool) -> Route {
    let event = Event::Pilot2Batch {
        batch_size: k,
        dif,
        source_verified,
    };
    match step(State::Pilot2 { appealed: false }, event) {
        Ok(State::ActivePool) => Route::Active,
        Ok(State::Contested) => Route::Contested,
        Ok(State::Rejected(_)) => Route::Rejected,
        Ok(State::Pilot2 { .. }) => Route::Pilot,
        _ => Route::Invalid,
    }
}

/// What the draw of every contested item returned; `Reduced` is the harness's reading of a
/// `NoBalancedDraw`, the largest feasible size drawn, not a protocol policy.
#[derive(Clone, Debug, PartialEq)]
pub enum Draw {
    NotNeeded,
    NoCurves,
    PoolRefused(String),
    OutOfDomain,
    Selected(Vec<usize>),
    Reduced {
        requested: usize,
        feasible: Vec<usize>,
        members: Vec<usize>,
    },
}

impl Draw {
    pub fn members(&self) -> Option<&[usize]> {
        match self {
            Draw::Selected(m) | Draw::Reduced { members: m, .. } => Some(m),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Removal {
    Rejected,
    Indeterminate,
    Contested,
    NotDrawn,
    DrawFailed,
    Invalid,
}

impl Removal {
    fn name(self) -> &'static str {
        match self {
            Removal::Rejected => "rejected",
            Removal::Indeterminate => "indeterminate",
            Removal::Contested => "contested",
            Removal::NotDrawn => "not_drawn",
            Removal::DrawFailed => "draw_failed",
            Removal::Invalid => "invalid",
        }
    }
}

/// A form's members against its reference set (the eight items, or the contested ones for a
/// selection), and why each missing reference item is missing.
#[derive(Clone, Debug, PartialEq)]
pub struct Form {
    pub members: Vec<usize>,
    pub reference: usize,
    pub removed: Vec<(usize, Removal)>,
}

impl Form {
    pub fn completeness(&self) -> &'static str {
        match self.members.len() {
            0 => "empty",
            m if m == self.reference => "complete",
            _ => "incomplete",
        }
    }
}

/// One source input's reading of a fit: routes, the draw, its cost `D` and the forms.
#[derive(Clone, Debug, PartialEq)]
pub struct Reading {
    pub verified: bool,
    pub routes: Vec<Route>,
    pub contested: Vec<usize>,
    pub draw: Draw,
    pub cost: Option<f64>,
    pub active: Form,
    pub composed: Form,
    pub selection: Option<Form>,
}

impl Reading {
    /// A returned selection whose cost exceeds `DTF_MAX`: a code defect, never a probe result.
    pub fn defect(&self) -> bool {
        self.draw.members().is_some() && self.cost.is_some_and(|c| c > DTF_MAX)
    }
}

fn item_cid(j: usize) -> Cid {
    cid(format!("a2-probe/item/{j}").as_bytes())
}

fn removal(route: Route, drawn: Option<bool>) -> Removal {
    match (route, drawn) {
        (Route::Rejected, _) => Removal::Rejected,
        (Route::Pilot, _) => Removal::Indeterminate,
        (Route::Contested, None) => Removal::Contested,
        (Route::Contested, Some(true)) => Removal::NotDrawn,
        (Route::Contested, Some(false)) => Removal::DrawFailed,
        (Route::Active | Route::Invalid, _) => Removal::Invalid,
    }
}

fn form(k: usize, members: Vec<usize>, routes: &[Route], drawn: Option<bool>) -> Form {
    let removed = (0..k)
        .filter(|j| !members.contains(j))
        .map(|j| (j, removal(routes[j], drawn)))
        .collect();
    Form {
        members,
        reference: k,
        removed,
    }
}

fn draw(curves: Option<&ClassCurves>, contested: &[usize], seed: u64) -> (Draw, Option<f64>) {
    if contested.is_empty() {
        return (Draw::NotNeeded, None);
    }
    let Some(curves) = curves else {
        return (Draw::NoCurves, None);
    };
    let members: Vec<(Cid, usize)> = contested.iter().map(|&j| (item_cid(j), j)).collect();
    let mut pool = ContestedPool::new();
    if let Err(e) = pool.record(curves.clone(), &members) {
        return (Draw::PoolRefused(format!("{e:?}")), None);
    }
    let index = |c: &Cid| members.iter().find(|(m, _)| m == c).map(|&(_, j)| j);
    let indices = |sel: &[Cid]| -> Vec<usize> {
        let mut out: Vec<usize> = sel.iter().filter_map(index).collect();
        out.sort_unstable();
        out
    };
    let outcome = match pool.draw(contested.len(), DTF_MAX, seed) {
        Ok(sel) => Ok((Draw::Selected(indices(&sel)), sel)),
        Err(DrawError::OutOfDomain(_)) => Err(Draw::OutOfDomain),
        Err(DrawError::NoBalancedDraw(nb)) => {
            let size = nb.feasible.iter().copied().max().unwrap_or(0);
            match pool.draw(size, DTF_MAX, seed) {
                Ok(sel) => Ok((
                    Draw::Reduced {
                        requested: nb.requested,
                        feasible: nb.feasible,
                        members: indices(&sel),
                    },
                    sel,
                )),
                Err(e) => Err(Draw::PoolRefused(format!("{e:?}"))),
            }
        }
    };
    match outcome {
        Ok((d, sel)) => (d, pool.dtf(&sel)),
        Err(d) => (d, None),
    }
}

/// The reading of `rechecks` under one source input: routes by `lifecycle::step`, the contested
/// items recorded in a one-fit pool with `curves` and drawn at `DTF_MAX`, and the forms.
pub fn assess(
    rechecks: &[Recheck],
    curves: Option<&ClassCurves>,
    verified: bool,
    seed: u64,
) -> Reading {
    let k = rechecks.len();
    let routes: Vec<Route> = rechecks.iter().map(|&r| route(k, r, verified)).collect();
    let with = |want: Route| -> Vec<usize> { (0..k).filter(|&j| routes[j] == want).collect() };
    let (active, contested) = (with(Route::Active), with(Route::Contested));
    let (drawn, cost) = draw(curves, &contested, seed);
    let selected = drawn.members().unwrap_or(&[]).to_vec();
    let mut composed: Vec<usize> = active.iter().chain(&selected).copied().collect();
    composed.sort_unstable();
    let selection = (!contested.is_empty()).then(|| Form {
        members: selected.clone(),
        reference: contested.len(),
        removed: contested
            .iter()
            .filter(|j| !selected.contains(j))
            .map(|&j| {
                (
                    j,
                    removal(Route::Contested, Some(drawn.members().is_some())),
                )
            })
            .collect(),
    });
    Reading {
        verified,
        active: form(k, active, &routes, None),
        composed: form(k, composed, &routes, Some(drawn.members().is_some())),
        selection,
        routes,
        contested,
        draw: drawn,
        cost,
    }
}

/// The run's measured cost: wall and CPU time of the run's process, its peak resident set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cost {
    pub wall_ms: u64,
    pub cpu_ms: Option<u64>,
    pub hwm_kib: Option<u64>,
}

fn respondents(n: usize) -> NullifierSet {
    let mut set = NullifierSet::new();
    for i in 0..n {
        let mut id = [0u8; 32];
        id[..8].copy_from_slice(&(i as u64).to_le_bytes());
        set.spend(Nym(id)).expect("distinct pseudonyms");
    }
    set
}

/// The production path on `task`'s batch: `latent_batch`'s gate and fit, engine seeded.
pub fn gate_and_fit(task: &Task, d: &DifDesign, batch: &DifBatch) -> Result<LatentDif, PilotError> {
    latent_batch(
        &respondents(d.n),
        &batch.anchors,
        &batch.x,
        &Formats::open(d.anchors, d.k),
        &Templates::none(d.anchors, d.k),
        engine_seed(task.seed()),
    )
}

/// `latent_batch`'s gate, its checks in its order, without the fit: the diagnostic path runs
/// the same fit through [`latent_dif_candidates`] instead (`docs/19` §13.14).
pub fn gate(
    respondents: &NullifierSet,
    anchors: &[Vec<f64>],
    responses: &[Vec<f64>],
    formats: &Formats,
    templates: &Templates,
) -> Result<(), PilotError> {
    let n = respondents.len();
    let m = responses.first().map_or(0, |row| row.len());
    admit_dif_batch(m, n, N_LATENT_MIN)?;
    let mismatch =
        |rows: usize, respondents: usize| PilotError::RowCountMismatch { rows, respondents };
    if responses.len() != n {
        return Err(mismatch(responses.len(), n));
    }
    if let Some(row) = responses.iter().find(|row| row.len() != m) {
        return Err(mismatch(row.len(), m));
    }
    if anchors.len() != n {
        return Err(mismatch(anchors.len(), n));
    }
    let k_anchor = anchors.first().map_or(0, Vec::len);
    if let Some(row) = anchors.iter().find(|row| row.len() != k_anchor) {
        return Err(mismatch(row.len(), k_anchor));
    }
    if formats.anchors.len() != k_anchor || formats.items.len() != m {
        return Err(PilotError::BadFormats);
    }
    admit_templates(templates, k_anchor, m)?;
    admit_anchors(anchors).map(|_| ())
}

/// The diagnostic path on `task`'s batch: [`gate`], then `latent_dif`'s fit with its candidates.
pub fn gate_and_diagnose(
    task: &Task,
    d: &DifDesign,
    batch: &DifBatch,
) -> Result<(LatentDif, Vec<Candidate>), PilotError> {
    let formats = Formats::open(d.anchors, d.k);
    let templates = Templates::none(d.anchors, d.k);
    gate(
        &respondents(d.n),
        &batch.anchors,
        &batch.x,
        &formats,
        &templates,
    )?;
    let lp = LatentParams {
        seed: engine_seed(task.seed()),
        ..LatentParams::default()
    };
    latent_dif_candidates(&batch.anchors, &batch.x, &formats, &lp)
        .map_err(|_| PilotError::BadFormats)
}

/// One run: the batch drawn from the task's seed, the gate and fit, then [`record_of`], whose
/// `cost` is measured once everything else in the record is computed.
pub fn run(task: &Task, cost: impl FnOnce() -> Option<Cost>) -> String {
    let d = task.design();
    let batch = dif_batch(&d, task.seed());
    if task.phase.diagnostic() {
        let diagnosed = gate_and_diagnose(task, &d, &batch);
        return diagnosed_record(task, &d, &batch, &diagnosed, cost);
    }
    let gate = gate_and_fit(task, &d, &batch);
    record_of(task, &d, &batch, Outcome::Gate(&gate), cost)
}

fn verdict_name(v: Verdict) -> &'static str {
    match v {
        Verdict::Selected => "selected",
        Verdict::Superseded => "superseded",
        Verdict::NotConverged => "not_converged",
        Verdict::BicNotLower => "bic_not_lower",
    }
}

/// Per candidate: its search fields, its classes counted at `MIN_CLASS_SHARE`, and against the
/// one-class fit `2 Δ log L`, the penalty `Δk ln N` and `Δ BIC`; its whole-form `DTF_F`.
fn candidates_json(seen: &[Candidate], n: usize, k: usize) -> J {
    let one = &seen[0];
    let ln_n = libm::log(n as f64);
    let all: Vec<usize> = (0..k).collect();
    J::Arr(
        seen.iter()
            .map(|c| {
                let (dtf, basis) = match ClassCurves::of(&c.fit) {
                    Ok(curves) if curves.classes() < 2 => (J::Num(0.0), "one counted class"),
                    Ok(curves) => (J::opt(curves.dtf(&all)), "counted classes"),
                    Err(_) => (J::Null, "unavailable"),
                };
                let counted = c
                    .fit
                    .pi
                    .iter()
                    .filter(|&&p| p >= scoring::dif::MIN_CLASS_SHARE);
                J::Obj(vec![
                    ("classes", J::Int(c.classes as u64)),
                    ("non_uniform", J::Bool(c.non_uniform)),
                    ("status", J::Str(status_name(c.status).into())),
                    ("verdict", J::Str(verdict_name(c.verdict).into())),
                    ("log_lik", J::Num(c.log_lik)),
                    ("params", J::Int(c.params as u64)),
                    ("bic", J::Num(c.bic)),
                    ("gain", J::Num(2.0 * (c.log_lik - one.log_lik))),
                    (
                        "penalty",
                        J::Num((c.params as f64 - one.params as f64) * ln_n),
                    ),
                    ("delta_bic", J::Num(c.bic - one.bic)),
                    (
                        "one_converged",
                        J::Bool(one.status == Convergence::Converged),
                    ),
                    ("pi", J::nums(&c.fit.pi)),
                    ("counted", J::Int(counted.count() as u64)),
                    ("dtf", dtf),
                    ("dtf_basis", J::Str(basis.into())),
                ])
            })
            .collect(),
    )
}

/// The record of the diagnostic path: [`record_of`]'s fields for the same fit, then every
/// candidate, then the cost.
pub fn diagnosed_record(
    task: &Task,
    d: &DifDesign,
    batch: &DifBatch,
    diagnosed: &Result<(LatentDif, Vec<Candidate>), PilotError>,
    cost: impl FnOnce() -> Option<Cost>,
) -> String {
    let gate = diagnosed
        .as_ref()
        .map(|(fit, _)| fit.clone())
        .map_err(|e| *e);
    let seen = diagnosed
        .as_ref()
        .ok()
        .map(|(_, seen)| candidates_json(seen, d.n, d.k));
    record_with(task, d, batch, Outcome::Gate(&gate), seen, cost)
}

fn status_name(s: Convergence) -> &'static str {
    match s {
        Convergence::Converged => "converged",
        Convergence::MaxIters => "max_iters",
        Convergence::LineSearchFailed => "line_search_failed",
    }
}

fn recheck_code(r: Recheck) -> char {
    match r {
        Recheck::Dif => 'D',
        Recheck::NoDif => 'N',
        Recheck::Indeterminate => 'I',
    }
}

enum J {
    Null,
    Bool(bool),
    Num(f64),
    Int(u64),
    Str(String),
    Arr(Vec<J>),
    Obj(Vec<(&'static str, J)>),
}

impl J {
    fn opt(v: Option<f64>) -> J {
        v.map_or(J::Null, J::Num)
    }

    fn nums(v: &[f64]) -> J {
        J::Arr(v.iter().map(|&x| J::Num(x)).collect())
    }

    fn ints(v: &[usize]) -> J {
        J::Arr(v.iter().map(|&x| J::Int(x as u64)).collect())
    }

    fn write(&self, out: &mut String) {
        match self {
            J::Null => out.push_str("null"),
            J::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            J::Num(x) if x.is_finite() => out.push_str(&format!("{x:?}")),
            J::Num(_) => out.push_str("null"),
            J::Int(i) => out.push_str(&i.to_string()),
            J::Str(s) => {
                out.push('"');
                for ch in s.chars() {
                    match ch {
                        '"' => out.push_str("\\\""),
                        '\\' => out.push_str("\\\\"),
                        c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                        c => out.push(c),
                    }
                }
                out.push('"');
            }
            J::Arr(v) => {
                out.push('[');
                for (i, x) in v.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    x.write(out);
                }
                out.push(']');
            }
            J::Obj(v) => {
                out.push('{');
                for (i, (k, x)) in v.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    out.push_str(&format!("\"{k}\":"));
                    x.write(out);
                }
                out.push('}');
            }
        }
    }
}

fn form_json(
    form: &Form,
    items: &Items,
    grid: Option<&ClassCurves>,
    fit: Option<&ClassCurves>,
) -> J {
    let removed = form
        .removed
        .iter()
        .map(|&(j, why)| J::Arr(vec![J::Int(j as u64), J::Str(why.name().into())]))
        .collect();
    let on = |c: Option<&ClassCurves>| J::opt(c.and_then(|c| c.dtf(&form.members)));
    J::Obj(vec![
        ("members", J::ints(&form.members)),
        ("size", J::Int(form.members.len() as u64)),
        ("reference", J::Int(form.reference as u64)),
        ("completeness", J::Str(form.completeness().into())),
        ("removed", J::Arr(removed)),
        ("dtf", J::Num(generating_dtf(items, &form.members))),
        ("grid", on(grid)),
        ("fitted", on(fit)),
    ])
}

fn draw_json(d: &Draw) -> J {
    let (outcome, extra) = match d {
        Draw::NotNeeded => ("not_needed", vec![]),
        Draw::NoCurves => ("no_curves", vec![]),
        Draw::PoolRefused(why) => ("pool_refused", vec![("reason", J::Str(why.clone()))]),
        Draw::OutOfDomain => ("out_of_domain", vec![]),
        Draw::Selected(m) => ("selected", vec![("members", J::ints(m))]),
        Draw::Reduced {
            requested,
            feasible,
            members,
        } => (
            "reduced",
            vec![
                ("requested", J::Int(*requested as u64)),
                ("feasible", J::ints(feasible)),
                ("members", J::ints(members)),
            ],
        ),
    };
    let mut fields = vec![("outcome", J::Str(outcome.into()))];
    fields.extend(extra);
    J::Obj(fields)
}

fn reading_json(
    r: &Reading,
    items: &Items,
    grid: Option<&ClassCurves>,
    fit: Option<&ClassCurves>,
) -> J {
    J::Obj(vec![
        (
            "source",
            J::Str(if r.verified { "S1" } else { "S0" }.into()),
        ),
        (
            "routes",
            J::Str(r.routes.iter().map(|x| x.code()).collect()),
        ),
        ("contested", J::ints(&r.contested)),
        ("draw", draw_json(&r.draw)),
        ("cost", J::opt(r.cost)),
        ("active", form_json(&r.active, items, grid, fit)),
        ("composed", form_json(&r.composed, items, grid, fit)),
        (
            "selection",
            r.selection
                .as_ref()
                .map_or(J::Null, |s| form_json(s, items, grid, fit)),
        ),
    ])
}

fn layout_name(layout: Layout) -> String {
    match layout {
        Layout::Campaign(n) => format!("c{n}"),
        Layout::Mirror => "m".into(),
        Layout::TwoAxes(n) => format!("x{n}"),
    }
}

fn head(task: &Task, d: &DifDesign, batch: &DifBatch, truth: J) -> Vec<(&'static str, J)> {
    vec![
        ("schema", J::Str(SCHEMA.into())),
        ("key", J::Str(task.key())),
        ("phase", J::Str(task.phase.name().into())),
        ("case", J::Str(task.case.name().into())),
        ("replicate", J::Int(u64::from(task.replicate))),
        ("seed", J::Int(task.seed())),
        ("engine_seed", J::Int(engine_seed(task.seed()))),
        ("draw_seed", J::Int(draw_seed("contested", task.seed(), 0))),
        (
            "design",
            J::Obj(vec![
                ("n", J::Int(d.n as u64)),
                ("anchors", J::Int(d.anchors as u64)),
                ("k", J::Int(d.k as u64)),
                ("layout", J::Str(layout_name(d.layout))),
                ("delta", J::Num(d.delta)),
                ("impact", J::Num(d.impact)),
                ("pi", J::Num(d.pi)),
            ]),
        ),
        (
            "items",
            J::Obj(vec![
                ("a", J::nums(&batch.a)),
                ("b", J::nums(&batch.b)),
                ("signs", J::nums(&batch.signs)),
                ("roles", J::Str(batch.roles.clone())),
            ]),
        ),
        ("truth", truth),
        ("kr20", J::Num(kr20(&batch.anchors))),
    ]
}

fn cost_json(cost: Option<Cost>) -> J {
    cost.map_or(J::Null, |c| {
        J::Obj(vec![
            ("wall_ms", J::Int(c.wall_ms)),
            ("cpu_ms", c.cpu_ms.map_or(J::Null, J::Int)),
            ("hwm_kib", c.hwm_kib.map_or(J::Null, J::Int)),
        ])
    })
}

/// What the record reads: the gate's result, or the supervisor's `outcome` and `reason`.
pub enum Outcome<'a> {
    Gate(&'a Result<LatentDif, PilotError>),
    Stopped { outcome: &'a str, reason: &'a str },
}

/// The run's record: the design's truth, then the refusal, the stop, or the fit read under S0
/// and S1; `cost` is called last.
pub fn record_of(
    task: &Task,
    d: &DifDesign,
    batch: &DifBatch,
    outcome: Outcome,
    cost: impl FnOnce() -> Option<Cost>,
) -> String {
    record_with(task, d, batch, outcome, None, cost)
}

fn record_with(
    task: &Task,
    d: &DifDesign,
    batch: &DifBatch,
    outcome: Outcome,
    seen: Option<J>,
    cost: impl FnOnce() -> Option<Cost>,
) -> String {
    let items = Items::of(d, batch);
    let grid = true_curves(d, batch);
    let all: Vec<usize> = (0..d.k).collect();
    let truth = J::Obj(vec![
        ("dtf", J::Num(generating_dtf(&items, &all))),
        ("grid", J::opt(grid.as_ref().and_then(|g| g.dtf(&all)))),
        ("raw_gap", J::Num(raw_gap(&items, &all))),
    ]);
    let mut fields = head(task, d, batch, truth);
    match outcome {
        Outcome::Stopped { outcome, reason } => {
            fields.push(("outcome", J::Str(outcome.into())));
            fields.push(("reason", J::Str(reason.into())));
        }
        Outcome::Gate(Err(e)) => {
            fields.push(("outcome", J::Str("refused".into())));
            fields.push(("reason", J::Str(format!("{e:?}"))));
        }
        Outcome::Gate(Ok(fit)) => {
            let curves = ClassCurves::of(fit).ok();
            let rechecks = target_rechecks(fit);
            let seed = draw_seed("contested", task.seed(), 0);
            let fixed = Form {
                members: all.clone(),
                reference: d.k,
                removed: Vec::new(),
            };
            let counted = curves
                .as_ref()
                .map_or(J::Null, |c| J::Int(c.classes() as u64));
            fields.push(("outcome", J::Str("read".into())));
            fields.push((
                "fit",
                J::Obj(vec![
                    ("status", J::Str(status_name(fit.status).into())),
                    ("classes", J::Int(fit.classes as u64)),
                    ("counted", counted),
                    ("non_uniform", J::Bool(fit.non_uniform)),
                    ("bic_gain", J::Num(fit.bic_gain)),
                    ("candidates", J::Int(fit.candidates.len() as u64)),
                    ("pi", J::nums(&fit.pi)),
                    ("eta", J::nums(&fit.eta)),
                    ("dif", J::nums(&fit.dif)),
                    ("a_gap", J::nums(&fit.a_gap)),
                ]),
            ));
            let codes: String = rechecks.iter().map(|&r| recheck_code(r)).collect();
            fields.push(("rechecks", J::Str(codes)));
            let (g, c) = (grid.as_ref(), curves.as_ref());
            fields.push(("fixed", form_json(&fixed, &items, g, c)));
            let assessed: Vec<Reading> = [false, true]
                .iter()
                .map(|&verified| assess(&rechecks, c, verified, seed))
                .collect();
            let readings = assessed
                .iter()
                .map(|r| reading_json(r, &items, g, c))
                .collect();
            fields.push(("readings", J::Arr(readings)));
            if assessed.iter().any(Reading::defect) {
                let why = "a returned selection's cost exceeds DTF_MAX";
                fields.push(("defect", J::Str(why.into())));
            }
        }
    }
    if let Some(seen) = seen {
        fields.push(("candidates", seen));
    }
    fields.push(("cost", cost_json(cost())));
    let mut out = String::new();
    J::Obj(fields).write(&mut out);
    out
}

/// The record of a run the supervisor killed or saw fail: the design's truth, no reading.
pub fn stopped_record(task: &Task, outcome: &str, reason: &str, cost: Cost) -> String {
    let d = task.design();
    let batch = dif_batch(&d, task.seed());
    let stop = Outcome::Stopped { outcome, reason };
    record_of(task, &d, &batch, stop, || Some(cost))
}
