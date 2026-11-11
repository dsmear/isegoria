//! A ceremony member taking a flight's payloads (`docs/03` T19-P3): one sender's payload arbitrary
//! or genuine with a window overwritten, or an extra sender. No panic; the genuine flight is taken;
//! an extra sender is refused by name; a state reached re-decodes as itself.
#![no_main]

#[path = "../common/ceremony.rs"]
mod common;

use arbitrary::Arbitrary;
use identity::credential::ceremony::{Party, Refusal};
use libfuzzer_sys::fuzz_target;
use std::sync::OnceLock;

#[derive(Arbitrary, Debug)]
struct Input {
    step: u16,
    payload: Option<(u8, Option<u16>, Vec<u8>)>,
    extra: Option<(u16, Vec<u8>)>,
}

fn ceremony() -> &'static common::Ceremony {
    static CEREMONY: OnceLock<common::Ceremony> = OnceLock::new();
    CEREMONY.get_or_init(common::run)
}

fuzz_target!(|input: Input| {
    let c = ceremony();
    let step = &c.steps[usize::from(input.step) % c.steps.len()];
    let mut incoming = step.incoming.clone();
    if let Some((sender, splice, bytes)) = input.payload {
        let senders: Vec<u16> = step.incoming.keys().copied().collect();
        if !senders.is_empty() {
            let j = senders[usize::from(sender) % senders.len()];
            let genuine = &step.incoming[&j];
            let payload = match splice {
                Some(at) if !genuine.is_empty() => {
                    let mut p = genuine.clone();
                    let at = usize::from(at) % p.len();
                    for (dst, src) in p[at..].iter_mut().zip(&bytes) {
                        *dst = *src;
                    }
                    p
                }
                _ => bytes,
            };
            incoming.insert(j, payload);
        }
    }
    let extra = (input.extra).filter(|(j, _)| !step.incoming.contains_key(j));
    if let Some((j, bytes)) = &extra {
        incoming.insert(*j, bytes.clone());
    }
    let result = step.party.advance(&incoming);
    if let Some((j, _)) = extra {
        assert!(
            matches!(result, Err(Refusal::Unexpected(k)) if k == j),
            "an extra sender not refused by name: {result:?}"
        );
    } else if incoming == step.incoming {
        assert!(result.is_ok(), "the genuine flight refused: {result:?}");
    }
    if let Ok((after, _)) = result {
        let bytes = after.encode();
        let again = Party::decode(&bytes).map(|p| p.encode());
        assert!(
            again == Some(bytes),
            "a state reached that does not re-decode"
        );
    }
});
