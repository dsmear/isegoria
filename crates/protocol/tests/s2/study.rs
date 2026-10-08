//! The check's derivations (`docs/20` §§3–5, §7; `17` §§8–9): freezes, K4's procedures, the
//! assignments' cases, B-b's contributions, counts and cohorts, the snapshot and its costs.

use super::fixture::{consortium, Main, ANCHORS};
use super::order::{
    answers_reference, commitments, existence_level, replay_level, Evidence, References,
};
use super::records::{Attempt, Construction, Group, Outcome, Reading, Reason, Record, Terminal};
use super::replay::{Examined, LogView};
use identity::nym::Nym;
use network::cid::{cid, Cid};
use protocol::gate::GateOutcome;
use protocol::lifecycle::{Event, State};
use protocol::panel_scores::{first_panel_baselines, Forecast};
use scoring::reputation::difference_score;
use std::collections::BTreeMap;

/// `Φ_j` (`16` §5) from an item's applied steps: `Score` on `Pass` or `Reject`; `Resolve`
/// unless appealable; `Appeal` or `AppealExpires` after an appealable decision.
pub fn freeze(steps: &[(u64, Event)]) -> Option<u64> {
    let mut appealable = false;
    for (pos, event) in steps {
        match event {
            Event::Score {
                outcome: GateOutcome::Pass | GateOutcome::Reject,
            } => return Some(*pos),
            Event::Score {
                outcome: GateOutcome::AppealEligible,
            }
            | Event::Resolve {
                outcome: GateOutcome::AppealEligible,
            } => appealable = true,
            Event::Resolve { .. } => return Some(*pos),
            Event::Appeal { .. } | Event::AppealExpires if appealable => return Some(*pos),
            _ => {}
        }
    }
    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Procedure {
    Concluded(Outcome),
    Exhausted(u8),
    Pending,
}

/// K4 on one item: its procedure, the attempts it used (stage, number, CID), the readings left
/// unused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct K4 {
    pub procedure: Procedure,
    pub used: Vec<(u8, u8, Cid)>,
    pub unused: Vec<(u8, u8, Reading)>,
}

/// Why a group's attempt records support no derivation (K4, K6); none is repaired.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SequenceError {
    /// One attempt's record logged twice.
    Repeated { stage: u8, attempt: u8 },
    /// Two different records claiming one attempt.
    Conflicting { stage: u8, attempt: u8 },
    /// A stage's attempt recorded out of its number's order, or numbered past the budget.
    Misnumbered { stage: u8, attempt: u8 },
}

/// K6 and K4 on a group's attempt records (record, CID, cut), in the order they were recorded:
/// one record per attempt, each stage's numbered 1, 2, … in that order, within the budget.
pub fn sequence(group: &Group, attempts: &[(&Attempt, Cid, u64)]) -> Result<(), SequenceError> {
    let mut seen: Vec<(u8, u8, Cid)> = Vec::new();
    let mut next = [1u8; 2];
    for (a, c, _) in attempts.iter().filter(|(a, _, _)| a.group == group.group) {
        let (stage, attempt) = (a.stage, a.attempt);
        if let Some((_, _, first)) = seen.iter().find(|(s, n, _)| (*s, *n) == (stage, attempt)) {
            return Err(if first == c {
                SequenceError::Repeated { stage, attempt }
            } else {
                SequenceError::Conflicting { stage, attempt }
            });
        }
        let expected = usize::from(stage)
            .checked_sub(1)
            .and_then(|i| next.get_mut(i));
        match expected {
            Some(n) if *n == attempt && u64::from(attempt) <= group.budget => *n += 1,
            _ => return Err(SequenceError::Misnumbered { stage, attempt }),
        }
        seen.push((stage, attempt, *c));
    }
    Ok(())
}

