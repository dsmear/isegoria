//! The answer review on cuts against the offline path (`docs/05` §Distributed integration, the
//! mapping): generated histories, valid and invalid, give the same draws, outcomes and records.

mod answer_net;

use answer_net::*;
use identity::credential::IssuerPublic;
use identity::nym::Nym;
use network::cut::{added, Cut, CutError, MemberObject};
use network::replica::{EntryId, Replica};
use network::tier::Tier;
use proptest::prelude::*;
use protocol::answer::{Answer, Question};
use protocol::answer_events::AnswerEvent;
use protocol::answer_ledger::Refusal;
use protocol::answer_ledger::{
    fits, Accepted, AnswerLedger, AnswerLedgerError, CheckFailure, Next, MAX_QUESTION,
};
use protocol::answer_path::{AnswerConfig, AnswerPath, Refused, Round};
use protocol::answer_rule::Verdict;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

#[derive(Clone, Debug)]
enum Op {
    Join {
        person: u8,
        via: u8,
    },
    Deposit {
        author: u8,
        tag: u8,
        via: u8,
        skew: u8,
    },
    Commit {
        question: u8,
        slot: u8,
        answer: u8,
        via: u8,
        bad: bool,
    },
    CommitGroup {
        question: u8,
        pattern: u8,
        via: u8,
    },
    Reveal {
        question: u8,
        slot: u8,
        honest: bool,
        via: u8,
    },
    RevealGroup {
        question: u8,
        skip: u16,
        via: u8,
    },
    Request {
        kind: u8,
        target: u8,
        member: u8,
    },
    RequestAll {
        kind: u8,
        target: u8,
        from: u8,
    },
    Again {
        via: u8,
    },
    Noise {
        kind: u8,
        via: u8,
    },
    /// The first open round's next step: its group's commits, then `t` requests to close them,
    /// its reveals, then `t` requests to close it.
    Advance {
        pattern: u8,
        skip: u16,
        from: u8,
    },
    /// `t` members ask to close the open epoch.
    CloseEpoch {
        from: u8,
    },
    Cut,
    /// A cut whose signatures are given in view 1 or 2.
    CutIn {
        view: u8,
    },
    BadCut {
        kind: u8,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Commits(u64),
    Round(u64),
    Epoch(u64),
}

/// What an entry stands for offline: a person's event, a member's request, or nothing.
#[derive(Clone)]
enum Mapped {
    Person(Rc<AnswerEvent>),
    Request(Kind, usize),
    Refused(Refusal),
}

/// The network and the offline path side by side, the model counting the requests.
struct Model {
    net: Net,
    config: AnswerConfig,
    pk: IssuerPublic,
    offline: AnswerPath,
    people: Vec<Person>,
    authors: Vec<Person>,
    mapped: BTreeMap<EntryId, Mapped>,
    last: Option<(Vec<u8>, Rc<AnswerEvent>)>,
    secrets: BTreeMap<(u64, Nym), (Answer, [u8; 32])>,
    requests: BTreeMap<Kind, BTreeSet<usize>>,
    lacking: BTreeSet<u64>,
    withhold: Vec<BTreeSet<usize>>,
    values: BTreeMap<u64, Option<[u8; 32]>>,
    nonce: u64,
    attempted: BTreeMap<(u64, bool), usize>,
}

fn answer_of(code: u8) -> Answer {
    match code % 8 {
        0..=2 => Answer::Alternative(0),
        3 => Answer::Alternative(1),
        4 => Answer::Alternative(2),
        5 => Answer::NoneCorrect,
        6 => Answer::DontKnow,
        _ => Answer::Alternative(3),
    }
}

/// The `i`-th group member's answer under `pattern`: all the key, all "none", half, or mostly
/// "don't know".
fn patterned(pattern: u8, i: usize) -> Answer {
    match pattern % 4 {
        0 => Answer::Alternative(0),
        1 => Answer::NoneCorrect,
        2 if i % 2 == 0 => Answer::Alternative(0),
        2 => Answer::Alternative(2),
        _ if i % 3 == 0 => Answer::Alternative(0),
        _ => Answer::DontKnow,
    }
}

fn same_state(net: &AnswerPath, off: &AnswerPath, authors: &[Person]) {
    assert_eq!(net.epoch(), off.epoch());
    assert_eq!(net.closings(), off.closings());
    assert_eq!(net.pool(), off.pool());
    assert_eq!(net.questions(), off.questions());
    for q in 0..off.questions() {
        assert_eq!(net.entry(q), off.entry(q), "question {q}");
    }
    let usable = |p: &AnswerPath| p.usable().map(|(q, _)| q).collect::<Vec<_>>();
    assert_eq!(usable(net), usable(off));
    for a in authors.iter().map(|a| &a.propose) {
        assert_eq!(net.author(a), off.author(a));
        assert_eq!(net.reputation(a).to_bits(), off.reputation(a).to_bits());
        assert_eq!(net.quota(a), off.quota(a));
        for e in 0..=off.epoch() {
            assert_eq!(net.slots(a, e), off.slots(a, e));
        }
    }
}

impl Model {
    fn new(config: AnswerConfig, people: u8, withhold: Vec<BTreeSet<usize>>) -> Model {
        let net = Net::new(config);
        let pk = net.issuer.public();
        Model {
            people: (0..people).map(|s| person(&net.issuer, 100 + s)).collect(),
            authors: (1..3).map(|s| person(&net.issuer, s)).collect(),
            offline: AnswerPath::new(config).unwrap(),
            net,
            config,
            pk,
            mapped: BTreeMap::new(),
            last: None,
            secrets: BTreeMap::new(),
            requests: BTreeMap::new(),
            lacking: BTreeSet::new(),
            withhold,
            values: BTreeMap::new(),
            nonce: 0,
            attempted: BTreeMap::new(),
        }
    }

