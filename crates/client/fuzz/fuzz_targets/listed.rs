//! The key configuration and relay address a person copies from other operators (`isegoria-client
//! --send`, `--read`): arbitrary text, or a genuine one with a window overwritten. No panic; only
//! a member's own signed configuration with its genesis key, and only a listed relay, are taken.
#![no_main]

#[path = "../common/network.rs"]
mod common;

use arbitrary::Arbitrary;
use client::event::{announcement, relay};
use client::{hex, Network};
use libfuzzer_sys::fuzz_target;
use libp2p::multiaddr::Protocol;
use libp2p::PeerId;
use p2p::blind::GatewayKey;
use std::sync::OnceLock;

struct Fixture {
    net: Network,
    /// Each member's configuration in hex, as `isegoria-node --announce` prints it.
    announcements: Vec<String>,
    relay: PeerId,
    address: String,
}

fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let net = common::network(&common::issuer().public());
        let announcements = (0..common::N)
            .map(|i| {
                let key = GatewayKey::new(&net.tiers, &common::member(i), i, &common::hpke_seed(i));
                hex(&key.expect("a gateway key").announcement().encode())
            })
            .collect();
        let relay = common::relay_peer();
        Fixture {
            net,
            announcements,
            address: format!("/ip4/127.0.0.1/tcp/4001/p2p/{relay}"),
            relay,
        }
    })
}

#[derive(Arbitrary, Debug)]
enum Input {
    Announcement(Option<(u8, u16)>, String),
    Relay(Option<u16>, String),
}

/// `genuine` with its characters from `at` overwritten by `text`'s.
fn spliced(genuine: &str, at: u16, text: &str) -> String {
    let mut c: Vec<char> = genuine.chars().collect();
    let at = usize::from(at) % c.len();
    for (dst, src) in c[at..].iter_mut().zip(text.chars()) {
        *dst = src;
    }
    c.into_iter().collect()
}

fuzz_target!(|input: Input| {
    let f = fixture();
    match input {
        Input::Announcement(splice, text) => {
            let text = match splice {
                Some((which, at)) => {
                    let genuine = &f.announcements[usize::from(which) % f.announcements.len()];
                    spliced(genuine, at, &text)
                }
                None => text,
            };
            let taken = announcement(&f.net, &text);
            if f.announcements.contains(&text) {
                assert!(taken.is_ok(), "a member's own configuration refused");
            }
            if let Ok(a) = taken {
                let genuine = hex(&a.encode());
                assert!(
                    f.announcements.contains(&genuine),
                    "another configuration taken"
                );
            }
        }
        Input::Relay(splice, text) => {
            let text = match splice {
                Some(at) => spliced(&f.address, at, &text),
                None => text,
            };
            let taken = relay(&f.net, &text);
            if text == f.address {
                assert!(taken.is_ok(), "the listed relay refused");
            }
            if let Ok(address) = taken {
                let last = address.iter().last();
                assert_eq!(
                    last,
                    Some(Protocol::P2p(f.relay)),
                    "an unlisted relay taken"
                );
            }
        }
    }
});
