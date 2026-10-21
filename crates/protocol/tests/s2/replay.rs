//! The check's log view (`docs/20` §2): signed cuts applied in turn by the real `Ledger`,
//! every examined entry at its position, the lifecycle states after each cut.

use super::fixture::{consortium, issuer, signatures, NET};
use super::records::Record;
use network::cid::Cid;
use network::cut::{added, Cut};
use network::replica::{EntryId, Replica};
use protocol::events::NodeEvent;
use protocol::ledger::{CutReport, Ledger, LedgerError, Refusal};
use protocol::lifecycle::{Event, State};
use std::collections::BTreeMap;

/// An entry a cut counts: `pos` is its index in the examination order of every cut so far.
pub struct Examined {
    pub id: EntryId,
    pub cut: u64,
    pub pos: u64,
    pub object: Vec<u8>,
    pub applied: bool,
}

pub struct LogView {
    pub cuts: Vec<Cut>,
    pub reports: Vec<CutReport>,
    pub examined: Vec<Examined>,
    pub states: Vec<BTreeMap<Cid, State>>,
    /// The ledger's refusal of the first cut it could not apply; later cuts are not tried.
    pub error: Option<LedgerError>,
}

impl LogView {
    pub fn replay(replica: &Replica, cuts: &[Cut], items: &[Cid]) -> LogView {
        let mut ledger = Ledger::new(NET, consortium(), issuer().public());
        let mut view = LogView {
            cuts: Vec::new(),
            reports: Vec::new(),
            examined: Vec::new(),
            states: Vec::new(),
            error: None,
        };
        for cut in cuts {
            let report = match ledger.apply(replica, cut, &signatures(cut)) {
                Ok(report) => report,
                Err(e) => {
                    view.error = Some(e);
                    break;
                }
            };
            let order = added(replica, view.cuts.last(), cut).expect("the cut the ledger applied");
            for id in order {
                view.examined.push(Examined {
                    id,
                    cut: cut.number,
                    pos: view.examined.len() as u64,
                    object: replica.get(&id).unwrap().1.to_vec(),
                    applied: report.applied.contains(&id),
                });
            }
            let state = ledger.state();
            let states = items
                .iter()
                .filter_map(|c| state.item(c).map(|s| (*c, s.clone())))
                .collect();
            view.states.push(states);
            view.reports.push(report);
            view.cuts.push(cut.clone());
        }
        view
    }

    /// Every cut through `prefix` applied, so each entry they count was held.
    pub fn complete_through(&self, prefix: u64) -> bool {
        self.cuts.len() as u64 > prefix
    }

    pub fn prefix(&self, prefix: u64) -> impl Iterator<Item = &Examined> {
        self.examined.iter().filter(move |e| e.cut <= prefix)
    }

    pub fn at(&self, id: &EntryId, prefix: u64) -> Option<&Examined> {
        self.prefix(prefix).find(|e| e.id == *id)
    }

    pub fn refusal(&self, id: &EntryId) -> Option<&Refusal> {
        self.reports
            .iter()
            .flat_map(|r| &r.refused)
            .find(|(r, _)| r == id)
            .map(|(_, refusal)| refusal)
    }

    pub fn state(&self, prefix: u64, item: &Cid) -> Option<&State> {
        self.states.get(prefix as usize)?.get(item)
    }

    /// The lifecycle steps applied through `prefix`, per item, in order.
    pub fn steps(&self, prefix: u64) -> BTreeMap<Cid, Vec<(u64, Event)>> {
        let mut steps: BTreeMap<Cid, Vec<(u64, Event)>> = BTreeMap::new();
        for e in self.prefix(prefix).filter(|e| e.applied) {
            if let Some(NodeEvent::Step { item, event }) = NodeEvent::decode(&e.object) {
                steps.entry(item).or_default().push((e.pos, event));
            }
        }
        steps
    }

    /// The study records counted through `prefix`, applied or not, with their entries.
    pub fn records(&self, prefix: u64) -> Vec<(&Examined, Record)> {
        self.prefix(prefix)
            .filter_map(|e| Record::decode(&e.object).map(|r| (e, r)))
            .collect()
    }
}
