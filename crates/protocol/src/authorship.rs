//! An epoch's author records from a node's state (`docs/02` §C.1, `docs/15` C5–C6): the pilot
//! conclusions not yet recorded, then the epoch's appeals selected one by one in CID order.

use crate::appeal::{AuthorError, AuthorHistory, InsufficientReputation};
use crate::author_time::AuthorPoint;
use crate::lifecycle::State;
use crate::node::NodeState;
use crate::results::{ResultRecord, ResultsState};
use identity::nym::Nym;
use network::cid::Cid;
use scoring::reputation::AuthorPrior;
use std::collections::BTreeMap;

/// An appeal request: whether it came within its item's appeal window, an explicit input
/// the producer reads before the request may consume coverage (`docs/15` C5–C6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppealRequest {
    pub within_window: bool,
}

/// Why a request was refused without a record.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AppealRefusal {
    WindowClosed,
    Uncovered(InsufficientReputation),
}

/// The records for one results event and the appeals they start or refuse.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AuthorRecords {
    /// Outcomes and settlements, then filings, each in ascending item CID.
    pub records: Vec<ResultRecord>,
    pub admitted: Vec<Cid>,
    pub refused: Vec<(Cid, AppealRefusal)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AuthorRecordsError {
    /// An appeal on an item no accepted deposit started.
    UnknownItem(Cid),
    NotAppealable(Cid),
    FilingBatchRecorded,
    Refused(Cid, AuthorError),
    /// An appealed conclusion with no open escrow: a defense, since only a filing record starts
    /// an appeal, escrowing it in the same event, and only a settlement closes it.
    Unescrowed(Cid),
}

/// Plans records at the caller's applying point; the consumer rechecks it (`docs/15` C6).
/// `appeals` is the supplied closed request set, selected in CID order.
pub fn author_records(
    node: &NodeState,
    at: AuthorPoint,
    appeals: &BTreeMap<Cid, AppealRequest>,
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
            h.conclude(*item, passed, at).map_err(refused)?;
            let record = ResultRecord::AuthorOutcome {
                author,
                item: *item,
                passed,
            };
            out.records.push(record);
        } else if h.is_open(item) {
            h.settle(*item, passed, at).map_err(refused)?;
            let record = ResultRecord::AppealSettled {
                author,
                item: *item,
                passed,
            };
            out.records.push(record);
        } else {
            return Err(AuthorRecordsError::Unescrowed(*item));
        }
    }
    if appeals.is_empty() {
        return Ok(out);
    }
    if results.filing_batch_at(at.epoch) {
        return Err(AuthorRecordsError::FilingBatchRecorded);
    }
    let config = results.author_config();
    for (item, request) in appeals {
        let author = node
            .proposer(item)
            .ok_or(AuthorRecordsError::UnknownItem(*item))?;
        if node.item(item) != Some(&State::AppealEligible) {
            return Err(AuthorRecordsError::NotAppealable(*item));
        }
        if !request.within_window {
            out.refused.push((*item, AppealRefusal::WindowClosed));
            continue;
        }
        let h = held(&mut working, results, author);
        match h.file_appeal(*item, at, config, &prior) {
            Ok(()) => {
                out.records.push(ResultRecord::AppealStarted {
                    author,
                    item: *item,
                    within_window: true,
                });
                out.admitted.push(*item);
            }
            Err(AuthorError::Uncovered(short)) => {
                out.refused.push((*item, AppealRefusal::Uncovered(short)))
            }
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
