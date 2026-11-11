//! The credential's private encodings (`docs/05` §Distributed integration, credentials before T19):
//! credential, issuance request, reply and pending state from arbitrary bytes or a genuine one with
//! a window overwritten. No panic; one encoding each; only the untouched genuine one is usable.
#![no_main]

use identity::credential::{
    AnonymousCredential, BlindSignature, Credential, IssuanceRequest, Issuer, IssuerPublic,
    PendingIssuance,
};
use identity::enrollment::Label;
use libfuzzer_sys::fuzz_target;
use std::sync::OnceLock;

struct Fixture {
    issuer: Issuer,
    public: IssuerPublic,
    blind: BlindSignature,
    pending: PendingIssuance,
    credential: Vec<u8>,
    request: Vec<u8>,
    reply: Vec<u8>,
    state: Vec<u8>,
}

fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let issuer = Issuer::new([1; 32]);
        let public = issuer.public();
        let holder = Credential::from_secret([2; 32]);
        let (request, pending) = holder.request_issuance(&Label([3; 32]), &public);
        let blind = issuer.issue(&request).expect("a valid request");
        let credential = pending.clone().finalize(blind.clone()).encode();
        Fixture {
            request: request.encode(),
            reply: blind.encode(),
            state: pending.encode(),
            issuer,
            public,
            blind,
            pending,
            credential,
        }
    })
}

fuzz_target!(|input: (Option<(u8, u16)>, Vec<u8>)| {
    let (splice, bytes) = input;
    let f = fixture();
    let genuine = [&f.credential, &f.request, &f.reply, &f.state];
    let candidate = match splice {
        Some((which, at)) => {
            let mut c = genuine[usize::from(which) % genuine.len()].clone();
            let at = usize::from(at) % c.len();
            for (dst, src) in c[at..].iter_mut().zip(&bytes) {
                *dst = *src;
            }
            c
        }
        None => bytes,
    };
    if let Some(credential) = AnonymousCredential::decode(&candidate) {
        assert_eq!(
            credential.encode(),
            candidate,
            "a credential from another encoding"
        );
        let verifies = credential.verify(&f.public);
        assert_eq!(
            verifies,
            candidate == f.credential,
            "only the genuine credential verifies"
        );
    }
    if let Some(request) = IssuanceRequest::decode(&candidate) {
        assert_eq!(
            request.encode(),
            candidate,
            "a request from another encoding"
        );
        let issued = f.issuer.issue(&request).is_ok();
        assert_eq!(
            issued,
            candidate == f.request,
            "only the genuine request is issued"
        );
    }
    if let Some(blind) = BlindSignature::decode(&candidate) {
        assert_eq!(blind.encode(), candidate, "a reply from another encoding");
        let credential = f.pending.clone().finalize(blind);
        let verifies = credential.verify(&f.public);
        assert_eq!(
            verifies,
            candidate == f.reply,
            "only the genuine reply finalizes"
        );
    }
    if let Some(pending) = PendingIssuance::decode(&candidate) {
        assert_eq!(
            pending.encode(),
            candidate,
            "a pending issuance from another encoding"
        );
        let verifies = pending.finalize(f.blind.clone()).verify(&f.public);
        assert_eq!(
            verifies,
            candidate == f.state,
            "only the genuine pending state finalizes"
        );
    }
});
