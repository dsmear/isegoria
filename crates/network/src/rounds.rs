//! The epoch's beacon rounds run on cuts (`docs/04` §The beacon on cuts, §The withholders on
//! cuts, T74, T76): the commits and reveals each cut counts, the outcomes, and rule 6.

use crate::beacon::{BeaconCommit, BeaconOutcome, BeaconReveal, BeaconRound, RoundError};
use crate::consortium::Consortium;
use crate::cut::{Cut, MemberObject};
use std::collections::BTreeMap;

/// Why a cut does not count a beacon message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Uncounted {
    /// A commit outside its epoch's cuts, or a reveal outside the cut after the closing one.
    OutOfWindow,
    Beacon(RoundError),
}

#[derive(Clone, Default)]
struct Round {
    commits: Vec<BeaconCommit>,
    reveals: Vec<BeaconReveal>,
}

/// A round's end: the cut it finished at, that cut's epoch, and the outcome.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finished {
    pub at: u64,
    pub epoch: u64,
    pub outcome: BeaconOutcome,
}

#[derive(Clone, Default)]
pub struct Rounds {
    open: BTreeMap<u64, Round>,
    finished: BTreeMap<u64, Finished>,
}

/// The round of `epoch` rebuilt from what was counted, then `reveal` tried on it.
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

/// The epoch whose reveals `cut` counts: the one `last`, the cut before it, closed.
fn revealing(last: Option<&Cut>) -> Option<u64> {
    last.filter(|l| l.closes).map(|l| l.epoch)
}

impl Rounds {
    pub fn new() -> Self {
        Self::default()
    }

    /// Counts a beacon commit or reveal that `cut`, after `last`, applies; any other object
    /// is not a beacon message and counts as nothing.
    pub fn count(
        &mut self,
        consortium: &Consortium,
        network_id: [u8; 32],
        object: &MemberObject,
        cut: &Cut,
        last: Option<&Cut>,
    ) -> Result<(), Uncounted> {
        match object {
            MemberObject::Commit(commit) => {
                let epoch = commit.round.epoch;
                if epoch != cut.epoch {
                    return Err(Uncounted::OutOfWindow);
                }
                let round = self.open.entry(epoch).or_default();
                let mut r = replay(consortium, network_id, epoch, round, None)
                    .expect("counted commits replay");
                r.commit(commit).map_err(Uncounted::Beacon)?;
                round.commits.push(*commit);
                Ok(())
            }
            MemberObject::Reveal { epoch, reveal } => {
                if revealing(last) != Some(*epoch) {
                    return Err(Uncounted::OutOfWindow);
                }
                let round = self.open.entry(*epoch).or_default();
                replay(consortium, network_id, *epoch, round, Some(reveal))
                    .map_err(Uncounted::Beacon)?;
                round.reveals.push(*reveal);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Finishes the round whose reveals `cut` counted, once all of them are counted.
    pub fn finish(
        &mut self,
        consortium: &Consortium,
        network_id: [u8; 32],
        cut: &Cut,
        last: Option<&Cut>,
    ) {
        let Some(epoch) = revealing(last) else {
            return;
        };
        let round = self.open.remove(&epoch).unwrap_or_default();
        let outcome = replay(consortium, network_id, epoch, &round, None)
            .and_then(|mut r| {
                if round.reveals.is_empty() {
                    r.close_commits()?;
                    r.close_deposits()?;
                }
                r.finish()
            })
            .expect("a round of counted commits and reveals finishes");
        let finished = Finished {
            at: cut.number,
            epoch: cut.epoch,
            outcome,
        };
        self.finished.insert(epoch, finished);
    }

    pub fn outcome(&self, epoch: u64) -> Option<&BeaconOutcome> {
        self.finished.get(&epoch).map(|f| &f.outcome)
    }

    /// The members whose signatures `cut` does not count: the withholders of each round that
    /// finished with a value before it, in its epoch.
    pub fn excluded(&self, cut: &Cut) -> Vec<usize> {
        let mut out: Vec<usize> = self
            .finished
            .values()
            .filter(|f| f.outcome.value().is_some() && cut.number > f.at && cut.epoch == f.epoch)
            .flat_map(|f| f.outcome.withheld().iter().copied())
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }
}
