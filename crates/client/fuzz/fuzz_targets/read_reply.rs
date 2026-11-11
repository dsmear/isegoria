//! A person's opening of a read's sealed reply (`docs/05` T79-P7, `isegoria-client --read`):
//! arbitrary sealed bytes, and arbitrary or genuine replies with a window overwritten sealed to the
//! read's key. No panic; a reply opens only under its read's key and kind, as decoded; it prints.
#![no_main]

#[path = "../common/network.rs"]
mod common;

use arbitrary::Arbitrary;
use client::read::{render, request, Ask, Opener};
use client::Error;
use identity::credential::Credential;
use libfuzzer_sys::fuzz_target;
use protocol::answer_read::{Assigned, Authored, EventStatus, Logged, Phase, Reply, ReviewPhase};
use protocol::answer_rule::Verdict;
use protocol::answer_seal::seal;
use std::sync::OnceLock;

struct Fixture {
    /// A status, a reviewer's and an author's read.
    openers: Vec<Opener>,
    genuine: Vec<Vec<u8>>,
}

fn replies() -> Vec<Reply> {
    let logged = Logged {
        writer: [1; 32],
        seq: 2,
        status: EventStatus::Applied { question: Some(3) },
    };
    vec![
        Reply::Status(vec![logged]),
        Reply::Reviewer(None),
        Reply::Reviewer(Some(Assigned {
            question: 4,
            context: b"context".to_vec(),
            stem: b"stem".to_vec(),
            alternatives: vec![b"a".to_vec(), b"b\n\"".to_vec()],
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
            ],
        }),
    ]
}

fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let issuer = common::issuer();
        let net = common::network(&issuer.public());
        let label = client::credential::slot_label(1);
        let (asked, pending) =
            Credential::from_secret([5; 32]).request_issuance(&label, &net.issuer);
        let cred = pending.finalize(issuer.issue(&asked).expect("a valid request"));
        let join = client::event::join(&net, &cred).expect("a verified credential joins");
        let asks = [Ask::Status(join), Ask::Reviewer { from: 0 }, Ask::Author];
        let openers = (asks.iter())
            .map(|ask| request(&net, &cred, ask).map(|(_, opener)| opener))
            .collect::<Result<Vec<_>, _>>()
            .expect("each read is requested");
        let genuine = replies().iter().map(Reply::encode).collect();
        Fixture { openers, genuine }
    })
}

/// Whether `reply` answers the read of opener `i`.
fn answers(i: usize, reply: &Reply) -> bool {
    matches!(
        (i, reply),
        (0, Reply::Status(_)) | (1, Reply::Reviewer(_)) | (2, Reply::Author(_))
    )
}

#[derive(Arbitrary, Debug)]
enum Input {
    Sealed(Vec<u8>),
    Plain {
        splice: Option<(u8, u16)>,
        bytes: Vec<u8>,
    },
}

fuzz_target!(|input: (u8, Input)| {
    let (which, input) = input;
    let f = fixture();
    let i = usize::from(which) % f.openers.len();
    let opener = &f.openers[i];
    let plain = match input {
        Input::Sealed(bytes) => {
            assert_eq!(
                opener.open(&bytes).err(),
                Some(Error::Reply),
                "foreign bytes opened"
            );
            return;
        }
        Input::Plain {
            splice: Some((g, at)),
            bytes,
        } => {
            let mut c = f.genuine[usize::from(g) % f.genuine.len()].clone();
            let at = usize::from(at) % c.len();
            for (dst, src) in c[at..].iter_mut().zip(&bytes) {
                *dst = *src;
            }
            c
        }
        Input::Plain {
            splice: None,
            bytes,
        } => bytes,
    };
    let sealed = seal(&opener.response_key(), &plain).expect("a reply seals");
    let expected = Reply::decode(&plain).filter(|r| answers(i, r));
    let opened = opener.open(&sealed);
    assert_eq!(
        opened.as_ref().ok().map(Reply::encode),
        expected.as_ref().map(Reply::encode),
        "a reply opened otherwise than it decodes for this read"
    );
    for (k, other) in f.openers.iter().enumerate() {
        if k != i {
            assert!(
                other.open(&sealed).is_err(),
                "a reply opened under another read's key"
            );
        }
    }
    if let Ok(reply) = opened {
        let printed = render(&reply);
        assert!(
            printed.ends_with('\n'),
            "a reply printed without its last line"
        );
        if let Reply::Status(logged) = &reply {
            assert_eq!(
                printed.lines().count(),
                1 + logged.len(),
                "a status printed otherwise"
            );
        }
    }
});
