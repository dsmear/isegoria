//! A network of four members, `t = 3`, one relay, for the client's tests: its genesis on disk,
//! members' key configurations, credentials, and the client run as a process.
#![allow(dead_code)]

use client::Network;
use identity::credential::{AnonymousCredential, Credential, Issuer, IssuerPublic};
use libp2p::PeerId;
use network::consortium::Member;
use network::genesis::{Genesis, GenesisMember};
use p2p::blind::{GatewayKey, KeyAnnouncement};
use protocol::answer_path::AnswerConfig;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub const N: usize = 4;
pub const RELAY: [u8; 32] = [40; 32];

pub fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("client")
        .join(format!("{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

pub fn tool(args: &[&dyn AsRef<std::ffi::OsStr>]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_isegoria-client"));
    for arg in args {
        command.arg(arg);
    }
    command.output().unwrap()
}

pub fn succeeded(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    String::from_utf8(output.stdout.clone()).unwrap()
}

/// Exit 2, nothing printed on stdout; the error line on stderr.
pub fn refused(output: &Output) -> String {
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[cfg(unix)]
pub fn private(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path).unwrap().permissions().mode() & 0o777 == 0o600
}

#[cfg(not(unix))]
pub fn private(_: &Path) -> bool {
    true
}

pub fn member(i: usize) -> Member {
    Member::from_seed([10 + i as u8; 32])
}

fn hpke_seed(i: usize) -> [u8; 32] {
    [30 + i as u8; 32]
}

pub fn relay_key() -> [u8; 32] {
    Member::from_seed(RELAY).public().to_bytes()
}

pub fn peer_of(key: [u8; 32]) -> PeerId {
    let key = libp2p::identity::ed25519::PublicKey::try_from_bytes(&key).unwrap();
    PeerId::from_public_key(&key.into())
}

pub fn relay_peer() -> PeerId {
    peer_of(relay_key())
}

pub fn test_issuer() -> Issuer {
    Issuer::new([77; 32])
}

/// The genesis of four members, `t = 3`, one relay, under `issuer`.
pub fn genesis(issuer: &IssuerPublic) -> Genesis {
    let members = (0..N)
        .map(|i| GenesisMember {
            consortium: member(i).public().to_bytes(),
            circle: Member::from_seed([20 + i as u8; 32]).public().to_bytes(),
            hpke: p2p::blind::public_key(&hpke_seed(i)).unwrap(),
        })
        .collect();
    Genesis {
        threshold: 3,
        members,
        answer_config: AnswerConfig::default().encode(),
        issuer: issuer.encode(),
        relays: vec![relay_key()],
    }
}

/// `genesis` written to `dir/genesis` and loaded as the client loads it.
pub fn written(dir: &Path, genesis: &Genesis) -> (PathBuf, Network) {
    let path = dir.join("genesis");
    fs::write(&path, genesis.encode()).unwrap();
    (path.clone(), Network::load(&path).unwrap())
}

/// Member `i`'s signed key configuration under `net`, from its HPKE seed `seed`.
pub fn announcement_with(net: &Network, i: usize, seed: [u8; 32]) -> KeyAnnouncement {
    let key = GatewayKey::new(&net.tiers, &member(i), i, &seed).unwrap();
    key.announcement().clone()
}

pub fn announcement(net: &Network, i: usize) -> KeyAnnouncement {
    announcement_with(net, i, hpke_seed(i))
}

/// A credential for a fresh secret from `issuer`, issued in process.
pub fn credential(issuer: &Issuer, slot: u64) -> AnonymousCredential {
    let mut secret = [0; 32];
    getrandom::getrandom(&mut secret).unwrap();
    let label = client::credential::slot_label(slot);
    let (request, pending) =
        Credential::from_secret(secret).request_issuance(&label, &issuer.public());
    pending.finalize(issuer.issue(&request).unwrap())
}

/// The test issuer's network on disk and a person's credential file for it.
pub fn person(dir: &Path) -> (PathBuf, Network, PathBuf, AnonymousCredential) {
    let issuer = test_issuer();
    let (path, net) = written(dir, &genesis(&issuer.public()));
    let held = credential(&issuer, 1);
    let file = dir.join("person.credential");
    fs::write(&file, held.encode()).unwrap();
    (path, net, file, held)
}
