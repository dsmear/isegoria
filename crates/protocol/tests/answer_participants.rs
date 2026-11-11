//! Participants on a network (`docs/05` §Built for participants, `docs/01` D60, T79-P7): gateway
//! admission, read requests, the replies a member computes, their encodings.

mod answer_net;

use answer_net::*;
use identity::credential::Issuer;
use identity::nullifier::prove;
use identity::nym::Role;
use network::cid::{cid, Cid};
use network::codec::Writer;
use network::tier::Tier;
use protocol::admission::Unproven;
use protocol::answer::{Answer, Malformed, Question};
use protocol::answer_author::Observation;
use protocol::answer_events::AnswerEvent;
use protocol::answer_gateway::{admit, Inadmissible};
use protocol::answer_ledger::{Refusal, MAX_QUESTION};
use protocol::answer_path::{network_context, Refused, Round, POOL_CONTEXT};
use protocol::answer_read::{
    answer, read_context, reply, Assigned, Authored, EventStatus, Logged, Phase, Read, ReadRequest,
    Reply, ReviewPhase,
};
use protocol::answer_rule::Verdict;
use protocol::answer_seal::{ResponseKey, OVERHEAD};
use protocol::author_time::AuthorPoint;

fn encoded(q: &Question) -> usize {
    let mut w = Writer::new();
    q.write(&mut w);
    w.finish().len()
}

fn key(seed: u8) -> [u8; 32] {
    ResponseKey::derive(&[seed; 32]).public()
}

fn request(net: &Net, read: Read, who: &Person, role: Role) -> ReadRequest {
    let pk = net.issuer.public();
    ReadRequest::new(read, key(7), &who.cred, role, &pk, &NET)
}

fn status(net: &Net, who: &Person, role: Role, event: &AnswerEvent) -> Vec<Logged> {
    let read = Read::Status {
        event: cid(&event.encode()),
    };
    match reply(&net.ledger, &net.replica, &request(net, read, who, role)) {
        Some(Reply::Status(logged)) => logged,
        other => panic!("{other:?}"),
    }
}

fn reviewer(net: &Net, who: &Person, from: u64) -> Option<Assigned> {
    let req = request(net, Read::Reviewer { from }, who, Role::Judge);
    match reply(&net.ledger, &net.replica, &req) {
        Some(Reply::Reviewer(assigned)) => assigned,
        other => panic!("{other:?}"),
    }
}

fn author(net: &Net, who: &Person) -> Authored {
    match reply(
        &net.ledger,
        &net.replica,
        &request(net, Read::Author, who, Role::Propose),
    ) {
        Some(Reply::Author(authored)) => authored,
        other => panic!("{other:?}"),
    }
}

fn logged(id: network::replica::EntryId, status: EventStatus) -> Logged {
    Logged {
        writer: id.writer,
        seq: id.seq,
        status,
    }
}

fn sorted(mut l: Vec<Logged>) -> Vec<Logged> {
    l.sort_by_key(|l| (l.writer, l.seq));
    l
}

