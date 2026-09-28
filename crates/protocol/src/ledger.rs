//! The protocol state from the replicated set (`docs/04` §Cuts, `docs/08` PROTO-015, T74):
//! each signed cut's entries applied in its order, the first of conflicting events winning.

use crate::events::NodeEvent;
use crate::node::{NodeState, Rejection};
use ed25519_dalek::Signature;
use identity::credential::IssuerPublic;
use network::beacon::{BeaconCommit, BeaconOutcome, BeaconReveal, BeaconRound, RoundError};
use network::consortium::Consortium;
use network::cut::{added, Cut, CutError, MemberObject};
use network::replica::{EntryId, Replica};
use std::collections::BTreeMap;

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
    /// Fewer than the threshold of member signatures over the cut's checkpoint.
    Unsigned,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct CutReport {
    pub applied: Vec<EntryId>,
    pub refused: Vec<(EntryId, Refusal)>,
}

#[derive(Default)]
struct Round {
    commits: Vec<BeaconCommit>,
    reveals: Vec<BeaconReveal>,
}

pub struct Ledger {
    network_id: [u8; 32],
    consortium: Consortium,
    issuer: IssuerPublic,
    state: NodeState,
    last: Option<Cut>,
    refused: Vec<EntryId>,
    rounds: BTreeMap<u64, Round>,
    beacons: BTreeMap<u64, BeaconOutcome>,
}

/// The round of `epoch` rebuilt from what was counted, then `extra` tried on it.
fn replay<'a>(
    consortium: &'a Consortium,
    network_id: [u8; 32],
    epoch: u64,
    round: &Round,
    reveal: Option<&BeaconReveal>,
) -> Result<BeaconRound<'a>, RoundError> {
    let mut r = BeaconRound::open(consortium, network_id, epoch);
    for c in &round.commits {
        r.commit(c)?;
    }
    if round.reveals.is_empty() && reveal.is_none() {
        return Ok(r);
    }
    r.close_commits()?;
    r.close_deposits()?;
    for s in round.reveals.iter().chain(reveal) {
        r.reveal(s)?;
    }
    Ok(r)
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
            rounds: BTreeMap::new(),
            beacons: BTreeMap::new(),
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
        let revealing = self.last.as_ref().filter(|l| l.closes).map(|l| l.epoch);
        let mut report = CutReport::default();
        for id in order {
            let (_, object) = replica.get(&id).expect("`added` names only held entries");
            match self.apply_one(id, object, cut, revealing) {
                Ok(()) => report.applied.push(id),
                Err(refusal) => report.refused.push((id, refusal)),
            }
        }
        if let Some(epoch) = revealing {
            let round = self.rounds.remove(&epoch).unwrap_or_default();
            let outcome = replay(&self.consortium, self.network_id, epoch, &round, None)
                .and_then(|mut r| {
                    if round.reveals.is_empty() {
                        r.close_commits()?;
                        r.close_deposits()?;
                    }
                    r.finish()
                })
                .expect("a round of counted commits and reveals finishes");
            self.beacons.insert(epoch, outcome);
        }
        self.refused
            .extend(report.refused.iter().map(|(id, _)| *id));
        self.last = Some(cut.clone());
        Ok(report)
    }

    fn apply_one(
        &mut self,
        id: EntryId,
        object: &[u8],
        cut: &Cut,
        revealing: Option<u64>,
    ) -> Result<(), Refusal> {
        if let Some(member) = MemberObject::decode(object) {
            return self.member_object(id, member, cut, revealing);
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
        revealing: Option<u64>,
    ) -> Result<(), Refusal> {
        let own = self.consortium.keys().get(object.member());
        if own.is_none_or(|k| k.as_bytes() != &id.writer) {
            return Err(Refusal::NotAuthorized);
        }
        let network_id = self.network_id;
        match object {
            MemberObject::CutSignature { .. } => Ok(()),
            MemberObject::Commit(commit) => {
                let epoch = commit.round.epoch;
                if epoch != cut.epoch {
                    return Err(Refusal::OutOfWindow);
                }
                let round = self.rounds.entry(epoch).or_default();
                let mut r = replay(&self.consortium, network_id, epoch, round, None)
                    .expect("counted commits replay");
                r.commit(&commit).map_err(Refusal::Beacon)?;
                round.commits.push(commit);
                Ok(())
            }
            MemberObject::Reveal { epoch, reveal } => {
                if revealing != Some(epoch) {
                    return Err(Refusal::OutOfWindow);
                }
                let round = self.rounds.entry(epoch).or_default();
                replay(&self.consortium, network_id, epoch, round, Some(&reveal))
                    .map_err(Refusal::Beacon)?;
                round.reveals.push(reveal);
                Ok(())
            }
        }
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
        self.beacons.get(&epoch)
    }
}
