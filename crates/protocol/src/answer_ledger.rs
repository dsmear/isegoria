//! The answer review on signed cuts (`docs/05` §Distributed integration, `docs/01` D60, T79):
//! one event family per network, its derived steps at each cut's end, the members' checks.

use crate::answer::Question;
use crate::answer_events::AnswerEvent;
use crate::answer_path::{AnswerConfig, AnswerPath, Applied, Refused, Round};
use ed25519_dalek::Signature;
use identity::credential::IssuerPublic;
use network::beacon::{BeaconOutcome, RoundError};
use network::codec::Writer;
use network::cut::{added, Cut, CutError, MemberObject};
use network::hash::tagged;
use network::replica::{EntryId, Replica};
use network::rounds::{Rounds, Uncounted};
use network::tier::{Tier, Tiers};
use std::collections::{BTreeMap, BTreeSet};

/// A question's most encoded bytes on a network, so its view fits one reply (`docs/14`).
pub const MAX_QUESTION: usize = 48 << 10;

const DIGEST: &str = "isegoria/answer-review/state/v1";

/// Whether `question`'s encoding is within [`MAX_QUESTION`]; the gateway's check too.
pub fn fits(question: &Question) -> bool {
    let mut w = Writer::new();
    question.write(&mut w);
    w.finish().len() <= MAX_QUESTION
}

/// What an entry a cut named did.
#[derive(Clone, Debug, PartialEq)]
pub enum Accepted {
    /// A member's own object, counted by the beacon round or by the cuts.
    Member,
    Applied(Applied),
    /// A member's closure request, the `count`-th; the closure not taken by it.
    Requested {
        count: usize,
    },
}

/// Why an entry a cut named left the state as it was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    NotAnEvent,
    /// A member object on another feed than its member's public one.
    NotAuthorized,
    /// An answer event on a public feed.
    PublicFeed,
    /// A `Draw` or `CloseEpoch`: derived at each cut's end, never logged.
    Derived,
    /// A request to close another epoch than its cut's.
    WrongEpoch,
    Repeated,
    /// A deposit whose question encodes past [`MAX_QUESTION`].
    TooLarge,
    /// A beacon commit or reveal outside its round's cuts.
    OutOfWindow,
    Beacon(RoundError),
    Path(Refused),
}

/// Why the members' checks refuse a cut (`docs/05` T79-P3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckFailure {
    Epoch {
        expected: u64,
    },
    Closes {
        expected: bool,
    },
    /// No epoch follows this one.
    Overflow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnswerLedgerError {
    /// A replica of another writer set than the ledger's tiers.
    Replica,
    /// Not the cut after the last one applied: an earlier, a repeated or a later number.
    OutOfSequence {
        expected: u64,
    },
    /// Fewer than the threshold of member signatures, a withholder's not counted (rule 6).
    Unsigned,
    /// A cut `t` members signed that the members' checks refuse: not applied.
    Checks(CheckFailure),
    Cut(CutError),
}

/// What the next cut must be: its number, its epoch, whether it closes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Next {
    pub number: u64,
    pub epoch: u64,
    pub closes: bool,
}

impl Next {
    /// The cut after `last`, `requested` members having asked to close its epoch in the cuts
    /// of that epoch so far; `None` past `u64`'s range.
    pub fn after(last: Option<&Cut>, requested: usize, threshold: usize) -> Option<Next> {
        let (number, epoch) = match last {
            None => (0, 0),
            Some(l) => (
                l.number.checked_add(1)?,
                l.epoch.checked_add(l.closes.into())?,
            ),
        };
        Some(Next {
            number,
            epoch,
            closes: requested >= threshold,
        })
    }

    /// `OutOfSequence` unless `cut` has the next number, then the members' checks.
    pub fn check(&self, cut: &Cut) -> Result<(), AnswerLedgerError> {
        if cut.number != self.number {
            return Err(AnswerLedgerError::OutOfSequence {
                expected: self.number,
            });
        }
        self.checks(cut).map_err(AnswerLedgerError::Checks)
    }

    fn checks(&self, cut: &Cut) -> Result<(), CheckFailure> {
        if cut.epoch != self.epoch {
            return Err(CheckFailure::Epoch {
                expected: self.epoch,
            });
        }
        if cut.closes != self.closes {
            return Err(CheckFailure::Closes {
                expected: self.closes,
            });
        }
        if cut.closes && cut.epoch == u64::MAX {
            return Err(CheckFailure::Overflow);
        }
        Ok(())
    }
}

