//! A read reply's inner seal (`docs/05` T79-P7): a requester opening arbitrary bytes, or a genuine
//! seal with a window overwritten, under its key; a member sealing to an arbitrary response key.
//! No panic; only the untouched genuine seal opens, to its reply; a seal has its fixed overhead.
#![no_main]

use libfuzzer_sys::fuzz_target;
use protocol::answer_seal::{seal, ResponseKey, OVERHEAD};
use std::sync::OnceLock;

const REPLY: &[u8] = b"isegoria: a sealed read reply";

struct Fixture {
    key: ResponseKey,
    sealed: Vec<u8>,
}

fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let key = ResponseKey::derive(&[7; 32]);
        let sealed = seal(&key.public(), REPLY).expect("a derived key");
        Fixture { key, sealed }
    })
}

fuzz_target!(|input: (Option<u16>, [u8; 32], Vec<u8>)| {
    let (splice, response_key, bytes) = input;
    let f = fixture();
    if let Some(sealed) = seal(&response_key, &bytes) {
        assert_eq!(
            sealed.len(),
            bytes.len() + OVERHEAD,
            "a seal of another length"
        );
    }
    let candidate = match splice {
        Some(at) => {
            let mut c = f.sealed.clone();
            let at = usize::from(at) % c.len();
            for (dst, src) in c[at..].iter_mut().zip(&bytes) {
                *dst = *src;
            }
            c
        }
        None => bytes,
    };
    let opened = f.key.open(&candidate);
    if candidate == f.sealed {
        assert_eq!(
            opened.as_deref(),
            Some(REPLY),
            "the genuine seal did not open"
        );
    } else {
        assert_eq!(opened, None, "an altered or foreign seal opened");
    }
});