    fn withheld(&self, epoch: u64) -> BTreeSet<usize> {
        self.withhold[epoch as usize % self.withhold.len()].clone()
    }

    fn value(&mut self, epoch: u64) -> Option<[u8; 32]> {
        let revealing: BTreeSet<usize> = (0..N)
            .filter(|m| !self.withheld(epoch).contains(m))
            .collect();
        *self
            .values
            .entry(epoch)
            .or_insert_with(|| value(epoch, &revealing))
    }

    /// The value epoch `e` takes: the first round from `e` on that has one.
    fn value_for(&mut self, e: u64) -> [u8; 32] {
        let horizon = e + self.withhold.len() as u64 + 1;
        (e..=horizon)
            .find_map(|f| self.value(f))
            .unwrap_or([0xee; 32])
    }

    fn question(&self, pick: u8) -> u64 {
        u64::from(pick) % (self.offline.questions() + 1)
    }

    fn group(&self, q: u64) -> Vec<Nym> {
        match self.offline.entry(q).map(|e| &e.round) {
            Some(Round::Committing { group, .. } | Round::Revealing { group, .. }) => group.clone(),
            _ => Vec::new(),
        }
    }

    /// Group member `slot` of `q`, or for a slot past the group any person.
    fn reviewer(&self, q: u64, slot: u8) -> usize {
        let group = self.group(q);
        let i = usize::from(slot) % (group.len() + 2);
        let by_nym = |n: &Nym| self.people.iter().position(|p| p.judge == *n);
        group
            .get(i)
            .and_then(by_nym)
            .unwrap_or(usize::from(slot) % self.people.len())
    }

    fn person_event(&mut self, via: u8, net: AnswerEvent, off: AnswerEvent) {
        let bytes = net.encode();
        let id = self
            .net
            .publish(usize::from(via) % N, Tier::Circle, bytes.clone());
        let off = Rc::new(off);
        self.mapped.insert(id, Mapped::Person(off.clone()));
        self.last = Some((bytes, off));
    }