/// A cut's entries in the order applied, then its derived steps.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CutReport {
    pub entries: Vec<(EntryId, Result<Accepted, Refusal>)>,
    pub sealed: Option<u64>,
    /// The epochs that took the value of the round this cut finished.
    pub valued: Vec<u64>,
    /// Their pending questions, drawn in question order.
    pub drawn: Vec<(u64, Applied)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Request {
    Commits(u64),
    Round(u64),
    Epoch(u64),
}

pub struct AnswerLedger {
    tiers: Tiers,
    issuer: IssuerPublic,
    path: AnswerPath,
    last: Option<Cut>,
    rounds: Rounds,
    requests: BTreeMap<Request, BTreeSet<usize>>,
    view: u64,
    refused: Vec<(EntryId, Refusal)>,
}

impl AnswerLedger {
    /// An empty ledger for the network `tiers` configures; `None` for an invalid configuration.
    pub fn new(tiers: Tiers, issuer: IssuerPublic, config: AnswerConfig) -> Option<Self> {
        let path = AnswerPath::for_network(config, tiers.network_id())?;
        Some(AnswerLedger {
            tiers,
            issuer,
            path,
            last: None,
            rounds: Rounds::new(),
            requests: BTreeMap::new(),
            view: 0,
            refused: Vec::new(),
        })
    }

    /// Applies the next cut, signed by `sigs` in view 0; an error leaves the ledger as it was.
    pub fn apply(
        &mut self,
        replica: &Replica,
        cut: &Cut,
        sigs: &[(usize, Signature)],
    ) -> Result<CutReport, AnswerLedgerError> {
        self.apply_in_view(replica, cut, 0, sigs)
    }

    /// As [`Self::apply`], for signatures given in `view`.
    pub fn apply_in_view(
        &mut self,
        replica: &Replica,
        cut: &Cut,
        view: u64,
        sigs: &[(usize, Signature)],
    ) -> Result<CutReport, AnswerLedgerError> {
        if replica.writers() != self.tiers.writers() {
            return Err(AnswerLedgerError::Replica);
        }
        let overflow = AnswerLedgerError::Checks(CheckFailure::Overflow);
        let next = self.next().ok_or(overflow)?;
        if cut.number != next.number {
            return Err(AnswerLedgerError::OutOfSequence {
                expected: next.number,
            });
        }
        let (consortium, network) = (self.tiers.consortium(), self.tiers.network_id());
        let checkpoint = cut.checkpoint_in(view, network, consortium.member_set_hash());
        if !consortium.verify_excluding(&checkpoint, sigs, &self.rounds.excluded(cut)) {
            return Err(AnswerLedgerError::Unsigned);
        }
        next.check(cut)?;
        let order = added(replica, self.last.as_ref(), cut).map_err(AnswerLedgerError::Cut)?;
        let mut report = CutReport::default();
        for id in order {
            let (_, object) = replica.get(&id).expect("`added` names only held entries");
            let result = self.entry(id, object, cut);
            if let Err(refusal) = result {
                self.refused.push((id, refusal));
            }
            report.entries.push((id, result));
        }
        if cut.closes {
            let sealed = self.path.seal().expect("the checks refuse a seal past u64");
            report.sealed = Some(sealed);
        }
        let last = self.last.replace(cut.clone());
        self.view = view;
        let consortium = self.tiers.consortium();
        self.rounds.finish(consortium, network, cut, last.as_ref());
        let finished = last.filter(|l| l.closes).map(|l| l.epoch);
        let value = finished.and_then(|f| Some((f, self.rounds.outcome(f)?.value()?)));
        if let Some((round, value)) = value {
            (report.valued, report.drawn) = self.path.assign(round, value);
        }
        Ok(report)
    }

