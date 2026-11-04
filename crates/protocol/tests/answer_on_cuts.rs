//! The answer review on signed cuts (`docs/05` §Distributed integration, `docs/01` D60, T79):
//! the family's refusals, closures by `t` members, derived seals and draws, the members' checks.

mod answer_net;

use answer_net::*;
use identity::credential::Issuer;
use identity::nullifier::prove;
use identity::nym::Role;
use network::codec::Writer;
use network::cut::{Cut, CutError, MemberObject};
use network::replica::{EntryId, Replica};
use network::tier::Tier;
use protocol::admission::Unproven;
use protocol::answer::{Answer, Question};
use protocol::answer_events::AnswerEvent;
use protocol::answer_ledger::{
    fits, Accepted, AnswerLedger, AnswerLedgerError, CheckFailure, CutReport, Next, Refusal,
    MAX_QUESTION,
};
use protocol::answer_path::{
    network_context, AnswerConfig, AnswerPath, Applied, Refused, Round, Slots, NETWORK_FAMILY,
    POOL_CONTEXT,
};
use protocol::answer_rule::Verdict;
use protocol::deposit::{deposit_context, Draft};
use protocol::events::NodeEvent;
use protocol::lottery;
use protocol::randomness::{Beacon, ANSWER_DRAW};
use std::collections::BTreeSet;

use Accepted::{Applied as Done, Requested};
use CheckFailure::{Closes, Epoch, NotNext};

fn outcome(report: &CutReport, id: EntryId) -> Result<Accepted, Refusal> {
    let found = report.entries.iter().find(|(i, _)| *i == id);
    found.expect("the cut applied the entry").1.clone()
}

fn path(r: Refused) -> Result<Accepted, Refusal> {
    Err(Refusal::Path(r))
}

fn legacy(issuer: &Issuer) -> Vec<u8> {
    let draft = Draft {
        item: b"item".to_vec(),
        primary_source: b"source".to_vec(),
    };
    let ctx = deposit_context(draft.content_id(), 0);
    let proof = prove(
        &person(issuer, 2).cred,
        &issuer.public(),
        Role::Propose,
        &ctx,
    );
    let event = NodeEvent::Deposit {
        epoch: 0,
        quota: 1,
        draft,
        proof,
    };
    event.encode()
}

fn group(net: &Net, q: u64) -> Vec<identity::nym::Nym> {
    match &net.ledger.state().entry(q).unwrap().round {
        Round::Committing { group, .. } | Round::Revealing { group, .. } => group.clone(),
        other => panic!("{other:?}"),
    }
}

fn beacon(net: &Net, epoch: u64) -> Beacon {
    Beacon::from_outcome(net.ledger.beacon(epoch).unwrap()).unwrap()
}

fn encoded(q: &Question) -> usize {
    let mut w = Writer::new();
    q.write(&mut w);
    w.finish().len()
}

