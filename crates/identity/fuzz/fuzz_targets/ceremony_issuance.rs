//! Issuance from the ceremony (`docs/03` T19-P5): slot requests and signature shares, arbitrary or
//! genuine with a window overwritten, answered by every member and combined. No panic; one encoding
//! each; only a genuine request is answered, with its share; a genuine set combines.
#![no_main]

#[path = "../common/ceremony.rs"]
mod common;

use common::{SIGNERS, SLOTS};
use identity::credential::ceremony::{combine, AnswerError, Party, SignatureShare, SlotRequest};
use identity::credential::{Credential, IssuerPublic, PendingIssuance};
use libfuzzer_sys::fuzz_target;
use std::sync::OnceLock;

struct Fixture {
    prepared: Vec<Party>,
    issuer: IssuerPublic,
    /// By slot: the request, the participant's pending state and each signer's share.
    slots: Vec<(Vec<u8>, PendingIssuance, Vec<SignatureShare>)>,
    genuine: Vec<Vec<u8>>,
}

fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let prepared = common::run().prepared;
        let issuer = prepared[0].issuer().expect("a finished key generation");
        let holder = Credential::from_secret([3; 32]);
        let slots: Vec<_> = (SLOTS.iter())
            .map(|&slot| {
                let (request, pending) = SlotRequest::new(&holder, slot, &issuer);
                let shares = (SIGNERS.iter())
                    .map(|&i| prepared[usize::from(i)].answer(&request))
                    .collect::<Result<Vec<_>, _>>()
                    .expect("a signer answers a genuine request");
                (request.encode(), pending, shares)
            })
            .collect();
        let requests = slots.iter().map(|(r, _, _)| r.clone());
        let shares = (slots.iter()).flat_map(|(_, _, s)| s.iter().map(SignatureShare::encode));
        let genuine = requests.chain(shares).collect();
        Fixture {
            prepared,
            issuer,
            slots,
            genuine,
        }
    })
}

fn requested(f: &Fixture, candidate: &[u8]) {
    let Some(request) = SlotRequest::decode(candidate) else {
        return;
    };
    assert_eq!(
        request.encode(),
        candidate,
        "a request from another encoding"
    );
    let slot = f.slots.iter().position(|(r, _, _)| r == candidate);
    for (i, party) in f.prepared.iter().enumerate() {
        let answer = party.answer(&request);
        let Some(k) = SIGNERS.iter().position(|&s| usize::from(s) == i) else {
            assert!(
                matches!(answer, Err(AnswerError::NotPrepared)),
                "a non-signer answered"
            );
            continue;
        };
        match (slot, answer) {
            (Some(s), Ok(share)) => assert_eq!(share, f.slots[s].2[k], "another share"),
            (Some(_), Err(e)) => panic!("a genuine request refused: {e:?}"),
            (None, Ok(_)) => panic!("an altered request answered"),
            (None, Err(_)) => {}
        }
    }
}

fn shared(f: &Fixture, candidate: &[u8]) {
    let Some(share) = SignatureShare::decode(candidate) else {
        return;
    };
    assert_eq!(share.encode(), candidate, "a share from another encoding");
    let s = SLOTS.iter().position(|&x| x == share.slot()).unwrap_or(0);
    let (_, pending, genuine) = &f.slots[s];
    let mut shares = genuine.clone();
    let k = (shares.iter()).position(|g| g.member() == share.member());
    shares[k.unwrap_or(0)] = share;
    let combined = combine(pending.clone(), &shares, &f.issuer);
    if shares == *genuine {
        assert!(combined.is_ok(), "the genuine shares do not combine");
    }
    if let Ok(credential) = combined {
        assert!(
            credential.verify(&f.issuer),
            "a combined credential that does not verify"
        );
    }
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
    requested(f, &candidate);
    shared(f, &candidate);
});