    fn entry(&mut self, id: EntryId, object: &[u8], cut: &Cut) -> Result<Accepted, Refusal> {
        let (member, tier) = self.tiers.place(&id.writer).ok_or(Refusal::NotAuthorized)?;
        if let Some(object) = MemberObject::decode(object) {
            if tier != Tier::Public || object.member() != member {
                return Err(Refusal::NotAuthorized);
            }
            let (c, net, last) = (self.tiers.consortium(), self.tiers.network_id(), &self.last);
            let counted = self.rounds.count(c, net, &object, cut, last.as_ref());
            return counted.map(|()| Accepted::Member).map_err(|e| match e {
                Uncounted::OutOfWindow => Refusal::OutOfWindow,
                Uncounted::Beacon(e) => Refusal::Beacon(e),
            });
        }
        let event = AnswerEvent::decode(object).ok_or(Refusal::NotAnEvent)?;
        if matches!(
            event,
            AnswerEvent::Draw { .. } | AnswerEvent::CloseEpoch { .. }
        ) {
            return Err(Refusal::Derived);
        }
        if tier != Tier::Circle {
            return Err(Refusal::PublicFeed);
        }
        match &event {
            AnswerEvent::CloseCommits { question } => {
                self.close(Request::Commits(*question), member, &event)
            }
            AnswerEvent::Close { question } => {
                self.close(Request::Round(*question), member, &event)
            }
            AnswerEvent::RequestCloseEpoch { epoch } if *epoch != cut.epoch => {
                Err(Refusal::WrongEpoch)
            }
            AnswerEvent::RequestCloseEpoch { epoch } => {
                let members = self.requests.entry(Request::Epoch(*epoch)).or_default();
                if !members.insert(member) {
                    return Err(Refusal::Repeated);
                }
                Ok(Accepted::Requested {
                    count: members.len(),
                })
            }
            AnswerEvent::Deposit { question, .. } if !fits(question) => Err(Refusal::TooLarge),
            _ => (self.path.apply(&event, &self.issuer))
                .map(Accepted::Applied)
                .map_err(Refusal::Path),
        }
    }

    /// A member's request where it is defined; the `t`-th distinct one applies `event` (T79-P5).
    fn close(
        &mut self,
        request: Request,
        member: usize,
        event: &AnswerEvent,
    ) -> Result<Accepted, Refusal> {
        let (Request::Commits(q) | Request::Round(q) | Request::Epoch(q)) = request;
        let entry = (self.path.entry(q)).ok_or(Refusal::Path(Refused::UnknownQuestion))?;
        let defined = matches!(
            (request, &entry.round),
            (Request::Commits(_), Round::Committing { .. })
                | (Request::Round(_), Round::Revealing { .. })
        );
        if !defined {
            return Err(Refusal::Path(Refused::WrongRound));
        }
        let threshold = self.tiers.consortium().threshold();
        let members = self.requests.entry(request).or_default();
        if members.contains(&member) {
            return Err(Refusal::Repeated);
        }
        if members.len() + 1 < threshold {
            members.insert(member);
            return Ok(Accepted::Requested {
                count: members.len(),
            });
        }
        let applied = (self.path.apply(event, &self.issuer)).map_err(Refusal::Path)?;
        self.requests.entry(request).or_default().insert(member);
        Ok(Accepted::Applied(applied))
    }

    /// What the next cut must be under the members' checks; `None` if no cut can follow.
    pub fn next(&self) -> Option<Next> {
        let epoch = Request::Epoch(self.path.epoch());
        let requested = self.requests.get(&epoch).map_or(0, BTreeSet::len);
        let threshold = self.tiers.consortium().threshold();
        Next::after(self.last.as_ref(), requested, threshold)
    }

    /// The members' checks on a proposal after the last cut applied (`docs/05` T79-P3).
    pub fn check(&self, cut: &Cut) -> Result<(), AnswerLedgerError> {
        let next = self
            .next()
            .ok_or(AnswerLedgerError::Checks(CheckFailure::Overflow))?;
        next.check(cut)
    }

    pub fn state(&self) -> &AnswerPath {
        &self.path
    }

    pub fn last(&self) -> Option<&Cut> {
        self.last.as_ref()
    }

    /// Every entry refused so far, with its reason, in the order the cuts applied them.
    pub fn refused(&self) -> &[(EntryId, Refusal)] {
        &self.refused
    }

    pub fn beacon(&self, epoch: u64) -> Option<&BeaconOutcome> {
        self.rounds.outcome(epoch)
    }

    /// The members whose signatures `cut` would not count (rule 6).
    pub fn excluded(&self, cut: &Cut) -> Vec<usize> {
        self.rounds.excluded(cut)
    }