/// AT-T79-1: derived kinds on any feed, answer events in public, other bytes and misfiled member
/// objects are refused, the state unchanged; offline, a closure request is refused.
#[test]
fn at_t79_1_what_each_feed_may_carry() {
    let mut net = Net::new(AnswerConfig::default());
    let joined = join(&net.issuer, &person(&net.issuer, 100), Some(&NET)).encode();
    let draw = AnswerEvent::Draw { question: 0 }.encode();
    let seal = AnswerEvent::CloseEpoch {
        epoch: 0,
        beacon: [1; 32],
    }
    .encode();
    let ask = AnswerEvent::RequestCloseEpoch { epoch: 0 }.encode();
    let commit = MemberObject::Commit(members()[1].beacon_commit(round(0), 1).0).encode();
    let old = legacy(&net.issuer);
    let expected = vec![
        (net.publish(0, Tier::Circle, draw.clone()), Refusal::Derived),
        (net.publish(1, Tier::Public, draw), Refusal::Derived),
        (net.publish(2, Tier::Circle, seal.clone()), Refusal::Derived),
        (net.publish(3, Tier::Public, seal), Refusal::Derived),
        (net.publish(0, Tier::Public, joined), Refusal::PublicFeed),
        (net.publish(1, Tier::Public, ask), Refusal::PublicFeed),
        (net.publish(0, Tier::Circle, old), Refusal::NotAnEvent),
        (
            net.publish(2, Tier::Circle, b"no event".to_vec()),
            Refusal::NotAnEvent,
        ),
        (
            net.publish(1, Tier::Circle, commit.clone()),
            Refusal::NotAuthorized,
        ),
        (net.publish(2, Tier::Public, commit), Refusal::NotAuthorized),
    ];
    let fresh = AnswerLedger::new(tiers(), net.issuer.public(), AnswerConfig::default()).unwrap();
    let report = net.cut();
    for (id, why) in &expected {
        assert_eq!(outcome(&report, *id), Err(*why));
    }
    assert_eq!(report.entries.len(), expected.len() + N);
    assert_eq!(net.ledger.state(), fresh.state());
    assert_eq!(net.ledger.digest(), fresh.digest());
    let request = AnswerEvent::RequestCloseEpoch { epoch: 7 };
    let bytes = request.encode();
    assert_eq!(AnswerEvent::decode(&bytes).map(|e| e.encode()), Some(bytes));
    let mut offline = AnswerPath::new(AnswerConfig::default()).unwrap();
    let before = offline.clone();
    let refused = offline.apply(&request, &net.issuer.public());
    assert_eq!((refused, &offline), (Err(Refused::NetworkOnly), &before));
}

/// AT-T79-2: pool, deposit and commit proofs count only in this network's prefixed context, from
/// another network or offline never; the nyms are the offline ones, a reveal opens the commit.
#[test]
fn at_t79_2_proofs_bind_the_network() {
    let ctx = deposit_context(question("q").content_id(), 0);
    for offline in [POOL_CONTEXT, &ctx[..]] {
        let ours = network_context(&NET, offline);
        assert!(ours.starts_with(NETWORK_FAMILY) && ours.ends_with(offline));
        assert_ne!(ours, network_context(&OTHER, offline));
        assert_ne!(ours, offline);
    }
    let mut net = Net::new(small());
    let people: Vec<Person> = (100..103).map(|s| person(&net.issuer, s)).collect();
    let author = person(&net.issuer, 1);
    let (i, q) = (&net.issuer, question("q"));
    let wrong = [
        join(i, &people[0], None),
        join(i, &people[0], Some(&OTHER)),
        deposit(i, &author, &q, 0, None),
        deposit(i, &author, &q, 0, Some(&OTHER)),
    ];
    let mut right: Vec<_> = people.iter().map(|p| join(i, p, Some(&NET))).collect();
    right.push(deposit(i, &author, &q, 0, Some(&NET)));
    let bad: Vec<_> = wrong.iter().map(|e| net.log(0, e)).collect();
    let good: Vec<_> = right.iter().map(|e| net.log(1, e)).collect();
    let report = net.cut();
    for id in bad {
        assert_eq!(
            outcome(&report, id),
            path(Refused::Unproven(Unproven::BadProof))
        );
    }
    for (id, p) in good.iter().zip(&people) {
        assert_eq!(outcome(&report, *id), Ok(Done(Applied::Joined(p.judge))));
    }
    let deposited = Applied::Deposited {
        question: 0,
        author: author.propose,
    };
    assert_eq!(outcome(&report, good[3]), Ok(Done(deposited)));
    net.close_epoch();
    let p = people
        .iter()
        .find(|p| p.judge == group(&net, 0)[0])
        .unwrap();
    let (a, nonce) = (Answer::Alternative(0), [7; 32]);
    let commits = [None, Some(&OTHER), Some(&NET)].map(|s| commit(&net.issuer, p, 0, a, nonce, s));
    let ids: Vec<_> = commits.iter().map(|e| net.log(2, e)).collect();
    let report = net.cut();
    let bad = path(Refused::Unproven(Unproven::BadProof));
    let got: Vec<_> = ids.iter().map(|id| outcome(&report, *id)).collect();
    assert_eq!(
        got,
        [bad.clone(), bad, Ok(Done(Applied::Committed(p.judge)))]
    );
    net.request(0..T, &AnswerEvent::CloseCommits { question: 0 });
    net.cut();
    let opened = net.log(0, &reveal(p, 0, a, nonce));
    assert_eq!(outcome(&net.cut(), opened), Ok(Done(Applied::Revealed)));
}

