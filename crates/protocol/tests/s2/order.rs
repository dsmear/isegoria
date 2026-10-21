//! The order evidence of `docs/17` §§13.2–13.4, as `docs/20` §6 checks it: a pair's outcome at
//! the replay check (R) and at documented existence precedence (E), on a declared prefix.

use super::replay::LogView;
use network::cid::Cid;
use network::consortium::Consortium;
use network::cut::{member_objects, Cut, MemberObject};
use network::replica::{EntryId, Replica};
use protocol::events::NodeEvent;
use protocol::lifecycle::Event;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Precedence {
    Verified,
    Contrary,
    Absent,
    Indeterminate,
}

/// The accepted entries a pair compares against `x_O`, and whether the set is closed.
pub struct Relevant {
    pub entries: Vec<EntryId>,
    pub complete: bool,
}

/// `item`'s relevant commitments (`17` §13.3): the accepted `AssignReviewers`, its panelists'
/// commits accepted in `InReview`, and its first accepted `CloseCommits`, which closes the set.
pub fn commitments(view: &LogView, prefix: u64, item: Cid) -> Relevant {
    let (mut entries, mut panel, mut complete) = (Vec::new(), None, false);
    for e in view.prefix(prefix).filter(|e| e.applied) {
        if complete {
            break;
        }
        let Some(NodeEvent::Step { item: k, event }) = NodeEvent::decode(&e.object) else {
            continue;
        };
        if k != item {
            continue;
        }
        match event {
            Event::AssignReviewers { panel: p, .. } => {
                panel = Some(p);
                entries.push(e.id);
            }
            Event::Commit { nym, .. } if panel.as_ref().is_some_and(|p| p.contains(&nym)) => {
                entries.push(e.id)
            }
            Event::CloseCommits if panel.is_some() => {
                entries.push(e.id);
                complete = true;
            }
            _ => {}
        }
    }
    Relevant { entries, complete }
}

/// A reference binding answers (`17` §13.4, (iv)): an accepted results event; none closes no set.
pub fn answers_reference(view: &LogView, prefix: u64) -> Relevant {
    let entries: Vec<EntryId> = view
        .prefix(prefix)
        .filter(|e| {
            e.applied && matches!(NodeEvent::decode(&e.object), Some(NodeEvent::Results(_)))
        })
        .map(|e| e.id)
        .collect();
    Relevant {
        complete: !entries.is_empty(),
        entries,
    }
}

/// What a `Contrary` or an `Absent` outcome rests on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Witness {
    /// The entry `x` of `x_O` shown before the accepted relevant entry `c`.
    Before { x: EntryId, c: EntryId },
    /// The entry `x` examined (R) or held (E) while the relevant set is not closed.
    Unclosed { x: EntryId },
}

/// A pair's result at one level (`17` §13.2): its outcome, the witness it rests on, and the
/// entries of `x_O` whose evidence at that level is missing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Evidence {
    pub outcome: Precedence,
    pub witness: Option<Witness>,
    pub uncovered: Vec<EntryId>,
}

/// One entry of `x_O` at one level: not examined (R) or not held (E); held with no reference
/// either way (E); shown before relevant entry `c`; shown after every relevant entry.
enum Status {
    Lacking,
    Unordered,
    Precedes(EntryId),
    Follows,
}

/// `17` §13.2's aggregation: a contrary order or an absent precondition for one entry stands
/// whatever the others; `Verified` needs every entry shown after every relevant entry.
fn aggregate(relevant: &Relevant, status: Vec<(EntryId, Status)>) -> Evidence {
    let available = |s: &Status| !matches!(s, Status::Lacking);
    let witness = if relevant.complete {
        status.iter().find_map(|(x, s)| match s {
            Status::Precedes(c) => Some(Witness::Before { x: *x, c: *c }),
            _ => None,
        })
    } else {
        let held = status.iter().find(|(_, s)| available(s));
        held.map(|(x, _)| Witness::Unclosed { x: *x })
    };
    let missing =
        |s: &Status| !available(s) || (relevant.complete && matches!(s, Status::Unordered));
    let uncovered: Vec<EntryId> = status
        .iter()
        .filter(|(_, s)| missing(s))
        .map(|(x, _)| *x)
        .collect();
    let outcome = match witness {
        Some(Witness::Before { .. }) => Precedence::Contrary,
        Some(Witness::Unclosed { .. }) => Precedence::Absent,
        None if relevant.complete && uncovered.is_empty() => Precedence::Verified,
        None => Precedence::Indeterminate,
    };
    Evidence {
        outcome,
        witness,
        uncovered,
    }
}

