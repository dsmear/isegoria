//! The protocol state from the replicated set (`docs/04` §Cuts, `docs/08` PROTO-015, T74):
//! each signed cut's entries applied in its order, the first of conflicting events winning.

use crate::events::NodeEvent;
use crate::node::{NodeState, Rejection};
use ed25519_dalek::Signature;
use identity::credential::IssuerPublic;
use network::consortium::Consortium;
use network::cut::{added, Cut, CutError};
use network::replica::{EntryId, Replica};

/// Why an entry a cut named left the state as it was.
#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    NotAnEvent,
    /// A lifecycle step or epoch results from a writer outside the consortium.
    NotAuthorized,
    Rejected(Rejection),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LedgerError {
    Cut(CutError),
    /// Fewer than the threshold of member signatures over the cut's checkpoint.
    Unsigned,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct CutReport {
    pub applied: Vec<EntryId>,
    pub refused: Vec<(EntryId, Refusal)>,
}

pub struct Ledger {
    network_id: [u8; 32],
    consortium: Consortium,
    issuer: IssuerPublic,
    state: NodeState,
    last: Option<Cut>,
    refused: Vec<EntryId>,
}

impl Ledger {
    pub fn new(network_id: [u8; 32], consortium: Consortium, issuer: IssuerPublic) -> Self {
        Ledger {
            network_id,
            consortium,
            issuer,
            state: NodeState::default(),
            last: None,
            refused: Vec::new(),
        }
    }

    /// Applies the next cut, signed by `sigs`, once `replica` holds all it names; an error
    /// leaves the ledger as it was.
    pub fn apply(
        &mut self,
        replica: &Replica,
        cut: &Cut,
        sigs: &[(usize, Signature)],
    ) -> Result<CutReport, LedgerError> {
        let checkpoint = cut.checkpoint(self.network_id, self.consortium.member_set_hash());
        if !self.consortium.verify(&checkpoint, sigs) {
            return Err(LedgerError::Unsigned);
        }
        let order = added(replica, self.last.as_ref(), cut).map_err(LedgerError::Cut)?;
        let mut report = CutReport::default();
        for id in order {
            let (_, object) = replica.get(&id).expect("`added` names only held entries");
            match self.apply_one(id, object) {
                Ok(()) => report.applied.push(id),
                Err(refusal) => report.refused.push((id, refusal)),
            }
        }
        self.refused
            .extend(report.refused.iter().map(|(id, _)| *id));
        self.last = Some(cut.clone());
        Ok(report)
    }

    fn apply_one(&mut self, id: EntryId, object: &[u8]) -> Result<(), Refusal> {
        let event = NodeEvent::decode(object).ok_or(Refusal::NotAnEvent)?;
        let orchestration = matches!(event, NodeEvent::Step { .. } | NodeEvent::Results(_));
        if orchestration && !self.consortium.is_member(&id.writer) {
            return Err(Refusal::NotAuthorized);
        }
        self.state
            .apply(&event, &self.issuer)
            .map(|_| ())
            .map_err(Refusal::Rejected)
    }

    pub fn state(&self) -> &NodeState {
        &self.state
    }

    /// The last cut applied.
    pub fn last(&self) -> Option<&Cut> {
        self.last.as_ref()
    }

    /// Every refused entry so far, in the order the cuts applied them.
    pub fn refused(&self) -> &[EntryId] {
        &self.refused
    }
}