/// AT-T79-3: the t-th distinct member's request closes commits, then the round; Pending, unknown,
/// repeated and late requests and late events are refused, uncounted; an excluded member counts.
#[test]
fn at_t79_3_closures_take_t_distinct_members() {
    let mut net = Net::new(small());
    let people: Vec<Person> = (100..103).map(|s| person(&net.issuer, s)).collect();
    let author = person(&net.issuer, 1);
    let mut events: Vec<_> = (people.iter())
        .map(|p| join(&net.issuer, p, Some(&NET)))
        .collect();
    events.push(deposit(&net.issuer, &author, &question("q"), 0, Some(&NET)));
    for e in &events {
        net.log(0, e);
    }
    let close_commits = AnswerEvent::CloseCommits { question: 0 };
    let close = AnswerEvent::Close { question: 0 };
    let early = net.log(0, &close_commits);
    let unknown = net.log(1, &AnswerEvent::Close { question: 5 });
    let report = net.cut();
    assert_eq!(outcome(&report, early), path(Refused::WrongRound));
    assert_eq!(outcome(&report, unknown), path(Refused::UnknownQuestion));
    net.withhold = BTreeSet::from([3]);
    net.close_epoch();
    net.withhold.clear();
    let answer = Answer::Alternative(0);
    let commits: Vec<_> = (people.iter().enumerate())
        .map(|(i, p)| commit(&net.issuer, p, 0, answer, [i as u8; 32], Some(&NET)))
        .collect();
    for c in &commits {
        net.log(1, c);
    }
    let first = net.log(3, &close_commits);
    let report = net.cut();
    assert!(net
        .ledger
        .excluded(&net.cuts.last().unwrap().0)
        .contains(&3));
    assert_eq!(outcome(&report, first), Ok(Requested { count: 1 }));
    let again = net.log(3, &close_commits);
    let second = net.log(0, &close_commits);
    let report = net.cut();
    assert_eq!(outcome(&report, again), Err(Refusal::Repeated));
    assert_eq!(outcome(&report, second), Ok(Requested { count: 2 }));
    assert!(matches!(group_round(&net), Round::Committing { .. }));
    let third = net.log(1, &close_commits);
    assert_eq!(outcome(&net.cut(), third), Ok(Done(Applied::CommitsClosed)));
    let fourth = net.log(2, &close_commits);
    let late_commit = net.log(2, &commits[0]);
    let report = net.cut();
    assert_eq!(outcome(&report, fourth), path(Refused::WrongRound));
    assert_eq!(outcome(&report, late_commit), path(Refused::WrongRound));
    let reveals: Vec<_> = (people.iter().enumerate())
        .map(|(i, p)| reveal(p, 0, answer, [i as u8; 32]))
        .collect();
    for r in &reveals {
        net.log(2, r);
    }
    net.cut();
    for (m, count) in [(2, 1), (3, 2)] {
        let asked = net.log(m, &close);
        assert_eq!(outcome(&net.cut(), asked), Ok(Requested { count }));
    }
    let repeated = net.log(2, &close);
    assert_eq!(outcome(&net.cut(), repeated), Err(Refusal::Repeated));
    let closing = net.log(0, &close);
    let report = net.cut();
    let Ok(Done(Applied::Closed(closure))) = outcome(&report, closing) else {
        panic!("the third request closes the round")
    };
    assert_eq!(
        (closure.verdict, closure.tally.usable()),
        (Verdict::Accepted, 3)
    );
    let late_reveal = net.log(1, &reveals[0]);
    let late_close = net.log(1, &close);
    let report = net.cut();
    assert_eq!(outcome(&report, late_reveal), path(Refused::WrongRound));
    assert_eq!(outcome(&report, late_close), path(Refused::WrongRound));
}