    fn commit(&mut self, q: u64, p: usize, answer: Answer, via: u8, bad: bool) {
        self.nonce += 1;
        let nonce: [u8; 32] = std::array::from_fn(|i| (self.nonce >> (8 * (i % 8))) as u8);
        let (net, off) = if bad {
            (Some(&OTHER), Some(&OTHER))
        } else {
            (Some(&NET), None)
        };
        let (i, who) = (&self.net.issuer, &self.people[p]);
        let events = (
            commit(i, who, q, answer, nonce, net),
            commit(i, who, q, answer, nonce, off),
        );
        self.secrets
            .entry((q, who.judge))
            .or_insert((answer, nonce));
        self.person_event(via, events.0, events.1);
    }

    fn reveal(&mut self, q: u64, p: usize, honest: bool, via: u8) {
        let who = &self.people[p];
        let secret = self.secrets.get(&(q, who.judge)).copied();
        let (answer, mut nonce) = secret.unwrap_or((Answer::Alternative(0), [0xab; 32]));
        nonce[0] ^= u8::from(!honest);
        let events = (reveal(who, q, answer, nonce), reveal(who, q, answer, nonce));
        self.person_event(via, events.0, events.1);
    }

    fn request(&mut self, kind: u8, target: u8, member: usize) {
        let epoch = self.offline.epoch();
        let kind = match kind % 3 {
            0 => Kind::Commits(self.question(target)),
            1 => Kind::Round(self.question(target)),
            _ => Kind::Epoch(match target % 5 {
                0 => epoch.saturating_sub(1),
                1 => epoch + 1,
                _ => epoch,
            }),
        };
        self.ask(kind, member);
    }

    fn ask(&mut self, kind: Kind, member: usize) {
        let event = match kind {
            Kind::Commits(question) => AnswerEvent::CloseCommits { question },
            Kind::Round(question) => AnswerEvent::Close { question },
            Kind::Epoch(epoch) => AnswerEvent::RequestCloseEpoch { epoch },
        };
        let id = self.net.log(member % N, &event);
        self.mapped.insert(id, Mapped::Request(kind, member % N));
    }

    fn advance(&mut self, pattern: u8, skip: u16, from: u8) {
        let open = |q: &u64| match self.offline.entry(*q).map(|e| &e.round) {
            Some(Round::Committing { .. }) => Some((*q, false)),
            Some(Round::Revealing { .. }) => Some((*q, true)),
            _ => None,
        };
        let Some((q, revealing)) = (0..self.offline.questions()).find_map(|q| open(&q)) else {
            return;
        };
        let (group, cuts) = (self.group(q), self.net.cuts.len());
        match self.attempted.get(&(q, revealing)) {
            None => {
                self.attempted.insert((q, revealing), cuts);
                for (i, _) in group.iter().enumerate() {
                    let p = self.reviewer(q, i as u8);
                    if !revealing {
                        self.commit(q, p, patterned(pattern, i), from, false);
                    } else if (skip >> (i % 16)) & 1 == 0 {
                        self.reveal(q, p, true, from);
                    }
                }
            }
            Some(&at) if at < cuts => {
                let kind = if revealing {
                    Kind::Round(q)
                } else {
                    Kind::Commits(q)
                };
                for m in 0..T {
                    self.ask(kind, usize::from(from) + m);
                }
            }
            Some(_) => {}
        }
    }

    fn noise(&mut self, kind: u8, via: u8) {
        let m = usize::from(via) % N;
        let epoch = self.offline.epoch();
        let own = |m: usize| MemberObject::Commit(members()[m].beacon_commit(round(epoch), m).0);
        let ask = AnswerEvent::RequestCloseEpoch { epoch }.encode();
        let (tier, bytes, why) = match kind % 6 {
            0 => (
                Tier::Circle,
                AnswerEvent::Draw { question: 0 }.encode(),
                Refusal::Derived,
            ),
            1 => {
                let seal = AnswerEvent::CloseEpoch {
                    epoch,
                    beacon: [3; 32],
                };
                (Tier::Public, seal.encode(), Refusal::Derived)
            }
            2 => {
                let bytes = self.last.as_ref().map_or(ask, |(b, _)| b.clone());
                (Tier::Public, bytes, Refusal::PublicFeed)
            }
            3 => (Tier::Circle, b"noise".to_vec(), Refusal::NotAnEvent),
            4 => (Tier::Circle, own(m).encode(), Refusal::NotAuthorized),
            _ => (
                Tier::Public,
                own((m + 1) % N).encode(),
                Refusal::NotAuthorized,
            ),
        };
        let id = self.net.publish(m, tier, bytes);
        self.mapped.insert(id, Mapped::Refused(why));
    }

