//! A person's events built by the client, as a member's gateway admits them, and the checks
//! before one is sent (`docs/05` T79-P1, T79-P2, T79-P7; `docs/04` T85-P3, T85-P4).

mod support;

use client::event::{self, answer_of, nym};
use client::{hex, Error, Network};
use identity::credential::Issuer;
use identity::nym::Role;
use network::cid::cid;
use p2p::blind::KeyAnnouncement;
use protocol::admission::Unproven;
use protocol::answer::{commit_answer, Answer, Question};
use protocol::answer_events::AnswerEvent;
use protocol::answer_gateway::{admit, Inadmissible};
use protocol::answer_ledger::MAX_QUESTION;
use protocol::answer_path::{network_context, POOL_CONTEXT};
use std::fs;
use std::path::Path;
use support::*;

fn question() -> Question {
    Question {
        context: b"A context".to_vec(),
        stem: b"Which one?".to_vec(),
        alternatives: vec![b"first".to_vec(), b"second".to_vec(), b"third".to_vec()],
        key: 1,
    }
}

fn unproven(e: Result<(), Inadmissible>) -> bool {
    matches!(e, Err(Inadmissible::Unproven(Unproven::BadProof)))
}

// The four events are admitted on their network only, under their role's nym (AT-T79-45).
#[test]
fn at_t79_45_the_events_are_admitted_under_their_roles() {
    let dir = scratch("events");
    let (_, net, _, held) = person(&dir);
    let other = Network::decode(&{
        let mut g = genesis(&test_issuer().public());
        g.relays.clear();
        g.encode()
    })
    .unwrap();
    assert_ne!(other.id, net.id);
    let stranger = Issuer::new([78; 32]).public();

    let join = event::join(&net, &held).unwrap();
    assert_eq!(admit(&join, &net.id, &net.issuer), Ok(()));
    assert!(unproven(admit(&join, &other.id, &net.issuer)));
    assert!(unproven(admit(&join, &net.id, &stranger)));
    let Some(AnswerEvent::JoinPool { proof }) = AnswerEvent::decode(&join) else {
        panic!("a join")
    };
    assert_eq!(proof.role(), Role::Judge);
    let prefixed = network_context(&net.id, POOL_CONTEXT);
    assert!(identity::nullifier::verify(&proof, &net.issuer, &prefixed));
    assert!(!identity::nullifier::verify(
        &proof,
        &net.issuer,
        POOL_CONTEXT
    ));
    let judge = nym(&net, &held, Role::Judge).unwrap();
    assert_eq!(proof.id(), judge);
    assert_ne!(
        event::join(&net, &held).unwrap(),
        join,
        "a fresh proof each time"
    );
    assert_eq!(
        nym(&net, &held, Role::Judge).unwrap(),
        judge,
        "one nym per role"
    );

    let deposit = event::deposit(&net, &held, 0, question()).unwrap();
    assert_eq!(admit(&deposit, &net.id, &net.issuer), Ok(()));
    assert!(unproven(admit(&deposit, &other.id, &net.issuer)));
    let Some(AnswerEvent::Deposit {
        epoch,
        question: q,
        proof,
    }) = AnswerEvent::decode(&deposit)
    else {
        panic!("a deposit")
    };
    assert_eq!((epoch, q), (0, question()));
    assert_eq!(proof.role(), Role::Propose);
    assert_eq!(proof.id(), nym(&net, &held, Role::Propose).unwrap());
    assert_ne!(
        proof.id(),
        judge,
        "author and reviewer nyms apart (invariant 4)"
    );

    let (commit, opened) = event::commit(&net, &held, 4, Answer::NoneCorrect).unwrap();
    assert_eq!(admit(&commit, &net.id, &net.issuer), Ok(()));
    assert!(unproven(admit(&commit, &other.id, &net.issuer)));
    let Some(AnswerEvent::Commit {
        question,
        commitment,
        proof,
    }) = AnswerEvent::decode(&commit)
    else {
        panic!("a commit")
    };
    assert_eq!(
        (question, opened.question, opened.answer),
        (4, 4, Answer::NoneCorrect)
    );
    assert_eq!(proof.id(), judge);
    assert_eq!(
        commitment,
        commit_answer(Answer::NoneCorrect, &opened.nonce, judge, 4)
    );
    let (_, again) = event::commit(&net, &held, 4, Answer::NoneCorrect).unwrap();
    assert_ne!(again.nonce, opened.nonce);

    let reveal = event::reveal(&net, &held, &opened).unwrap();
    assert_eq!(admit(&reveal, &net.id, &net.issuer), Ok(()));
    let Some(AnswerEvent::Reveal {
        question,
        reviewer,
        answer,
        nonce,
    }) = AnswerEvent::decode(&reveal)
    else {
        panic!("a reveal")
    };
    assert_eq!(
        (question, reviewer, answer, nonce),
        (4, judge, Answer::NoneCorrect, opened.nonce)
    );
    assert_eq!(
        commit_answer(answer, &nonce, reviewer, question),
        commitment
    );
}