/// No entry can be weighed: the prefix is incomplete, so the relevant set is unidentifiable.
fn unidentifiable(x: &[EntryId]) -> Evidence {
    Evidence {
        outcome: Precedence::Indeterminate,
        witness: None,
        uncovered: x.to_vec(),
    }
}

/// (R): the order in which the replay of a complete prefix examines the entries.
pub fn replay_level(view: &LogView, prefix: u64, relevant: &Relevant, x: &[EntryId]) -> Evidence {
    if !view.complete_through(prefix) {
        return unidentifiable(x);
    }
    let cs: Vec<(EntryId, u64)> = relevant
        .entries
        .iter()
        .map(|id| (*id, view.at(id, prefix).unwrap().pos))
        .collect();
    let status = x
        .iter()
        .map(|id| {
            let s = match view.at(id, prefix) {
                None => Status::Lacking,
                Some(e) => cs
                    .iter()
                    .find(|(_, pos)| e.pos < *pos)
                    .map_or(Status::Follows, |(c, _)| Status::Precedes(*c)),
            };
            (*id, s)
        })
        .collect();
    aggregate(relevant, status)
}

/// `17` §13.1's references: each writer's feed, and the cut signatures members carry on theirs.
pub struct References {
    feeds: BTreeMap<[u8; 32], Vec<EntryId>>,
    signatures: Vec<(EntryId, Cut)>,
    replica: Replica,
}

impl References {
    /// A cut signature counts when its object is on its member's own feed, authenticated by
    /// that entry's signature, and its cut is one the prefix applied; the member's signature
    /// inside it is not checked again.
    pub fn new(replica: &Replica, consortium: &Consortium, view: &LogView, prefix: u64) -> Self {
        let feeds = replica
            .summary()
            .0
            .iter()
            .map(|w| {
                (
                    w.writer,
                    replica.feed(&w.writer).iter().map(|e| e.id()).collect(),
                )
            })
            .collect();
        let signatures = member_objects(replica, consortium)
            .filter_map(|(id, o)| match o {
                MemberObject::CutSignature { cut, .. }
                    if cut.number <= prefix && view.cuts.get(cut.number as usize) == Some(&cut) =>
                {
                    Some((id, cut))
                }
                _ => None,
            })
            .collect();
        References {
            feeds,
            signatures,
            replica: replica.clone(),
        }
    }

    fn on_feed(&self, id: &EntryId) -> bool {
        self.feeds
            .get(&id.writer)
            .is_some_and(|f| f.get(id.seq as usize) == Some(id))
    }

    fn counts(&self, cut: &Cut, id: &EntryId) -> bool {
        cut.marks.iter().any(|m| {
            m.writer == id.writer
                && m.len > id.seq
                && self
                    .replica
                    .chain(&m.writer, m.len, m.head)
                    .is_some_and(|c| c[id.seq as usize].id() == *id)
        })
    }

    /// `a` existed before `b`: `b` follows, on one feed, `a` or a cut signature counting `a`.
    pub fn before(&self, a: &EntryId, b: &EntryId) -> bool {
        let follows = |s: &EntryId| s.writer == b.writer && s.seq < b.seq && self.on_feed(s);
        self.on_feed(b)
            && (follows(a)
                || self
                    .signatures
                    .iter()
                    .any(|(s, cut)| follows(s) && self.counts(cut, a)))
    }
}

/// (E): §13.1's chains and references between the relevant entries and each entry of `x_O`.
pub fn existence_level(
    view: &LogView,
    prefix: u64,
    refs: &References,
    relevant: &Relevant,
    x: &[EntryId],
) -> Evidence {
    if !view.complete_through(prefix) {
        return unidentifiable(x);
    }
    let c = &relevant.entries;
    let status = x
        .iter()
        .map(|x| {
            let s = if !refs.replica.contains(x) {
                Status::Lacking
            } else if let Some(c) = c.iter().find(|c| refs.before(x, c)) {
                Status::Precedes(*c)
            } else if c.iter().all(|c| refs.before(c, x)) {
                Status::Follows
            } else {
                Status::Unordered
            };
            (*x, s)
        })
        .collect();
    aggregate(relevant, status)
}