fn group_round(net: &Net) -> Round {
    net.ledger.state().entry(0).unwrap().round.clone()
}

/// AT-T79-4: an epoch's closure requests count only in its own cuts, once per member, past the
/// t-th too; the cut after the t-th closes it.
#[test]
fn at_t79_4_epoch_closure_requests() {
    let mut net = Net::new(AnswerConfig::default());
    let ask = |epoch| AnswerEvent::RequestCloseEpoch { epoch };
    let (a, b) = (net.log(0, &ask(0)), net.log(1, &ask(1)));
    let report = net.cut();
    assert_eq!(outcome(&report, a), Ok(Requested { count: 1 }));
    assert_eq!(outcome(&report, b), Err(Refusal::WrongEpoch));
    assert!(!net.ledger.next().unwrap().closes);
    let (a, b) = (net.log(0, &ask(0)), net.log(1, &ask(0)));
    let report = net.cut();
    assert_eq!(outcome(&report, a), Err(Refusal::Repeated));
    assert_eq!(outcome(&report, b), Ok(Requested { count: 2 }));
    assert!(!net.ledger.next().unwrap().closes);
    let c = net.log(3, &ask(0));
    assert_eq!(outcome(&net.cut(), c), Ok(Requested { count: 3 }));
    let next = Next {
        number: 3,
        epoch: 0,
        closes: true,
    };
    assert_eq!(net.ledger.next(), Some(next));
    let d = net.log(2, &ask(0));
    let report = net.cut();
    assert_eq!(
        (report.sealed, outcome(&report, d)),
        (Some(0), Ok(Requested { count: 4 }))
    );
    let next = Next {
        number: 4,
        epoch: 1,
        closes: false,
    };
    assert_eq!(net.ledger.next(), Some(next));
    let (late, early) = (net.log(2, &ask(0)), net.log(1, &ask(2)));
    let report = net.cut();
    assert_eq!(outcome(&report, late), Err(Refusal::WrongEpoch));
    assert_eq!(outcome(&report, early), Err(Refusal::WrongEpoch));
    let state = net.ledger.state();
    assert_eq!((state.epoch(), state.closings()), (1, 1));
    assert!(!net.ledger.next().unwrap().closes);
}