// The commands write each event to a new 0600 file and print its content id (AT-T79-45).
#[test]
fn at_t79_45_the_commands_write_the_events() {
    let dir = scratch("commands");
    let (genesis, net, credential, _) = person(&dir);
    let out = dir.join("join.event");
    let printed = succeeded(&tool(&[&"--join", &genesis, &credential, &out]));
    let bytes = fs::read(&out).unwrap();
    assert!(private(&out));
    assert_eq!(printed, format!("event {}\n", hex(&cid(&bytes).0)));
    assert!(matches!(
        AnswerEvent::decode(&bytes),
        Some(AnswerEvent::JoinPool { .. })
    ));
    assert_eq!(event::admitted(&net, &bytes), Ok(()));
    let stranger = dir.join("stranger.credential");
    fs::write(
        &stranger,
        support::credential(&Issuer::new([78; 32]), 2).encode(),
    )
    .unwrap();
    let err = refused(&tool(&[&"--join", &genesis, &stranger, &dir.join("x")]));
    assert!(err.contains("Unverified"), "{err}");
    assert!(!dir.join("x").exists());
}

// A credential that decodes but not under the genesis's key proves nothing, no panic (AT-T79-42).
#[test]
fn at_t79_42_a_credential_under_another_key_proves_nothing() {
    let dir = scratch("unverified");
    let (genesis, net, _, held) = person(&dir);
    let stranger = support::credential(&Issuer::new([78; 32]), 2);
    let file = dir.join("stranger.credential");
    fs::write(&file, stranger.encode()).unwrap();
    let decoded = identity::credential::AnonymousCredential::decode(&fs::read(&file).unwrap());
    assert!(decoded.is_some_and(|c| !c.verify(&net.issuer)));
    let unverified = Err(Error::Unverified);
    assert_eq!(event::join(&net, &stranger), unverified);
    assert_eq!(event::deposit(&net, &stranger, 0, question()), unverified);
    assert!(matches!(
        event::commit(&net, &stranger, 0, Answer::DontKnow),
        Err(Error::Unverified)
    ));
    let (_, opened) = event::commit(&net, &held, 0, Answer::DontKnow).unwrap();
    assert_eq!(event::reveal(&net, &stranger, &opened), unverified);
    assert_eq!(nym(&net, &stranger, Role::Judge), Err(Error::Unverified));
    for ask in [
        client::read::Ask::Author,
        client::read::Ask::Reviewer { from: 0 },
    ] {
        assert!(matches!(
            client::read::request(&net, &stranger, &ask),
            Err(Error::Unverified)
        ));
    }
    let opened_file = dir.join("opened");
    fs::write(&opened_file, opened.encode()).unwrap();
    let toml = dir.join("q.toml");
    fs::write(
        &toml,
        "context = \"c\"\nstem = \"s\"\nalternatives = [\"a\", \"b\"]\nkey = 0\n",
    )
    .unwrap();
    let (out, kept) = (dir.join("out"), dir.join("c"));
    let runs: [Vec<&dyn AsRef<std::ffi::OsStr>>; 4] = [
        vec![&"--join", &genesis, &file, &out],
        vec![&"--deposit", &genesis, &file, &"0", &toml, &out],
        vec![&"--commit", &genesis, &file, &"0", &"none", &kept, &out],
        vec![&"--reveal", &genesis, &file, &opened_file, &out],
    ];
    for args in runs {
        let err = refused(&tool(&args));
        assert!(
            err.contains("Unverified") && !err.contains("panicked"),
            "{err}"
        );
        assert!(!out.exists() && !kept.exists());
    }
}

