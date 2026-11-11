//! The network of the client's targets, as `tests/support` builds it: four members, `t = 3`,
//! one relay, under a test issuer, decoded as the client decodes its genesis.
#![allow(dead_code)]

use client::Network;
use identity::credential::{Issuer, IssuerPublic};
use libp2p::PeerId;
use network::consortium::Member;
use network::genesis::{Genesis, GenesisMember};
use protocol::answer_path::AnswerConfig;

pub const N: usize = 4;
const RELAY: [u8; 32] = [40; 32];

pub fn member(i: usize) -> Member {
    Member::from_seed([10 + i as u8; 32])
}

pub fn hpke_seed(i: usize) -> [u8; 32] {
    [30 + i as u8; 32]
}

pub fn issuer() -> Issuer {
    Issuer::new([77; 32])
}

pub fn relay_peer() -> PeerId {
    let key = Member::from_seed(RELAY).public().to_bytes();
    let key = libp2p::identity::ed25519::PublicKey::try_from_bytes(&key).expect("a key");
    PeerId::from_public_key(&key.into())
}

pub fn network(issuer: &IssuerPublic) -> Network {
    let members = (0..N)
        .map(|i| GenesisMember {
            consortium: member(i).public().to_bytes(),
            circle: Member::from_seed([20 + i as u8; 32]).public().to_bytes(),
            hpke: p2p::blind::public_key(&hpke_seed(i)).expect("an HPKE key"),
        })
        .collect();
    let genesis = Genesis {
        threshold: 3,
        members,
        answer_config: AnswerConfig::default().encode(),
        issuer: issuer.encode(),
        relays: vec![Member::from_seed(RELAY).public().to_bytes()],
    };
    Network::decode(&genesis.encode()).unwrap_or_else(|_| panic!("the genesis decodes"))
}
