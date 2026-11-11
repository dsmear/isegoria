//! A person's credential by files with `isegoria-client`: from the test issuer (D65) and from
//! the committee's signers (`docs/03` T19-P5); the files new, private, strict (`docs/05`).

mod support;

use client::credential::{read_credential, slot_label};
use client::event::Commitment;
use client::Error;
use identity::credential::ceremony::{self, Party, Setup, SignatureShare, SlotRequest, FLIGHTS};
use identity::credential::{AnonymousCredential, IssuanceRequest, IssuerPublic};
use identity::nullifier;
use identity::nym::Role;
use protocol::answer::Answer;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use support::*;

/// The test issuer's new key in `dir` (its own command's function) and a genesis naming it.
fn issuer_key(dir: &Path, name: &str) -> (PathBuf, IssuerPublic, PathBuf) {
    let key = dir.join(name);
    let public = test_issuer::new_key(&key).unwrap();
    let genesis = dir.join(format!("{name}.genesis"));
    fs::write(&genesis, support::genesis(&public).encode()).unwrap();
    (key, public, genesis)
}

fn request(dir: &Path, issuer: &Path, slot: &str, name: &str) -> (PathBuf, PathBuf) {
    let (pending, message) = (
        dir.join(format!("{name}.pending")),
        dir.join(format!("{name}.req")),
    );
    let out = succeeded(&tool(&[&"--request", &issuer, &slot, &pending, &message]));
    assert_eq!(out, format!("slot {slot}\n"));
    (pending, message)
}

// The client's request is the test issuer's; its credential 0600, proving (AT-T79-43).
#[test]
fn at_t79_43_a_credential_from_the_test_issuer_by_files() {
    let dir = scratch("test-issuer");
    let (key, public, issuer) = issuer_key(&dir, "issuer.key");
    let (pending, message) = request(&dir, &issuer, "7", "person");
    assert!(private(&pending));
    let sent = IssuanceRequest::decode(&fs::read(&message).unwrap()).expect("a request");
    assert_eq!(sent.label(), &slot_label(7));
    assert_eq!(slot_label(7), test_issuer::slot_label(7));
    let reply = dir.join("person.reply");
    assert_eq!(test_issuer::issue(&key, &message, &reply), Ok(7));
    let credential = dir.join("person.credential");
    let args: [&dyn AsRef<std::ffi::OsStr>; 5] =
        [&"--finalize", &issuer, &pending, &reply, &credential];
    assert_eq!(succeeded(&tool(&args)), "");
    assert!(private(&credential));
    let held = read_credential(&credential, &public).expect("a valid credential");
    let bytes = fs::read(&credential).unwrap();
    assert_eq!(AnonymousCredential::decode(&bytes).unwrap().encode(), bytes);
    let proof = nullifier::prove(&held, &public, Role::Judge, b"context");
    assert!(nullifier::verify(&proof, &public, b"context"));
    let (_, stranger, stranger_genesis) = issuer_key(&dir, "stranger.key");
    assert_eq!(
        read_credential(&credential, &stranger).unwrap_err(),
        Error::Unverified
    );
    let other = dir.join("other.credential");
    let err = refused(&tool(&[
        &"--finalize",
        &stranger_genesis,
        &pending,
        &reply,
        &other,
    ]));
    assert!(err.contains("Unverified"), "{err}");
    assert!(!other.exists());
    for slot in ["slot", "+7", "07", "-1", "18446744073709551616"] {
        let p = dir.join("p");
        refused(&tool(&[&"--request", &issuer, &slot, &p, &dir.join("r")]));
        assert!(!p.exists());
    }
    refused(&tool(&[
        &"--request",
        &"00",
        &"1",
        &dir.join("p"),
        &dir.join("r"),
    ]));
}