/// AT-T79-23: a gateway admits persons' joins, deposits and commits proved in this network's
/// context, and reveals; refuses other proofs, questions past the bound or malformed, members'
/// requests, derived kinds and other bytes; without state.
#[test]
fn at_t79_23_gateway_admission() {
    let issuer = Issuer::new([1u8; 32]);
    let pk = issuer.public();
    let (r, a) = (person(&issuer, 100), person(&issuer, 101));
    let q = question("q");
    let nonce = [3; 32];
    let admitted = [
        join(&issuer, &r, Some(&NET)),
        join(&issuer, &r, Some(&NET)),
        deposit(&issuer, &a, &q, 0, Some(&NET)),
        deposit(&issuer, &a, &q, 7, Some(&NET)),
        commit(&issuer, &r, 0, Answer::Alternative(0), nonce, Some(&NET)),
        reveal(&r, 0, Answer::Alternative(0), nonce),
        reveal(&r, 99, Answer::NoneCorrect, [0; 32]),
    ];
    for e in &admitted {
        assert_eq!(admit(&e.encode(), &NET, &pk), Ok(()));
    }
    let unproven = Err(Inadmissible::Unproven(Unproven::BadProof));
    for scope in [None, Some(&OTHER)] {
        let others = [
            join(&issuer, &r, scope),
            deposit(&issuer, &a, &q, 0, scope),
            commit(&issuer, &r, 0, Answer::Alternative(0), nonce, scope),
        ];
        for e in &others {
            assert_eq!(admit(&e.encode(), &NET, &pk), unproven);
        }
    }
    let stranger = Issuer::new([2u8; 32]).public();
    let joined = join(&issuer, &r, Some(&NET)).encode();
    assert_eq!(admit(&joined, &NET, &stranger), unproven);
    let ctx = network_context(&NET, POOL_CONTEXT);
    let wrong = AnswerEvent::JoinPool {
        proof: prove(&a.cred, &pk, Role::Propose, &ctx),
    };
    let wrong_role = Err(Inadmissible::Unproven(Unproven::WrongRole));
    assert_eq!(admit(&wrong.encode(), &NET, &pk), wrong_role);

    let mut bad = question("bad");
    bad.key = 9;
    let malformed = Err(Inadmissible::Malformed(Malformed::KeyOutOfRange));
    let bad = deposit(&issuer, &a, &bad, 0, Some(&NET)).encode();
    assert_eq!(admit(&bad, &NET, &pk), malformed);
    let mut edge = question("edge");
    edge.context.clear();
    edge.context = vec![b'x'; MAX_QUESTION - encoded(&edge)];
    let mut over = edge.clone();
    over.context.push(b'x');
    let at = deposit(&issuer, &a, &edge, 0, Some(&NET)).encode();
    assert_eq!(admit(&at, &NET, &pk), Ok(()));
    let over = deposit(&issuer, &a, &over, 0, Some(&NET)).encode();
    assert_eq!(admit(&over, &NET, &pk), Err(Inadmissible::TooLarge));

    let members = [
        AnswerEvent::CloseCommits { question: 0 },
        AnswerEvent::Close { question: 0 },
        AnswerEvent::RequestCloseEpoch { epoch: 0 },
    ];
    for e in &members {
        assert_eq!(admit(&e.encode(), &NET, &pk), Err(Inadmissible::MemberOnly));
    }
    let derived = [
        AnswerEvent::Draw { question: 0 },
        AnswerEvent::CloseEpoch {
            epoch: 0,
            beacon: [1; 32],
        },
    ];
    for e in &derived {
        assert_eq!(admit(&e.encode(), &NET, &pk), Err(Inadmissible::Derived));
    }
    let mut extended = joined.clone();
    extended.push(0);
    for bytes in [
        b"no event".to_vec(),
        joined[..joined.len() - 1].to_vec(),
        extended,
    ] {
        assert_eq!(admit(&bytes, &NET, &pk), Err(Inadmissible::NotAnEvent));
    }
}

