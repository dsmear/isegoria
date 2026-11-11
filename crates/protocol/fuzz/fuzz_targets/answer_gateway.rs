//! A gateway's admission of a person's bytes (`docs/05` §Distributed integration, D60): arbitrary
//! bytes or a genuine join, deposit, commit or reveal with a window overwritten. No panic; a
//! genuine event is admitted; a proved kind is admitted only as one of the genuine events.
#![no_main]

use identity::credential::{AnonymousCredential, Credential, Issuer, IssuerPublic};
use identity::enrollment::Label;
use identity::nullifier::prove;
use identity::nym::Role;
use libfuzzer_sys::fuzz_target;
use protocol::answer::{commit_answer, Answer, Question};
use protocol::answer_events::AnswerEvent;
use protocol::answer_gateway::admit;
use protocol::answer_path::{commit_context, network_context, POOL_CONTEXT};
use protocol::deposit::deposit_context;
use std::sync::OnceLock;

const NET: [u8; 32] = [0x79; 32];

struct Fixture {
    issuer: IssuerPublic,
    genuine: Vec<Vec<u8>>,
}

fn person(issuer: &Issuer, secret: u8) -> AnonymousCredential {
    let holder = Credential::from_secret([secret; 32]);
    let (request, pending) = holder.request_issuance(&Label([secret; 32]), &issuer.public());
    pending.finalize(issuer.issue(&request).expect("a valid request"))
}

fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let issuer = Issuer::new([1; 32]);
        let public = issuer.public();
        let (reviewer, author) = (person(&issuer, 2), person(&issuer, 3));
        let context = |offline: &[u8]| network_context(&NET, offline);
        let join = AnswerEvent::JoinPool {
            proof: prove(&reviewer, &public, Role::Judge, &context(POOL_CONTEXT)),
        };
        let question = Question {
            context: b"context".to_vec(),
            stem: b"stem".to_vec(),
            alternatives: vec![b"a".to_vec(), b"b".to_vec()],
            key: 0,
        };
        let offline = deposit_context(question.content_id(), 0);
        let deposit = AnswerEvent::Deposit {
            epoch: 0,
            proof: prove(&author, &public, Role::Propose, &context(&offline)),
            question,
        };
        let (answer, nonce) = (Answer::Alternative(1), [5; 32]);
        let judge = prove(&reviewer, &public, Role::Judge, b"nym").id();
        let commitment = commit_answer(answer, &nonce, judge, 0);
        let offline = commit_context(0, &commitment);
        let commit = AnswerEvent::Commit {
            question: 0,
            commitment,
            proof: prove(&reviewer, &public, Role::Judge, &context(&offline)),
        };
        let reveal = AnswerEvent::Reveal {
            question: 0,
            reviewer: judge,
            answer,
            nonce,
        };
        Fixture {
            issuer: public,
            genuine: [join, deposit, commit, reveal]
                .iter()
                .map(AnswerEvent::encode)
                .collect(),
        }
    })
}

fuzz_target!(|input: (Option<(u8, u16)>, Vec<u8>)| {
    let (splice, bytes) = input;
    let f = fixture();
    let candidate = match splice {
        Some((which, at)) => {
            let mut c = f.genuine[usize::from(which) % f.genuine.len()].clone();
            let at = usize::from(at) % c.len();
            for (dst, src) in c[at..].iter_mut().zip(&bytes) {
                *dst = *src;
            }
            c
        }
        None => bytes,
    };
    let admitted = admit(&candidate, &NET, &f.issuer).is_ok();
    if f.genuine.contains(&candidate) {
        assert!(admitted, "a genuine event refused");
    }
    if !admitted {
        return;
    }
    let event = AnswerEvent::decode(&candidate).expect("an admitted event decodes");
    match event {
        AnswerEvent::Reveal { .. } => {}
        AnswerEvent::JoinPool { .. } | AnswerEvent::Deposit { .. } | AnswerEvent::Commit { .. } => {
            assert!(
                f.genuine.contains(&candidate),
                "an altered proved event admitted"
            )
        }
        _ => panic!("a member's or derived event admitted"),
    }
});