fn deposit_file(dir: &Path, name: &str, toml: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    fs::write(&path, toml).unwrap();
    path
}

// A question's file is read strictly; a deposit the gateway refuses is not written (AT-T79-46).
#[test]
fn at_t79_46_a_deposit_from_a_question_file() {
    let dir = scratch("deposit");
    let (genesis, _, credential, _) = person(&dir);
    let good = "context = \"A context\"\nstem = \"Which one?\"\n\
        alternatives = [\"first\", \"second\", \"third\"]\nkey = 1\n";
    let file = deposit_file(&dir, "good.toml", good);
    assert_eq!(event::question_file(&file), Ok(question()));
    let out = dir.join("deposit.event");
    succeeded(&tool(&[
        &"--deposit",
        &genesis,
        &credential,
        &"3",
        &file,
        &out,
    ]));
    let Some(AnswerEvent::Deposit {
        epoch, question: q, ..
    }) = AnswerEvent::decode(&fs::read(&out).unwrap())
    else {
        panic!("a deposit")
    };
    assert_eq!((epoch, q), (3, question()));
    let cases = [
        (
            "unknown",
            format!("{good}author = \"x\"\n"),
            "Malformed(\"question\")",
        ),
        (
            "no-key",
            good.replace("key = 1\n", ""),
            "Malformed(\"question\")",
        ),
        (
            "wide-key",
            good.replace("key = 1", "key = 256"),
            "Malformed(\"question\")",
        ),
        (
            "one",
            good.replace(", \"second\", \"third\"", ""),
            "TooFewAlternatives",
        ),
        (
            "repeated",
            good.replace("third", "first"),
            "RepeatedAlternative",
        ),
        ("key", good.replace("key = 1", "key = 3"), "KeyOutOfRange"),
        ("empty", good.replace("A context", ""), "EmptyContext"),
    ];
    for (name, text, why) in cases {
        let file = deposit_file(&dir, &format!("{name}.toml"), &text);
        let out = dir.join(format!("{name}.event"));
        let err = refused(&tool(&[
            &"--deposit",
            &genesis,
            &credential,
            &"0",
            &file,
            &out,
        ]));
        assert!(err.contains(why), "{name}: {err}");
        assert!(!out.exists());
    }
    let long = "x".repeat(MAX_QUESTION);
    let file = deposit_file(&dir, "long.toml", &good.replace("Which one?", &long));
    let out = dir.join("long.event");
    let err = refused(&tool(&[
        &"--deposit",
        &genesis,
        &credential,
        &"0",
        &file,
        &out,
    ]));
    assert!(err.contains("TooLarge"), "{err}");
    assert!(!out.exists());
    for (text, answer) in [
        ("0", Some(Answer::Alternative(0))),
        ("255", Some(Answer::Alternative(255))),
        ("none", Some(Answer::NoneCorrect)),
        ("dont-know", Some(Answer::DontKnow)),
        ("256", None),
        ("+1", None),
        ("01", None),
        ("None", None),
        ("", None),
    ] {
        assert_eq!(answer_of(text), answer, "{text}");
    }
    refused(&tool(&[
        &"--deposit",
        &genesis,
        &credential,
        &"+3",
        &file,
        &out,
    ]));
}

