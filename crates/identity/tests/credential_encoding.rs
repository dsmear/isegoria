//! The private test encodings of a credential and of its blind issuance's messages (`docs/05`
//! §Distributed integration, credentials before T19, AT-ID-10, AT-ID-11): one each, strict.

use ark_bls12_381::Fr;
use ark_ff::{BigInteger, PrimeField};
use identity::credential::{
    AnonymousCredential, BlindSignature, Credential, IssuanceRequest, Issuer, PendingIssuance,
};
use identity::enrollment::Label;
use identity::nullifier;
use identity::nym::Role;
use proptest::prelude::*;
use std::sync::OnceLock;

struct Issued {
    request: Vec<u8>,
    reply: Vec<u8>,
    pending: Vec<u8>,
    credential: Vec<u8>,
}

fn issuer() -> Issuer {
    Issuer::new([1; 32])
}

fn issued() -> &'static Issued {
    static ISSUED: OnceLock<Issued> = OnceLock::new();
    ISSUED.get_or_init(|| {
        let issuer = issuer();
        let holder = Credential::from_secret([3; 32]);
        let (request, pending) = holder.request_issuance(&Label([4; 32]), &issuer.public());
        let reply = issuer.issue(&request).unwrap();
        Issued {
            request: request.encode(),
            reply: reply.encode(),
            pending: pending.encode(),
            credential: pending.finalize(reply).encode(),
        }
    })
}

type Decoder = fn(&[u8]) -> Option<Vec<u8>>;

/// Each kind's decoder, re-encoding what it accepts, with that kind's bytes.
fn kinds() -> [(&'static str, Decoder, &'static [u8]); 4] {
    let i = issued();
    [
        (
            "credential",
            |b| AnonymousCredential::decode(b).map(|c| c.encode()),
            &i.credential,
        ),
        (
            "request",
            |b| IssuanceRequest::decode(b).map(|r| r.encode()),
            &i.request,
        ),
        (
            "reply",
            |b| BlindSignature::decode(b).map(|r| r.encode()),
            &i.reply,
        ),
        (
            "pending",
            |b| PendingIssuance::decode(b).map(|p| p.encode()),
            &i.pending,
        ),
    ]
}

fn modulus() -> Vec<u8> {
    Fr::MODULUS.to_bytes_le()
}

// A decoded credential verifies, salts and proves as the original, under its issuer (AT-ID-10).
#[test]
fn a_decoded_credential_acts_as_the_original() {
    let bytes = &issued().credential;
    assert_eq!(bytes.len(), 22 + 112 + 64);
    let credential = AnonymousCredential::decode(bytes).expect("its own encoding");
    assert_eq!(&credential.encode(), bytes);
    assert!(credential.verify(&issuer().public()));
    assert!(!credential.verify(&Issuer::new([2; 32]).public()));
    let again = AnonymousCredential::decode(bytes).unwrap();
    for role in [Role::Propose, Role::Judge, Role::Respond] {
        assert_eq!(credential.state_salt_key(role), again.state_salt_key(role));
        let proof = nullifier::prove(&credential, &issuer().public(), role, b"context");
        assert!(nullifier::verify(&proof, &issuer().public(), b"context"));
        let other = nullifier::prove(&again, &issuer().public(), role, b"other");
        assert_eq!(proof.id(), other.id());
    }
}

// Issuance through decoded request, reply and pending state gives a valid credential (AT-ID-11).
#[test]
fn issuance_runs_through_the_encodings() {
    let issuer = issuer();
    let holder = Credential::from_secret([5; 32]);
    let (request, pending) = holder.request_issuance(&Label([6; 32]), &issuer.public());
    let request = IssuanceRequest::decode(&request.encode()).expect("its own encoding");
    assert_eq!(request.label(), &Label([6; 32]));
    assert_eq!(request.encode().len(), 28 + 48 + 32 + 48 + 64);
    let reply = issuer
        .issue(&request)
        .expect("a decoded request keeps its proof");
    assert_eq!(reply.encode().len(), 26 + 112);
    let reply = BlindSignature::decode(&reply.encode()).expect("its own encoding");
    assert_eq!(pending.encode().len(), 28 + 96);
    let pending = PendingIssuance::decode(&pending.encode()).expect("its own encoding");
    assert!(pending.finalize(reply).verify(&issuer.public()));
}