/// `attempts` as [`sequence`] reads them, never reordered; one past the term is not executed.
pub fn k4(
    group: &Group,
    item: Cid,
    attempts: &[(&Attempt, Cid, u64)],
) -> Result<K4, SequenceError> {
    sequence(group, attempts)?;
    let mut k = K4 {
        procedure: Procedure::Pending,
        used: Vec::new(),
        unused: Vec::new(),
    };
    let (mut stage, mut misses) = (1u8, 0u64);
    for (a, c, cut) in attempts {
        let Some(at) = a.batch.iter().position(|i| *i == item) else {
            continue;
        };
        if a.group != group.group || *cut > group.term {
            continue;
        }
        let reading = a.readings[at];
        if k.procedure != Procedure::Pending {
            k.unused.push((a.stage, a.attempt, reading));
            continue;
        }
        if a.stage != stage {
            continue;
        }
        k.used.push((a.stage, a.attempt, *c));
        match reading {
            Reading::Survives => (stage, misses) = (2, 0),
            Reading::NotConverged => {
                misses += 1;
                if misses == group.budget {
                    k.procedure = Procedure::Exhausted(stage);
                }
            }
            r => k.procedure = Procedure::Concluded(r.verdict().unwrap()),
        }
    }
    Ok(k)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TerminalError {
    Rule,
    Sequence(SequenceError),
    Unreferenced(u8, u8),
    NoEvidence(u8, u8),
    Mismatch(Procedure),
}

/// `17` §9.2 on one terminal record: its rule is its group's record; K4 on the attempts recorded
/// before it gives its outcome and reason; each attempt used is referenced by its own CID.
pub fn validate(
    t: &Terminal,
    group: &Group,
    rule: Cid,
    before: &[(&Attempt, Cid, u64)],
) -> Result<Outcome, TerminalError> {
    if t.rule != rule || !group.items.contains(&t.item) {
        return Err(TerminalError::Rule);
    }
    let k = k4(group, t.item, before).map_err(TerminalError::Sequence)?;
    for (s, n, c) in &k.used {
        let (a, _, _) = before.iter().find(|(_, x, _)| x == c).unwrap();
        if !t.attempts.contains(c) {
            return Err(TerminalError::Unreferenced(*s, *n));
        }
        if a.evidence == [0; 32] {
            return Err(TerminalError::NoEvidence(*s, *n));
        }
    }
    match (k.procedure, t.reason) {
        (Procedure::Concluded(o), Reason::Concluded) if o == t.outcome => Ok(o),
        (Procedure::Exhausted(_), Reason::Exhausted) if t.outcome == Outcome::I => Ok(Outcome::I),
        (p, _) => Err(TerminalError::Mismatch(p)),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pending {
    NotStarted,
    /// The group's attempt records break K4 or K6 ([`SequenceError`]): nothing is derived.
    Incoherent,
    AttemptIndeterminate,
    Shortfall,
    RefusedBatch,
    Held,
    FreezeUnreached,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemState {
    Terminal { outcome: Outcome, pos: u64 },
    Pending(Pending),
}

/// `17` §8.2's cases 2–7 (case 1 does not arise under A).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Case {
    Pending,
    Positive,
    Negative,
    Inconclusive,
    MissingReport,
    Unfrozen,
}

impl Case {
    pub fn number(self) -> u8 {
        match self {
            Case::Pending => 2,
            Case::Positive => 3,
            Case::Negative => 4,
            Case::Inconclusive => 5,
            Case::MissingReport => 6,
            Case::Unfrozen => 7,
        }
    }
}

fn case_of(completed: bool, freeze: Option<u64>, state: ItemState) -> Case {
    match (completed, freeze, state) {
        (false, _, _) => Case::MissingReport,
        (true, None, _) => Case::Unfrozen,
        (true, Some(_), ItemState::Pending(_)) => Case::Pending,
        (true, Some(_), ItemState::Terminal { outcome, .. }) => match outcome {
            Outcome::A => Case::Positive,
            Outcome::R => Case::Negative,
            Outcome::I => Case::Inconclusive,
        },
    }
}

/// Why a verdict's term is undefined (`17` §14.4), apart from cases 2, 6 and 7.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Undefined {
    /// `first_panel_baselines` gave `None`: no other first panelist carries a positive weight.
    NoBaseline,
}

/// B-b's term under A (`17` §8.1, `π = 1`): `g(I) = 0` needs no report; cases 2, 6, 7 have none.
fn contribution(case: Case, p: Option<f64>, b: Option<f64>) -> Result<Option<f64>, Undefined> {
    let o = match case {
        Case::Positive => 1.0,
        Case::Negative => 0.0,
        Case::Inconclusive => return Ok(Some(0.0)),
        _ => return Ok(None),
    };
    let p = p.expect("a verdict's case has a completed report");
    let b = b.ok_or(Undefined::NoBaseline)?;
    Ok(Some(difference_score(p, b, o)))
}

/// A declared member's term from its case, report and first-panel baseline, read on a verdict.
pub fn member(item: usize, case: Case, report: Option<f64>, baseline: Option<f64>) -> Member {
    let baseline = baseline.filter(|_| matches!(case, Case::Positive | Case::Negative));
    let (contribution, undefined) = match contribution(case, report, baseline) {
        Ok(g) => (g, None),
        Err(why) => (None, Some(why)),
    };
    Member {
        item,
        case,
        baseline,
        contribution,
        undefined,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Assignment {
    pub nym: Nym,
    pub item: usize,
    pub extra: bool,
    pub committed: bool,
    pub report: Option<f64>,
    pub case: Case,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    pub item: usize,
    pub case: Case,
    pub baseline: Option<f64>,
    pub contribution: Option<f64>,
    pub undefined: Option<Undefined>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Final(f64),
    /// D3 under `π = 1`: the sum within the pending members' count of its known part.
    Bound {
        sum: (f64, f64),
        mean: (f64, f64),
    },
    Unresolved(Case),
    /// No case 6 or 7, but a member's term undefined: no value and no interval.
    Undefined(Undefined),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cohort {
    pub nym: Nym,
    pub members: Vec<Member>,
    pub n: usize,
    pub o: usize,
    pub v: usize,
    /// The sum of the defined terms.
    pub known: f64,
    /// The members whose term is undefined, whatever else blocks the value.
    pub undefined: Vec<usize>,
    pub value: Value,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Start {
    NotStarted,
    Ordered { freeze: u64, start: u64 },
    Held { production: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartError {
    BeforeFreeze { start: u64 },
    RecordWithoutStart { record: u64 },
    RecordBeforeStart { record: u64 },
    LoggedWhileHeld { record: u64 },
}

/// T1 (K7): the start is a synthetic event the fixture supplies on the log; it must follow
/// `Φ_G` and precede the group's first attempt, shortfall or refused-batch record.
pub fn t1_start(
    freeze: Option<u64>,
    start: Option<u64>,
    first: Option<u64>,
) -> Result<Start, StartError> {
    match (start, first, freeze) {
        (None, None, _) => Ok(Start::NotStarted),
        (None, Some(record), _) => Err(StartError::RecordWithoutStart { record }),
        (Some(start), _, None) => Err(StartError::BeforeFreeze { start }),
        (Some(start), _, Some(f)) if f >= start => Err(StartError::BeforeFreeze { start }),
        (Some(start), Some(record), _) if record < start => {
            Err(StartError::RecordBeforeStart { record })
        }
        (Some(start), _, Some(freeze)) => Ok(Start::Ordered { freeze, start }),
    }
}

/// T2 (K7): no record of the group logged before `Φ_G`.
pub fn t2_hold(
    freeze: Option<u64>,
    logged: Option<u64>,
    production: u64,
) -> Result<Start, StartError> {
    match logged {
        Some(record) if freeze.is_none_or(|f| record < f) => {
            Err(StartError::LoggedWhileHeld { record })
        }
        _ => Ok(Start::Held { production }),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupReport {
    pub group: u8,
    pub construction: Construction,
    pub freeze: Option<u64>,
    pub start: Result<Start, StartError>,
    pub sequence: Result<(), SequenceError>,
    /// Per stage, the distinct attempts recorded by the term, however often each is logged.
    pub executed: [u64; 2],
    pub refused: [u64; 2],
    pub term: u64,
    /// The fixture's held records of the group, never read from the log.
    pub held: usize,
    pub survivors_batched: bool,
}

/// A declared count split by source: records in the log's prefix, records the fixture holds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub log: u64,
    pub held: u64,
}

impl Tally {
    pub fn total(self) -> u64 {
        self.log + self.held
    }

    fn add(&mut self, held: bool, n: u64) {
        *if held { &mut self.held } else { &mut self.log } += n;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Costs {
    pub participations: [Tally; 2],
    pub answers: Tally,
    pub fits: Tally,
    pub searches: Tally,
    pub runs: Tally,
    pub refused: Tally,
    /// Per batch: group, stage, pseudonyms its gate counted; never summed across batches.
    pub batches: Vec<(u8, u8, u64)>,
    pub search_runs: Vec<u64>,
}

impl Costs {
    fn account(&mut self, record: &Record, held: bool) {
        let (group, stage, admitted, items) = match record {
            Record::Attempt(a) => {
                if a.stage == 1 {
                    self.fits.add(held, 1);
                } else {
                    self.searches.add(held, 1);
                    self.runs.add(held, a.runs);
                    self.search_runs.push(a.runs);
                }
                (a.group, a.stage, a.participations, a.batch.len())
            }
            Record::Shortfall(s) => (s.group, s.stage, s.have, s.batch.len()),
            Record::Refused(b) => (b.group, b.stage, b.participations, b.batch.len()),
            _ => return,
        };
        if !matches!(record, Record::Attempt(_)) {
            self.refused.add(held, 1);
        }
        self.participations[usize::from(stage) - 1].add(held, admitted);
        self.answers.add(held, admitted * (items as u64 + ANCHORS));
        self.batches.push((group, stage, admitted));
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ItemReport {
    pub state: ItemState,
    pub lifecycle: Option<State>,
    pub freeze: Option<u64>,
    pub unused: Vec<(u8, u8, Reading)>,
}

/// A pair of `17` §13.3: a logged terminal record (its item) against an item (ii) or a
/// group (iv), with its evidence at (R) and at (E).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pair {
    pub record: usize,
    pub target: usize,
    pub r: Evidence,
    pub e: Evidence,
}

/// `docs/20` §7's report on a prefix. Every field derives from the log's prefix except `held`,
/// each group's `held` and the `held` side of `costs`: the fixture's held records.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub prefix: u64,
    pub items: Vec<ItemReport>,
    pub assignments: Vec<Assignment>,
    pub cohorts: Vec<Cohort>,
    pub groups: Vec<GroupReport>,
    pub ii: Vec<Pair>,
    pub iv: Vec<Pair>,
    pub costs: Costs,
    pub invalid: Vec<(usize, TerminalError)>,
    pub held: Vec<(usize, Result<Outcome, TerminalError>)>,
    /// The items whose first-panel baselines the check formed: those past their freeze.
    pub baselines: Vec<usize>,
}

struct Logged<'a> {
    groups: Vec<(&'a Examined, &'a Group)>,
    starts: Vec<(&'a Examined, u8)>,
    attempts: Vec<(&'a Examined, &'a Attempt)>,
    others: Vec<(&'a Examined, &'a Record)>,
    terminals: Vec<(&'a Examined, &'a Terminal)>,
}

fn sort<'a>(records: &'a [(&'a Examined, Record)]) -> Logged<'a> {
    let mut l = Logged {
        groups: Vec::new(),
        starts: Vec::new(),
        attempts: Vec::new(),
        others: Vec::new(),
        terminals: Vec::new(),
    };
    for (e, r) in records {
        match r {
            Record::Group(g) => l.groups.push((e, g)),
            Record::Start(g) => l.starts.push((e, *g)),
            Record::Attempt(a) => l.attempts.push((e, a)),
            Record::Shortfall(_) | Record::Refused(_) => l.others.push((e, r)),
            Record::Terminal(t) => l.terminals.push((e, t)),
            Record::Setup(_) => {}
        }
    }
    l
}

fn group_of(record: &Record) -> Option<u8> {
    match record {
        Record::Shortfall(s) => Some(s.group),
        Record::Refused(b) => Some(b.group),
        Record::Attempt(a) => Some(a.group),
        Record::Start(g) => Some(*g),
        _ => None,
    }
}

fn batch_of(record: &Record) -> &[Cid] {
    match record {
        Record::Shortfall(s) => &s.batch,
        Record::Refused(b) => &b.batch,
        _ => &[],
    }
}

impl Snapshot {
    pub fn of(main: &Main, view: &LogView, prefix: u64) -> Snapshot {
        let records = view.records(prefix);
        let setup = records
            .iter()
            .find_map(|(_, r)| match r {
                Record::Setup(s) => Some(s),
                _ => None,
            })
            .expect("S0, in cut 0");
        let logged = sort(&records);
        let index = |c: &Cid| main.items.iter().position(|i| i == c).unwrap();
        let group = |item: &Cid| {
            let (e, g) = logged
                .groups
                .iter()
                .find(|(_, g)| g.items.contains(item))
                .unwrap();
            (*g, cid(&e.object))
        };
        let steps = view.steps(prefix);
        let freezes: Vec<Option<u64>> = main
            .items
            .iter()
            .map(|c| steps.get(c).and_then(|s| freeze(s)))
            .collect();
        let group_freeze = |g: &Group| {
            let all: Option<Vec<u64>> = g.items.iter().map(|c| freezes[index(c)]).collect();
            all.map(|f| f.into_iter().max().unwrap())
        };
        let attempts_of = |g: u8, below: u64| {
            logged
                .attempts
                .iter()
                .filter(|(e, a)| a.group == g && e.pos < below)
                .map(|(e, a)| (*a, cid(&e.object), e.cut))
                .collect::<Vec<_>>()
        };

        let mut terminal: BTreeMap<usize, (Outcome, u64)> = BTreeMap::new();
        let mut invalid = Vec::new();
        for (e, t) in &logged.terminals {
            let (g, rule) = group(&t.item);
            match validate(t, g, rule, &attempts_of(g.group, e.pos)) {
                Ok(o) => {
                    terminal.entry(index(&t.item)).or_insert((o, e.pos));
                }
                Err(err) => invalid.push((index(&t.item), err)),
            }
        }

        let mut items = Vec::new();
        for (j, item) in main.items.iter().enumerate() {
            let (g, _) = group(item);
            let in_batch = |kind: fn(&Record) -> bool| {
                logged
                    .others
                    .iter()
                    .any(|(_, r)| kind(r) && batch_of(r).contains(item))
            };
            let state = if let Some((outcome, pos)) = terminal.get(&j) {
                ItemState::Terminal {
                    outcome: *outcome,
                    pos: *pos,
                }
            } else if g.construction == Construction::T2 && group_freeze(g).is_none() {
                ItemState::Pending(Pending::Held)
            } else if freezes[j].is_none() {
                ItemState::Pending(Pending::FreezeUnreached)
            } else if !logged.starts.iter().any(|(_, s)| *s == g.group) {
                ItemState::Pending(Pending::NotStarted)
            } else if sequence(g, &attempts_of(g.group, u64::MAX)).is_err() {
                ItemState::Pending(Pending::Incoherent)
            } else if in_batch(|r| matches!(r, Record::Refused(_))) {
                ItemState::Pending(Pending::RefusedBatch)
            } else if in_batch(|r| matches!(r, Record::Shortfall(_))) {
                ItemState::Pending(Pending::Shortfall)
            } else {
                ItemState::Pending(Pending::AttemptIndeterminate)
            };
            let k = k4(g, *item, &attempts_of(g.group, u64::MAX));
            items.push(ItemReport {
                state,
                lifecycle: view.state(prefix, item).cloned(),
                freeze: freezes[j],
                unused: k.map_or(Vec::new(), |k| k.unused),
            });
        }

        let weight = |n: &Nym| {
            setup
                .weights
                .iter()
                .find(|(m, _)| m == n)
                .map_or(1.0, |(_, w)| *w as f64)
        };
        let mut assignments = Vec::new();
        let mut baselines: BTreeMap<usize, Vec<(Nym, Option<f64>)>> = BTreeMap::new();
        for (j, item) in main.items.iter().enumerate() {
            let (g, _) = group(item);
            let close = g.reveal_close[g.items.iter().position(|c| c == item).unwrap()];
            let (mut first, mut extra, mut commits, mut reveals) =
                (Vec::new(), Vec::new(), Vec::new(), BTreeMap::new());
            for (pos, event) in steps.get(item).map_or(&[][..], |s| &s[..]) {
                match event {
                    Event::AssignReviewers { panel, .. } => first = panel.clone(),
                    Event::AssignExtraReviewers { panel } => extra = panel.clone(),
                    Event::Commit { nym, .. } => commits.push(*nym),
                    Event::Reveal { nym, prob, .. } => {
                        reveals.insert(*nym, (*pos, *prob));
                    }
                    _ => {}
                }
            }
            if freezes[j].is_some() {
                let forecasts: Vec<Forecast> = first
                    .iter()
                    .map(|n| Forecast {
                        prob: reveals[n].1,
                        weight: weight(n),
                    })
                    .collect();
                let b = first_panel_baselines(&forecasts);
                baselines.insert(j, first.iter().copied().zip(b).collect());
            }
            let panels = first
                .iter()
                .map(|n| (n, false))
                .chain(extra.iter().map(|n| (n, true)));
            for (n, is_extra) in panels {
                let report = reveals.get(n).copied();
                let completed = report.is_some_and(|(pos, _)| is_extra || pos < close);
                assignments.push(Assignment {
                    nym: *n,
                    item: j,
                    extra: is_extra,
                    committed: commits.contains(n),
                    report: report.map(|(_, p)| p),
                    case: case_of(completed, freezes[j], items[j].state),
                });
            }
        }

        let cohorts = setup
            .cohorts
            .iter()
            .map(|(n, declared)| {
                let members: Vec<Member> = declared
                    .iter()
                    .map(|c| {
                        let j = index(c);
                        let a = assignments
                            .iter()
                            .find(|a| a.nym == *n && a.item == j && !a.extra)
                            .expect("a declared member is a first-panel assignment");
                        let baseline = baselines
                            .get(&j)
                            .and_then(|b| b.iter().find(|(m, _)| m == n).unwrap().1);
                        member(j, a.case, a.report, baseline)
                    })
                    .collect();
                cohort(*n, members)
            })
            .collect();

        let mut groups = Vec::new();
        for (_, g) in &logged.groups {
            let freeze = group_freeze(g);
            let of_group = |r: &Record| group_of(r) == Some(g.group);
            let records_of: Vec<(&Examined, &Record)> = records
                .iter()
                .filter(|(_, r)| of_group(r))
                .map(|(e, r)| (*e, r))
                .collect();
            let first = records_of
                .iter()
                .filter(|(_, r)| !matches!(r, Record::Start(_)))
                .map(|(e, _)| e.pos)
                .min();
            let start = match g.construction {
                Construction::T1 => {
                    let start = logged.starts.iter().find(|(_, s)| *s == g.group);
                    t1_start(freeze, start.map(|(e, _)| e.pos), first)
                }
                Construction::T2 => {
                    let terminals = logged
                        .terminals
                        .iter()
                        .filter(|(_, t)| g.items.contains(&t.item));
                    let any = records_of.iter().map(|(e, _)| e.pos);
                    let logged_first = any.chain(terminals.map(|(e, _)| e.pos)).min();
                    t2_hold(freeze, logged_first, g.production.unwrap())
                }
            };
            let recorded = attempts_of(g.group, u64::MAX);
            let mut executed = [0; 2];
            let mut seen = Vec::new();
            for (a, _, _) in recorded.iter().filter(|(_, _, cut)| *cut <= g.term) {
                if !seen.contains(&(a.stage, a.attempt)) {
                    seen.push((a.stage, a.attempt));
                    executed[usize::from(a.stage) - 1] += 1;
                }
            }
            let mut refused = [0; 2];
            for (_, r) in logged.others.iter().filter(|(_, r)| of_group(r)) {
                let stage = match r {
                    Record::Shortfall(s) => s.stage,
                    Record::Refused(b) => b.stage,
                    _ => unreachable!(),
                };
                refused[usize::from(stage) - 1] += 1;
            }
            let held = main
                .held
                .iter()
                .filter(|r| match r {
                    Record::Terminal(t) => g.items.contains(&t.item),
                    r => of_group(r),
                })
                .count();
            groups.push(GroupReport {
                group: g.group,
                construction: g.construction,
                freeze,
                start,
                sequence: sequence(g, &recorded),
                executed,
                refused,
                term: g.term,
                held,
                survivors_batched: survivors_batched(g, &records_of),
            });
        }

        let refs = References::new(&main.replica, &consortium(), view, prefix);
        let relevant: Vec<_> = main
            .items
            .iter()
            .map(|c| commitments(view, prefix, *c))
            .collect();
        let reference = answers_reference(view, prefix);
        let administered: Vec<u8> = logged
            .groups
            .iter()
            .filter(|(_, g)| {
                logged.starts.iter().any(|(_, s)| *s == g.group)
                    || g.production.is_some_and(|p| p <= prefix)
            })
            .map(|(_, g)| g.group)
            .collect();
        let (mut ii, mut iv) = (Vec::new(), Vec::new());
        for (e, t) in &logged.terminals {
            let j = index(&t.item);
            if terminal.get(&j).is_none_or(|(_, pos)| *pos != e.pos) {
                continue;
            }
            let (g, _) = group(&t.item);
            let mut x = vec![e.id];
            let of_group = logged.attempts.iter().filter(|(_, a)| a.group == g.group);
            x.extend(of_group.map(|(attempt, _)| attempt.id));
            for (k, rel) in relevant.iter().enumerate() {
                ii.push(Pair {
                    record: j,
                    target: k,
                    r: replay_level(view, prefix, rel, &x),
                    e: existence_level(view, prefix, &refs, rel, &x),
                });
            }
            for g in &administered {
                iv.push(Pair {
                    record: j,
                    target: usize::from(*g),
                    r: replay_level(view, prefix, &reference, &x),
                    e: existence_level(view, prefix, &refs, &reference, &x),
                });
            }
        }

        let mut costs = Costs::default();
        let mut accounted = Vec::new();
        let logged_records = records.iter().map(|(_, r)| (r, false));
        for (r, held) in logged_records.chain(main.held.iter().map(|r| (r, true))) {
            if let Record::Attempt(a) = r {
                if accounted.contains(&(a.group, a.stage, a.attempt)) {
                    continue;
                }
                accounted.push((a.group, a.stage, a.attempt));
            }
            costs.account(r, held);
        }

        let (g6, rule) = group(&main.items[10]);
        let production = g6.production.unwrap();
        let held_attempts: Vec<(&Attempt, Cid, u64)> = main
            .held
            .iter()
            .filter_map(|r| match r {
                Record::Attempt(a) => Some((a, r.cid(), production)),
                _ => None,
            })
            .collect();
        let held = main
            .held
            .iter()
            .filter_map(|r| match r {
                Record::Terminal(t) => {
                    Some((index(&t.item), validate(t, g6, rule, &held_attempts)))
                }
                _ => None,
            })
            .collect();

        Snapshot {
            prefix,
            items,
            assignments,
            cohorts,
            groups,
            ii,
            iv,
            costs,
            invalid,
            held,
            baselines: baselines.into_keys().collect(),
        }
    }
}

/// K2's cohort (`17` §8.3): a final value once every member is in case 3–5 with its term
/// defined; with only case-2 members pending, D3's bound; otherwise, no value and no interval.
pub fn cohort(nym: Nym, members: Vec<Member>) -> Cohort {
    let n = members.len();
    let count = |f: fn(Case) -> bool| members.iter().filter(|m| f(m.case)).count();
    let o = count(|c| matches!(c, Case::Positive | Case::Negative | Case::Inconclusive));
    let v = count(|c| matches!(c, Case::Positive | Case::Negative));
    let known: f64 = members.iter().filter_map(|m| m.contribution).sum();
    let pending = count(|c| c == Case::Pending) as f64;
    let blocking = members
        .iter()
        .map(|m| m.case)
        .find(|c| matches!(c, Case::MissingReport | Case::Unfrozen));
    let undefined: Vec<usize> = members
        .iter()
        .filter(|m| m.undefined.is_some())
        .map(|m| m.item)
        .collect();
    let why = members.iter().find_map(|m| m.undefined);
    let value = match (blocking, why) {
        (Some(c), _) => Value::Unresolved(c),
        (None, Some(why)) => Value::Undefined(why),
        (None, None) if pending == 0.0 => Value::Final(known / n as f64),
        (None, None) => {
            let sum = (known - pending, known + pending);
            Value::Bound {
                sum,
                mean: (sum.0 / n as f64, sum.1 / n as f64),
            }
        }
    };
    Cohort {
        nym,
        members,
        n,
        o,
        v,
        known,
        undefined,
        value,
    }
}

/// K4: every stage-2 batch, executed or refused, is the stage-1 survivors.
fn survivors_batched(g: &Group, records: &[(&Examined, &Record)]) -> bool {
    let Some(stage1) = records.iter().find_map(|(_, r)| match r {
        Record::Attempt(a) if a.stage == 1 => Some(a),
        _ => None,
    }) else {
        return true;
    };
    let survivors: Vec<Cid> = stage1
        .batch
        .iter()
        .zip(&stage1.readings)
        .filter(|(_, r)| **r == Reading::Survives)
        .map(|(c, _)| *c)
        .collect();
    records.iter().all(|(_, r)| match r {
        Record::Attempt(a) if a.stage == 2 => a.batch == survivors,
        Record::Refused(b) if b.stage == 2 => b.batch == survivors,
        _ => true,
    }) && g.items.iter().all(|c| stage1.batch.contains(c))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CohortError {
    Undeclared { nym: Nym, item: Cid },
    Unassigned { nym: Nym, item: Cid },
    DeclaredAfter { assignment: u64 },
}

/// K2: `S0`'s cohorts precede every assignment and are exactly their nyms' assignments in all
/// the log the check holds; an assignment outside one is reported, never added to it.
pub fn cohorts_complete(view: &LogView, last: u64) -> Vec<CohortError> {
    let records = view.records(last);
    let (s0, setup) = records
        .iter()
        .find_map(|(e, r)| match r {
            Record::Setup(s) => Some((e.pos, s)),
            _ => None,
        })
        .expect("S0");
    let mut assigned: Vec<(u64, Nym, Cid)> = Vec::new();
    for (item, steps) in view.steps(last) {
        for (pos, event) in steps {
            if let Event::AssignReviewers { panel, .. } | Event::AssignExtraReviewers { panel } =
                event
            {
                assigned.extend(panel.into_iter().map(|n| (pos, n, item)));
            }
        }
    }
    let mut errors: Vec<CohortError> = assigned
        .iter()
        .filter(|(pos, _, _)| *pos < s0)
        .map(|(pos, _, _)| CohortError::DeclaredAfter { assignment: *pos })
        .collect();
    for (nym, items) in &setup.cohorts {
        let of: Vec<Cid> = assigned
            .iter()
            .filter(|a| a.1 == *nym)
            .map(|a| a.2)
            .collect();
        errors.extend(
            of.iter()
                .filter(|c| !items.contains(c))
                .map(|c| CohortError::Undeclared {
                    nym: *nym,
                    item: *c,
                }),
        );
        errors.extend(
            items
                .iter()
                .filter(|c| !of.contains(c))
                .map(|c| CohortError::Unassigned {
                    nym: *nym,
                    item: *c,
                }),
        );
    }
    errors
}