/// AT-T79-24: a read request has one encoding; its proof answers only in its network's context
/// with its kind and response key, under its role; `answer` seals the reply to that key.
#[test]
fn at_t79_24_read_requests_and_their_proofs() {
    let net = Net::new(small());
    let (r, a) = (person(&net.issuer, 100), person(&net.issuer, 101));
    let pk = net.issuer.public();
    let event = Cid([4; 32]);
    let reads = [
        (Read::Status { event }, Role::Judge),
        (Read::Status { event }, Role::Propose),
        (Read::Reviewer { from: 3 }, Role::Judge),
        (Read::Author, Role::Propose),
    ];
    for (read, role) in reads {
        let req = ReadRequest::new(read, key(7), &r.cred, role, &pk, &NET);
        let bytes = req.encode();
        let back = ReadRequest::decode(&bytes).unwrap();
        assert_eq!((back.read, back.response_key), (read, key(7)));
        assert_eq!(back.encode(), bytes);
        for len in 0..bytes.len() {
            assert!(ReadRequest::decode(&bytes[..len]).is_none(), "{len}");
        }
        let mut extended = bytes.clone();
        extended.push(0);
        assert!(ReadRequest::decode(&extended).is_none());
        for (at, value) in [(0, 2), (1, 0), (1, 4)] {
            let mut altered = bytes.clone();
            altered[at] = value;
            assert!(ReadRequest::decode(&altered).is_none());
        }
        assert!(reply(&net.ledger, &net.replica, &req).is_some());
    }

    let read = Read::Reviewer { from: 0 };
    let made =
        |read, key, cred, role, network| ReadRequest::new(read, key, cred, role, &pk, network);
    let other = made(read, key(7), &r.cred, Role::Judge, &OTHER);
    assert!(reply(&net.ledger, &net.replica, &other).is_none());
    let mut swapped = made(read, key(8), &r.cred, Role::Judge, &NET);
    swapped.response_key = key(7);
    assert!(reply(&net.ledger, &net.replica, &swapped).is_none());
    let mut kind = made(Read::Status { event }, key(7), &r.cred, Role::Judge, &NET);
    kind.read = read;
    assert!(reply(&net.ledger, &net.replica, &kind).is_none());
    let mut cursor = made(read, key(7), &r.cred, Role::Judge, &NET);
    cursor.read = Read::Reviewer { from: 9 };
    assert!(reply(&net.ledger, &net.replica, &cursor).is_some());
    let stranger = Issuer::new([2u8; 32]);
    let foreign = person(&stranger, 100);
    let foreign = ReadRequest::new(
        read,
        key(7),
        &foreign.cred,
        Role::Judge,
        &stranger.public(),
        &NET,
    );
    assert!(reply(&net.ledger, &net.replica, &foreign).is_none());
    for (read, role) in [
        (read, Role::Propose),
        (Read::Author, Role::Judge),
        (Read::Status { event }, Role::Respond),
    ] {
        let req = made(read, key(7), &a.cred, role, &NET);
        assert!(reply(&net.ledger, &net.replica, &req).is_none());
    }
    let elsewhere = ReadRequest {
        read,
        response_key: key(7),
        proof: prove(
            &r.cred,
            &pk,
            Role::Judge,
            &read_context(&OTHER, read, &key(7)),
        ),
    };
    assert!(answer(&net.ledger, &net.replica, &elsewhere.encode()).is_none());

    let req = made(Read::Author, key(7), &a.cred, Role::Propose, &NET);
    let expected = reply(&net.ledger, &net.replica, &req).unwrap();
    let sealed = answer(&net.ledger, &net.replica, &req.encode()).unwrap();
    let opened = ResponseKey::derive(&[7; 32]).open(&sealed).unwrap();
    assert_eq!(Reply::decode(&opened), Some(expected));
    assert!(ResponseKey::derive(&[8; 32]).open(&sealed).is_none());
    assert!(answer(&net.ledger, &net.replica, b"no request").is_none());
}

