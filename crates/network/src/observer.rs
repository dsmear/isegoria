//! A reader without circle access following the cuts (`docs/04` §Who reads, `docs/10` T77):
//! signatures, succession and public chains checked; circle chains attested by the signers.

use crate::beacon::BeaconOutcome;
use crate::cut::{observed, Cut, CutError, MemberObject, Observed};
use crate::replica::Replica;
use crate::rounds::Rounds;
use crate::tier::Tiers;
use ed25519_dalek::Signature;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserveError {
    Cut(CutError),
    /// Fewer than the threshold of member signatures, a withholder's not counted (rule 6).
    Unsigned,
    /// Tiers other than the configuration the observer follows.
    Context,
    /// A replica whose writer set is not the tiers'.
    Replica,
}

#[derive(Clone)]
pub struct Observer {
    context: [u8; 32],
    last: Option<Cut>,
    rounds: Rounds,
}

impl Observer {
    /// An observer of the network and configuration `tiers` describe, and of no other.
    pub fn new(tiers: &Tiers) -> Self {
        Observer {
            context: tiers.context(),
            last: None,
            rounds: Rounds::new(),
        }
    }

    /// Applies the next cut, signed by `sigs` in `view`, once `replica` holds every public
    /// entry it names; an error leaves the observer as it was.
    pub fn apply_in_view(
        &mut self,
        replica: &Replica,
        tiers: &Tiers,
        cut: &Cut,
        view: u64,
        sigs: &[(usize, Signature)],
    ) -> Result<Observed, ObserveError> {
        if !self.follows(tiers) {
            return Err(ObserveError::Context);
        }
        if replica.writers() != tiers.writers() {
            return Err(ObserveError::Replica);
        }
        let (consortium, net) = (tiers.consortium(), tiers.network_id());
        let checkpoint = cut.checkpoint_in(view, net, consortium.member_set_hash());
        let excluded = self.rounds.excluded(cut);
        if !consortium.verify_excluding(&checkpoint, sigs, &excluded) {
            return Err(ObserveError::Unsigned);
        }
        let seen = observed(replica, tiers, self.last.as_ref(), cut).map_err(ObserveError::Cut)?;
        let last = self.last.as_ref();
        for id in &seen.public {
            let Some(object) = replica.get(id).and_then(|(_, b)| MemberObject::decode(b)) else {
                continue;
            };
            let key = consortium.keys().get(object.member());
            if key.is_some_and(|k| k.as_bytes() == &id.writer) {
                let _ = self.rounds.count(consortium, net, &object, cut, last);
            }
        }
        self.rounds.finish(consortium, net, cut, last);
        self.last = Some(cut.clone());
        Ok(seen)
    }

    /// Whether `tiers` is the configuration this observer follows.
    pub fn follows(&self, tiers: &Tiers) -> bool {
        tiers.context() == self.context
    }

    /// The last cut applied.
    pub fn last(&self) -> Option<&Cut> {
        self.last.as_ref()
    }

    /// The outcome of epoch `epoch`'s beacon round, once the cut after its closing one applied.
    pub fn beacon(&self, epoch: u64) -> Option<&BeaconOutcome> {
        self.rounds.outcome(epoch)
    }

    /// The members whose signatures `cut` would not count (rule 6).
    pub fn excluded(&self, cut: &Cut) -> Vec<usize> {
        self.rounds.excluded(cut)
    }
}