// No output over a file, no pair half left, each kind decoded only as itself (AT-T79-42).
#[test]
fn at_t79_42_the_files_are_new_private_and_strict() {
    let dir = scratch("files");
    let (key, _, issuer) = issuer_key(&dir, "issuer.key");
    let kept = dir.join("kept");
    fs::write(&kept, "kept").unwrap();
    let fresh = dir.join("fresh");
    refused(&tool(&[&"--request", &issuer, &"1", &kept, &fresh]));
    assert!(!fresh.exists());
    refused(&tool(&[&"--request", &issuer, &"1", &fresh, &kept]));
    assert!(!fresh.exists(), "the pending file is not left behind");
    let (pending, message) = request(&dir, &issuer, "1", "person");
    let reply = dir.join("person.reply");
    test_issuer::issue(&key, &message, &reply).unwrap();
    refused(&tool(&[&"--finalize", &issuer, &pending, &reply, &kept]));
    refused(&tool(&[&"--finalize", &issuer, &message, &reply, &fresh]));
    refused(&tool(&[&"--finalize", &issuer, &pending, &message, &fresh]));
    let mut longer = fs::read(&pending).unwrap();
    longer.push(0);
    let longer_path = dir.join("longer.pending");
    fs::write(&longer_path, longer).unwrap();
    refused(&tool(&[
        &"--finalize",
        &issuer,
        &longer_path,
        &reply,
        &fresh,
    ]));
    assert!(!fresh.exists());
    assert_eq!(fs::read_to_string(&kept).unwrap(), "kept");

    let (genesis, _, credential, _) = person(&dir);
    let opened = dir.join("vote.commitment");
    refused(&tool(&[
        &"--commit",
        &genesis,
        &credential,
        &"3",
        &"1",
        &kept,
        &fresh,
    ]));
    assert!(!fresh.exists());
    refused(&tool(&[
        &"--commit",
        &genesis,
        &credential,
        &"3",
        &"1",
        &opened,
        &kept,
    ]));
    assert!(!opened.exists(), "the commitment is not left behind");
    let event = dir.join("vote.event");
    succeeded(&tool(&[
        &"--commit",
        &genesis,
        &credential,
        &"3",
        &"1",
        &opened,
        &event,
    ]));
    assert!(private(&opened) && private(&event));
    let bytes = fs::read(&opened).unwrap();
    assert_eq!(bytes.len(), 25 + 8 + 2 + 32);
    assert!(bytes.starts_with(b"isegoria/client/commit/v1"));
    let read = Commitment::decode(&bytes).unwrap();
    assert_eq!((read.question, read.answer), (3, Answer::Alternative(1)));
    assert_eq!(read.encode(), bytes);
    for n in 0..bytes.len() {
        assert_eq!(Commitment::decode(&bytes[..n]), None);
    }
    assert_eq!(Commitment::decode(&[&bytes[..], &[0]].concat()), None);
    let mut code = bytes.clone();
    code[33] = 3;
    assert_eq!(Commitment::decode(&code), None);
    refused(&tool(&[&"--reveal", &genesis, &credential, &opened, &kept]));
    let mut tagged = bytes.clone();
    tagged[0] ^= 1;
    fs::write(dir.join("bad.commitment"), tagged).unwrap();
    let revealed = dir.join("reveal.event");
    let bad = dir.join("bad.commitment");
    refused(&tool(&[
        &"--reveal",
        &genesis,
        &credential,
        &bad,
        &revealed,
    ]));
    refused(&tool(&[
        &"--reveal",
        &genesis,
        &pending,
        &opened,
        &revealed,
    ]));
    assert!(!revealed.exists());
    succeeded(&tool(&[
        &"--reveal",
        &genesis,
        &credential,
        &opened,
        &revealed,
    ]));
    assert!(private(&revealed));
}

/// The four members' parties after the ceremony's 13 flights, in process.
fn prepared(setup: &Setup) -> Vec<Party> {
    let mut parties: Vec<Party> = (0..4)
        .map(|i| Party::new(setup.clone(), i, [7; 32]).unwrap())
        .collect();
    let mut inbox: BTreeMap<u16, BTreeMap<u16, Vec<u8>>> = BTreeMap::new();
    for _ in 2..=FLIGHTS {
        let mut next: BTreeMap<u16, BTreeMap<u16, Vec<u8>>> = BTreeMap::new();
        for p in parties.iter_mut() {
            let incoming = inbox.remove(&p.me()).unwrap_or_default();
            let (after, out) = p.advance(&incoming).unwrap();
            for (to, bytes) in out {
                next.entry(to).or_default().insert(after.me(), bytes);
            }
            *p = after;
        }
        inbox = next;
    }
    parties
}

/// Each named signer's share of the request in `request`, written to `dir/<name>-<member>`.
fn shares(dir: &Path, parties: &[Party], by: &[u16], request: &Path, name: &str) -> Vec<PathBuf> {
    let request = SlotRequest::decode(&fs::read(request).unwrap()).expect("a slot request");
    by.iter()
        .map(|&i| {
            let share = parties[usize::from(i)].answer(&request).unwrap();
            let path = dir.join(format!("{name}-{i}"));
            fs::write(&path, share.encode()).unwrap();
            path
        })
        .collect()
}

