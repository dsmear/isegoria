//! The binding exchange (`docs/04` §Transport, `/isegoria/bind/1`): arbitrary messages or a
//! genuine hello with a window overwritten, and arbitrary peer ids. No panic; one encoding;
//! only the genuine binding, from its own transport key, grants the circle.
#![no_main]

use libfuzzer_sys::fuzz_target;
use libp2p::PeerId;
use network::consortium::{Consortium, Member};
use network::replica::FeedWriter;
use network::tier::{Tiers, TransportBinding};
use p2p::tiered::{transport_key, Bind};
use std::sync::OnceLock;

const NET: [u8; 32] = [5; 32];
const TRANSPORT: [u8; 32] = [8; 32];

fn fixture() -> &'static (Tiers, TransportBinding) {
    static FIXTURE: OnceLock<(Tiers, TransportBinding)> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let members: Vec<Member> = (1..=3).map(|i| Member::from_seed([i; 32])).collect();
        let circle: Vec<_> = (4..=6)
            .map(|i| FeedWriter::from_seed(NET, [i; 32]).public())
            .collect();
        let keys = members.iter().map(|m| m.public()).collect();
        let tiers = Tiers::new(NET, Consortium::new(keys, 2), &circle).expect("a tiered set");
        let binding = tiers.bind(&members[1], 1, TRANSPORT);
        (tiers, binding)
    })
}

fuzz_target!(|input: (bool, u16, Vec<u8>, bool)| {
    let (splice, at, bytes, other_transport) = input;
    let (tiers, genuine) = fixture();
    if let Ok(peer) = PeerId::from_bytes(&bytes) {
        let _ = transport_key(&peer);
    }
    let candidate = if splice {
        let mut c = Bind::Hello(Some(*genuine)).encode();
        let at = usize::from(at) % c.len();
        for (dst, src) in c[at..].iter_mut().zip(&bytes) {
            *dst = *src;
        }
        c
    } else {
        bytes
    };
    let Some(bind) = Bind::decode(&candidate) else {
        return;
    };
    assert_eq!(
        bind.encode(),
        candidate,
        "a message decoded from another encoding"
    );
    let (Bind::Hello(binding) | Bind::Reply { binding, .. }) = bind;
    let transport = if other_transport { [9; 32] } else { TRANSPORT };
    if tiers
        .member(tiers.access(&transport, binding.as_ref()))
        .is_some()
    {
        assert!(
            binding == Some(*genuine) && !other_transport,
            "a forged binding granted"
        );
    }
});