// Before sending: the key and the relay are the genesis's, the event admissible (AT-T79-47).
#[test]
fn at_t79_47_what_is_checked_before_sending() {
    let dir = scratch("send");
    let (genesis, net, _, held) = person(&dir);
    let key = announcement(&net, 2);
    let text = hex(&key.encode());
    assert_eq!(event::announcement(&net, &text), Ok(key.clone()));
    let moved = KeyAnnouncement {
        member: 1,
        ..key.clone()
    };
    assert_eq!(
        event::announcement(&net, &hex(&moved.encode())),
        Err(Error::Unlisted("announcement"))
    );
    let unlisted = announcement_with(&net, 2, [99; 32]);
    let err = event::announcement(&net, &hex(&unlisted.encode()));
    assert_eq!(err, Err(Error::Unlisted("announcement")));
    let other = Network::decode(&{
        let mut g = support::genesis(&test_issuer().public());
        g.relays.clear();
        g.encode()
    })
    .unwrap();
    let elsewhere = announcement(&other, 2);
    let err = event::announcement(&net, &hex(&elsewhere.encode()));
    assert_eq!(err, Err(Error::Unlisted("announcement")));
    assert_eq!(
        event::announcement(&net, "zz"),
        Err(Error::Malformed("announcement"))
    );

    let listed = format!("/ip4/127.0.0.1/tcp/4001/p2p/{}", relay_peer());
    assert_eq!(event::relay(&net, &listed).unwrap().to_string(), listed);
    assert_eq!(
        event::relay(&net, "/ip4/127.0.0.1/tcp/4001"),
        Err(Error::Malformed("relay"))
    );
    let stranger = peer_of(member(0).public().to_bytes());
    let unknown = format!("/ip4/127.0.0.1/tcp/4001/p2p/{stranger}");
    assert_eq!(event::relay(&net, &unknown), Err(Error::Unlisted("relay")));
    assert_eq!(
        event::relay(&net, "nowhere"),
        Err(Error::Malformed("relay"))
    );

    let join = event::join(&net, &held).unwrap();
    let sealed = event::sealed(&net, &key, &join).expect("sealed");
    assert!(!sealed.request.windows(join.len()).any(|w| w == join));
    let member_only = AnswerEvent::CloseCommits { question: 0 }.encode();
    assert!(matches!(
        event::sealed(&net, &key, &member_only),
        Err(Error::Inadmissible(Inadmissible::MemberOnly))
    ));
    let foreign = event::join(&other, &held).unwrap();
    assert!(matches!(
        event::sealed(&net, &key, &foreign),
        Err(Error::Inadmissible(Inadmissible::Unproven(_)))
    ));
    let file = dir.join("join.event");
    fs::write(&file, &join).unwrap();
    let err = refused(&tool(&[&"--send", &genesis, &text, &unknown, &file]));
    assert!(err.contains("Unlisted(\"relay\")"), "{err}");
    let err = refused(&tool(&[
        &"--send",
        &genesis,
        &hex(&moved.encode()),
        &listed,
        &file,
    ]));
    assert!(err.contains("Unlisted(\"announcement\")"), "{err}");
}

// A key configuration is read in lower-case hex digit pairs alone, no other spelling (AT-NET-47).
#[test]
fn at_net_47_an_announcement_in_lower_case_hex_only() {
    let net = Network::decode(&support::genesis(&test_issuer().public()).encode()).unwrap();
    let key = announcement(&net, 2);
    let text = hex(&key.encode());
    assert_eq!(event::announcement(&net, &text), Ok(key));
    let letter = text.find(|c: char| c.is_ascii_lowercase()).unwrap();
    let mut one_upper = text.clone();
    one_upper[letter..=letter].make_ascii_uppercase();
    assert!(text.starts_with("01"));
    for spelling in [
        text.to_uppercase(),
        one_upper,
        format!("+1{}", &text[2..]),
        format!("-1{}", &text[2..]),
        format!(" 1{}", &text[2..]),
        format!("{text}\n"),
        format!(" {text}"),
        format!("{text}0"),
        text[..text.len() - 2].to_string(),
        String::new(),
    ] {
        let taken = event::announcement(&net, &spelling);
        assert_eq!(taken, Err(Error::Malformed("announcement")), "{spelling:?}");
    }
}