/// AT-T79-5: the members' checks — cut 0 in epoch 0, one epoch more only after a closing cut,
/// closing exactly at t requests, no gap; a signed cut failing them is not applied.
#[test]
fn at_t79_5_the_members_checks() {
    let c = |number, epoch, closes| Cut {
        number,
        epoch,
        closes,
        marks: Vec::new(),
    };
    let first = Next::after(None, 0, T).unwrap();
    let start = Next {
        number: 0,
        epoch: 0,
        closes: false,
    };
    assert_eq!(first, start);
    assert_eq!(first.check(&c(0, 0, false)), Ok(()));
    assert_eq!(first.check(&c(0, 1, false)), Err(Epoch { expected: 0 }));
    assert_eq!(first.check(&c(0, 0, true)), Err(Closes { expected: false }));
    assert_eq!(first.check(&c(1, 0, false)), Err(NotNext { expected: 0 }));
    let open = Next::after(Some(&c(4, 2, false)), T - 1, T).unwrap();
    assert_eq!((open.number, open.epoch, open.closes), (5, 2, false));
    assert_eq!(open.check(&c(5, 3, false)), Err(Epoch { expected: 2 }));
    let asked = Next::after(Some(&c(4, 2, false)), T, T).unwrap();
    assert_eq!(asked.check(&c(5, 2, false)), Err(Closes { expected: true }));
    assert_eq!(asked.check(&c(5, 2, true)), Ok(()));
    let closed = Next::after(Some(&c(4, 2, true)), 0, T).unwrap();
    assert_eq!((closed.number, closed.epoch, closed.closes), (5, 3, false));
    for epoch in [2, 4] {
        assert_eq!(
            closed.check(&c(5, epoch, false)),
            Err(Epoch { expected: 3 })
        );
    }
    assert_eq!(Next::after(Some(&c(u64::MAX, 0, false)), 0, T), None);
    assert_eq!(Next::after(Some(&c(0, u64::MAX, true)), 0, T), None);
    let last = Next::after(Some(&c(0, u64::MAX, false)), T, T).unwrap();
    assert_eq!(
        last.check(&c(1, u64::MAX, true)),
        Err(CheckFailure::Overflow)
    );

    let mut net = Net::new(AnswerConfig::default());
    let (fresh, digest) = (net.ledger.state().clone(), net.ledger.digest());
    let refused = |net: &mut Net, cut: Cut, failure: CheckFailure| {
        let (last, digest) = (net.ledger.last().cloned(), net.ledger.digest());
        assert_eq!(net.ledger.check(&cut), Err(failure));
        assert_eq!(net.apply(cut), Err(AnswerLedgerError::Checks(failure)));
        assert_eq!(
            (net.ledger.last().cloned(), net.ledger.digest()),
            (last, digest)
        );
    };
    let proposal =
        |net: &Net, epoch, closes| Cut::next(&net.replica, net.ledger.last(), epoch, closes);
    for (epoch, closes, failure) in [
        (1, false, Epoch { expected: 0 }),
        (0, true, Closes { expected: false }),
    ] {
        let cut = proposal(&net, epoch, closes);
        refused(&mut net, cut, failure);
    }
    assert_eq!((net.ledger.state(), net.ledger.digest()), (&fresh, digest));
    net.request(0..T, &AnswerEvent::RequestCloseEpoch { epoch: 0 });
    net.cut();
    let open = proposal(&net, 0, false);
    refused(&mut net, open, Closes { expected: true });
    let gap = Cut {
        number: 2,
        ..proposal(&net, 0, true)
    };
    refused(&mut net, gap, NotNext { expected: 1 });
    assert_eq!(net.cut().sealed, Some(0));
    for epoch in [0, 2] {
        let cut = proposal(&net, epoch, false);
        refused(&mut net, cut, Epoch { expected: 1 });
    }
    assert_eq!(net.ledger.check(&proposal(&net, 1, false)), Ok(()));
    net.cut();
}

/// AT-T79-6: the closing cut seals the pool; the next cut's round value draws its questions in
/// number order; a later epoch's questions wait for their own value; a late deposit is refused.
#[test]
fn at_t79_6_the_seal_the_value_and_the_draws() {
    let mut net = Net::new(AnswerConfig::default());
    let people: Vec<Person> = (100..114).map(|s| person(&net.issuer, s)).collect();
    let author = person(&net.issuer, 1);
    let (i, a) = (&net.issuer, &author);
    let mut events: Vec<_> = people[..12]
        .iter()
        .map(|p| join(i, p, Some(&NET)))
        .collect();
    events.push(deposit(i, a, &question("q0"), 0, Some(&NET)));
    events.push(deposit(i, a, &question("q1"), 0, Some(&NET)));
    let thirteenth = join(i, &people[12], Some(&NET));
    let after = [
        join(i, &people[13], Some(&NET)),
        deposit(i, a, &question("q2"), 1, Some(&NET)),
        deposit(i, a, &question("q3"), 0, Some(&NET)),
        commit(
            i,
            &people[0],
            0,
            Answer::Alternative(0),
            [0; 32],
            Some(&NET),
        ),
    ];
    for e in &events {
        net.log(0, e);
    }
    net.request(0..T, &AnswerEvent::RequestCloseEpoch { epoch: 0 });
    net.cut();
    net.log(1, &thirteenth);
    let closing = net.cut();
    assert_eq!(
        (closing.sealed, closing.valued.len(), closing.drawn.len()),
        (Some(0), 0, 0)
    );
    assert_eq!(
        (net.ledger.state().epoch(), net.ledger.state().closings()),
        (1, 1)
    );
    let ids: Vec<_> = after.iter().map(|e| net.log(2, e)).collect();
    let report = net.cut();
    let later = Applied::Deposited {
        question: 2,
        author: author.propose,
    };
    assert_eq!(outcome(&report, ids[1]), Ok(Done(later)));
    assert_eq!(outcome(&report, ids[2]), path(Refused::WrongEpoch));
    assert_eq!(outcome(&report, ids[3]), path(Refused::WrongRound));
    let (v0, pool) = (beacon(&net, 0), net.ledger.state().pool().to_vec());
    assert_eq!(pool.len(), 14);
    let drawn = |v: &Beacon, q: u64, sealed: usize, epoch: u64| Applied::Drawn {
        group: lottery::admit(&pool[..sealed], 12, v.seed(ANSWER_DRAW, q), epoch),
    };
    assert_eq!(report.valued, [0]);
    assert_eq!(
        report.drawn,
        [(0, drawn(&v0, 0, 13, 0)), (1, drawn(&v0, 1, 13, 0))]
    );
    assert_eq!(net.ledger.state().entry(2).unwrap().round, Round::Pending);
    let last = net.close_epoch();
    let v1 = beacon(&net, 1);
    assert_eq!(last.valued, [1]);
    assert_eq!(last.drawn, [(2, drawn(&v1, 2, 14, 1))]);
    assert_ne!(v0.seed(ANSWER_DRAW, 2), v1.seed(ANSWER_DRAW, 2));
}

