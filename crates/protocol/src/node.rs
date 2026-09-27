//! A node's state rebuilt from its durable log (`docs/04` §Events and replay, `docs/10`
//! T73): events are checked, applied, then written; opening replays them all.

use crate::admission::{NullifierSet, QuotaLedger};
use crate::deposit::{deposit_with_identity, DepositRejected};
use crate::events::NodeEvent;
use crate::pilot::{submit_response, ResponseRejected};
use crate::review::{submit_review, ReviewRejected};
use identity::credential::IssuerPublic;
use identity::nym::Nym;
use network::cid::Cid;
use network::log::TransparencyLog;
use network::store::{DurableLog, ObjectStore, StoreError};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub enum Rejection {
    Deposit(DepositRejected),
    Review(ReviewRejected),
    Response(ResponseRejected),
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeError {
    Store(StoreError),
    Rejected(Rejection),
    /// Log entry `entry` names no stored object.
    MissingObject {
        entry: u64,
    },
    /// Log entry `entry` names an object that is not an event.
    Undecodable {
        entry: u64,
    },
    /// Log entry `entry` names an event this node rejects on replay.
    ReplayRejected {
        entry: u64,
        rejection: Rejection,
    },
    /// A write failed after the state changed: reopen the node.
    Poisoned,
}

/// The admission state: drafts on record, quotas per epoch, panels and respondents per
/// `(item or batch, epoch)`.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct AdmissionState {
    drafts: TransparencyLog,
    quotas: BTreeMap<u64, QuotaLedger>,
    panels: BTreeMap<(Cid, u64), NullifierSet>,
    respondents: BTreeMap<(Cid, u64), NullifierSet>,
}

impl AdmissionState {
    /// Checks and applies one event, as the entry points do (`deposit_with_identity`,
    /// `submit_review`, `submit_response`); the proven id on success.
    pub fn apply(&mut self, event: &NodeEvent, issuer: &IssuerPublic) -> Result<Nym, Rejection> {
        match event {
            NodeEvent::Deposit {
                epoch,
                quota,
                draft,
                proof,
            } => {
                let drafts = &mut self.drafts;
                with_entry(&mut self.quotas, *epoch, |ledger| {
                    deposit_with_identity(drafts, draft, proof, issuer, *epoch, ledger, *quota)
                })
                .map(|(_, proposer)| proposer)
                .map_err(Rejection::Deposit)
            }
            NodeEvent::AdmitReviewer { item, epoch, proof } => {
                with_entry(&mut self.panels, (*item, *epoch), |panel| {
                    submit_review(proof, issuer, *item, *epoch, panel)
                })
                .map_err(Rejection::Review)
            }
            NodeEvent::AdmitRespondent {
                batch,
                epoch,
                proof,
            } => with_entry(&mut self.respondents, (*batch, *epoch), |set| {
                submit_response(proof, issuer, *batch, *epoch, set)
            })
            .map_err(Rejection::Response),
        }
    }

    pub fn drafts(&self) -> &TransparencyLog {
        &self.drafts
    }

    pub fn quota_used(&self, epoch: u64, proposer: &Nym) -> u32 {
        self.quotas.get(&epoch).map_or(0, |l| l.used(proposer))
    }

    pub fn panel(&self, item: Cid, epoch: u64) -> Option<&NullifierSet> {
        self.panels.get(&(item, epoch))
    }

    pub fn respondents(&self, batch: Cid, epoch: u64) -> Option<&NullifierSet> {
        self.respondents.get(&(batch, epoch))
    }
}

/// Runs `f` on the entry for `key`, keeping it only if it is not empty afterwards: a
/// rejected event leaves the map as it found it.
fn with_entry<K: Ord, V: Default + PartialEq, T>(
    map: &mut BTreeMap<K, V>,
    key: K,
    f: impl FnOnce(&mut V) -> T,
) -> T {
    let mut value = map.remove(&key).unwrap_or_default();
    let out = f(&mut value);
    if value != V::default() {
        map.insert(key, value);
    }
    out
}

impl From<StoreError> for NodeError {
    fn from(e: StoreError) -> Self {
        NodeError::Store(e)
    }
}

pub struct Node {
    log: DurableLog,
    objects: ObjectStore,
    issuer: IssuerPublic,
    state: AdmissionState,
    poisoned: bool,
}

impl Node {
    /// Opens or creates the node in `dir` (files `log` and `objects`) and replays its log.
    pub fn open(dir: &Path, issuer: IssuerPublic) -> Result<Self, NodeError> {
        fs::create_dir_all(dir).map_err(|e| StoreError::Io(e.kind()))?;
        let (objects, _) = ObjectStore::open(&dir.join("objects"))?;
        let (log, _) = DurableLog::open(&dir.join("log"))?;
        let mut state = AdmissionState::default();
        for (i, entry) in log.log().entries().iter().enumerate() {
            let entry_no = i as u64;
            let bytes = objects
                .get(&entry.payload)?
                .ok_or(NodeError::MissingObject { entry: entry_no })?;
            let event =
                NodeEvent::decode(&bytes).ok_or(NodeError::Undecodable { entry: entry_no })?;
            state
                .apply(&event, &issuer)
                .map_err(|rejection| NodeError::ReplayRejected {
                    entry: entry_no,
                    rejection,
                })?;
        }
        Ok(Node {
            log,
            objects,
            issuer,
            state,
            poisoned: false,
        })
    }

    /// Checks and applies `event`, then writes it; the proven id on success.
    pub fn submit(&mut self, event: NodeEvent) -> Result<Nym, NodeError> {
        if self.poisoned {
            return Err(NodeError::Poisoned);
        }
        let id = self
            .state
            .apply(&event, &self.issuer)
            .map_err(NodeError::Rejected)?;
        let written = self
            .objects
            .put(&event.encode())
            .and_then(|object| self.log.append(object));
        if let Err(e) = written {
            self.poisoned = true;
            return Err(NodeError::Store(e));
        }
        Ok(id)
    }

    pub fn state(&self) -> &AdmissionState {
        &self.state
    }

    pub fn log_len(&self) -> usize {
        self.log.log().len()
    }
}