    fn next(&self) -> Next {
        let epoch = self.offline.epoch();
        let asked = self
            .requests
            .get(&Kind::Epoch(epoch))
            .map_or(0, BTreeSet::len);
        Next {
            number: self.net.cuts.len() as u64,
            epoch,
            closes: asked >= T,
        }
    }

    fn run(&mut self, op: &Op) {
        match *op {
            Op::Join { person, via } => {
                let p = &self.people[usize::from(person) % self.people.len()];
                let events = (
                    join(&self.net.issuer, p, Some(&NET)),
                    join(&self.net.issuer, p, None),
                );
                self.person_event(via, events.0, events.1);
            }
            Op::Deposit {
                author,
                tag,
                via,
                skew,
            } => {
                let epoch = self.offline.epoch();
                let named = match skew % 8 {
                    0 => epoch + 1,
                    1 => epoch.saturating_sub(1),
                    _ => epoch,
                };
                let q = match tag % 6 {
                    4 => Question {
                        context: Vec::new(),
                        ..question("malformed")
                    },
                    5 => Question {
                        context: vec![b'x'; MAX_QUESTION],
                        ..question("oversized")
                    },
                    t => question(&format!("q{t}")),
                };
                let a = &self.authors[usize::from(author) % self.authors.len()];
                let i = &self.net.issuer;
                let events = (
                    deposit(i, a, &q, named, Some(&NET)),
                    deposit(i, a, &q, named, None),
                );
                if fits(&q) {
                    self.person_event(via, events.0, events.1);
                } else {
                    let id = self.net.log(usize::from(via) % N, &events.0);
                    self.mapped.insert(id, Mapped::Refused(Refusal::TooLarge));
                }
            }
            Op::Commit {
                question,
                slot,
                answer,
                via,
                bad,
            } => {
                let q = self.question(question);
                let p = self.reviewer(q, slot);
                self.commit(q, p, answer_of(answer), via, bad);
            }
            Op::CommitGroup {
                question,
                pattern,
                via,
            } => {
                let q = self.question(question);
                for (i, _) in self.group(q).iter().enumerate() {
                    let p = self.reviewer(q, i as u8);
                    self.commit(q, p, patterned(pattern, i), via, false);
                }
            }
            Op::Reveal {
                question,
                slot,
                honest,
                via,
            } => {
                let q = self.question(question);
                let p = self.reviewer(q, slot);
                self.reveal(q, p, honest, via);
            }
            Op::RevealGroup {
                question,
                skip,
                via,
            } => {
                let q = self.question(question);
                for (i, _) in self.group(q).iter().enumerate() {
                    if (skip >> (i % 16)) & 1 == 0 {
                        let p = self.reviewer(q, i as u8);
                        self.reveal(q, p, true, via);
                    }
                }
            }
            Op::Request {
                kind,
                target,
                member,
            } => self.request(kind, target, usize::from(member)),
            Op::RequestAll { kind, target, from } => {
                for m in 0..T {
                    self.request(kind, target, usize::from(from) + m);
                }
            }
            Op::Again { via } => {
                if let Some((bytes, off)) = self.last.clone() {
                    let id = self.net.publish(usize::from(via) % N, Tier::Circle, bytes);
                    self.mapped.insert(id, Mapped::Person(off));
                }
            }
            Op::Noise { kind, via } => self.noise(kind, via),
            Op::Advance {
                pattern,
                skip,
                from,
            } => self.advance(pattern, skip, from),
            Op::CloseEpoch { from } => {
                let epoch = Kind::Epoch(self.offline.epoch());
                for m in 0..T {
                    self.ask(epoch, usize::from(from) + m);
                }
            }
            Op::Cut => self.cut(),
            Op::CutIn { view } => self.cut_in(1 + u64::from(view % 2)),
            Op::BadCut { kind } => self.bad_cut(kind),
        }
    }

