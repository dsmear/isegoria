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
    /// Not the cut after the last one this ledger applied.
    NotNext {
        expected: u64,
    },
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

    pub fn check(&self, cut: &Cut) -> Result<(), CheckFailure> {
        if cut.number != self.number {
            return Err(CheckFailure::NotNext {
                expected: self.number,
            });
        }
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
        let (consortium, network) = (self.tiers.consortium(), self.tiers.network_id());
        let checkpoint = cut.checkpoint_in(view, network, consortium.member_set_hash());
        if !consortium.verify_excluding(&checkpoint, sigs, &self.rounds.excluded(cut)) {
            return Err(AnswerLedgerError::Unsigned);
        }
        self.check(cut).map_err(AnswerLedgerError::Checks)?;
        let order = added(replica, self.last.as_ref(), cut).map_err(AnswerLedgerError::Cut)?;
        let mut report = CutReport::default();
        for id in order {
            let (_, object) = replica.get(&id).expect("`added` names only held entries");
            report.entries.push((id, self.entry(id, object, cut)));
        }
        if cut.closes {
            let sealed = self.path.seal().expect("the checks refuse a seal past u64");
            report.sealed = Some(sealed);
        }
        let last = self.last.replace(cut.clone());
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
    pub fn check(&self, cut: &Cut) -> Result<(), CheckFailure> {
        self.next().ok_or(CheckFailure::Overflow)?.check(cut)
    }

    pub fn state(&self) -> &AnswerPath {
        &self.path
    }

    pub fn last(&self) -> Option<&Cut> {
        self.last.as_ref()
    }

    pub fn beacon(&self, epoch: u64) -> Option<&BeaconOutcome> {
        self.rounds.outcome(epoch)
    }

    /// The members whose signatures `cut` would not count (rule 6).
    pub fn excluded(&self, cut: &Cut) -> Vec<usize> {
        self.rounds.excluded(cut)
    }

    /// The tagged SHA-256 of the network id, the path's state and the requests counted (D60).
    pub fn digest(&self) -> [u8; 32] {
        let mut state = Writer::new();
        self.path.write_state(&mut state);
        let mut requests = Writer::new();
        requests.u64(self.requests.len() as u64);
        for (request, members) in &self.requests {
            let (kind, n) = match request {
                Request::Commits(q) => (0, q),
                Request::Round(q) => (1, q),
                Request::Epoch(e) => (2, e),
            };
            requests.u8(kind).u64(*n).u64(members.len() as u64);
            for m in members {
                requests.u64(*m as u64);
            }
        }
        let network = self.tiers.network_id();
        tagged(DIGEST, &[&network, &state.finish(), &requests.finish()])
    }
}
