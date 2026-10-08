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

/// (R): the order in which the replay of a complete prefix examines the entries.
pub fn replay_level(view: &LogView, prefix: u64, relevant: &Relevant, x: &[EntryId]) -> Precedence {
    if !view.complete_through(prefix) {
        return Precedence::Indeterminate;
    }
    let Some(xs) = x
        .iter()
        .map(|id| view.at(id, prefix).map(|e| e.pos))
        .collect::<Option<Vec<u64>>>()
    else {
        return Precedence::Indeterminate;
    };
    if !relevant.complete {
        return Precedence::Absent;
    }
    let cs: Vec<u64> = relevant
        .entries
        .iter()
        .map(|id| view.at(id, prefix).unwrap().pos)
        .collect();
    if xs.iter().any(|x| cs.iter().any(|c| x < c)) {
        Precedence::Contrary
    } else {
        Precedence::Verified
    }
}

/// `17` §13.1's references: each writer's feed, and the cut signatures members carry on theirs.
pub struct References {
    feeds: BTreeMap<[u8; 32], Vec<EntryId>>,
    signatures: Vec<(EntryId, Cut)>,
    replica: Replica,
}

impl References {
    /// Only signatures of a cut the prefix applied count.
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
) -> Precedence {
    if !view.complete_through(prefix) || x.iter().any(|id| !refs.replica.contains(id)) {
        return Precedence::Indeterminate;
    }
    if !relevant.complete {
        return Precedence::Absent;
    }
    let c = &relevant.entries;
    if c.iter().any(|c| x.iter().any(|x| refs.before(x, c))) {
        Precedence::Contrary
    } else if c.iter().all(|c| x.iter().all(|x| refs.before(c, x))) {
        Precedence::Verified
    } else {
        Precedence::Indeterminate
    }
}