// Prefixes, trailing bytes, other tags and another kind's bytes do not decode (AT-ID-10, AT-ID-11).
#[test]
fn what_decoding_refuses() {
    let kinds = kinds();
    for (name, decode, bytes) in kinds {
        assert_eq!(decode(bytes).as_deref(), Some(bytes), "{name}");
        for n in 0..bytes.len() {
            assert!(decode(&bytes[..n]).is_none(), "{name} prefix {n}");
        }
        assert!(decode(&[bytes, &[0]].concat()).is_none(), "{name} trailing");
        for at in [0, 9, 20] {
            let mut tagged = bytes.to_vec();
            tagged[at] ^= 1;
            assert!(decode(&tagged).is_none(), "{name} tag byte {at}");
        }
        for (other, _, theirs) in kinds {
            if other != name {
                assert!(decode(theirs).is_none(), "{name} decoding a {other}");
            }
        }
    }
}

// A scalar at or over the group order, in any scalar's place, does not decode (AT-ID-10, AT-ID-11).
#[test]
fn non_canonical_scalars_are_refused() {
    let i = issued();
    let places: [(Decoder, &[u8], &[usize]); 4] = [
        (
            |b| AnonymousCredential::decode(b).map(|c| c.encode()),
            &i.credential,
            &[22 + 48, 22 + 80, 22 + 112, 22 + 144],
        ),
        (
            |b| IssuanceRequest::decode(b).map(|r| r.encode()),
            &i.request,
            &[28 + 128, 28 + 160],
        ),
        (
            |b| BlindSignature::decode(b).map(|r| r.encode()),
            &i.reply,
            &[26 + 48, 26 + 80],
        ),
        (
            |b| PendingIssuance::decode(b).map(|p| p.encode()),
            &i.pending,
            &[28, 28 + 32, 28 + 64],
        ),
    ];
    for (decode, bytes, offsets) in places {
        for &at in offsets {
            for value in [modulus(), vec![0xff; 32]] {
                let mut bad = bytes.to_vec();
                bad[at..at + 32].copy_from_slice(&value);
                assert!(decode(&bad).is_none(), "scalar at {at}");
            }
        }
    }
}

// A point's flags set to uncompressed or infinity do not decode (AT-ID-10, AT-ID-11).
#[test]
fn altered_points_are_refused() {
    let [credential, request, reply, _] = kinds();
    let places = [
        (credential, 22),
        (request, 28),
        (request, 28 + 80),
        (reply, 26),
    ];
    for ((name, decode, bytes), at) in places {
        for flags in [0x00, 0x40, 0xc0, 0xe0] {
            let mut bad = bytes.to_vec();
            bad[at] = (bad[at] & 0x1f) | flags;
            assert!(decode(&bad).is_none(), "{name} flags {flags:#04x} at {at}");
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // A changed, inserted or removed byte decodes only as its own encoding (AT-ID-10, AT-ID-11).
    #[test]
    fn only_canonical_bytes_decode(
        kind in 0usize..4,
        at in any::<prop::sample::Index>(),
        value in any::<u8>(),
        edit in 0usize..3,
    ) {
        let (_, decode, bytes) = kinds()[kind];
        let mut bytes = bytes.to_vec();
        let i = at.index(bytes.len());
        match edit {
            0 => bytes[i] ^= value | 1,
            1 => bytes.insert(i, value),
            _ => { bytes.remove(i); }
        }
        if let Some(again) = decode(&bytes) {
            prop_assert_eq!(again, bytes);
        }
    }
}
