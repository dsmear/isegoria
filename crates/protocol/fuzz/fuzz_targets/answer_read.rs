//! Private reads (`docs/05` T79-P7): read requests and replies from arbitrary bytes or genuine ones
//! with a window overwritten, and a member's `answer` over an empty ledger. No panic; one encoding
//! each; an answer only for a genuine request's kind, key and proof, opening to a reply.
#![no_main]

use identity::credential::{Credential, Issuer};
use identity::enrollment::Label;
use identity::nym::Role;
use libfuzzer_sys::fuzz_target;
use network::cid::Cid;
use network::consortium::{Consortium, Member};
use network::replica::{FeedWriter, Replica};
use network::tier::Tiers;
use protocol::answer_author::Observation;
use protocol::answer_ledger::{AnswerLedger, Refusal};
use protocol::answer_path::{AnswerConfig, Refused};
use protocol::answer_read::{
    answer, Assigned, Authored, EventStatus, Logged, Phase, Read, ReadRequest, Reply, ReviewPhase,
};
use protocol::answer_rule::Verdict;
use protocol::answer_seal::ResponseKey;
use protocol::author_time::AuthorPoint;
use std::mem::discriminant;
use std::sync::OnceLock;

const NET: [u8; 32] = [0x79; 32];

struct Fixture {
    ledger: AnswerLedger,
    replica: Replica,
    key: ResponseKey,
    requests: Vec<ReadRequest>,
    genuine: Vec<Vec<u8>>,
}

fn tiers() -> Tiers {
    let members: Vec<_> = (0..4u8).map(|i| Member::from_seed([10 + i; 32])).collect();
    let consortium = Consortium::new(members.iter().map(Member::public).collect(), 3);
    let circle: Vec<_> = (0..4u8)
        .map(|i| FeedWriter::from_seed(NET, [20 + i; 32]).public())
        .collect();
    Tiers::new(NET, consortium, &circle).expect("a valid tiered set")
}

fn replies() -> Vec<Reply> {
    let status = |status| Logged {
        writer: [1; 32],
        seq: 2,
        status,
    };
    let observation = Observation {
        success: true,
        weight: 0.75,
        origin: AuthorPoint {
            epoch: 1,
            closings: 2,
        },
    };
    vec![
        Reply::Status(vec![
            status(EventStatus::Pending),
            status(EventStatus::Applied { question: None }),
            status(EventStatus::Applied { question: Some(3) }),
            status(EventStatus::Refused(Refusal::Repeated)),
            status(EventStatus::Refused(Refusal::Path(Refused::Duplicate {
                question: 3,
            }))),
        ]),
        Reply::Reviewer(None),
        Reply::Reviewer(Some(Assigned {
            question: 4,
            context: b"context".to_vec(),
            stem: b"stem".to_vec(),
            alternatives: vec![b"a".to_vec(), b"b".to_vec()],
            committed: true,
            revealed: false,
            phase: ReviewPhase::RevealsOpen,
        })),
        Reply::Author(Authored {
            reputation: 0.5,
            quota: 2,
            questions: vec![
                (0, Phase::Pending),
                (1, Phase::Committing),
                (
                    2,
                    Phase::Closed {
                        verdict: Verdict::Accepted,
                        observation: Some(observation),
                    },
                ),
            ],
        }),
    ]
}

fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let issuer = Issuer::new([1; 32]);
        let public = issuer.public();
        let holder = Credential::from_secret([2; 32]);
        let (request, pending) = holder.request_issuance(&Label([2; 32]), &public);
        let cred = pending.finalize(issuer.issue(&request).expect("a valid request"));
        let tiers = tiers();
        let replica = Replica::new(tiers.writers().clone());
        let ledger = AnswerLedger::new(tiers, public.clone(), AnswerConfig::default())
            .expect("the default holds");
        let key = ResponseKey::derive(&[7; 32]);
        let reads = [
            (
                Read::Status {
                    event: Cid([3; 32]),
                },
                Role::Judge,
            ),
            (
                Read::Status {
                    event: Cid([3; 32]),
                },
                Role::Propose,
            ),
            (Read::Reviewer { from: 0 }, Role::Judge),
            (Read::Author, Role::Propose),
        ];
        let requests: Vec<ReadRequest> = (reads.iter())
            .map(|&(read, role)| ReadRequest::new(read, key.public(), &cred, role, &public, &NET))
            .collect();
        let genuine = (requests.iter().map(ReadRequest::encode))
            .chain(replies().iter().map(Reply::encode))
            .collect();
        Fixture {
            ledger,
            replica,
            key,
            requests,
            genuine,
        }
    })
}

fn proved_as(request: &ReadRequest, genuine: &ReadRequest) -> bool {
    discriminant(&request.read) == discriminant(&genuine.read)
        && request.response_key == genuine.response_key
        && request.proof.encode() == genuine.proof.encode()
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
    if let Some(reply) = Reply::decode(&candidate) {
        assert_eq!(
            reply.encode(),
            candidate,
            "a reply decoded from another encoding"
        );
    }
    let request = ReadRequest::decode(&candidate);
    if let Some(r) = &request {
        assert_eq!(
            r.encode(),
            candidate,
            "a request decoded from another encoding"
        );
    }
    let genuine = f.requests.iter().any(|g| g.encode() == candidate);
    let Some(sealed) = answer(&f.ledger, &f.replica, &candidate) else {
        assert!(!genuine, "a genuine request got no answer");
        return;
    };
    let request = request.expect("an answered request decodes");
    assert!(
        f.requests.iter().any(|g| proved_as(&request, g)),
        "an answer for a request not proved under its kind and key"
    );
    let opened = f
        .key
        .open(&sealed)
        .expect("the answer opens under the request's key");
    assert!(
        Reply::decode(&opened).is_some(),
        "an answer that is no reply"
    );
});