/// AT-T79-25: an event's status lists each circle entry holding it — pending, applied (a deposit
/// with its number) or refused with the reason — only to a request proved under its nym.
#[test]
fn at_t79_25_an_events_status() {
    let mut net = Net::new(small());
    let (r, a, other) = (
        person(&net.issuer, 100),
        person(&net.issuer, 101),
        person(&net.issuer, 102),
    );
    let joined = join(&net.issuer, &r, Some(&NET));
    let first = net.log(0, &joined);
    net.publish(3, Tier::Public, joined.encode());
    assert_eq!(
        status(&net, &r, Role::Judge, &joined),
        vec![logged(first, EventStatus::Pending)]
    );
    net.cut();
    let second = net.log(1, &joined);
    let applied = logged(first, EventStatus::Applied { question: None });
    let expected = sorted(vec![applied, logged(second, EventStatus::Pending)]);
    assert_eq!(sorted(status(&net, &r, Role::Judge, &joined)), expected);
    net.cut();
    let expected = sorted(vec![
        applied,
        logged(
            second,
            EventStatus::Refused(Refusal::Path(Refused::AlreadyInPool)),
        ),
    ]);
    assert_eq!(sorted(status(&net, &r, Role::Judge, &joined)), expected);
    assert_eq!(status(&net, &other, Role::Judge, &joined), Vec::new());
    assert_eq!(status(&net, &r, Role::Propose, &joined), Vec::new());
    assert_eq!(
        status(&net, &r, Role::Judge, &join(&net.issuer, &r, Some(&NET))),
        Vec::new()
    );

    let deposited = deposit(&net.issuer, &a, &question("q"), 0, Some(&NET));
    let late = deposit(&net.issuer, &a, &question("late"), 5, Some(&NET));
    let (once, late_id) = (net.log(0, &deposited), net.log(0, &late));
    net.cut();
    let again = net.log(2, &deposited);
    let early = reveal(&r, 0, Answer::Alternative(0), [3; 32]);
    let revealed = net.log(1, &early);
    net.cut();
    let expected = sorted(vec![
        logged(once, EventStatus::Applied { question: Some(0) }),
        logged(
            again,
            EventStatus::Refused(Refusal::Path(Refused::Duplicate { question: 0 })),
        ),
    ]);
    assert_eq!(
        sorted(status(&net, &a, Role::Propose, &deposited)),
        expected
    );
    let wrong_epoch = EventStatus::Refused(Refusal::Path(Refused::WrongEpoch));
    assert_eq!(
        status(&net, &a, Role::Propose, &late),
        vec![logged(late_id, wrong_epoch)]
    );
    assert_eq!(status(&net, &a, Role::Judge, &deposited), Vec::new());
    let wrong_round = EventStatus::Refused(Refusal::Path(Refused::WrongRound));
    assert_eq!(
        status(&net, &r, Role::Judge, &early),
        vec![logged(revealed, wrong_round)]
    );
    assert_eq!(status(&net, &other, Role::Judge, &early), Vec::new());
}

/// Four reviewers in the pool, two questions of one author drawn in epoch 0 under `small`.
fn drawn(key: u8) -> (Net, Vec<Person>, Person) {
    let mut net = Net::new(small());
    let pool: Vec<Person> = (100..104).map(|s| person(&net.issuer, s)).collect();
    let a = person(&net.issuer, 1);
    for p in &pool {
        net.log(0, &join(&net.issuer, p, Some(&NET)));
    }
    for tag in ["q0", "q1"] {
        let q = Question {
            key,
            ..question(tag)
        };
        net.log(1, &deposit(&net.issuer, &a, &q, 0, Some(&NET)));
    }
    net.close_epoch();
    (net, pool, a)
}

fn group(net: &Net, q: u64) -> Vec<identity::nym::Nym> {
    match &net.ledger.state().entry(q).unwrap().round {
        Round::Committing { group, .. } | Round::Revealing { group, .. } => group.clone(),
        other => panic!("{other:?}"),
    }
}

fn view(q: u64, committed: bool, revealed: bool, phase: ReviewPhase) -> Assigned {
    let question = question(["q0", "q1"][q as usize]);
    Assigned {
        question: q,
        context: question.context,
        stem: question.stem,
        alternatives: question.alternatives,
        committed,
        revealed,
        phase,
    }
}