    /// The tagged SHA-256 of the network id, the path's state, the requests, the beacon rounds,
    /// the last cut's number and view, and the refusals, in one canonical encoding (D60).
    pub fn digest(&self) -> [u8; 32] {
        let mut state = Writer::new();
        self.path.write_state(&mut state);
        let mut w = Writer::new();
        w.u64(self.requests.len() as u64);
        for (request, members) in &self.requests {
            let (kind, n) = match request {
                Request::Commits(q) => (0, q),
                Request::Round(q) => (1, q),
                Request::Epoch(e) => (2, e),
            };
            w.u8(kind).u64(*n).u64(members.len() as u64);
            for m in members {
                w.u64(*m as u64);
            }
        }
        self.rounds.write(&mut w);
        match &self.last {
            None => w.u8(0),
            Some(cut) => w.u8(1).u64(cut.number).u64(self.view),
        };
        w.u64(self.refused.len() as u64);
        for (id, refusal) in &self.refused {
            w.fixed(&id.writer).u64(id.seq).fixed(&id.hash);
            write_refusal(&mut w, refusal);
        }
        let network = self.tiers.network_id();
        tagged(DIGEST, &[&network, &state.finish(), &w.finish()])
    }
}

fn write_refusal(w: &mut Writer, refusal: &Refusal) {
    let (code, detail) = match refusal {
        Refusal::NotAnEvent => (0, 0),
        Refusal::NotAuthorized => (1, 0),
        Refusal::PublicFeed => (2, 0),
        Refusal::Derived => (3, 0),
        Refusal::WrongEpoch => (4, 0),
        Refusal::Repeated => (5, 0),
        Refusal::TooLarge => (6, 0),
        Refusal::OutOfWindow => (7, 0),
        Refusal::Beacon(e) => (8, *e as u64),
        Refusal::Path(_) => (9, 0),
    };
    w.u8(code).u64(detail);
    if let Refusal::Path(r) = refusal {
        let (code, detail) = match r {
            Refused::WrongEpoch => (0, 0),
            Refused::Malformed(m) => (1, *m as u64),
            Refused::Unproven(u) => (2, *u as u64),
            Refused::AlreadyInPool => (3, 0),
            Refused::Duplicate { question } => (4, *question),
            Refused::OverQuota { quota } => (5, u64::from(*quota)),
            Refused::UnknownQuestion => (6, 0),
            Refused::BeaconPending => (7, 0),
            Refused::WrongRound => (8, 0),
            Refused::NotDrawn => (9, 0),
            Refused::AlreadyCommitted => (10, 0),
            Refused::NoCommit => (11, 0),
            Refused::AlreadyRevealed => (12, 0),
            Refused::RevealMismatch => (13, 0),
            Refused::AnswerOutOfRange => (14, 0),
            Refused::Overflow => (15, 0),
            Refused::NetworkOnly => (16, 0),
        };
        w.u8(code).u64(detail);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admission::Unproven;
    use crate::answer::Malformed;
    use identity::credential::Issuer;
    use network::beacon::RoundId;
    use network::consortium::{Consortium, Member};
    use network::replica::FeedWriter;

    const NET: [u8; 32] = [0x79; 32];

    fn id(b: u8) -> EntryId {
        EntryId {
            writer: [b; 32],
            seq: u64::from(b),
            hash: [b; 32],
        }
    }

    /// A ledger with requests, a cut point, refusals and a counted beacon commit.
    fn ledger(network: [u8; 32]) -> AnswerLedger {
        let members: Vec<_> = (0..4).map(|i| Member::from_seed([10 + i; 32])).collect();
        let keys = members.iter().map(Member::public).collect();
        let circle: Vec<_> = (0..4)
            .map(|i| FeedWriter::from_seed(network, [20 + i; 32]).public())
            .collect();
        let tiers = Tiers::new(network, Consortium::new(keys, 3), &circle).unwrap();
        let issuer = Issuer::new([1; 32]).public();
        let mut l = AnswerLedger::new(tiers, issuer, AnswerConfig::default()).unwrap();
        l.requests
            .insert(Request::Commits(0), BTreeSet::from([0, 1]));
        l.requests.insert(Request::Epoch(0), BTreeSet::from([2]));
        let cut = Cut {
            number: 4,
            epoch: 0,
            closes: false,
            marks: Vec::new(),
        };
        let round = RoundId {
            network_id: network,
            member_set_hash: l.tiers.consortium().member_set_hash(),
            epoch: 0,
        };
        let commit = MemberObject::Commit(members[0].beacon_commit(round, 0).0);
        let c = l.tiers.consortium();
        l.rounds.count(c, network, &commit, &cut, None).unwrap();
        (l.last, l.view) = (Some(cut), 1);
        let duplicate = Refusal::Path(Refused::Duplicate { question: 0 });
        l.refused = vec![(id(1), Refusal::NotAnEvent), (id(2), duplicate)];
        l
    }

    fn rekey(l: &mut AnswerLedger, from: Request, to: Request) {
        let members = l.requests.remove(&from).unwrap();
        l.requests.insert(to, members);
    }

    // AT-T79-15: ledgers differing in one part of the digest — network, requests, cut point,
    // refusals, rounds, path — digest apart.
    #[test]
    fn every_part_is_in_the_digest() {
        type Change = fn(&mut AnswerLedger);
        let changes: Vec<(&str, Change)> = vec![
            ("a requester", |l| {
                l.requests.get_mut(&Request::Commits(0)).unwrap().insert(3);
            }),
            ("a request's kind", |l| {
                rekey(l, Request::Commits(0), Request::Round(0))
            }),
            ("a request's question", |l| {
                rekey(l, Request::Commits(0), Request::Commits(1))
            }),
            ("a request's epoch", |l| {
                rekey(l, Request::Epoch(0), Request::Epoch(1))
            }),
            ("no cut", |l| l.last = None),
            ("the cut's number", |l| l.last.as_mut().unwrap().number += 1),
            ("the view", |l| l.view = 2),
            ("a refused writer", |l| l.refused[0].0.writer[0] ^= 1),
            ("a refused seq", |l| l.refused[0].0.seq += 1),
            ("a refused hash", |l| l.refused[0].0.hash[0] ^= 1),
            ("a reason", |l| l.refused[0].1 = Refusal::NotAuthorized),
            ("a reason's detail", |l| {
                l.refused[1].1 = Refusal::Path(Refused::Duplicate { question: 1 })
            }),
            ("a refusal more", |l| {
                l.refused.push((id(4), Refusal::Derived))
            }),
            ("their order", |l| l.refused.swap(0, 1)),
            ("a round", |l| l.rounds = Rounds::new()),
            ("the path", |l| {
                l.path.seal().unwrap();
            }),
        ];
        let reference = ledger(NET).digest();
        assert_eq!(ledger(NET).digest(), reference);
        for (part, change) in changes {
            let mut l = ledger(NET);
            change(&mut l);
            assert_ne!(l.digest(), reference, "{part}");
        }
        assert_ne!(ledger([9; 32]).digest(), reference);
    }

    // AT-T79-15: each refusal, with each detail it carries, encodes apart from every other.
    #[test]
    fn every_refusal_encodes_apart() {
        use Refused as R;
        let mut path = vec![R::WrongEpoch, R::AlreadyInPool, R::UnknownQuestion];
        path.extend([
            R::BeaconPending,
            R::WrongRound,
            R::NotDrawn,
            R::AlreadyCommitted,
        ]);
        path.extend([R::NoCommit, R::AlreadyRevealed, R::RevealMismatch]);
        path.extend([R::AnswerOutOfRange, R::Overflow, R::NetworkOnly]);
        path.extend([R::Duplicate { question: 0 }, R::Duplicate { question: 1 }]);
        path.extend([R::OverQuota { quota: 1 }, R::OverQuota { quota: 2 }]);
        path.extend([Unproven::BadProof, Unproven::WrongRole].map(R::Unproven));
        let malformed = [
            Malformed::EmptyContext,
            Malformed::EmptyStem,
            Malformed::TooFewAlternatives,
            Malformed::TooManyAlternatives,
            Malformed::EmptyAlternative,
            Malformed::RepeatedAlternative,
            Malformed::KeyOutOfRange,
        ];
        path.extend(malformed.map(R::Malformed));
        let beacon = [
            RoundError::WrongNetwork,
            RoundError::WrongMemberSet,
            RoundError::WrongEpoch,
            RoundError::NotAMember,
            RoundError::BadSignature,
            RoundError::DuplicateCommit,
            RoundError::CommitsClosed,
            RoundError::RevealsNotOpen,
            RoundError::NoCommit,
            RoundError::RevealMismatch,
            RoundError::AlreadyRevealed,
            RoundError::OutOfPhase,
        ];
        let mut all = vec![
            Refusal::NotAnEvent,
            Refusal::NotAuthorized,
            Refusal::PublicFeed,
        ];
        all.extend([Refusal::Derived, Refusal::WrongEpoch, Refusal::Repeated]);
        all.extend([Refusal::TooLarge, Refusal::OutOfWindow]);
        all.extend(beacon.map(Refusal::Beacon));
        all.extend(path.into_iter().map(Refusal::Path));
        let encoded: BTreeSet<Vec<u8>> = (all.iter())
            .map(|r| {
                let mut w = Writer::new();
                write_refusal(&mut w, r);
                w.finish()
            })
            .collect();
        assert_eq!(encoded.len(), all.len());
    }
}
