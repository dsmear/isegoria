//! The snapshot of B-b's scorer's inputs (`docs/17` §14.10): a pure function of a ledger prefix
//! and of experimental proposed records, which no producer writes and which are no production
//! evidence; no production caller.

use crate::cohort_scores::{
    score, Cohort, Counts, Extra, InputError, Item, Outcome, Panelist, Report, Scores, Selection,
};
use crate::events::NodeEvent;
use crate::ledger::Refusal;
use crate::lifecycle::{self, Event, Invalid, State};
use identity::nym::Nym;
use network::cid::{cid, Cid};
use network::codec::Writer;
use network::cut::MemberObject;
use network::replica::EntryId;
use std::collections::{BTreeMap, BTreeSet};

const PROPOSED: u8 = 0xE0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Design {
    A,
    C { inclusion: f64 },
}

/// Experimental records, never `NodeEvent`s; `encode` is canonical, `id` its CID, the identity
/// duplicates are judged by.
#[derive(Clone, Debug, PartialEq)]
pub enum Proposed {
    /// The frozen weights of the first panels that cuts of `epoch` assign.
    Weights {
        epoch: u64,
        weights: BTreeMap<Nym, f64>,
    },
    /// `group` and `rule` declare the item's association, which its terminal must name.
    Design {
        item: Cid,
        group: Cid,
        design: Design,
        rule: Cid,
    },
    /// C's draw in `round`, as recorded: the round is not checked here.
    Draw {
        item: Cid,
        round: u64,
        selected: bool,
    },
    /// `references` and `verified` are the verifier's, supplied: no verifier runs here.
    Terminal {
        item: Cid,
        group: Cid,
        rule: Cid,
        outcome: Outcome,
        references: Vec<Cid>,
        verified: bool,
    },
    Cohort {
        nym: Nym,
        epoch: u64,
        items: BTreeSet<Cid>,
    },
}

impl Proposed {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(PROPOSED);
        match self {
            Proposed::Weights { epoch, weights } => {
                w.u8(1).u64(*epoch).u64(weights.len() as u64);
                for (nym, x) in weights {
                    w.fixed(&nym.0).u64(x.to_bits());
                }
            }
            Proposed::Design {
                item,
                group,
                design,
                rule,
            } => {
                w.u8(2).fixed(&item.0).fixed(&group.0).fixed(&rule.0);
                match design {
                    Design::A => w.u8(0),
                    Design::C { inclusion } => w.u8(1).u64(inclusion.to_bits()),
                };
            }
            Proposed::Draw {
                item,
                round,
                selected,
            } => {
                w.u8(3).fixed(&item.0).u64(*round).u8(u8::from(*selected));
            }
            Proposed::Terminal {
                item,
                group,
                rule,
                outcome,
                references,
                verified,
            } => {
                let o = match outcome {
                    Outcome::A => 0,
                    Outcome::R => 1,
                    Outcome::I => 2,
                };
                w.u8(4).fixed(&item.0).fixed(&group.0).fixed(&rule.0).u8(o);
                w.u64(references.len() as u64);
                for r in references {
                    w.fixed(&r.0);
                }
                w.u8(u8::from(*verified));
            }
            Proposed::Cohort { nym, epoch, items } => {
                w.u8(5).fixed(&nym.0).u64(*epoch).u64(items.len() as u64);
                for c in items {
                    w.fixed(&c.0);
                }
            }
        }
        w.finish()
    }

    pub fn id(&self) -> Cid {
        cid(&self.encode())
    }
}

/// One slot of a cut, in application order: an entry the ledger applied, its object as the
/// replica holds it, in `CutReport::applied`'s order; or a proposed record the input places.
#[derive(Clone, Debug, PartialEq)]
pub enum Slot {
    Applied { id: EntryId, object: Vec<u8> },
    Proposed(Proposed),
}

/// A cut the ledger applied, whether or not it applied a `NodeEvent`. `digest` names the ledger's
/// prefix only: it authenticates no proposed record, slot or refusal. `refused`, the ledger's
/// refusals as supplied (`CutReport::refused`): an empty list does not show that there were none.
#[derive(Debug, PartialEq)]
pub struct CutInput {
    pub number: u64,
    pub epoch: u64,
    pub closes: bool,
    pub digest: [u8; 32],
    pub slots: Vec<Slot>,
    pub refused: Vec<(EntryId, Refusal)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub cut: u64,
    pub slot: usize,
}

/// Why an item is invalid, settled at its first assignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invalidity {
    NoDesign,
    NoWeights,
    Unweighted(Nym),
    Weight(Nym),
}