/// AT-T79-26: a reviewer reads each question whose group holds it, one per reply from a number,
/// by number and view, whether its commit and reveal counted, that question's phase, after the
/// closure too; the reply is the same whatever the key and the others' commits and answers.
#[test]
fn at_t79_26_a_reviewers_questions() {
    let (open, reveals, closed) = (
        ReviewPhase::CommitsOpen,
        ReviewPhase::RevealsOpen,
        ReviewPhase::Closed,
    );
    let (mut net, pool, _) = drawn(0);
    let groups = [group(&net, 0), group(&net, 1)];
    let mut reads = Vec::new();
    for p in &pool {
        let held: Vec<u64> = (0..2)
            .filter(|&q| groups[q as usize].contains(&p.judge))
            .collect();
        let first = held.first().map(|&q| view(q, false, false, open));
        assert_eq!(reviewer(&net, p, 0), first);
        let next = held
            .iter()
            .find(|&&q| q >= 1)
            .map(|&q| view(q, false, false, open));
        assert_eq!(reviewer(&net, p, 1), next);
        assert_eq!(reviewer(&net, p, 2), None);
        let req = request(&net, Read::Reviewer { from: 0 }, p, Role::Judge);
        reads.push(reply(&net.ledger, &net.replica, &req).unwrap().encode());
    }
    let (keyed, _, _) = drawn(1);
    for (p, bytes) in pool.iter().zip(&reads) {
        let req = request(&keyed, Read::Reviewer { from: 0 }, p, Role::Judge);
        assert_eq!(
            &reply(&keyed.ledger, &keyed.replica, &req).unwrap().encode(),
            bytes
        );
    }

    let drawn0: Vec<&Person> = (groups[0].iter())
        .map(|nym| pool.iter().find(|p| p.judge == *nym).unwrap())
        .collect();
    let answer = Answer::Alternative(0);
    let (x, y, z) = (drawn0[0], drawn0[1], drawn0[2]);
    let before = reviewer(&net, x, 0);
    for (i, p) in [y, z].into_iter().enumerate() {
        net.log(
            2,
            &commit(&net.issuer, p, 0, answer, [i as u8; 32], Some(&NET)),
        );
    }
    net.cut();
    assert_eq!(reviewer(&net, x, 0), before);
    assert_eq!(reviewer(&net, y, 0), Some(view(0, true, false, open)));
    net.log(0, &commit(&net.issuer, x, 0, answer, [9; 32], Some(&NET)));
    net.cut();
    assert_eq!(reviewer(&net, x, 0), Some(view(0, true, false, open)));
    net.request(0..T, &AnswerEvent::CloseCommits { question: 0 });
    net.cut();
    assert_eq!(reviewer(&net, x, 0), Some(view(0, true, false, reveals)));
    net.log(1, &reveal(y, 0, answer, [0; 32]));
    net.log(1, &reveal(x, 0, answer, [9; 32]));
    net.cut();
    assert_eq!(reviewer(&net, x, 0), Some(view(0, true, true, reveals)));
    assert_eq!(reviewer(&net, z, 0), Some(view(0, true, false, reveals)));
    net.request(0..T, &AnswerEvent::Close { question: 0 });
    net.cut();
    assert!(net.ledger.state().closure(0).is_some());
    assert_eq!(reviewer(&net, x, 0), Some(view(0, true, true, closed)));
    assert_eq!(reviewer(&net, z, 0), Some(view(0, true, false, closed)));
    for p in pool.iter().filter(|p| groups[1].contains(&p.judge)) {
        assert_eq!(reviewer(&net, p, 1), Some(view(1, false, false, open)));
    }

    let mut most = question("most");
    most.context.clear();
    most.context = vec![b'x'; MAX_QUESTION - encoded(&most)];
    let mut view_len = Writer::new();
    view_len
        .u64(u64::MAX)
        .field(&most.context)
        .field(&most.stem);
    view_len.u64(most.alternatives.len() as u64);
    for alternative in &most.alternatives {
        view_len.field(alternative);
    }
    let biggest = Reply::Reviewer(Some(Assigned {
        question: u64::MAX,
        context: most.context,
        stem: most.stem,
        alternatives: most.alternatives,
        committed: true,
        revealed: true,
        phase: closed,
    }));
    assert!(biggest.encode().len() + OVERHEAD <= view_len.finish().len() + 1024);
}