/// AT-T79-7: sealed epochs without a value keep their pools and pending questions, slots spent,
/// until the next round with one, which draws each by number (P4).
#[test]
fn at_t79_7_a_missing_value_waits() {
    let mut net = Net::new(small());
    let people: Vec<Person> = (100..104).map(|s| person(&net.issuer, s)).collect();
    let (a, b) = (person(&net.issuer, 1), person(&net.issuer, 2));
    let (i, q0, q1) = (&net.issuer, question("q0"), question("q1"));
    let mut first: Vec<_> = people[..3].iter().map(|p| join(i, p, Some(&NET))).collect();
    first.push(deposit(i, &a, &q0, 0, Some(&NET)));
    let second = [
        join(i, &people[3], Some(&NET)),
        deposit(i, &b, &q1, 1, Some(&NET)),
    ];
    let early = commit(
        i,
        &people[0],
        0,
        Answer::Alternative(0),
        [0; 32],
        Some(&NET),
    );
    for e in &first {
        net.log(0, e);
    }
    net.withhold = BTreeSet::from([2, 3]);
    let report = net.close_epoch();
    assert_eq!(net.ledger.beacon(0).unwrap().value(), None);
    assert!(report.valued.is_empty() && report.drawn.is_empty());
    for e in &second {
        net.log(1, e);
    }
    let waiting = net.log(1, &early);
    assert_eq!(outcome(&net.cut(), waiting), path(Refused::WrongRound));
    let report = net.close_epoch();
    assert!(report.valued.is_empty() && report.drawn.is_empty());
    for q in 0..2 {
        assert_eq!(net.ledger.state().entry(q).unwrap().round, Round::Pending);
    }
    net.withhold.clear();
    let report = net.close_epoch();
    let (v2, pool) = (beacon(&net, 2), net.ledger.state().pool().to_vec());
    let drawn = |q: u64, sealed: usize, epoch: u64| Applied::Drawn {
        group: lottery::admit(&pool[..sealed], 3, v2.seed(ANSWER_DRAW, q), epoch),
    };
    assert_eq!(report.valued, [0, 1, 2]);
    assert_eq!(report.drawn, [(0, drawn(0, 3, 0)), (1, drawn(1, 4, 1))]);
    let slots = Slots { quota: 2, used: 1 };
    assert_eq!(net.ledger.state().slots(&a.propose, 0), Some(slots));
    assert_eq!(net.ledger.state().slots(&b.propose, 1), Some(slots));
}

