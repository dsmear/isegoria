//! A ceremony member's state (`docs/03` §Committee setup for the trial, the state file): arbitrary
//! bytes or a genuine state at any flight with a window overwritten. No panic; one encoding.
#![no_main]

#[path = "../common/ceremony.rs"]
mod common;

use identity::credential::ceremony::Party;
use libfuzzer_sys::fuzz_target;
use std::sync::OnceLock;

fn states() -> &'static Vec<Vec<u8>> {
    static STATES: OnceLock<Vec<Vec<u8>>> = OnceLock::new();
    STATES.get_or_init(|| {
        let c = common::run();
        let before = c.steps.iter().map(|s| &s.party);
        before.chain(&c.prepared).map(Party::encode).collect()
    })
}

fuzz_target!(|input: (Option<(u8, u32)>, Vec<u8>)| {
    let (splice, bytes) = input;
    let states = states();
    let candidate = match splice {
        Some((which, at)) => {
            let mut c = states[usize::from(which) % states.len()].clone();
            let at = (at as usize) % c.len();
            for (dst, src) in c[at..].iter_mut().zip(&bytes) {
                *dst = *src;
            }
            c
        }
        None => bytes,
    };
    if let Some(party) = Party::decode(&candidate) {
        assert_eq!(party.encode(), candidate, "a state from another encoding");
        let _ = (party.issuer(), party.is_prepared(), party.next());
    }
});
