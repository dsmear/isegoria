//! B-b's scorer (`docs/17` §§8, 14.9): each assignment's case and term, the counts and each
//! prefixed cohort's value or the causes of none, on typed inputs, the baselines through
//! `panel_scores`; the inputs' authenticity and links are the caller's (`docs/17` §14.8 D).

use crate::panel_scores::{extra_round_baseline, first_panel_baselines, Forecast};
use identity::nym::Nym;
use scoring::reputation::difference_score;

/// A reference procedure's terminal outcome (`docs/17` §8.1): admissible, rejected, inconclusive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    A,
    R,
    I,
}

/// `Open` while the reveal period runs; `Missing` once it closed without a valid reveal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Report {
    Revealed(f64),
    Open,
    Missing,
}

/// The recorded `S_j` and `π_j`; `Undrawn` until a drawn design records the draw, `outcome`
/// `None` while the selected procedure is pending.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Selection {
    Undrawn,
    NotSelected {
        inclusion: f64,
    },
    Selected {
        inclusion: f64,
        outcome: Option<Outcome>,
    },
}

impl Selection {
    pub fn inclusion(self) -> Option<f64> {
        match self {
            Selection::Undrawn => None,
            Selection::NotSelected { inclusion } | Selection::Selected { inclusion, .. } => {
                Some(inclusion)
            }
        }
    }
}

/// A first panelist: its report and its frozen review weight.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Panelist {
    pub nym: Nym,
    pub report: Report,
    pub weight: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extra {
    pub nym: Nym,
    pub report: Report,
}

/// One item: its first panel, its extra round (empty without one), `Φ_j` reached, its selection.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub first: Vec<Panelist>,
    pub extra: Vec<Extra>,
    pub frozen: bool,
    pub selection: Selection,
}

/// A prefixed cohort: `nym`'s assignments on `items`, indices into the scorer's items.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cohort {
    pub nym: Nym,
    pub items: Vec<usize>,
}

/// `docs/17` §8.2's cases 1–7, and a drawn design's frozen item before its draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Case {
    NotSelected,
    Pending,
    Positive,
    Negative,
    Inconclusive,
    MissingReport,
    Unfrozen,
    Undrawn,
}

impl Case {
    /// In `O`: an observed terminal outcome, cases 3–5.
    pub fn observed(self) -> bool {
        matches!(self, Case::Positive | Case::Negative | Case::Inconclusive)
    }

    /// In `V`: a conclusive verdict, cases 3–4, with or without a baseline.
    pub fn conclusive(self) -> bool {
        matches!(self, Case::Positive | Case::Negative)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Undefined {
    NoBaseline,
}

/// `term`: `Ok(Some(_))` a defined term, divided by `π_j`; `Ok(None)` none in this case; `Err`
/// a verdict whose term is undefined. `baseline`: composed once the item is frozen.
#[derive(Clone, Debug, PartialEq)]
pub struct Assignment {
    pub nym: Nym,
    pub item: usize,
    pub extra: bool,
    pub case: Case,
    pub inclusion: Option<f64>,
    pub baseline: Option<f64>,
    pub term: Result<Option<f64>, Undefined>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    MissingReport,
    Unfrozen,
    Undrawn,
    NoBaseline,
}

/// `Bound` holds the final value where one comes to exist; `Unavailable` names every member
/// (item, cause) that leaves no value and no interval, in the cohort's order.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Final(f64),
    Bound { sum: (f64, f64), mean: (f64, f64) },
    Unavailable(Vec<(usize, Cause)>),
}

/// `size` is `|K|`, the denominator; `o` and `v` count `K`'s members; `known` sums its terms.
#[derive(Clone, Debug, PartialEq)]
pub struct CohortScore {
    pub nym: Nym,
    pub members: Vec<Assignment>,
    pub size: usize,
    pub o: usize,
    pub v: usize,
    pub known: f64,
    pub value: Value,
}

/// `N_u`, `O_u`, `V_u` over a reviewer's assignments among the items supplied.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub n: usize,
    pub o: usize,
    pub v: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Scores {
    pub assignments: Vec<Assignment>,
    pub cohorts: Vec<CohortScore>,
}

impl Scores {
    pub fn counts(&self, nym: Nym) -> Counts {
        count(self.assignments.iter().filter(|a| a.nym == nym))
    }
}

/// Inputs refused, never repaired (`docs/17` §14.9); the first found, items before cohorts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputError {
    Probability { item: usize, nym: Nym },
    Weight { item: usize, nym: Nym },
    Inclusion { item: usize },
    NotSelectedAtCertainty { item: usize },
    Reassigned { item: usize, nym: Nym },
    OpenAtFreeze { item: usize, nym: Nym },
    MissingFirstAtFreeze { item: usize, nym: Nym },
    OutcomeBeforeFreeze { item: usize },
    EmptyCohort { cohort: usize },
    UnknownItem { cohort: usize, item: usize },
    Unassigned { cohort: usize, item: usize },
    RepeatedMember { cohort: usize, item: usize },
}

