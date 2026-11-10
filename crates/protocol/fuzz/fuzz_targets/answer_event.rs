//! The answer review's events from untrusted bytes (`docs/05` §The answer review): arbitrary
//! bytes or a genuine join, deposit or reveal with a window overwritten. No panic; one
//! encoding; applying a decoded event to a path holding one reviewer never panics.
#![no_main]

use identity::credential::{AnonymousCredential, Credential, Issuer, IssuerPublic};
use identity::enrollment::Label;
use identity::nullifier::prove;
use identity::nym::{Nym, Role};
use libfuzzer_sys::fuzz_target;
use protocol::answer::{Answer, Question};
use protocol::answer_events::AnswerEvent;
use protocol::answer_path::{AnswerConfig, AnswerPath, POOL_CONTEXT};
use protocol::deposit::deposit_context;
use std::sync::OnceLock;

struct Fixture {
    issuer: IssuerPublic,
    path: AnswerPath,
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
        let join = AnswerEvent::JoinPool {
            proof: prove(&person(&issuer, 2), &public, Role::Judge, POOL_CONTEXT),
        };
        let mut path = AnswerPath::new(AnswerConfig::default()).expect("the default holds");
        path.apply(&join, &public).expect("a genuine join");
        let question = Question {
            context: b"context".to_vec(),
            stem: b"stem".to_vec(),
            alternatives: vec![b"a".to_vec(), b"b".to_vec()],
            key: 0,
        };
        let context = deposit_context(question.content_id(), 0);
        let deposit = AnswerEvent::Deposit {
            epoch: 0,
            proof: prove(&person(&issuer, 3), &public, Role::Propose, &context),
            question,
        };
        let reveal = AnswerEvent::Reveal {
            question: 0,
            reviewer: Nym([4; 32]),
            answer: Answer::Alternative(1),
            nonce: [5; 32],
        };
        Fixture {
            issuer: public,
            path,
            genuine: [join, deposit, reveal]
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
    if let Some(event) = AnswerEvent::decode(&candidate) {
        assert_eq!(
            event.encode(),
            candidate,
            "an event decoded from another encoding"
        );
        let _ = f.path.clone().apply(&event, &f.issuer);
    }
});