    /// What the model expects of entry `id` in `cut`, the offline path stepped alongside.
    fn step(&mut self, id: &EntryId, cut: &Cut) -> Result<Accepted, Refusal> {
        if self.net.beacon.contains(id) {
            return Ok(Accepted::Member);
        }
        let (kind, m) = match self.mapped.get(id).cloned().expect("every entry is mapped") {
            Mapped::Refused(why) => return Err(why),
            Mapped::Person(e) => {
                let applied = self.offline.apply(&e, &self.pk);
                return applied.map(Accepted::Applied).map_err(Refusal::Path);
            }
            Mapped::Request(kind, m) => (kind, m),
        };
        let event = match kind {
            Kind::Epoch(e) if e != cut.epoch => return Err(Refusal::WrongEpoch),
            Kind::Epoch(_) => {
                let set = self.requests.entry(kind).or_default();
                let count = set.len() + 1;
                return if set.insert(m) {
                    Ok(Accepted::Requested { count })
                } else {
                    Err(Refusal::Repeated)
                };
            }
            Kind::Commits(question) => AnswerEvent::CloseCommits { question },
            Kind::Round(question) => AnswerEvent::Close { question },
        };
        let (Kind::Commits(q) | Kind::Round(q)) = kind else {
            unreachable!()
        };
        let entry = self
            .offline
            .entry(q)
            .ok_or(Refusal::Path(Refused::UnknownQuestion))?;
        let defined = matches!(
            (kind, &entry.round),
            (Kind::Commits(_), Round::Committing { .. })
                | (Kind::Round(_), Round::Revealing { .. })
        );
        if !defined {
            return Err(Refusal::Path(Refused::WrongRound));
        }
        let set = self.requests.entry(kind).or_default();
        if !set.insert(m) {
            return Err(Refusal::Repeated);
        }
        if set.len() < T {
            return Ok(Accepted::Requested { count: set.len() });
        }
        let applied = self.offline.apply(&event, &self.pk);
        Ok(Accepted::Applied(
            applied.expect("the t-th request's transition is defined"),
        ))
    }

    fn cut(&mut self) {
        self.cut_in(0);
    }

    fn cut_in(&mut self, view: u64) {
        let next = self.next();
        let (replica, last) = (&self.net.replica, self.net.ledger.last());
        let cut = Cut::next(replica, last, next.epoch, next.closes);
        let order = added(replica, last, &cut).unwrap();
        let previous = last.filter(|c| c.closes).map(|c| c.epoch);
        if cut.closes {
            self.net.withhold = self.withheld(cut.epoch);
        }
        let report = (self.net.apply_in(cut.clone(), view)).expect("the model's cut applies");
        let expected: Vec<_> = order.iter().map(|id| (*id, self.step(id, &cut))).collect();
        assert_eq!(report.entries, expected);
        let mut sealed = None;
        if cut.closes {
            let beacon = self.value_for(cut.epoch);
            let seal = AnswerEvent::CloseEpoch {
                epoch: cut.epoch,
                beacon,
            };
            self.offline.apply(&seal, &self.pk).unwrap();
            self.lacking.insert(cut.epoch);
            sealed = Some(cut.epoch);
        }
        let (mut valued, mut drawn) = (Vec::new(), Vec::new());
        if let Some(f) = previous.filter(|f| self.value(*f).is_some()) {
            valued = self.lacking.range(..=f).copied().collect();
            self.lacking.retain(|e| *e > f);
            for q in 0..self.offline.questions() {
                let entry = self.offline.entry(q).unwrap();
                if valued.contains(&entry.epoch) && entry.round == Round::Pending {
                    let draw = AnswerEvent::Draw { question: q };
                    drawn.push((q, self.offline.apply(&draw, &self.pk).unwrap()));
                }
            }
        }
        assert_eq!(
            (report.sealed, report.valued, report.drawn),
            (sealed, valued, drawn)
        );
        same_state(self.net.ledger.state(), &self.offline, &self.authors);
    }

