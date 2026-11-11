//! A person's read requests built by the client, their proofs, and their replies opened only
//! by their own response key (`docs/05` T79-P7, §Built for participants; invariant 4).

mod support;

use client::event;
use client::read::{self, render, Ask};
use client::{hex, Error};
use identity::nym::Role;
use network::cid::cid;
use protocol::admission::{admit, Unproven};
use protocol::answer::Answer;
use protocol::answer_author::Observation;
use protocol::answer_events::AnswerEvent;
use protocol::answer_ledger::Refusal;
use protocol::answer_path::Refused;
use protocol::answer_read::{
    read_context, Assigned, Authored, EventStatus, Logged, Phase, Read, ReadRequest, Reply,
    ReviewPhase,
};
use protocol::answer_rule::Verdict;
use protocol::answer_seal::seal;
use protocol::author_time::AuthorPoint;
use std::fs;
use support::*;

// Each read is proved under its role for its network, kind and fresh key only (AT-T79-48).
#[test]
fn at_t79_48_read_requests_and_their_proofs() {
    let dir = scratch("requests");
    let (_, net, _, held) = person(&dir);
    let join = event::join(&net, &held).unwrap();
    let deposit = event::deposit(
        &net,
        &held,
        0,
        protocol::answer::Question {
            context: b"c".to_vec(),
            stem: b"s".to_vec(),
            alternatives: vec![b"a".to_vec(), b"b".to_vec()],
            key: 0,
        },
    )
    .unwrap();
    let (commit, opened) = event::commit(&net, &held, 1, Answer::DontKnow).unwrap();
    let reveal = event::reveal(&net, &held, &opened).unwrap();
    let judge = event::nym(&net, &held, Role::Judge).unwrap();
    let propose = event::nym(&net, &held, Role::Propose).unwrap();
    let cases = [
        (
            Ask::Status(join.clone()),
            Read::Status { event: cid(&join) },
            Role::Judge,
        ),
        (
            Ask::Status(deposit.clone()),
            Read::Status {
                event: cid(&deposit),
            },
            Role::Propose,
        ),
        (
            Ask::Status(commit.clone()),
            Read::Status {
                event: cid(&commit),
            },
            Role::Judge,
        ),
        (
            Ask::Status(reveal.clone()),
            Read::Status {
                event: cid(&reveal),
            },
            Role::Judge,
        ),
        (
            Ask::Reviewer { from: 7 },
            Read::Reviewer { from: 7 },
            Role::Judge,
        ),
        (Ask::Author, Read::Author, Role::Propose),
    ];
    let mut keys = Vec::new();
    for (ask, read, role) in cases {
        assert_eq!(read::role(&ask), Some(role));
        let (bytes, opener) = read::request(&net, &held, &ask).unwrap();
        let request = ReadRequest::decode(&bytes).expect("one request");
        assert_eq!(request.encode(), bytes);
        assert_eq!(
            (request.read, request.response_key),
            (read, opener.response_key())
        );
        let context = read_context(&net.id, read, &request.response_key);
        let nym = admit(&request.proof, &net.issuer, role, &context).expect("proved");
        assert_eq!(nym, if role == Role::Judge { judge } else { propose });
        let other = if role == Role::Judge {
            Role::Propose
        } else {
            Role::Judge
        };
        assert_eq!(
            admit(&request.proof, &net.issuer, other, &context),
            Err(Unproven::WrongRole)
        );
        let elsewhere = read_context(&[0; 32], read, &request.response_key);
        assert!(admit(&request.proof, &net.issuer, role, &elsewhere).is_err());
        let rekeyed = read_context(&net.id, read, &[9; 32]);
        assert!(admit(&request.proof, &net.issuer, role, &rekeyed).is_err());
        let (again, _) = read::request(&net, &held, &ask).unwrap();
        let again = ReadRequest::decode(&again).unwrap();
        assert_ne!(again.response_key, request.response_key);
        keys.push(request.response_key);
        keys.push(again.response_key);
    }
    let distinct: std::collections::BTreeSet<_> = keys.iter().collect();
    assert_eq!(
        distinct.len(),
        keys.len(),
        "no response key shared between requests"
    );
    for bytes in [
        AnswerEvent::CloseCommits { question: 0 }.encode(),
        b"not an event".to_vec(),
    ] {
        let ask = Ask::Status(bytes);
        assert_eq!(read::role(&ask), None);
        assert!(matches!(
            read::request(&net, &held, &ask),
            Err(Error::Malformed("event"))
        ));
    }
}

fn replies() -> [Reply; 4] {
    [
        Reply::Status(vec![
            Logged {
                writer: [1; 32],
                seq: 4,
                status: EventStatus::Pending,
            },
            Logged {
                writer: [2; 32],
                seq: 5,
                status: EventStatus::Applied { question: Some(3) },
            },
            Logged {
                writer: [3; 32],
                seq: 6,
                status: EventStatus::Refused(Refusal::Path(Refused::Duplicate { question: 3 })),
            },
        ]),
        Reply::Reviewer(None),
        Reply::Reviewer(Some(Assigned {
            question: 2,
            context: b"ctx".to_vec(),
            stem: b"line\nbreak".to_vec(),
            alternatives: vec![b"yes".to_vec(), b"no".to_vec()],
            committed: true,
            revealed: false,
            phase: ReviewPhase::RevealsOpen,
        })),
        Reply::Author(Authored {
            reputation: 0.5,
            quota: 2,
            questions: vec![
                (0, Phase::Pending),
                (
                    1,
                    Phase::Closed {
                        verdict: Verdict::Divided,
                        observation: None,
                    },
                ),
                (
                    2,
                    Phase::Closed {
                        verdict: Verdict::Accepted,
                        observation: Some(Observation {
                            success: true,
                            weight: 1.0,
                            origin: AuthorPoint {
                                epoch: 3,
                                closings: 2,
                            },
                        }),
                    },
                ),
            ],
        }),
    ]
}

