//! A participant's combination of the signers' shares (`docs/03` T19-P5, `isegoria-client
//! --combine`): any list of genuine, altered or arbitrary shares, in any order, for either slot's
//! pending state. No panic; the genuine set combines in any order; only to the genuine credential.
#![no_main]

#[path = "../common/ceremony.rs"]
mod common;

use arbitrary::Arbitrary;
use common::{SIGNERS, SLOTS};
use identity::credential::ceremony::{combine, SignatureShare, SlotRequest};
use identity::credential::{Credential, IssuerPublic, PendingIssuance};
use libfuzzer_sys::fuzz_target;
use std::sync::OnceLock;

struct Fixture {
    issuer: IssuerPublic,
    /// By slot: the pending state, each signer's share encoded, and the credential they make.
    slots: Vec<(PendingIssuance, Vec<Vec<u8>>, Vec<u8>)>,
}

fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let prepared = common::run().prepared;
        let issuer = prepared[0].issuer().expect("a finished key generation");
        let holder = Credential::from_secret([4; 32]);
        let slots = (SLOTS.iter())
            .map(|&slot| {
                let (request, pending) = SlotRequest::new(&holder, slot, &issuer);
                let shares = (SIGNERS.iter())
                    .map(|&i| prepared[usize::from(i)].answer(&request))
                    .collect::<Result<Vec<_>, _>>()
                    .expect("a signer answers a genuine request");
                let credential = combine(pending.clone(), &shares, &issuer)
                    .expect("the genuine shares combine")
                    .encode();
                let encoded = shares.iter().map(SignatureShare::encode).collect();
                (pending, encoded, credential)
            })
            .collect();
        Fixture { issuer, slots }
    })
}

#[derive(Arbitrary, Debug)]
enum Share {
    Genuine {
        slot: u8,
        signer: u8,
    },
    Spliced {
        slot: u8,
        signer: u8,
        at: u16,
        bytes: Vec<u8>,
    },
    Raw(Vec<u8>),
}

fn bytes_of(f: &Fixture, share: Share) -> Vec<u8> {
    let genuine = |slot: u8, signer: u8| {
        let shares = &f.slots[usize::from(slot) % f.slots.len()].1;
        shares[usize::from(signer) % shares.len()].clone()
    };
    match share {
        Share::Genuine { slot, signer } => genuine(slot, signer),
        Share::Spliced {
            slot,
            signer,
            at,
            bytes,
        } => {
            let mut c = genuine(slot, signer);
            let at = usize::from(at) % c.len();
            for (dst, src) in c[at..].iter_mut().zip(&bytes) {
                *dst = *src;
            }
            c
        }
        Share::Raw(bytes) => bytes,
    }
}

fuzz_target!(|input: (u8, Vec<Share>)| {
    let (slot, shares) = input;
    let f = fixture();
    let (pending, genuine, credential) = &f.slots[usize::from(slot) % f.slots.len()];
    let encoded: Vec<Vec<u8>> = shares.into_iter().take(8).map(|s| bytes_of(f, s)).collect();
    let Some(decoded) = (encoded.iter())
        .map(|b| SignatureShare::decode(b))
        .collect::<Option<Vec<_>>>()
    else {
        return;
    };
    let mut sorted = encoded.clone();
    sorted.sort();
    let mut expected = genuine.clone();
    expected.sort();
    let combined = combine(pending.clone(), &decoded, &f.issuer);
    if sorted == expected {
        assert!(
            combined.is_ok(),
            "the genuine shares in this order do not combine"
        );
    }
    let only_genuine = (encoded.iter()).all(|e| f.slots.iter().any(|(_, g, _)| g.contains(e)));
    if only_genuine && sorted != expected {
        assert!(
            combined.is_err(),
            "fewer, repeated or another slot's genuine shares combined"
        );
    }
    if let Ok(made) = combined {
        assert!(
            made.verify(&f.issuer),
            "a combined credential that does not verify"
        );
        assert_eq!(&made.encode(), credential, "combined to another credential");
    }
});