fn combine(genesis: &Path, pending: &Path, credential: &Path, shares: &[PathBuf]) -> String {
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_isegoria-client"));
    command
        .arg("--combine")
        .arg(genesis)
        .arg(pending)
        .arg(credential);
    let out = command.args(shares).output().unwrap();
    match out.status.success() {
        true => String::from_utf8(out.stdout).unwrap(),
        false => {
            assert_eq!(out.status.code(), Some(2));
            format!("refused {}", String::from_utf8_lossy(&out.stderr))
        }
    }
}

// Three signers' shares make a 0600 credential under the genesis's key, no fewer (AT-T79-44).
#[test]
fn at_t79_44_a_credential_from_the_committees_shares() {
    let dir = scratch("committee");
    let setup = Setup::new(4, 3, vec![0, 2, 3], vec![5, 9]).unwrap();
    let parties = prepared(&setup);
    let issuer = parties[0].issuer().unwrap();
    let (genesis, _) = written(&dir, &support::genesis(&issuer));
    let slot = |n: &str, name: &str| {
        let (pending, message) = (
            dir.join(format!("{name}.pending")),
            dir.join(format!("{name}.req")),
        );
        let out = succeeded(&tool(&[
            &"--slot-request",
            &genesis,
            &n,
            &pending,
            &message,
        ]));
        assert_eq!(out, format!("slot {n}\n"));
        assert!(private(&pending));
        (pending, message)
    };
    let (pending, message) = slot("5", "five");
    let sent = SlotRequest::decode(&fs::read(&message).unwrap()).unwrap();
    assert_eq!(sent.slot(), 5);
    assert_eq!(ceremony::slot_label(5), slot_label(5));
    let three = shares(&dir, &parties, &[0, 2, 3], &message, "five");
    let credential = dir.join("five.credential");
    let refusal = |s: &[PathBuf], why: &str| {
        let out = combine(&genesis, &pending, &credential, s);
        assert!(out.starts_with("refused") && out.contains(why), "{out}");
        assert!(!credential.exists());
    };
    refusal(&three[..2], "Signature");
    refusal(
        &[three[0].clone(), three[0].clone(), three[1].clone()],
        "Shares",
    );
    let (_, nine) = slot("9", "nine");
    let other = shares(&dir, &parties, &[0, 2, 3], &nine, "nine");
    refusal(&other, "Slot");
    refusal(
        &[three[0].clone(), three[1].clone(), other[2].clone()],
        "Slot",
    );
    let mut altered = fs::read(&three[2]).unwrap();
    let last = altered.len() - 1;
    altered[last] ^= 1;
    let altered_path = dir.join("altered-share");
    fs::write(&altered_path, &altered).unwrap();
    refusal(
        &[three[0].clone(), three[1].clone(), altered_path],
        "refused",
    );
    refusal(
        &[three[0].clone(), three[1].clone(), message.clone()],
        "Malformed",
    );
    let (stranger, _) = written(
        &scratch("stranger"),
        &support::genesis(&test_issuer().public()),
    );
    let out = combine(&stranger, &pending, &credential, &three);
    assert!(out.contains("Signature"), "{out}");
    assert!(!credential.exists());
    let kept = dir.join("kept");
    fs::write(&kept, "kept").unwrap();
    assert!(combine(&genesis, &pending, &kept, &three).starts_with("refused"));
    assert_eq!(fs::read_to_string(&kept).unwrap(), "kept");

    assert_eq!(combine(&genesis, &pending, &credential, &three), "");
    assert!(private(&credential));
    let held = read_credential(&credential, &issuer).expect("it verifies under the genesis's key");
    let proof = nullifier::prove(&held, &issuer, Role::Propose, b"context");
    assert!(nullifier::verify(&proof, &issuer, b"context"));
    assert!(read_credential(&credential, &test_issuer().public()).is_err());
    let decoded: Vec<SignatureShare> = three
        .iter()
        .map(|p| SignatureShare::decode(&fs::read(p).unwrap()).unwrap())
        .collect();
    assert_eq!(
        decoded
            .iter()
            .map(SignatureShare::member)
            .collect::<Vec<_>>(),
        [0, 2, 3]
    );
    refused(&tool(&[
        &"--slot-request",
        &genesis,
        &"4294967296",
        &dir.join("p"),
        &dir.join("r"),
    ]));
    refused(&tool(&[&"--combine", &genesis, &pending, &dir.join("c")]));
}
