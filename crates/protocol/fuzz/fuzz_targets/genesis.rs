//! A genesis document as a node loads it (`docs/04` T88-P1): arbitrary bytes or a genuine
//! document with a window overwritten, then its answer configuration and issuer key. No
//! panic; one encoding each; a decoded document yields its tiered set.
#![no_main]

use curve25519_dalek::MontgomeryPoint;
use identity::credential::{Issuer, IssuerPublic};
use libfuzzer_sys::fuzz_target;
use network::consortium::Member;
use network::genesis::{Genesis, GenesisMember};
use protocol::answer_path::AnswerConfig;
use std::sync::OnceLock;

fn key(seed: u8) -> [u8; 32] {
    Member::from_seed([seed; 32]).public().to_bytes()
}

fn genuine() -> &'static Vec<u8> {
    static GENUINE: OnceLock<Vec<u8>> = OnceLock::new();
    GENUINE.get_or_init(|| {
        let members = (0..4)
            .map(|i| GenesisMember {
                consortium: key(10 + i),
                circle: key(20 + i),
                hpke: MontgomeryPoint::mul_base_clamped([30 + i; 32]).to_bytes(),
            })
            .collect();
        Genesis {
            threshold: 3,
            members,
            answer_config: AnswerConfig::default().encode(),
            issuer: Issuer::new([1; 32]).public().encode(),
            relays: vec![key(40)],
        }
        .encode()
    })
}

fuzz_target!(|input: (Option<u16>, Vec<u8>)| {
    let (splice, bytes) = input;
    if let Some(config) = AnswerConfig::decode(&bytes) {
        assert_eq!(
            config.encode(),
            bytes,
            "a configuration from another encoding"
        );
    }
    let candidate = match splice {
        Some(at) => {
            let mut c = genuine().clone();
            let at = usize::from(at) % c.len();
            for (dst, src) in c[at..].iter_mut().zip(&bytes) {
                *dst = *src;
            }
            c
        }
        None => bytes,
    };
    let Ok(genesis) = Genesis::decode(&candidate) else {
        return;
    };
    assert_eq!(
        genesis.encode(),
        candidate,
        "a document from another encoding"
    );
    let tiers = genesis
        .tiers()
        .expect("a decoded document yields its tiered set");
    assert_eq!(tiers.network_id(), genesis.id());
    if let Some(config) = AnswerConfig::decode(&genesis.answer_config) {
        assert_eq!(config.encode(), genesis.answer_config);
    }
    if let Some(issuer) = IssuerPublic::decode(&genesis.issuer) {
        assert_eq!(issuer.encode(), genesis.issuer);
    }
});
