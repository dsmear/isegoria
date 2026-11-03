//! A tiered network's genesis for the node tests (`docs/04` T88-P1): its document under the test
//! issuer and D49–D51's configuration, written to a file with the TOML lines repeating it.
#![allow(dead_code)]

use identity::credential::Issuer;
use network::consortium::Member;
use network::genesis::{Genesis, GenesisMember};
use protocol::answer_path::AnswerConfig;
use std::path::Path;
use std::sync::OnceLock;

/// The test issuer's encoded key.
pub fn issuer() -> Vec<u8> {
    static KEY: OnceLock<Vec<u8>> = OnceLock::new();
    KEY.get_or_init(|| Issuer::new([77; 32]).public().encode())
        .clone()
}

pub fn key(seed: u8) -> [u8; 32] {
    Member::from_seed([seed; 32]).public().to_bytes()
}

pub fn hpke(seed: u8) -> [u8; 32] {
    p2p::blind::public_key(&[seed; 32]).unwrap()
}

/// The member whose consortium, circle and HPKE keys come from these seeds.
pub fn seeded(consortium: u8, circle: u8, hpke_seed: u8) -> GenesisMember {
    GenesisMember {
        consortium: key(consortium),
        circle: key(circle),
        hpke: hpke(hpke_seed),
    }
}

pub fn genesis(
    threshold: usize,
    members: Vec<GenesisMember>,
    mut relays: Vec<[u8; 32]>,
) -> Genesis {
    relays.sort();
    Genesis {
        threshold,
        members,
        answer_config: AnswerConfig::default().encode(),
        issuer: issuer(),
        relays,
    }
}

/// The TOML's top-level lines naming the document at `path` and repeating its family's parts.
pub fn lines(path: &Path, genesis: &Genesis) -> String {
    format!(
        "genesis = \"{}\"\nfamily = \"answer-review/v1\"\nanswer_config = \"{}\"\nissuer = \"{}\"\n",
        path.display(),
        node::hex(&genesis.answer_config),
        node::hex(&genesis.issuer)
    )
}

/// Writes `genesis` at `path` and returns [`lines`].
pub fn write(path: &Path, genesis: &Genesis) -> String {
    std::fs::write(path, genesis.encode()).unwrap();
    lines(path, genesis)
}
