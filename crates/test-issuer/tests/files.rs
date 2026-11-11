//! `isegoria-test-issuer` as a process: the issuer's key, issuance to a person by files, what
//! is refused (`docs/05` §Distributed integration, credentials before T19, AT-ID-12–14).

use identity::credential::{AnonymousCredential, Credential, IssuanceRequest, Issuer};
use identity::enrollment::Label;
use identity::nullifier;
use identity::nym::Role;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use test_issuer::{hex, issuer_from_hex, read_credential, slot_label, slot_of};

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

/// A person's request for `slot`: the pending state and the request files.
fn request(dir: &Path, issuer: &str, slot: u64, name: &str) -> (PathBuf, PathBuf) {
    let (pending, message) = (
        dir.join(format!("{name}.pending")),
        dir.join(format!("{name}.req")),
    );
    let slot = slot.to_string();
    let out = succeeded(&tool(&[&"--request", &issuer, &slot, &pending, &message]));
    assert_eq!(out, format!("slot {slot}\n"));
    (pending, message)
}

/// The whole flow for `slot`: the issuer's reply and the person's credential file.
fn obtain(dir: &Path, key: &Path, issuer: &str, slot: u64, name: &str) -> PathBuf {
    let (pending, message) = request(dir, issuer, slot, name);
    let reply = dir.join(format!("{name}.reply"));
    assert_eq!(
        succeeded(&tool(&[&"--issue", &key, &message, &reply])),
        format!("slot {slot}\n")
    );
    let credential = dir.join(format!("{name}.credential"));
    assert_eq!(
        succeeded(&tool(&[
            &"--finalize",
            &issuer,
            &pending,
            &reply,
            &credential
        ])),
        ""
    );
    credential
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

// A person obtains by files a 0600 credential that verifies and proves under the key (AT-ID-13).
#[test]
fn a_person_obtains_a_credential_by_files() {
    let dir = scratch("flow");
    let (key, issuer) = new_key(&dir, "issuer.key");
    let (pending, message) = request(&dir, &issuer, 7, "person");
    assert!(private(&pending));
    let sent = IssuanceRequest::decode(&fs::read(&message).unwrap()).expect("a request");
    assert_eq!(sent.label(), &slot_label(7));
    let reply = dir.join("person.reply");
    assert_eq!(
        succeeded(&tool(&[&"--issue", &key, &message, &reply])),
        "slot 7\n"
    );
    let credential = dir.join("person.credential");
    assert_eq!(
        succeeded(&tool(&[
            &"--finalize",
            &issuer,
            &pending,
            &reply,
            &credential
        ])),
        ""
    );
    assert!(private(&credential));
    let public = issuer_from_hex(&issuer).unwrap();
    let held = read_credential(&credential, &public).expect("a valid credential");
    let bytes = fs::read(&credential).unwrap();
    assert_eq!(AnonymousCredential::decode(&bytes).unwrap().encode(), bytes);
    let proof = nullifier::prove(&held, &public, Role::Respond, b"context");
    assert!(nullifier::verify(&proof, &public, b"context"));
    let (_, stranger) = new_key(&dir, "stranger.key");
    let stranger = issuer_from_hex(&stranger).unwrap();
    assert!(read_credential(&credential, &stranger).is_err());
    assert!(!nullifier::verify(&proof, &stranger, b"context"));
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

// No command writes over an existing file, and a refused request leaves no file (AT-ID-13).
#[test]
fn no_output_replaces_a_file() {
    let dir = scratch("exists");
    let (key, issuer) = new_key(&dir, "issuer.key");
    let kept = dir.join("kept");
    fs::write(&kept, "kept").unwrap();
    let fresh = dir.join("fresh");
    refused(&tool(&[&"--request", &issuer, &"1", &kept, &fresh]));
    assert!(!fresh.exists());
    refused(&tool(&[&"--request", &issuer, &"1", &fresh, &kept]));
    assert!(!fresh.exists(), "the pending file is not left behind");
    let (pending, message) = request(&dir, &issuer, 1, "person");
    refused(&tool(&[&"--issue", &key, &message, &kept]));
    let reply = dir.join("person.reply");
    succeeded(&tool(&[&"--issue", &key, &message, &reply]));
    refused(&tool(&[&"--finalize", &issuer, &pending, &reply, &kept]));
    assert_eq!(fs::read_to_string(&kept).unwrap(), "kept");
}

// The issuer signs any slot's valid request again and again: no uniqueness, no limit (AT-ID-14).
#[test]
fn the_test_issuer_mints_without_limit() {
    let dir = scratch("unlimited");
    let (key, issuer) = new_key(&dir, "issuer.key");
    let public = issuer_from_hex(&issuer).unwrap();
    let ids: Vec<_> = (0..3)
        .map(|n| {
            let credential = obtain(&dir, &key, &issuer, 7, &format!("copy{n}"));
            let held = read_credential(&credential, &public).unwrap();
            nullifier::prove(&held, &public, Role::Judge, b"context").id()
        })
        .collect();
    assert!(ids[0] != ids[1] && ids[1] != ids[2] && ids[0] != ids[2]);
}

// A label that is no slot, an altered request and another kind's file get no reply (AT-ID-14).
#[test]
fn the_issuer_signs_only_a_slots_valid_request() {
    let dir = scratch("issuer-refuses");
    let (key, issuer) = new_key(&dir, "issuer.key");
    let public = issuer_from_hex(&issuer).unwrap();
    let reply = dir.join("reply");
    let (message, _) = Credential::from_secret([3; 32]).request_issuance(&Label([4; 32]), &public);
    let not_a_slot = dir.join("not-a-slot.req");
    fs::write(&not_a_slot, message.encode()).unwrap();
    refused(&tool(&[&"--issue", &key, &not_a_slot, &reply]));
    let (pending, message) = request(&dir, &issuer, 2, "person");
    let mut altered = fs::read(&message).unwrap();
    let last = altered.len() - 1;
    altered[last] ^= 1;
    let altered_path = dir.join("altered.req");
    fs::write(&altered_path, altered).unwrap();
    refused(&tool(&[&"--issue", &key, &altered_path, &reply]));
    refused(&tool(&[&"--issue", &key, &pending, &reply]));
    refused(&tool(&[&"--issue", &pending, &message, &reply]));
    assert!(!reply.exists());
}

// A reply from another issuer, or the wrong key at the end, yields no credential (AT-ID-14).
#[test]
fn the_person_keeps_only_a_credential_that_verifies() {
    let dir = scratch("person-refuses");
    let (_, issuer) = new_key(&dir, "issuer.key");
    let (other_key, other) = new_key(&dir, "other.key");
    let (pending, message) = request(&dir, &issuer, 3, "person");
    let reply = dir.join("other.reply");
    succeeded(&tool(&[&"--issue", &other_key, &message, &reply]));
    let credential = dir.join("person.credential");
    refused(&tool(&[
        &"--finalize",
        &issuer,
        &pending,
        &reply,
        &credential,
    ]));
    assert!(!credential.exists());
    refused(&tool(&[
        &"--finalize",
        &"00",
        &pending,
        &reply,
        &credential,
    ]));
    refused(&tool(&[
        &"--finalize",
        &other,
        &message,
        &reply,
        &credential,
    ]));
    refused(&tool(&[
        &"--request",
        &issuer,
        &"slot",
        &dir.join("p"),
        &dir.join("r"),
    ]));
    assert!(!credential.exists() && !dir.join("p").exists());
}