/// `status`: the scorer index of a valid item, or every reason it is invalid.
#[derive(Clone, Debug, PartialEq)]
pub struct Registered {
    pub cid: Cid,
    pub assigned: Position,
    pub epoch: u64,
    pub frozen: Option<Position>,
    pub status: Result<usize, Vec<Invalidity>>,
}

/// `item` is a register index; `epoch` is the assigning cut's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Assigned {
    pub nym: Nym,
    pub item: usize,
    pub extra: bool,
    pub at: Position,
    pub epoch: u64,
}

/// `K(nym, epoch)`: `members` by register index, in the register's order; `scored`, its index
/// among the cohorts passed to `score`, once closed and holding no invalid item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CohortEntry {
    pub nym: Nym,
    pub epoch: u64,
    pub members: Vec<usize>,
    pub closed: bool,
    pub invalid: Vec<usize>,
    pub scored: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    Duplicate {
        first: Position,
    },
    Conflict {
        accepted: Position,
    },
    /// Placed after the first assignment it must precede.
    Late,
    Inclusion,
    NoDesign,
    /// A draw for an item of A's design.
    Shape,
    BeforeFreeze,
    NotSelectedAtCertainty,
    Unselected,
    Group,
    Rule,
    Unreferenced,
    Unverified,
    CohortOpen,
    Membership,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Judged {
    pub at: Position,
    pub id: Cid,
    pub refused: Option<Refused>,
}

/// `assignments` is `|R_u|`; `unscored`, those on invalid items; `valid`, `Scores::counts` over
/// the valid items only, never `R_u`'s.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReviewerCounts {
    pub assignments: usize,
    pub unscored: usize,
    pub valid: Counts,
}

/// `cut` and `digest` name the ledger's prefix, not the proposed records or slots read with it;
/// `scored[k]` is the register index of the scorer's item `k`; `inputs` and `scorer_cohorts` are
/// what `score` received; `refusals`, the supplied ledger refusals by cut number.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub cut: u64,
    pub digest: [u8; 32],
    pub items: Vec<Registered>,
    pub register: Vec<Assigned>,
    pub cohorts: Vec<CohortEntry>,
    pub records: Vec<Judged>,
    pub refusals: Vec<(u64, EntryId)>,
    pub scored: Vec<usize>,
    pub inputs: Vec<Item>,
    pub scorer_cohorts: Vec<Cohort>,
    pub scores: Scores,
}

impl Snapshot {
    pub fn counts(&self, nym: Nym) -> ReviewerCounts {
        let mine = || self.register.iter().filter(move |a| a.nym == nym);
        ReviewerCounts {
            assignments: mine().count(),
            unscored: mine()
                .filter(|a| self.items[a.item].status.is_err())
                .count(),
            valid: self.scores.counts(nym),
        }
    }
}

/// An input outside the preconditions the snapshot checks: no snapshot exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Incoherence {
    NoCut,
    Number { found: u64, expected: u64 },
    Epoch { cut: u64 },
    Repeated(EntryId),
    AppliedAndRefused(EntryId),
    Undecodable(Position),
    Redeposited(Position),
    UnknownItem(Position),
    ItemMismatch(Position),
    Lifecycle(Position, Invalid),
}

/// `Defect`: `score` refused what the snapshot gave it, a defect of this component.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Incoherent(Incoherence),
    Defect(InputError),
}

impl From<Incoherence> for Error {
    fn from(i: Incoherence) -> Self {
        Error::Incoherent(i)
    }
}

struct Track {
    cid: Cid,
    assigned: Position,
    epoch: u64,
    first: Vec<(Nym, Option<f64>)>,
    extra: Vec<(Nym, Option<f64>)>,
    frozen: Option<Position>,
    invalid: Vec<Invalidity>,
    valid: Option<(Design, Vec<f64>)>,
}

#[derive(Default)]
struct Walk {
    states: BTreeMap<Cid, State>,
    index: BTreeMap<Cid, usize>,
    tracks: Vec<Track>,
    register: Vec<Assigned>,
    started: BTreeSet<u64>,
    weights: BTreeMap<u64, (Position, BTreeMap<Nym, f64>)>,
    designs: BTreeMap<Cid, (Position, Design, Cid, Cid)>,
    draws: BTreeMap<Cid, (Position, bool)>,
    terminals: BTreeMap<Cid, (Position, Outcome)>,
    cohorts: BTreeMap<(Nym, u64), Position>,
    seen: BTreeMap<Cid, Position>,
    records: Vec<Judged>,
}

fn ends(cut: &CutInput, epoch: u64) -> bool {
    cut.epoch > epoch || (cut.epoch == epoch && cut.closes)
}