    /// A cut the members' checks refuse, one out of sequence, one short of signatures, or one on
    /// another replica: refused, the ledger unchanged.
    fn bad_cut(&mut self, kind: u8) {
        let next = self.next();
        let net = &self.net;
        let base = Cut::next(&net.replica, net.ledger.last(), next.epoch, next.closes);
        let checks = AnswerLedgerError::Checks;
        let out = AnswerLedgerError::OutOfSequence {
            expected: next.number,
        };
        let (cut, view, sigs, error) = match kind % 7 {
            0 => {
                let cut = Cut {
                    epoch: next.epoch + 1,
                    ..base
                };
                let epoch = CheckFailure::Epoch {
                    expected: next.epoch,
                };
                (cut, 0, None, checks(epoch))
            }
            1 => {
                let cut = Cut {
                    closes: !next.closes,
                    ..base
                };
                let closes = CheckFailure::Closes {
                    expected: next.closes,
                };
                (cut, 0, None, checks(closes))
            }
            3 if !net.cuts.is_empty() => {
                let (cut, view, sigs) = net.cuts.last().unwrap().clone();
                (cut, view, Some(sigs), out)
            }
            4 if net.cuts.len() > 1 => {
                let (cut, view, sigs) = net.cuts[0].clone();
                (cut, view, Some(sigs), out)
            }
            5 => {
                let short = net.signed_by(&base, 0, 0..T - 1);
                (base, 0, Some(short), AnswerLedgerError::Unsigned)
            }
            6 => (base, 0, None, AnswerLedgerError::Replica),
            _ => {
                let cut = Cut {
                    number: next.number + 1,
                    ..base
                };
                (cut, 0, None, out)
            }
        };
        let sigs = sigs.unwrap_or_else(|| net.sign_in(&cut, view));
        if !matches!(
            error,
            AnswerLedgerError::Unsigned | AnswerLedgerError::Replica
        ) {
            assert_eq!(net.ledger.check(&cut), Err(error));
        }
        let before = (net.ledger.digest(), net.ledger.last().cloned());
        let foreign = Replica::new(tiers_on(OTHER).writers().clone());
        let (ledger, replica) = (&mut self.net.ledger, &self.net.replica);
        let on = if error == AnswerLedgerError::Replica {
            &foreign
        } else {
            replica
        };
        let applied = ledger.apply_in_view(on, &cut, view, &sigs);
        assert_eq!(applied.map(|_| ()), Err(error));
        let after = (ledger.digest(), ledger.last().cloned());
        assert_eq!(after, before);
    }