fn count<'a>(of: impl Iterator<Item = &'a Assignment>) -> Counts {
    of.fold(Counts::default(), |c, a| Counts {
        n: c.n + 1,
        o: c.o + usize::from(a.case.observed()),
        v: c.v + usize::from(a.case.conclusive()),
    })
}

fn check(j: usize, item: &Item) -> Result<(), InputError> {
    if let Some(pi) = item.selection.inclusion() {
        if !(pi > 0.0 && pi <= 1.0) {
            return Err(InputError::Inclusion { item: j });
        }
    }
    if item.selection == (Selection::NotSelected { inclusion: 1.0 }) {
        return Err(InputError::NotSelectedAtCertainty { item: j });
    }
    let reports = item.first.iter().map(|f| (f.nym, f.report, false));
    let reports: Vec<(Nym, Report, bool)> = reports
        .chain(item.extra.iter().map(|e| (e.nym, e.report, true)))
        .collect();
    for (k, &(nym, report, extra)) in reports.iter().enumerate() {
        if matches!(report, Report::Revealed(p) if !(0.0..=1.0).contains(&p)) {
            return Err(InputError::Probability { item: j, nym });
        }
        if reports[..k].iter().any(|r| r.0 == nym) {
            return Err(InputError::Reassigned { item: j, nym });
        }
        match report {
            Report::Open if item.frozen => return Err(InputError::OpenAtFreeze { item: j, nym }),
            Report::Missing if item.frozen && !extra => {
                return Err(InputError::MissingFirstAtFreeze { item: j, nym })
            }
            _ => {}
        }
    }
    if let Some(f) = item
        .first
        .iter()
        .find(|f| !(f.weight.is_finite() && f.weight >= 0.0))
    {
        return Err(InputError::Weight {
            item: j,
            nym: f.nym,
        });
    }
    match item.selection {
        Selection::Selected {
            outcome: Some(_), ..
        } if !item.frozen => Err(InputError::OutcomeBeforeFreeze { item: j }),
        _ => Ok(()),
    }
}

fn case(report: Report, item: &Item) -> Case {
    match (report, item.frozen, item.selection) {
        (Report::Missing, _, _) => Case::MissingReport,
        (_, false, _) => Case::Unfrozen,
        (_, true, Selection::Undrawn) => Case::Undrawn,
        (_, true, Selection::NotSelected { .. }) => Case::NotSelected,
        (_, true, Selection::Selected { outcome, .. }) => match outcome {
            None => Case::Pending,
            Some(Outcome::A) => Case::Positive,
            Some(Outcome::R) => Case::Negative,
            Some(Outcome::I) => Case::Inconclusive,
        },
    }
}

/// `docs/17` §8.1: `g/π` on a verdict, 0 on `I` and on a non-selection, none otherwise.
fn term(
    case: Case,
    report: Report,
    baseline: Option<f64>,
    pi: Option<f64>,
) -> Result<Option<f64>, Undefined> {
    let o = match case {
        Case::Positive => 1.0,
        Case::Negative => 0.0,
        Case::Inconclusive | Case::NotSelected => return Ok(Some(0.0)),
        _ => return Ok(None),
    };
    let (Report::Revealed(p), Some(pi)) = (report, pi) else {
        unreachable!("a verdict has a revealed report and a selection")
    };
    let b = baseline.ok_or(Undefined::NoBaseline)?;
    Ok(Some(difference_score(p, b, o) / pi))
}