fn before_freeze(state: &State) -> bool {
    matches!(
        state,
        State::Deposited
            | State::Admitted
            | State::InReview { .. }
            | State::Revealing { .. }
            | State::SupplementaryReview { .. }
            | State::AppealEligible
    )
}

fn check(cuts: &[CutInput]) -> Result<(), Incoherence> {
    for (i, c) in cuts.iter().enumerate() {
        let expected = i as u64;
        if c.number != expected {
            return Err(Incoherence::Number {
                found: c.number,
                expected,
            });
        }
        let p = i.checked_sub(1).map(|k| &cuts[k]);
        if p.is_some_and(|p| c.epoch < p.epoch || (p.closes && c.epoch == p.epoch)) {
            return Err(Incoherence::Epoch { cut: c.number });
        }
    }
    let mut applied = BTreeSet::new();
    for slot in cuts.iter().flat_map(|c| &c.slots) {
        if let Slot::Applied { id, .. } = slot {
            if !applied.insert(*id) {
                return Err(Incoherence::Repeated(*id));
            }
        }
    }
    let mut refused = BTreeSet::new();
    for (id, _) in cuts.iter().flat_map(|c| &c.refused) {
        if applied.contains(id) {
            return Err(Incoherence::AppliedAndRefused(*id));
        }
        if !refused.insert(*id) {
            return Err(Incoherence::Repeated(*id));
        }
    }
    Ok(())
}

impl Walk {
    fn object(&mut self, at: Position, epoch: u64, object: &[u8]) -> Result<(), Incoherence> {
        if MemberObject::decode(object).is_some() {
            return Ok(());
        }
        match NodeEvent::decode(object).ok_or(Incoherence::Undecodable(at))? {
            NodeEvent::Deposit { draft, .. } => {
                if self
                    .states
                    .insert(draft.content_id(), State::Deposited)
                    .is_some()
                {
                    return Err(Incoherence::Redeposited(at));
                }
                Ok(())
            }
            NodeEvent::Step { item, event } => self.step(at, epoch, item, event),
            _ => Ok(()),
        }
    }

    /// `NodeState::step`'s checks, the transition itself left to `lifecycle::step`.
    fn step(
        &mut self,
        at: Position,
        epoch: u64,
        item: Cid,
        event: Event,
    ) -> Result<(), Incoherence> {
        let state = self
            .states
            .get(&item)
            .ok_or(Incoherence::UnknownItem(at))?
            .clone();
        if matches!(&event, Event::AssignReviewers { item: named, .. } if *named != item) {
            return Err(Incoherence::ItemMismatch(at));
        }
        let next = lifecycle::step(state.clone(), event.clone())
            .map_err(|invalid| Incoherence::Lifecycle(at, invalid))?;
        match (&state, event) {
            (_, Event::AssignReviewers { panel, .. }) => self.assign(at, epoch, item, &panel),
            (_, Event::AssignExtraReviewers { panel }) => {
                if let Some(&r) = self.index.get(&item) {
                    self.tracks[r].extra = panel.iter().map(|&n| (n, None)).collect();
                    self.seat(r, &panel, true, at, epoch);
                }
            }
            (State::Revealing { .. }, Event::Reveal { nym, prob, .. }) => {
                self.reveal(item, nym, prob, false)
            }
            (State::SupplementaryReview { .. }, Event::Reveal { nym, prob, .. }) => {
                self.reveal(item, nym, prob, true)
            }
            _ => {}
        }
        if !before_freeze(&next) {
            if let Some(&r) = self.index.get(&item) {
                self.tracks[r].frozen.get_or_insert(at);
            }
        }
        self.states.insert(item, next);
        Ok(())
    }

    fn assign(&mut self, at: Position, epoch: u64, item: Cid, panel: &[Nym]) {
        let mut invalid = Vec::new();
        let design = self.designs.get(&item).map(|&(_, d, ..)| d);
        if design.is_none() {
            invalid.push(Invalidity::NoDesign);
        }
        let mut weights = Vec::new();
        match self.weights.get(&epoch) {
            None => invalid.push(Invalidity::NoWeights),
            Some((_, record)) => {
                for &nym in panel {
                    match record.get(&nym) {
                        None => invalid.push(Invalidity::Unweighted(nym)),
                        Some(&w) if !(w.is_finite() && w >= 0.0) => {
                            invalid.push(Invalidity::Weight(nym))
                        }
                        Some(&w) => weights.push(w),
                    }
                }
            }
        }
        let valid = design.filter(|_| invalid.is_empty()).map(|d| (d, weights));
        let r = self.tracks.len();
        self.index.insert(item, r);
        self.tracks.push(Track {
            cid: item,
            assigned: at,
            epoch,
            first: panel.iter().map(|&n| (n, None)).collect(),
            extra: Vec::new(),
            frozen: None,
            invalid,
            valid,
        });
        self.seat(r, panel, false, at, epoch);
    }