// A reply opens under its request's key only, as its read's kind, printed by line (AT-T79-49).
#[test]
fn at_t79_49_a_reply_opens_only_under_its_key() {
    let dir = scratch("replies");
    let (_, net, _, held) = person(&dir);
    let join = event::join(&net, &held).unwrap();
    let (_, status) = read::request(&net, &held, &Ask::Status(join)).unwrap();
    let (_, reviewer) = read::request(&net, &held, &Ask::Reviewer { from: 0 }).unwrap();
    let (_, author) = read::request(&net, &held, &Ask::Author).unwrap();
    let [status_reply, none, assigned, authored] = replies();
    let to = |opener: &read::Opener, reply: &Reply| {
        seal(&opener.response_key(), &reply.encode()).unwrap()
    };
    assert_eq!(
        status.open(&to(&status, &status_reply)),
        Ok(status_reply.clone())
    );
    assert_eq!(reviewer.open(&to(&reviewer, &none)), Ok(none.clone()));
    assert_eq!(
        reviewer.open(&to(&reviewer, &assigned)),
        Ok(assigned.clone())
    );
    assert_eq!(author.open(&to(&author, &authored)), Ok(authored.clone()));
    assert_eq!(reviewer.open(&to(&author, &authored)), Err(Error::Reply));
    assert_eq!(author.open(&to(&reviewer, &assigned)), Err(Error::Reply));
    assert_eq!(author.open(&to(&author, &assigned)), Err(Error::Reply));
    assert_eq!(
        reviewer.open(&to(&reviewer, &status_reply)),
        Err(Error::Reply)
    );
    let sealed = to(&author, &authored);
    for i in 0..sealed.len() {
        let mut altered = sealed.clone();
        altered[i] ^= 1;
        assert_eq!(author.open(&altered), Err(Error::Reply));
    }
    assert_eq!(author.open(&sealed[..sealed.len() - 1]), Err(Error::Reply));
    let not_a_reply = seal(&author.response_key(), b"\x01\x03").unwrap();
    assert_eq!(author.open(&not_a_reply), Err(Error::Reply));

    let writer = |b: u8| hex(&[b; 32]);
    assert_eq!(
        render(&status_reply),
        format!(
            "entries 3\nentry {} 4 pending\nentry {} 5 applied question 3\n\
             entry {} 6 refused Path(Duplicate {{ question: 3 }})\n",
            writer(1),
            writer(2),
            writer(3)
        )
    );
    assert_eq!(render(&none), "question none\n");
    assert_eq!(
        render(&assigned),
        "question 2\nphase reveals-open\ncommitted true\nrevealed false\ncontext \"ctx\"\n\
         stem \"line\\nbreak\"\nalternative 0 \"yes\"\nalternative 1 \"no\"\n"
    );
    assert_eq!(
        render(&authored),
        "reputation 0.5\nquota 2\nquestion 0 pending\nquestion 1 closed divided\n\
         question 2 closed accepted observation success 1 3 2\n"
    );
}

// A read with no relay listening fails on the channel and prints nothing (AT-T79-49).
#[test]
fn at_t79_49_a_read_without_its_relay_fails() {
    let dir = scratch("unwired");
    let (genesis, net, credential, held) = person(&dir);
    let key = announcement(&net, 0);
    let relay = event::relay(
        &net,
        &format!("/ip4/127.0.0.1/tcp/4001/p2p/{}", relay_peer()),
    )
    .unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let got = runtime.block_on(read::read(&net, &held, &key, relay, &Ask::Author));
    assert!(matches!(got, Err(Error::Channel(_))), "{got:?}");
    let (text, address) = (
        hex(&key.encode()),
        format!("/ip4/127.0.0.1/tcp/4001/p2p/{}", relay_peer()),
    );
    let err = refused(&tool(&[
        &"--read",
        &genesis,
        &credential,
        &text,
        &address,
        &"author",
    ]));
    assert!(err.contains("Channel"), "{err}");
    let err = refused(&tool(&[
        &"--read",
        &genesis,
        &credential,
        &text,
        &address,
        &"reviewer",
        &"0",
    ]));
    assert!(err.contains("Channel"), "{err}");
    let event_file = dir.join("join.event");
    fs::write(&event_file, event::join(&net, &held).unwrap()).unwrap();
    let err = refused(&tool(&[
        &"--read",
        &genesis,
        &credential,
        &text,
        &address,
        &"status",
        &event_file,
    ]));
    assert!(err.contains("Channel"), "{err}");
    let err = refused(&tool(&[
        &"--read",
        &genesis,
        &credential,
        &text,
        &address,
        &"everything",
    ]));
    assert!(err.contains("usage"), "{err}");
}
