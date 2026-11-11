//! `isegoria-test-issuer` as a process: the issuer's key, its side of issuance by files, what is
//! refused, and a person's commands gone to the client (`docs/05` credentials before T19, D65).

use identity::credential::{
    AnonymousCredential, BlindSignature, Credential, IssuanceRequest, Issuer, IssuerPublic,
    PendingIssuance,
};
use identity::enrollment::Label;
use identity::nullifier;
use identity::nym::Role;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use test_issuer::{hex, issuer_from_hex, slot_label, slot_of};

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("test-issuer")
        .join(format!("{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn tool(args: &[&dyn AsRef<std::ffi::OsStr>]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_isegoria-test-issuer"));
    for arg in args {
        command.arg(arg);
    }
    command.output().unwrap()
}

fn succeeded(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn refused(output: &Output) {
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
}

#[cfg(unix)]
fn private(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path).unwrap().permissions().mode() & 0o777 == 0o600
}

#[cfg(not(unix))]
fn private(_: &Path) -> bool {
    true
}

/// A new issuer key in `dir`: its file and the hex it printed.
fn new_key(dir: &Path, name: &str) -> (PathBuf, String) {
    let key = dir.join(name);
    let out = succeeded(&tool(&[&"--new-key", &key]));
    let hex = out
        .strip_prefix("issuer ")
        .unwrap()
        .strip_suffix('\n')
        .unwrap();
    (key, hex.to_string())
}

fn fresh() -> [u8; 32] {
    let mut secret = [0; 32];
    getrandom::getrandom(&mut secret).unwrap();
    secret
}

/// A person's request for `slot` under `issuer`, made in process: its file and the pending state.
fn request(dir: &Path, issuer: &str, slot: u64, name: &str) -> (PathBuf, PendingIssuance) {
    let issuer = issuer_from_hex(issuer).unwrap();
    let (message, pending) =
        Credential::from_secret(fresh()).request_issuance(&slot_label(slot), &issuer);
    let path = dir.join(format!("{name}.req"));
    fs::write(&path, message.encode()).unwrap();
    (path, pending)
}

/// The person's credential from its pending state and the issuer's reply file.
fn finalize(pending: PendingIssuance, reply: &Path) -> AnonymousCredential {
    let signature = BlindSignature::decode(&fs::read(reply).unwrap()).expect("a reply");
    pending.finalize(signature)
}

/// The whole flow for `slot`: the issuer's reply by files and the person's credential.
fn obtain(dir: &Path, key: &Path, issuer: &str, slot: u64, name: &str) -> AnonymousCredential {
    let (message, pending) = request(dir, issuer, slot, name);
    let reply = dir.join(format!("{name}.reply"));
    assert_eq!(
        succeeded(&tool(&[&"--issue", &key, &message, &reply])),
        format!("slot {slot}\n")
    );
    finalize(pending, &reply)
}

fn public(issuer: &str) -> IssuerPublic {
    issuer_from_hex(issuer).unwrap()
}

// The issuer's seed is in a new 0600 file, its key printed as the genesis takes it (AT-ID-12).
#[test]
fn the_new_key_is_private_and_printed_for_the_genesis() {
    let dir = scratch("new-key");
    let (key, printed) = new_key(&dir, "issuer.key");
    assert!(private(&key));
    let seed = fs::read_to_string(&key).unwrap();
    assert_eq!(seed.len(), 64);
    let seed: [u8; 32] = (0..32)
        .map(|i| u8::from_str_radix(&seed[2 * i..2 * i + 2], 16).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    assert_eq!(printed, hex(&Issuer::new(seed).public().encode()));
    assert_eq!(printed.len(), 2 * 97);
    assert!(printed
        .bytes()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)));
    assert_eq!(issuer_from_hex(&printed).unwrap().encode().len(), 97);
    let (_, other) = new_key(&dir, "other.key");
    assert_ne!(printed, other);
}

// A new key never replaces an existing file, which stays as it was (AT-ID-12).
#[test]
fn a_new_key_never_replaces_a_file() {
    let dir = scratch("key-exists");
    let key = dir.join("issuer.key");
    fs::write(&key, "kept").unwrap();
    refused(&tool(&[&"--new-key", &key]));
    assert_eq!(fs::read_to_string(&key).unwrap(), "kept");
    refused(&tool(&[&"--new-key"]));
    refused(&tool(&[&"--new-key", &dir.join("a"), &dir.join("b")]));
}

