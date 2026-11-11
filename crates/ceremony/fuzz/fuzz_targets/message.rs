//! A ceremony message file (`docs/03` T19-P3): arbitrary bytes or a genuine message with a window
//! overwritten, opened by its recipient. No panic; a genuine message opens to its header and
//! payload; nothing else opens.
#![no_main]

use ceremony::message::{hpke_pair, open, seal, Header, HpkeSecret};
use ed25519_dalek::SigningKey;
use libfuzzer_sys::fuzz_target;
use std::sync::OnceLock;

struct Fixture {
    secret: HpkeSecret,
    members: Vec<[u8; 32]>,
    genuine: Vec<(Vec<u8>, Header, Vec<u8>)>,
}

fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let keys: Vec<SigningKey> = (0..4u8)
            .map(|i| SigningKey::from_bytes(&[10 + i; 32]))
            .collect();
        let members = keys.iter().map(|k| k.verifying_key().to_bytes()).collect();
        let (secret, recipient) = hpke_pair(&[30; 32]);
        let message = |step, sender: u8, payload: &[u8]| {
            let header = Header {
                id: [7; 32],
                step,
                sender,
                recipient: 0,
            };
            let key = &keys[usize::from(sender)];
            let bytes = seal(&header, payload, key, &recipient).expect("a derived key");
            (bytes, header, payload.to_vec())
        };
        let genuine = vec![
            message(1, 1, &[9; 32]),
            message(3, 2, b"a library object"),
            message(13, 3, b""),
        ];
        Fixture {
            secret,
            members,
            genuine,
        }
    })
}

fuzz_target!(|input: (Option<(u8, u16)>, Vec<u8>)| {
    let (splice, bytes) = input;
    let f = fixture();
    let candidate = match splice {
        Some((which, at)) => {
            let mut c = f.genuine[usize::from(which) % f.genuine.len()].0.clone();
            let at = usize::from(at) % c.len();
            for (dst, src) in c[at..].iter_mut().zip(&bytes) {
                *dst = *src;
            }
            c
        }
        None => bytes,
    };
    let opened = open(&candidate, 0, &f.secret, &f.members);
    let genuine = f.genuine.iter().find(|(bytes, _, _)| *bytes == candidate);
    match (genuine, opened) {
        (Some((_, header, payload)), Ok(o)) => {
            assert_eq!(o, (*header, payload.clone()), "a message opened as another")
        }
        (Some(_), Err(e)) => panic!("a genuine message refused: {e:?}"),
        (None, Ok(_)) => panic!("an altered or foreign message opened"),
        (None, Err(_)) => {}
    }
});