    /// The same cuts on a new ledger, the entries delivered shuffled, each twice, a cut applied
    /// once its entries are held.
    fn replay(&self, seed: u64) {
        let mut entries = self.net.entries();
        entries.shuffle(&mut ChaCha8Rng::seed_from_u64(seed));
        let mut replica = Replica::new(tiers().writers().clone());
        let mut ledger = AnswerLedger::new(tiers(), self.pk.clone(), self.config).unwrap();
        let mut next = 0;
        for (entry, object) in entries {
            replica.insert(entry.clone(), object.clone()).unwrap();
            replica.insert(entry, object).unwrap();
            while let Some((cut, view, sigs)) = self.net.cuts.get(next) {
                match ledger.apply_in_view(&replica, cut, *view, sigs) {
                    Ok(_) => next += 1,
                    Err(AnswerLedgerError::Cut(CutError::Missing { .. })) => break,
                    Err(e) => panic!("{e:?}"),
                }
            }
        }
        assert_eq!(next, self.net.cuts.len());
        assert_eq!(ledger.state(), self.net.ledger.state());
        assert_eq!(ledger.digest(), self.net.ledger.digest());
    }
}

fn op() -> impl Strategy<Value = Op> {
    let b = any::<u8>;
    prop_oneof![
        3 => (b(), b()).prop_map(|(person, via)| Op::Join { person, via }),
        2 => (b(), b(), b(), b()).prop_map(|(author, tag, via, skew)| Op::Deposit { author, tag, via, skew }),
        2 => (b(), b(), b(), b(), prop::bool::weighted(0.1))
            .prop_map(|(question, slot, answer, via, bad)| Op::Commit { question, slot, answer, via, bad }),
        2 => (b(), b(), b()).prop_map(|(question, pattern, via)| Op::CommitGroup { question, pattern, via }),
        1 => (b(), b(), prop::bool::weighted(0.85), b())
            .prop_map(|(question, slot, honest, via)| Op::Reveal { question, slot, honest, via }),
        2 => (b(), absent(), b())
            .prop_map(|(question, skip, via)| Op::RevealGroup { question, skip, via }),
        1 => (b(), b(), b()).prop_map(|(kind, target, member)| Op::Request { kind, target, member }),
        1 => (b(), b(), b()).prop_map(|(kind, target, from)| Op::RequestAll { kind, target, from }),
        1 => b().prop_map(|via| Op::Again { via }),
        1 => (b(), b()).prop_map(|(kind, via)| Op::Noise { kind, via }),
        8 => (b(), absent(), b())
            .prop_map(|(pattern, skip, from)| Op::Advance { pattern, skip, from }),
        2 => b().prop_map(|from| Op::CloseEpoch { from }),
        6 => Just(Op::Cut),
        1 => b().prop_map(|view| Op::CutIn { view }),
        1 => b().prop_map(|kind| Op::BadCut { kind }),
    ]
}

/// Which of a group do not reveal: none, exactly 3 or 4 of 12 (the quorum's edge), or any.
fn absent() -> impl Strategy<Value = u16> {
    let exactly = |k| {
        prop::sample::subsequence((0..12u16).collect::<Vec<_>>(), k)
            .prop_map(|bits| bits.iter().fold(0, |mask, b| mask | 1 << b))
    };
    prop_oneof![5 => Just(0u16), 2 => exactly(3), 1 => exactly(4), 1 => any::<u16>()]
}

/// Other ops spliced into the group-of-12 histories.
fn aside() -> impl Strategy<Value = Op> {
    let b = any::<u8>;
    prop_oneof![
        2 => b().prop_map(|view| Op::CutIn { view }),
        1 => (b(), b(), b(), b(), prop::bool::weighted(0.2))
            .prop_map(|(question, slot, answer, via, bad)| Op::Commit { question, slot, answer, via, bad }),
        1 => (b(), b(), prop::bool::weighted(0.7), b())
            .prop_map(|(question, slot, honest, via)| Op::Reveal { question, slot, honest, via }),
        1 => (b(), b(), b()).prop_map(|(kind, target, member)| Op::Request { kind, target, member }),
        1 => b().prop_map(|via| Op::Again { via }),
        1 => (b(), b()).prop_map(|(kind, via)| Op::Noise { kind, via }),
        2 => b().prop_map(|kind| Op::BadCut { kind }),
        1 => (b(), b(), b(), 2..8u8).prop_map(|(author, tag, via, skew)| Op::Deposit { author, tag, via, skew }),
    ]
}

/// One epoch of a group-of-12 history: its deposits, its closure and the draws, then each round
/// advanced through commits, their closure, reveals and its closure, other ops in between.
fn epoch12() -> impl Strategy<Value = Vec<Op>> {
    let b = any::<u8>;
    let deposits = prop::collection::vec(
        (b(), 0..4u8, b()).prop_map(|(author, tag, via)| Op::Deposit {
            author,
            tag,
            via,
            skew: 2,
        }),
        1..3,
    );
    let step = (b(), absent(), b(), prop::option::weighted(0.4, aside()));
    let rounds = prop::collection::vec(step, 4..10);
    (deposits, b(), rounds).prop_map(|(deposits, from, rounds)| {
        let mut ops = deposits;
        ops.extend([Op::CloseEpoch { from }, Op::Cut, Op::Cut, Op::Cut]);
        for (pattern, skip, from, aside) in rounds {
            ops.extend([
                Op::Advance {
                    pattern,
                    skip,
                    from,
                },
                Op::Cut,
            ]);
            ops.extend(aside);
        }
        ops
    })
}

/// Each epoch's withholders: none, one (a value, and that member excluded), or two (no value).
fn withhold() -> impl Strategy<Value = Vec<BTreeSet<usize>>> {
    let set = prop_oneof![
        5 => Just(BTreeSet::new()),
        2 => (0..N).prop_map(|m| BTreeSet::from([m])),
        2 => (0..N, 1..N).prop_map(|(m, d)| BTreeSet::from([m, (m + d) % N])),
    ];
    prop::collection::vec(set, 1..5)
}

fn check(
    config: AnswerConfig,
    people: u8,
    prelude: &[u8],
    ops: &[Op],
    withheld: Vec<BTreeSet<usize>>,
    seed: u64,
) -> Model {
    let mut model = Model::new(config, people, withheld);
    for &person in prelude {
        model.run(&Op::Join {
            person,
            via: person,
        });
    }
    for op in ops {
        model.run(op);
    }
    model.cut();
    model.replay(seed);
    model
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 24, max_shrink_iters: 64, ..ProptestConfig::default() })]

    /// AT-T79-12: under the mapping of `docs/05`, a small group's histories give on cuts the
    /// offline path's state and refusals, the same state after a reordered, doubled replay.
    #[test]
    fn at_t79_12_cuts_follow_the_offline_path(
        prelude in prop::collection::vec(0u8..5, 0..6),
        ops in prop::collection::vec(op(), 40..140),
        withheld in withhold(),
        seed in any::<u64>(),
    ) {
        check(small(), 5, &prelude, &ops, withheld, seed);
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 16, max_shrink_iters: 16, ..ProptestConfig::default() })]

    /// AT-T79-12: the same under D49's group of 12 and quorum of 9, the pool joined first, the
    /// rounds driven to their outcomes.
    #[test]
    fn at_t79_12_cuts_follow_the_offline_path_at_twelve(
        epochs in prop::collection::vec(epoch12(), 1..4),
        withheld in withhold(),
        seed in any::<u64>(),
    ) {
        let ops = epochs.concat();
        check(AnswerConfig::default(), 14, &(0..13).collect::<Vec<_>>(), &ops, withheld, seed);
    }
}

