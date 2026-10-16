//! An epoch's author records from a node's state (`docs/02` §C.1, `docs/15` C5–C6): the pilot
//! conclusions not yet recorded, then the epoch's appeals selected one by one in CID order.

use crate::appeal::{AuthorError, AuthorHistory};
use crate::lifecycle::State;
use crate::node::NodeState;
use crate::results::{ResultRecord, ResultsState};
use identity::nym::Nym;
use network::cid::Cid;
use scoring::reputation::AuthorPrior;
use std::collections::{BTreeMap, BTreeSet};

/// The records for one results event and the appeals they admit or refuse.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AuthorRecords {
    /// Outcomes and settlements, then filings, each in ascending item CID.
    pub records: Vec<ResultRecord>,
    pub admitted: Vec<Cid>,
    pub refused: Vec<Cid>,
    /// Appealed items concluded with no escrow on record: no record can count them.
    pub unescrowed: Vec<Cid>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AuthorRecordsError {
    /// An appeal on an item no accepted deposit started.
    UnknownItem(Cid),
    NotAppealable(Cid),
    /// Appeals with no `E_m` configured (`appeal::EpochScale`).
    NoEpochScale,
    Refused(Cid, AuthorError),
}

/// The author records of results epoch `epoch`; `appeals` is the epoch's closed set of
/// requests, so their arrival order cannot matter (`docs/15` C6).
pub fn author_records(
    node: &NodeState,
    epoch: u64,
    appeals: &BTreeSet<Cid>,
) -> Result<AuthorRecords, AuthorRecordsError> {
    let results = node.results();
    let prior = AuthorPrior::default();
    let mut out = AuthorRecords::default();
    let mut working: BTreeMap<Nym, AuthorHistory> = BTreeMap::new();
    for (item, conclusion) in node.conclusions() {
        let author = node.proposer(item).expect("a concluded item was deposited");
        let h = held(&mut working, results, author);
        let refused = |e| AuthorRecordsError::Refused(*item, e);
        let passed = conclusion.passed;
        if h.outcome(item).is_some() {
            continue;
        }
        if !conclusion.appealed {
            h.conclude(*item, passed, epoch).map_err(refused)?;
            let record = ResultRecord::AuthorOutcome {
                author,
                item: *item,
                passed,
            };
            out.records.push(record);
        } else if h.is_open(item) {
            h.settle(*item, passed, epoch).map_err(refused)?;
            let record = ResultRecord::AppealSettled {
                author,
                item: *item,
                passed,
            };
            out.records.push(record);
        } else {
            out.unescrowed.push(*item);
        }
    }
    if appeals.is_empty() {
        return Ok(out);
    }
    let scale = results
        .epoch_scale()
        .ok_or(AuthorRecordsError::NoEpochScale)?;
    for item in appeals {
        let author = node
            .proposer(item)
            .ok_or(AuthorRecordsError::UnknownItem(*item))?;
        if node.item(item) != Some(&State::AppealEligible) {
            return Err(AuthorRecordsError::NotAppealable(*item));
        }
        let h = held(&mut working, results, author);
        match h.file_appeal(*item, epoch, scale, &prior) {
            Ok(()) => {
                out.records.push(ResultRecord::AppealFiled {
                    author,
                    item: *item,
                });
                out.admitted.push(*item);
            }
            Err(AuthorError::Uncovered(_)) => out.refused.push(*item),
            Err(e) => return Err(AuthorRecordsError::Refused(*item, e)),
        }
    }
    Ok(out)
}

fn held<'a>(
    working: &'a mut BTreeMap<Nym, AuthorHistory>,
    results: &ResultsState,
    author: Nym,
) -> &'a mut AuthorHistory {
    working
        .entry(author)
        .or_insert_with(|| results.author(&author).cloned().unwrap_or_default())
}