/// AT-T79-27: an author reads only its own questions' rounds, verdicts and observations, its `C_a`
/// and quota, under its `Propose` nym; never a group, commit or answer.
#[test]
fn at_t79_27_an_authors_records() {
    let mut net = Net::new(small());
    let pool: Vec<Person> = (100..104).map(|s| person(&net.issuer, s)).collect();
    let (a, b) = (person(&net.issuer, 1), person(&net.issuer, 2));
    let fresh = author(&net, &a);
    let path = net.ledger.state();
    assert_eq!(fresh.questions, Vec::new());
    assert_eq!(
        (fresh.reputation, fresh.quota),
        (path.reputation(&a.propose), 2)
    );
    for p in &pool {
        net.log(0, &join(&net.issuer, p, Some(&NET)));
    }
    net.log(1, &deposit(&net.issuer, &a, &question("q0"), 0, Some(&NET)));
    net.log(1, &deposit(&net.issuer, &b, &question("b0"), 0, Some(&NET)));
    net.log(1, &deposit(&net.issuer, &a, &question("q1"), 0, Some(&NET)));
    net.cut();
    let pending = vec![(0, Phase::Pending), (2, Phase::Pending)];
    assert_eq!(author(&net, &a).questions, pending);
    assert_eq!(author(&net, &b).questions, vec![(1, Phase::Pending)]);
    net.close_epoch();
    let committing = vec![(0, Phase::Committing), (2, Phase::Committing)];
    assert_eq!(author(&net, &a).questions, committing);

    let drawn: Vec<&Person> = (group(&net, 0).iter())
        .map(|nym| pool.iter().find(|p| p.judge == *nym).unwrap())
        .collect();
    let answer = Answer::Alternative(0);
    for (i, p) in drawn.iter().enumerate() {
        net.log(
            2,
            &commit(&net.issuer, p, 0, answer, [i as u8; 32], Some(&NET)),
        );
    }
    net.cut();
    let before = author(&net, &a);
    net.request(0..T, &AnswerEvent::CloseCommits { question: 0 });
    net.cut();
    assert_eq!(before.questions[0], (0, Phase::Committing));
    assert_eq!(author(&net, &a).questions[0], (0, Phase::Revealing));
    for (i, p) in drawn.iter().enumerate() {
        net.log(0, &reveal(p, 0, answer, [i as u8; 32]));
    }
    net.cut();
    net.request(0..T, &AnswerEvent::Close { question: 0 });
    net.cut();
    let closure = net.ledger.state().closure(0).unwrap().clone();
    assert_eq!(closure.verdict, Verdict::Accepted);
    let closed = Phase::Closed {
        verdict: closure.verdict,
        observation: closure.observation,
    };
    let records = author(&net, &a);
    assert_eq!(records.questions, vec![(0, closed), (2, Phase::Committing)]);
    let path = net.ledger.state();
    assert_eq!(
        records.reputation.to_bits(),
        path.reputation(&a.propose).to_bits()
    );
    assert_eq!(records.quota, path.quota(&a.propose));
    assert_eq!(author(&net, &b).questions, vec![(1, Phase::Committing)]);
    let judge = request(&net, Read::Author, &a, Role::Judge);
    assert!(reply(&net.ledger, &net.replica, &judge).is_none());
}

