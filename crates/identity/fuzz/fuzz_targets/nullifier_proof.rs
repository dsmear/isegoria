//! Nullifier proofs from untrusted bytes (`docs/03` §M3, `docs/04` §Events and replay): the
//! wire format v1 — arbitrary bytes, or a genuine proof with bytes spliced in. Only the
//! untouched genuine proof verifies.
#![no_main]

use identity::credential::{Credential, Issuer, IssuerPublic};
use identity::enrollment::Label;
use identity::nullifier::{prove, verify, NullifierProof};
use identity::nym::Role;
use libfuzzer_sys::fuzz_target;
use std::sync::OnceLock;

const CONTEXT: &[u8] = b"fuzz-action";

/// The issuer, and one genuine Judge proof for `CONTEXT`.
fn genuine() -> &'static (IssuerPublic, Vec<u8>) {
    static GENUINE: OnceLock<(IssuerPublic, Vec<u8>)> = OnceLock::new();
    GENUINE.get_or_init(|| {
        let issuer = Issuer::new([1u8; 32]);
        let holder = Credential::from_secret([9u8; 32]);
        let (request, pending) = holder.request_issuance(&Label([7u8; 32]), &issuer.public());
        let credential = pending.finalize(issuer.issue(&request).expect("valid request"));
        let proof = prove(&credential, &issuer.public(), Role::Judge, CONTEXT);
        (issuer.public(), proof.encode())
    })
}

fuzz_target!(|input: (bool, u16, Vec<u8>, bool)| {
    let (splice, at, bytes, other_context) = input;
    let (issuer, genuine) = genuine();
    let candidate = if splice {
        // Overwrite a window of the genuine encoding, keeping its length.
        let mut c = genuine.clone();
        let at = usize::from(at) % c.len();
        for (dst, src) in c[at..].iter_mut().zip(&bytes) {
            *dst = *src;
        }
        c
    } else {
        bytes
    };
    let context: &[u8] = if other_context {
        b"another-action"
    } else {
        CONTEXT
    };

    if let Some(proof) = NullifierProof::decode(&candidate) {
        assert_eq!(proof.encode(), candidate, "a proof decoded from another encoding");
        if verify(&proof, issuer, context) {
            assert!(
                &candidate == genuine && context == CONTEXT,
                "a modified proof verified"
            );
        }
    }
});