/// AT-T79-8: fewer than 12 in the pool sealed with a question's epoch leave it unavailable for
/// good: a later reviewer recovers nothing, its slot stays spent, the same deposit a duplicate.
#[test]
fn at_t79_8_fewer_than_twelve_is_final() {
    let mut net = Net::new(AnswerConfig::default());
    let people: Vec<Person> = (100..112).map(|s| person(&net.issuer, s)).collect();
    let author = person(&net.issuer, 1);
    let (i, q0, q1) = (&net.issuer, question("q0"), question("q1"));
    let mut first: Vec<_> = people[..11]
        .iter()
        .map(|p| join(i, p, Some(&NET)))
        .collect();
    first.push(deposit(i, &author, &q0, 0, Some(&NET)));
    let second = [
        join(i, &people[11], Some(&NET)),
        deposit(i, &author, &q0, 1, Some(&NET)),
        deposit(i, &author, &q1, 1, Some(&NET)),
    ];
    for e in &first {
        net.log(0, e);
    }
    let report = net.close_epoch();
    let unavailable = Applied::Unavailable {
        available: 11,
        required: 12,
    };
    assert_eq!(report.drawn, [(0, unavailable)]);
    let ids: Vec<_> = second.iter().map(|e| net.log(0, e)).collect();
    let report = net.cut();
    assert_eq!(
        outcome(&report, ids[1]),
        path(Refused::Duplicate { question: 0 })
    );
    let report = net.close_epoch();
    assert!(matches!(report.drawn[..], [(1, Applied::Drawn { .. })]));
    let state = net.ledger.state();
    let round = &state.entry(0).unwrap().round;
    assert_eq!(*round, Round::Unavailable { available: 11 });
    assert_eq!(state.author(&author.propose), None);
    assert_eq!(
        state.slots(&author.propose, 0),
        Some(Slots { quota: 2, used: 1 })
    );
}

/// AT-T79-9: a person's event logged twice applies once, the second refused with its reason; one
/// signed history gives one state and digest on restart and under reordered, doubled delivery.
#[test]
fn at_t79_9_duplicates_restart_and_delivery() {
    let mut net = Net::new(small());
    let people: Vec<Person> = (100..104).map(|s| person(&net.issuer, s)).collect();
    let author = person(&net.issuer, 1);
    let joins: Vec<_> = (people.iter())
        .map(|p| join(&net.issuer, p, Some(&NET)))
        .collect();
    let deposited = deposit(&net.issuer, &author, &question("q"), 0, Some(&NET));
    for e in joins.iter().chain([&deposited]) {
        net.log(0, e);
    }
    net.request(0..T, &AnswerEvent::RequestCloseEpoch { epoch: 0 });
    net.cut();
    let twice = [net.log(1, &joins[0]), net.log(2, &deposited)];
    let report = net.cut();
    assert_eq!(outcome(&report, twice[0]), path(Refused::AlreadyInPool));
    assert_eq!(
        outcome(&report, twice[1]),
        path(Refused::Duplicate { question: 0 })
    );
    net.cut();
    let drawn: Vec<&Person> = (group(&net, 0).iter())
        .map(|nym| people.iter().find(|p| p.judge == *nym).unwrap())
        .collect();
    let answer = Answer::Alternative(0);
    let commits: Vec<_> = (drawn.iter().enumerate())
        .map(|(i, p)| commit(&net.issuer, p, 0, answer, [i as u8; 32], Some(&NET)))
        .collect();
    for c in &commits {
        net.log(1, c);
    }
    net.cut();
    let again = net.log(3, &commits[0]);
    assert_eq!(outcome(&net.cut(), again), path(Refused::AlreadyCommitted));
    net.request(0..T, &AnswerEvent::CloseCommits { question: 0 });
    net.cut();
    let reveals: Vec<_> = (drawn.iter().enumerate())
        .map(|(i, p)| reveal(p, 0, answer, [i as u8; 32]))
        .collect();
    for r in &reveals {
        net.log(2, r);
    }
    net.cut();
    let again = net.log(0, &reveals[0]);
    assert_eq!(outcome(&net.cut(), again), path(Refused::AlreadyRevealed));
    net.request(0..T, &AnswerEvent::Close { question: 0 });
    net.cut();
    net.close_epoch();
    assert!(net.ledger.state().closure(0).is_some());

    let fresh = || AnswerLedger::new(tiers(), net.issuer.public(), small()).unwrap();
    let mut restarted = fresh();
    for (cut, sigs) in &net.cuts {
        restarted.apply(&net.replica, cut, sigs).unwrap();
    }
    assert_eq!(restarted.state(), net.ledger.state());
    assert_eq!(restarted.digest(), net.ledger.digest());
    let mut entries = net.entries();
    entries.reverse();
    let (mut replica, mut late) = (Replica::new(tiers().writers().clone()), fresh());
    let (mut next, mut waited) = (0, 0);
    for (entry, object) in entries {
        for _ in 0..2 {
            replica.insert(entry.clone(), object.clone()).unwrap();
        }
        while let Some((cut, sigs)) = net.cuts.get(next) {
            match late.apply(&replica, cut, sigs) {
                Ok(_) => next += 1,
                Err(AnswerLedgerError::Cut(CutError::Missing { .. })) => {
                    waited += 1;
                    break;
                }
                Err(e) => panic!("{e:?}"),
            }
        }
    }
    assert_eq!(next, net.cuts.len());
    assert!(waited > 0);
    assert_eq!(late.state(), net.ledger.state());
    assert_eq!(late.digest(), net.ledger.digest());
}