fn replies() -> Vec<Reply> {
    let writer = [6; 32];
    let refused = Refusal::Path(Refused::Duplicate { question: 3 });
    let statuses = [
        EventStatus::Pending,
        EventStatus::Applied { question: None },
        EventStatus::Applied { question: Some(4) },
        EventStatus::Refused(refused),
        EventStatus::Refused(Refusal::Beacon(network::beacon::RoundError::OutOfPhase)),
    ];
    let logged = (statuses.iter().enumerate())
        .map(|(seq, &status)| Logged {
            writer,
            seq: seq as u64,
            status,
        })
        .collect();
    let observation = Observation {
        success: false,
        weight: 0.25,
        origin: AuthorPoint {
            epoch: 2,
            closings: 2,
        },
    };
    let phases = vec![
        (0, Phase::Pending),
        (1, Phase::Unavailable),
        (2, Phase::Committing),
        (3, Phase::Revealing),
        (
            4,
            Phase::Closed {
                verdict: Verdict::Divided,
                observation: Some(observation),
            },
        ),
        (
            5,
            Phase::Closed {
                verdict: Verdict::InsufficientParticipation,
                observation: None,
            },
        ),
    ];
    vec![
        Reply::Status(logged),
        Reply::Status(Vec::new()),
        Reply::Reviewer(None),
        Reply::Reviewer(Some(view(1, true, false, ReviewPhase::RevealsOpen))),
        Reply::Author(Authored {
            reputation: 0.4,
            quota: u32::MAX,
            questions: phases,
        }),
        Reply::Author(Authored {
            reputation: 0.0,
            quota: 2,
            questions: Vec::new(),
        }),
        Reply::Reviewer(Some(view(0, false, false, ReviewPhase::CommitsOpen))),
        Reply::Reviewer(Some(view(0, true, true, ReviewPhase::Closed))),
    ]
}

/// AT-T79-28: each reply kind has one encoding: re-encoded alike; truncated, extended,
/// re-versioned, of another kind, or with an out-of-range code, count, flag, phase, quota, refused.
#[test]
fn at_t79_28_reply_encodings() {
    for r in replies() {
        let bytes = r.encode();
        assert_eq!(Reply::decode(&bytes).as_ref(), Some(&r));
        for len in 0..bytes.len() {
            assert_eq!(Reply::decode(&bytes[..len]), None, "{r:?} at {len}");
        }
        let mut extended = bytes.clone();
        extended.push(0);
        assert_eq!(Reply::decode(&extended), None);
        for (at, value) in [(0, 2), (1, 0), (1, 4)] {
            let mut altered = bytes.clone();
            altered[at] = value;
            assert_eq!(Reply::decode(&altered), None);
        }
    }
    let assigned = replies()[3].encode();
    let (revealed, stage) = (assigned.len() - 2, assigned.len() - 1);
    assert_eq!((assigned[revealed], assigned[stage]), (0, 1));
    let mut flag = assigned.clone();
    flag[revealed] = 2;
    let mut review_phase = assigned.clone();
    review_phase[stage] = 3;
    let mut count = assigned.clone();
    let at = 3 + 8 + 8 + b"civic education, 2026".len() + 8 + 2;
    count[at..at + 8].copy_from_slice(&257u64.to_le_bytes());
    let mut option = assigned;
    option[2] = 2;
    let author = replies()[5].encode();
    let mut quota = author.clone();
    quota[10..18].copy_from_slice(&(u64::from(u32::MAX) + 1).to_le_bytes());
    let mut phase = replies()[4].encode();
    let first = 2 + 8 + 8 + 8 + 8;
    phase[first] = 5;
    let mut verdict = replies()[4].encode();
    let closed = first + 4 * 9;
    assert_eq!(verdict[closed], 4);
    verdict[closed + 1] = 4;
    let mut code = replies()[0].encode();
    code[2 + 8 + 32 + 8] = 3;
    for (at, phase) in [(6, 0), (3, 1), (7, 2)] {
        assert_eq!(*replies()[at].encode().last().unwrap(), phase);
    }
    let bytes = [
        flag,
        review_phase,
        count,
        option,
        quota,
        phase,
        verdict,
        code,
    ];
    for bytes in bytes {
        assert_eq!(Reply::decode(&bytes), None);
    }
}