/// AT-T79-12: two discards on cuts set their author's quota to 1 from the next epoch, as offline.
#[test]
fn at_t79_12_a_quota_change_on_cuts() {
    let deposit = |tag| Op::Deposit {
        author: 0,
        tag,
        via: tag,
        skew: 2,
    };
    let advance = Op::Advance {
        pattern: 1,
        skip: 0,
        from: 1,
    };
    let close = [Op::CloseEpoch { from: 2 }, Op::Cut, Op::Cut, Op::Cut];
    let round: Vec<Op> = [advance, Op::Cut].iter().cycle().take(8).cloned().collect();
    let ops = [
        &[deposit(0), deposit(1)][..],
        &close,
        &round,
        &round,
        &close,
        &[deposit(2), deposit(3)],
    ]
    .concat();
    let model = check(small(), 5, &[0, 1, 2, 3, 4], &ops, vec![BTreeSet::new()], 7);
    let (path, author) = (&model.offline, &model.authors[0].propose);
    let verdicts: Vec<_> = (0..2).map(|q| path.closure(q).unwrap().verdict).collect();
    assert_eq!(verdicts, [Verdict::Discarded; 2]);
    assert_eq!((path.epoch(), path.quota(author)), (2, 1));
    let slots = path.slots(author, 2).unwrap();
    assert_eq!((slots.quota, slots.used), (1, 1));
}