    fn seat(&mut self, item: usize, panel: &[Nym], extra: bool, at: Position, epoch: u64) {
        self.started.insert(epoch);
        self.register.extend(panel.iter().map(|&nym| Assigned {
            nym,
            item,
            extra,
            at,
            epoch,
        }));
    }

    fn reveal(&mut self, item: Cid, nym: Nym, prob: f64, extra: bool) {
        let Some(&r) = self.index.get(&item) else {
            return;
        };
        let t = &mut self.tracks[r];
        let seats = if extra { &mut t.extra } else { &mut t.first };
        if let Some(seat) = seats.iter_mut().find(|(n, _)| *n == nym) {
            seat.1 = Some(prob);
        }
    }

    fn frozen(&self, item: &Cid) -> bool {
        self.index
            .get(item)
            .is_some_and(|&r| self.tracks[r].frozen.is_some())
    }

    fn judge(&mut self, closed: impl Fn(u64) -> bool, at: Position, record: &Proposed) {
        let id = record.id();
        let refused = match self.seen.get(&id) {
            Some(&first) => Some(Refused::Duplicate { first }),
            None => {
                self.seen.insert(id, at);
                self.accept(closed, at, record).err()
            }
        };
        self.records.push(Judged { at, id, refused });
    }

    fn accept(
        &mut self,
        closed: impl Fn(u64) -> bool,
        at: Position,
        record: &Proposed,
    ) -> Result<(), Refused> {
        let conflict = |p: &Position| Refused::Conflict { accepted: *p };
        match record {
            Proposed::Weights { epoch, weights } => {
                if let Some((p, _)) = self.weights.get(epoch) {
                    return Err(conflict(p));
                }
                if self.started.contains(epoch) {
                    return Err(Refused::Late);
                }
                self.weights.insert(*epoch, (at, weights.clone()));
            }
            Proposed::Design {
                item,
                group,
                design,
                rule,
            } => {
                if let Some((p, ..)) = self.designs.get(item) {
                    return Err(conflict(p));
                }
                let outside = |pi: f64| !(pi > 0.0 && pi <= 1.0);
                if matches!(design, Design::C { inclusion } if outside(*inclusion)) {
                    return Err(Refused::Inclusion);
                }
                if self.index.contains_key(item) {
                    return Err(Refused::Late);
                }
                self.designs.insert(*item, (at, *design, *group, *rule));
            }
            Proposed::Draw { item, selected, .. } => {
                if let Some((p, _)) = self.draws.get(item) {
                    return Err(conflict(p));
                }
                let &(_, design, ..) = self.designs.get(item).ok_or(Refused::NoDesign)?;
                let Design::C { inclusion } = design else {
                    return Err(Refused::Shape);
                };
                if !self.frozen(item) {
                    return Err(Refused::BeforeFreeze);
                }
                if !selected && inclusion == 1.0 {
                    return Err(Refused::NotSelectedAtCertainty);
                }
                self.draws.insert(*item, (at, *selected));
            }
            Proposed::Terminal {
                item,
                group,
                rule,
                outcome,
                references,
                verified,
            } => {
                if let Some((p, _)) = self.terminals.get(item) {
                    return Err(conflict(p));
                }
                let &(_, design, own_group, own_rule) =
                    self.designs.get(item).ok_or(Refused::NoDesign)?;
                if *group != own_group {
                    return Err(Refused::Group);
                }
                if *rule != own_rule {
                    return Err(Refused::Rule);
                }
                if !self.frozen(item) {
                    return Err(Refused::BeforeFreeze);
                }
                let drawn = matches!(self.draws.get(item), Some((_, true)));
                if !(design == Design::A || drawn) {
                    return Err(Refused::Unselected);
                }
                if references.is_empty() {
                    return Err(Refused::Unreferenced);
                }
                if !verified {
                    return Err(Refused::Unverified);
                }
                self.terminals.insert(*item, (at, *outcome));
            }
            Proposed::Cohort { nym, epoch, items } => {
                if let Some(p) = self.cohorts.get(&(*nym, *epoch)) {
                    return Err(conflict(p));
                }
                if !closed(*epoch) {
                    return Err(Refused::CohortOpen);
                }
                let members: BTreeSet<Cid> = self
                    .register
                    .iter()
                    .filter(|a| a.nym == *nym && a.epoch == *epoch)
                    .map(|a| self.tracks[a.item].cid)
                    .collect();
                if members != *items {
                    return Err(Refused::Membership);
                }
                self.cohorts.insert((*nym, *epoch), at);
            }
        }
        Ok(())
    }

