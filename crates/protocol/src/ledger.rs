//! The protocol state from the replicated set (`docs/04` §Cuts, `docs/08` PROTO-015, T74):
//! each signed cut's entries applied in its order, the first of conflicting events winning.

use crate::author_time::{ContextError, CutContext};
use crate::events::NodeEvent;
use crate::node::{NodeState, Rejection};
use ed25519_dalek::Signature;
use identity::credential::IssuerPublic;
use network::beacon::{BeaconOutcome, RoundError};
use network::consortium::Consortium;
use network::cut::{added, Cut, CutError, MemberObject};
use network::replica::{EntryId, Replica};
use network::rounds::{Rounds, Uncounted};

/// Why an entry a cut named left the state as it was.
#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    NotAnEvent,
    /// Orchestration from outside the consortium, or a member object on another's feed.
    NotAuthorized,
    Rejected(Rejection),
    /// A deposit naming another epoch than its cut's.
    WrongEpoch,
    /// A beacon commit or reveal outside its round's cuts.
    OutOfWindow,
    Beacon(RoundError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LedgerError {
    Cut(CutError),
    /// Fewer than the threshold of member signatures over the cut's checkpoint, a withholder's
    /// not counted where rule 6 excludes it.
    Unsigned,
    Context(ContextError),
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
    rounds: Rounds,
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
            rounds: Rounds::new(),
        }
    }

    /// Applies the next cut, signed by `sigs` in view 0, once `replica` holds all it names; an
    /// error leaves the ledger as it was.
    pub fn apply(
        &mut self,
        replica: &Replica,
        cut: &Cut,
        sigs: &[(usize, Signature)],
    ) -> Result<CutReport, LedgerError> {
        self.apply_in_view(replica, cut, 0, sigs)
    }

    /// As [`Self::apply`], for signatures given in `view` (`docs/04` §Changing view).
    pub fn apply_in_view(
        &mut self,
        replica: &Replica,
        cut: &Cut,
        view: u64,
        sigs: &[(usize, Signature)],
    ) -> Result<CutReport, LedgerError> {
        let msh = self.consortium.member_set_hash();
        let checkpoint = cut.checkpoint_in(view, self.network_id, msh);
        let excluded = self.rounds.excluded(cut);
        if !self
            .consortium
            .verify_excluding(&checkpoint, sigs, &excluded)
        {
            return Err(LedgerError::Unsigned);
        }
        let order = added(replica, self.last.as_ref(), cut).map_err(LedgerError::Cut)?;
        self.state
            .begin_cut(CutContext {
                number: cut.number,
                epoch: cut.epoch,
                closes: cut.closes,
            })
            .map_err(LedgerError::Context)?;
        let mut report = CutReport::default();
        for id in order {
            let (_, object) = replica.get(&id).expect("`added` names only held entries");
            match self.apply_one(id, object, cut) {
                Ok(()) => report.applied.push(id),
                Err(refusal) => report.refused.push((id, refusal)),
            }
        }
        let last = self.last.take();
        self.rounds
            .finish(&self.consortium, self.network_id, cut, last.as_ref());
        self.state
            .end_cut()
            .expect("begin_cut prevalidates closing overflow");
        self.refused
            .extend(report.refused.iter().map(|(id, _)| *id));
        self.last = Some(cut.clone());
        Ok(report)
    }

    fn apply_one(&mut self, id: EntryId, object: &[u8], cut: &Cut) -> Result<(), Refusal> {
        if let Some(member) = MemberObject::decode(object) {
            return self.member_object(id, member, cut);
        }
        let event = NodeEvent::decode(object).ok_or(Refusal::NotAnEvent)?;
        let orchestration = matches!(event, NodeEvent::Step { .. } | NodeEvent::Results(_));
        if orchestration && !self.consortium.is_member(&id.writer) {
            return Err(Refusal::NotAuthorized);
        }
        if matches!(event, NodeEvent::Deposit { epoch, .. } if epoch != cut.epoch) {
            return Err(Refusal::WrongEpoch);
        }
        self.state
            .apply(&event, &self.issuer)
            .map(|_| ())
            .map_err(Refusal::Rejected)
    }

    fn member_object(
        &mut self,
        id: EntryId,
        object: MemberObject,
        cut: &Cut,
    ) -> Result<(), Refusal> {
        let own = self.consortium.keys().get(object.member());
        if own.is_none_or(|k| k.as_bytes() != &id.writer) {
            return Err(Refusal::NotAuthorized);
        }
        let (c, net, last) = (&self.consortium, self.network_id, self.last.as_ref());
        self.rounds
            .count(c, net, &object, cut, last)
            .map_err(|e| match e {
                Uncounted::OutOfWindow => Refusal::OutOfWindow,
                Uncounted::Beacon(e) => Refusal::Beacon(e),
            })
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

    /// The outcome of epoch `epoch`'s beacon round, once the cut after its closing one is
    /// applied.
    pub fn beacon(&self, epoch: u64) -> Option<&BeaconOutcome> {
        self.rounds.outcome(epoch)
    }

    /// The members whose signatures `cut` would not count (rule 6, `docs/04` §The withholders
    /// on cuts).
    pub fn excluded(&self, cut: &Cut) -> Vec<usize> {
        self.rounds.excluded(cut)
    }
}
