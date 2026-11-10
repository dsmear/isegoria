//! What a member's gateway accepts (`docs/04` §Blind relays): arbitrary encapsulated requests
//! from a relay, arbitrary inner requests, and events sealed as a client seals them. No panic;
//! a sealed event opens and reads back as itself, for this network only.
#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use network::consortium::{Consortium, Member};
use network::replica::FeedWriter;
use network::tier::Tiers;
use p2p::blind::{seal, Answer, GatewayKey};
use p2p::fuzz::{gateway_open, gateway_read};
use std::sync::OnceLock;

const NET: [u8; 32] = [2; 32];

fn fixture() -> &'static (Tiers, GatewayKey) {
    static FIXTURE: OnceLock<(Tiers, GatewayKey)> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let members: Vec<Member> = (1..=3).map(|i| Member::from_seed([i; 32])).collect();
        let circle: Vec<_> = (4..=6)
            .map(|i| FeedWriter::from_seed(NET, [i; 32]).public())
            .collect();
        let keys = members.iter().map(|m| m.public()).collect();
        let tiers = Tiers::new(NET, Consortium::new(keys, 2), &circle).expect("a tiered set");
        let key = GatewayKey::new(&tiers, &members[0], 0, &[7; 32]).expect("a gateway key");
        (tiers, key)
    })
}

#[derive(Arbitrary, Debug)]
enum Input {
    Request(Vec<u8>),
    Inner(Vec<u8>),
    Event(Vec<u8>),
}

fuzz_target!(|input: Input| {
    let (tiers, key) = fixture();
    match input {
        Input::Request(bytes) => {
            if let Ok(inner) = gateway_open(key, &bytes) {
                let _ = gateway_read(&inner, &NET);
            }
        }
        Input::Inner(bytes) => {
            let _ = gateway_read(&bytes, &NET);
        }
        Input::Event(event) => {
            let sealed = seal(tiers, key.announcement(), &event).expect("a verified key seals");
            let inner = gateway_open(key, &sealed.request).expect("a sealed request opens");
            assert_eq!(gateway_read(&inner, &NET), Ok(event));
            assert_eq!(gateway_read(&inner, &[9; 32]), Err(Answer::OtherNetwork));
        }
    }
});