    fn input(&self, t: &Track) -> Option<Item> {
        let (design, weights) = t.valid.as_ref()?;
        let report = |p: Option<f64>| p.map_or(Report::Open, Report::Revealed);
        let outcome = self.terminals.get(&t.cid).map(|&(_, o)| o);
        let selection = match (*design, self.draws.get(&t.cid)) {
            (Design::A, _) => Selection::Selected {
                inclusion: 1.0,
                outcome,
            },
            (Design::C { .. }, None) => Selection::Undrawn,
            (Design::C { inclusion }, Some((_, false))) => Selection::NotSelected { inclusion },
            (Design::C { inclusion }, Some((_, true))) => {
                Selection::Selected { inclusion, outcome }
            }
        };
        let first = t.first.iter().zip(weights);
        Some(Item {
            first: first
                .map(|(&(nym, p), &weight)| Panelist {
                    nym,
                    report: report(p),
                    weight,
                })
                .collect(),
            extra: t
                .extra
                .iter()
                .map(|&(nym, p)| Extra {
                    nym,
                    report: report(p),
                })
                .collect(),
            frozen: t.frozen.is_some(),
            selection,
        })
    }

    fn finish(self, cuts: &[CutInput], last: &CutInput) -> Result<Snapshot, Error> {
        let (mut scored, mut inputs) = (Vec::new(), Vec::new());
        let mut items = Vec::new();
        for (r, t) in self.tracks.iter().enumerate() {
            let status = match self.input(t) {
                Some(item) => {
                    scored.push(r);
                    inputs.push(item);
                    Ok(inputs.len() - 1)
                }
                None => Err(t.invalid.clone()),
            };
            items.push(Registered {
                cid: t.cid,
                assigned: t.assigned,
                epoch: t.epoch,
                frozen: t.frozen,
                status,
            });
        }
        let mut groups: BTreeMap<(u64, Nym), Vec<usize>> = BTreeMap::new();
        for a in &self.register {
            groups.entry((a.epoch, a.nym)).or_default().push(a.item);
        }
        let (mut cohorts, mut scorer_cohorts) = (Vec::new(), Vec::new());
        for ((epoch, nym), mut members) in groups {
            members.sort_unstable();
            let closed = cuts.iter().any(|c| ends(c, epoch));
            let invalid: Vec<usize> = members
                .iter()
                .copied()
                .filter(|&r| items[r].status.is_err())
                .collect();
            let scored = (closed && invalid.is_empty()).then(|| {
                let at = members.iter().filter_map(|&r| items[r].status.clone().ok());
                scorer_cohorts.push(Cohort {
                    nym,
                    items: at.collect(),
                });
                scorer_cohorts.len() - 1
            });
            cohorts.push(CohortEntry {
                nym,
                epoch,
                members,
                closed,
                invalid,
                scored,
            });
        }
        let scores = score(&inputs, &scorer_cohorts).map_err(Error::Defect)?;
        Ok(Snapshot {
            cut: last.number,
            digest: last.digest,
            items,
            register: self.register,
            cohorts,
            records: self.records,
            refusals: cuts
                .iter()
                .flat_map(|c| c.refused.iter().map(|(id, _)| (c.number, *id)))
                .collect(),
            scored,
            inputs,
            scorer_cohorts,
            scores,
        })
    }
}

/// The snapshot of `cuts`, cut 0 through the last (`docs/17` §14.10). A truncation of one input
/// gives the snapshot of that prefix, its proposed records and slots unchanged; equal digests with
/// other records or slots are other inputs, each read on its own data.
pub fn snapshot(cuts: &[CutInput]) -> Result<Snapshot, Error> {
    let last = cuts.last().ok_or(Incoherence::NoCut)?;
    check(cuts)?;
    let mut walk = Walk::default();
    for (i, cut) in cuts.iter().enumerate() {
        for (slot, s) in cut.slots.iter().enumerate() {
            let at = Position {
                cut: cut.number,
                slot,
            };
            match s {
                Slot::Applied { object, .. } => walk.object(at, cut.epoch, object)?,
                Slot::Proposed(record) => {
                    let closed = |e: u64| cuts[..i].iter().any(|c| ends(c, e)) || cut.epoch > e;
                    walk.judge(closed, at, record);
                }
            }
        }
    }
    walk.finish(cuts, last)
}