fn assignments(j: usize, item: &Item) -> Vec<Assignment> {
    let inclusion = item.selection.inclusion();
    let forecasts: Option<Vec<Forecast>> = item.frozen.then(|| {
        let revealed = item.first.iter().map(|f| match f.report {
            Report::Revealed(prob) => Forecast {
                prob,
                weight: f.weight,
            },
            _ => unreachable!("checked: every first report revealed at the freeze"),
        });
        revealed.collect()
    });
    let first_baselines = forecasts.as_deref().map(first_panel_baselines);
    let extra_baseline = forecasts.as_deref().and_then(extra_round_baseline);
    let first = item.first.iter().enumerate().map(|(k, f)| {
        let b = first_baselines.as_ref().and_then(|b| b[k]);
        (f.nym, f.report, false, b)
    });
    let extra = item
        .extra
        .iter()
        .map(|e| (e.nym, e.report, true, extra_baseline));
    first
        .chain(extra)
        .map(|(nym, report, extra, baseline)| {
            let case = case(report, item);
            Assignment {
                nym,
                item: j,
                extra,
                case,
                inclusion,
                baseline,
                term: term(case, report, baseline, inclusion),
            }
        })
        .collect()
}

fn cohort(
    k: usize,
    c: &Cohort,
    items: &[Item],
    all: &[Assignment],
) -> Result<CohortScore, InputError> {
    if c.items.is_empty() {
        return Err(InputError::EmptyCohort { cohort: k });
    }
    let mut members: Vec<Assignment> = Vec::new();
    for &j in &c.items {
        if j >= items.len() {
            return Err(InputError::UnknownItem { cohort: k, item: j });
        }
        if members.iter().any(|m| m.item == j) {
            return Err(InputError::RepeatedMember { cohort: k, item: j });
        }
        let a = all.iter().find(|a| a.item == j && a.nym == c.nym);
        members.push(
            a.ok_or(InputError::Unassigned { cohort: k, item: j })?
                .clone(),
        );
    }
    let counts = count(members.iter());
    let known: f64 = members.iter().filter_map(|m| m.term.ok().flatten()).sum();
    let causes: Vec<(usize, Cause)> = members
        .iter()
        .filter_map(|m| {
            let cause = match (m.case, m.term) {
                (Case::MissingReport, _) => Cause::MissingReport,
                (Case::Unfrozen, _) => Cause::Unfrozen,
                (Case::Undrawn, _) => Cause::Undrawn,
                (_, Err(Undefined::NoBaseline)) => Cause::NoBaseline,
                _ => return None,
            };
            Some((m.item, cause))
        })
        .collect();
    let pending = members.iter().filter(|m| m.case == Case::Pending);
    let radius: f64 = pending.map(|m| 1.0 / m.inclusion.expect("selected")).sum();
    let size = members.len() as f64;
    let value = if !causes.is_empty() {
        Value::Unavailable(causes)
    } else if members.iter().all(|m| m.case != Case::Pending) {
        Value::Final(known / size)
    } else {
        let sum = (known - radius, known + radius);
        Value::Bound {
            sum,
            mean: (sum.0 / size, sum.1 / size),
        }
    };
    Ok(CohortScore {
        nym: c.nym,
        size: members.len(),
        o: counts.o,
        v: counts.v,
        members,
        known,
        value,
    })
}

/// Every assignment of `items`, in item order, first panel then extra round; each cohort scored
/// on its members, in the order given (`docs/17` §14.9).
pub fn score(items: &[Item], cohorts: &[Cohort]) -> Result<Scores, InputError> {
    for (j, item) in items.iter().enumerate() {
        check(j, item)?;
    }
    let all: Vec<Assignment> = items
        .iter()
        .enumerate()
        .flat_map(|(j, item)| assignments(j, item))
        .collect();
    let cohorts = cohorts
        .iter()
        .enumerate()
        .map(|(k, c)| cohort(k, c, items, &all))
        .collect::<Result<_, _>>()?;
    Ok(Scores {
        assignments: all,
        cohorts,
    })
}
