//! Signed key configurations (`docs/04` §Blind relays, `consortium.hpke`): arbitrary bytes or a
//! genuine configuration with a window overwritten. No panic; one encoding; only the genuine
//! one verifies, and a client seals exactly under a verified one.
#![no_main]

use libfuzzer_sys::fuzz_target;
use network::consortium::{Consortium, Member};
use network::replica::FeedWriter;
use network::tier::Tiers;
use p2p::blind::{seal, GatewayKey, KeyAnnouncement};
use std::sync::OnceLock;

const NET: [u8; 32] = [6; 32];

fn fixture() -> &'static (Tiers, KeyAnnouncement) {
    static FIXTURE: OnceLock<(Tiers, KeyAnnouncement)> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let members: Vec<Member> = (1..=3).map(|i| Member::from_seed([i; 32])).collect();
        let circle: Vec<_> = (4..=6)
            .map(|i| FeedWriter::from_seed(NET, [i; 32]).public())
            .collect();
        let keys = members.iter().map(|m| m.public()).collect();
        let tiers = Tiers::new(NET, Consortium::new(keys, 2), &circle).expect("a tiered set");
        let key = GatewayKey::new(&tiers, &members[1], 1, &[7; 32]).expect("a gateway key");
        let announcement = key.announcement().clone();
        (tiers, announcement)
    })
}

fuzz_target!(|input: (bool, u16, Vec<u8>, Vec<u8>)| {
    let (splice, at, bytes, event) = input;
    let (tiers, genuine) = fixture();
    let candidate = if splice {
        let mut c = genuine.encode();
        let at = usize::from(at) % c.len();
        for (dst, src) in c[at..].iter_mut().zip(&bytes) {
            *dst = *src;
        }
        c
    } else {
        bytes
    };
    let Some(announcement) = KeyAnnouncement::decode(&candidate) else {
        return;
    };
    assert_eq!(
        announcement.encode(),
        candidate,
        "decoded from another encoding"
    );
    let verified = announcement.verify(tiers);
    if verified {
        assert_eq!(announcement, *genuine, "a modified configuration verified");
        assert!(announcement.hpke_key().is_some());
    }
    let _ = announcement.hpke_key();
    assert_eq!(seal(tiers, &announcement, &event).is_ok(), verified);
});