/// AT-T79-10: a question encoding to MAX_QUESTION bytes is deposited; one byte more is an invalid
/// request that spends no slot.
#[test]
fn at_t79_10_the_question_size_bound() {
    let mut at = question("q");
    at.context.clear();
    at.context = vec![b'x'; MAX_QUESTION - encoded(&at)];
    assert_eq!(encoded(&at), MAX_QUESTION);
    let mut over = at.clone();
    over.context.push(b'x');
    assert!(fits(&at) && !fits(&over) && fits(&question("q")));
    let mut net = Net::new(AnswerConfig::default());
    let (a, b) = (person(&net.issuer, 1), person(&net.issuer, 2));
    let events = [
        deposit(&net.issuer, &a, &at, 0, Some(&NET)),
        deposit(&net.issuer, &b, &over, 0, Some(&NET)),
    ];
    let ids: Vec<_> = events.iter().map(|e| net.log(0, e)).collect();
    let report = net.cut();
    let first = Applied::Deposited {
        question: 0,
        author: a.propose,
    };
    assert_eq!(outcome(&report, ids[0]), Ok(Done(first)));
    assert_eq!(outcome(&report, ids[1]), Err(Refusal::TooLarge));
    assert_eq!(net.ledger.state().slots(&b.propose, 0), None);
    let small = deposit(&net.issuer, &b, &question("small"), 0, Some(&NET));
    let id = net.log(0, &small);
    let second = Applied::Deposited {
        question: 1,
        author: b.propose,
    };
    assert_eq!(outcome(&net.cut(), id), Ok(Done(second)));
    let slots = Some(Slots { quota: 2, used: 1 });
    assert_eq!(net.ledger.state().slots(&b.propose, 0), slots);
}

/// AT-T79-11: the digest binds the network and the derived state: a refused entry or an empty
/// cut leaves it, an accepted event, a counted request and a seal change it.
#[test]
fn at_t79_11_the_state_digest() {
    let issuer = Issuer::new([1u8; 32]);
    let other = AnswerLedger::new(tiers_on(OTHER), issuer.public(), AnswerConfig::default());
    let mut net = Net::new(AnswerConfig::default());
    let empty = net.ledger.digest();
    assert_ne!(empty, other.unwrap().digest());
    net.cut();
    assert_eq!(net.ledger.digest(), empty);
    let joined = join(&net.issuer, &person(&net.issuer, 100), Some(&NET));
    net.log(0, &joined);
    net.cut();
    let once = net.ledger.digest();
    assert_ne!(once, empty);
    net.log(1, &joined);
    net.cut();
    assert_eq!(net.ledger.digest(), once);
    net.request(0..T, &AnswerEvent::RequestCloseEpoch { epoch: 0 });
    net.cut();
    let asked = net.ledger.digest();
    assert_ne!(asked, once);
    assert_eq!(net.cut().sealed, Some(0));
    assert_ne!(net.ledger.digest(), asked);
}
