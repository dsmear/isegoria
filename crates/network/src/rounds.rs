//! The epoch's beacon rounds run on cuts (`docs/04` §The beacon on cuts, §The withholders on
//! cuts, T74, T76): the commits and reveals each cut counts, the outcomes, and rule 6.

use crate::beacon::{BeaconCommit, BeaconOutcome, BeaconReveal, BeaconRound, RoundError};
use crate::codec::Writer;
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

    /// The open rounds' counted commits and reveals, then each finished round's cut, epoch and
    /// outcome record, by epoch: one canonical encoding.
    pub fn write(&self, w: &mut Writer) {
        w.u64(self.open.len() as u64);
        for (epoch, round) in &self.open {
            w.u64(*epoch).u64(round.commits.len() as u64);
            for c in &round.commits {
                w.u64(c.member as u64).fixed(&c.commitment);
                w.fixed(&c.signature.to_bytes());
            }
            w.u64(round.reveals.len() as u64);
            for r in &round.reveals {
                w.u64(r.member as u64).fixed(&r.secret);
            }
        }
        w.u64(self.finished.len() as u64);
        for (epoch, f) in &self.finished {
            w.u64(*epoch).u64(f.at).u64(f.epoch);
            w.fixed(&f.outcome.record().0);
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beacon::RoundId;
    use crate::consortium::Member;

    const NET: [u8; 32] = [3; 32];

    fn members() -> Vec<Member> {
        (0..3).map(|i| Member::from_seed([40 + i; 32])).collect()
    }

    fn consortium() -> Consortium {
        Consortium::new(members().iter().map(Member::public).collect(), 2)
    }

    fn cut(number: u64, epoch: u64, closes: bool) -> Cut {
        Cut {
            number,
            epoch,
            closes,
            marks: Vec::new(),
        }
    }

    fn pair(m: usize, epoch: u64) -> (BeaconCommit, BeaconReveal) {
        let c = consortium();
        let round = RoundId {
            network_id: NET,
            member_set_hash: c.member_set_hash(),
            epoch,
        };
        members()[m].beacon_commit(round, m)
    }

    /// Epoch 0's round finished with members 0 and 1 revealing; epoch 1's open with two commits.
    fn base(revealing: &[usize]) -> Rounds {
        let (c, mut rounds) = (consortium(), Rounds::new());
        let (closing, next) = (cut(0, 0, true), cut(1, 1, false));
        for m in 0..3 {
            let commit = MemberObject::Commit(pair(m, 0).0);
            rounds.count(&c, NET, &commit, &closing, None).unwrap();
        }
        for &m in revealing {
            let reveal = MemberObject::Reveal {
                epoch: 0,
                reveal: pair(m, 0).1,
            };
            rounds
                .count(&c, NET, &reveal, &next, Some(&closing))
                .unwrap();
        }
        for m in 0..2 {
            let commit = MemberObject::Commit(pair(m, 1).0);
            rounds
                .count(&c, NET, &commit, &next, Some(&closing))
                .unwrap();
        }
        rounds.finish(&c, NET, &next, Some(&closing));
        rounds
    }

    fn encoded(rounds: &Rounds) -> Vec<u8> {
        let mut w = Writer::new();
        rounds.write(&mut w);
        w.finish()
    }

    fn open(r: &mut Rounds) -> &mut Round {
        r.open.get_mut(&1).unwrap()
    }

    // AT-T79-15: rounds differing in one counted commit, reveal or finished outcome encode apart.
    #[test]
    fn every_counted_message_and_outcome_is_encoded() {
        type Change = fn(&mut Rounds);
        let changes: Vec<(&str, Change)> = vec![
            ("a committer", |r| open(r).commits[1].member = 2),
            ("a commitment", |r| open(r).commits[1].commitment[0] ^= 1),
            ("a signature", |r| {
                open(r).commits[1].signature = pair(2, 1).0.signature
            }),
            ("a commit more", |r| open(r).commits.push(pair(2, 1).0)),
            ("a reveal", |r| open(r).reveals.push(pair(0, 1).1)),
            ("an open epoch", |r| {
                let round = r.open.remove(&1).unwrap();
                r.open.insert(2, round);
            }),
            ("the finishing cut", |r| {
                r.finished.get_mut(&0).unwrap().at += 1
            }),
            ("its epoch", |r| r.finished.get_mut(&0).unwrap().epoch += 1),
            ("an outcome", |r| {
                let other = base(&[0]).finished.remove(&0).unwrap();
                r.finished.insert(0, other);
            }),
            ("a finished epoch", |r| {
                let finished = r.finished.remove(&0).unwrap();
                r.finished.insert(5, finished);
            }),
        ];
        let reference = encoded(&base(&[0, 1]));
        for (field, change) in changes {
            let mut rounds = base(&[0, 1]);
            change(&mut rounds);
            assert_ne!(encoded(&rounds), reference, "{field}");
        }
        let mut with_reveal = base(&[0, 1]);
        open(&mut with_reveal).reveals.push(pair(0, 1).1);
        let mut other_secret = with_reveal.clone();
        open(&mut other_secret).reveals[0].secret[0] ^= 1;
        assert_ne!(encoded(&with_reveal), encoded(&other_secret));
        let mut other_revealer = with_reveal.clone();
        open(&mut other_revealer).reveals[0].member = 1;
        assert_ne!(encoded(&with_reveal), encoded(&other_revealer));
    }
}