// A slot's request by file gets a reply that makes a credential proving under the key (AT-ID-13).
#[test]
fn the_issuer_answers_a_request_by_files() {
    let dir = scratch("flow");
    let (key, issuer) = new_key(&dir, "issuer.key");
    let (message, pending) = request(&dir, &issuer, 7, "person");
    let sent = IssuanceRequest::decode(&fs::read(&message).unwrap()).expect("a request");
    assert_eq!(sent.label(), &slot_label(7));
    let reply = dir.join("person.reply");
    assert_eq!(
        succeeded(&tool(&[&"--issue", &key, &message, &reply])),
        "slot 7\n"
    );
    let held = finalize(pending, &reply);
    assert!(held.verify(&public(&issuer)));
    let proof = nullifier::prove(&held, &public(&issuer), Role::Respond, b"context");
    assert!(nullifier::verify(&proof, &public(&issuer), b"context"));
    let (_, stranger) = new_key(&dir, "stranger.key");
    assert!(!held.verify(&public(&stranger)));
    assert!(!nullifier::verify(&proof, &public(&stranger), b"context"));
}

// A slot's label is its number in 8 little-endian bytes, then zeros, and nothing else (AT-ID-13).
#[test]
fn a_slot_is_an_opaque_number() {
    let mut expected = [0u8; 32];
    expected[..2].copy_from_slice(&[0x02, 0x01]);
    assert_eq!(slot_label(0x0102).0, expected);
    assert_eq!(slot_label(u64::MAX).0[..8], [0xff; 8]);
    for slot in [0, 7, u64::MAX] {
        assert_eq!(slot_of(&slot_label(slot)), Some(slot));
    }
    let mut wider = slot_label(7);
    wider.0[31] = 1;
    assert_eq!(slot_of(&wider), None);
}

// The issuer's reply never replaces a file, which stays as it was (AT-ID-13).
#[test]
fn no_reply_replaces_a_file() {
    let dir = scratch("exists");
    let (key, issuer) = new_key(&dir, "issuer.key");
    let kept = dir.join("kept");
    fs::write(&kept, "kept").unwrap();
    let (message, _) = request(&dir, &issuer, 1, "person");
    refused(&tool(&[&"--issue", &key, &message, &kept]));
    assert_eq!(fs::read_to_string(&kept).unwrap(), "kept");
    succeeded(&tool(&[&"--issue", &key, &message, &dir.join("reply")]));
}

// The issuer signs any slot's valid request again and again: no uniqueness, no limit (AT-ID-14).
#[test]
fn the_test_issuer_mints_without_limit() {
    let dir = scratch("unlimited");
    let (key, issuer) = new_key(&dir, "issuer.key");
    let ids: Vec<_> = (0..3)
        .map(|n| {
            let held = obtain(&dir, &key, &issuer, 7, &format!("copy{n}"));
            assert!(held.verify(&public(&issuer)));
            nullifier::prove(&held, &public(&issuer), Role::Judge, b"context").id()
        })
        .collect();
    assert!(ids[0] != ids[1] && ids[1] != ids[2] && ids[0] != ids[2]);
}

// A label that is no slot, an altered request and another kind's file get no reply (AT-ID-14).
#[test]
fn the_issuer_signs_only_a_slots_valid_request() {
    let dir = scratch("issuer-refuses");
    let (key, issuer) = new_key(&dir, "issuer.key");
    let reply = dir.join("reply");
    let (message, _) =
        Credential::from_secret([3; 32]).request_issuance(&Label([4; 32]), &public(&issuer));
    let not_a_slot = dir.join("not-a-slot.req");
    fs::write(&not_a_slot, message.encode()).unwrap();
    refused(&tool(&[&"--issue", &key, &not_a_slot, &reply]));
    let (message, pending) = request(&dir, &issuer, 2, "person");
    let mut altered = fs::read(&message).unwrap();
    let last = altered.len() - 1;
    altered[last] ^= 1;
    let altered_path = dir.join("altered.req");
    fs::write(&altered_path, altered).unwrap();
    refused(&tool(&[&"--issue", &key, &altered_path, &reply]));
    let pending_path = dir.join("person.pending");
    fs::write(&pending_path, pending.encode()).unwrap();
    refused(&tool(&[&"--issue", &key, &pending_path, &reply]));
    refused(&tool(&[&"--issue", &pending_path, &message, &reply]));
    assert!(!reply.exists());
}

// A person's `--request` and `--finalize` are no issuer command: nothing written (AT-T79-43).
#[test]
fn at_t79_43_a_persons_commands_left_the_issuer() {
    let dir = scratch("moved");
    let (key, issuer) = new_key(&dir, "issuer.key");
    let (pending, message) = (dir.join("p"), dir.join("r"));
    let out = tool(&[&"--request", &issuer, &"1", &pending, &message]);
    refused(&out);
    assert!(String::from_utf8_lossy(&out.stderr).contains("isegoria-client"));
    assert!(!pending.exists() && !message.exists());
    let (sent, state) = request(&dir, &issuer, 1, "person");
    let reply = dir.join("person.reply");
    succeeded(&tool(&[&"--issue", &key, &sent, &reply]));
    fs::write(&pending, state.encode()).unwrap();
    let credential = dir.join("person.credential");
    refused(&tool(&[
        &"--finalize",
        &issuer,
        &pending,
        &reply,
        &credential,
    ]));
    assert!(!credential.exists());
}
