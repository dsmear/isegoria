//! The single-key test issuer before T19 and a person's side of its blind issuance, by files
//! carried by hand (`docs/05` §Distributed integration, credentials before T19).

use identity::credential::{
    AnonymousCredential, BlindSignature, Credential, IssuanceError, IssuanceRequest, Issuer,
    IssuerPublic, PendingIssuance,
};
use identity::enrollment::Label;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// The named file could not be read or written; an output that exists is `AlreadyExists`.
    Io(&'static str, io::ErrorKind),
    /// The named file or argument is not its kind's one encoding.
    Malformed(&'static str),
    /// The request's label is no slot's.
    NotASlot,
    Issuance(IssuanceError),
    /// The finished credential does not verify under the issuer key given.
    Unverified,
}

/// The bound on a file read here: the largest, a request, is 220 bytes.
const MAX_FILE: u64 = 1024;

/// A slot's label: the slot number in 8 little-endian bytes, then 24 zero bytes.
pub fn slot_label(slot: u64) -> Label {
    let mut label = [0u8; 32];
    label[..8].copy_from_slice(&slot.to_le_bytes());
    Label(label)
}

/// The slot whose label `label` is, if any.
pub fn slot_of(label: &Label) -> Option<u64> {
    let (number, rest) = label.0.split_first_chunk::<8>()?;
    rest.iter()
        .all(|&b| b == 0)
        .then(|| u64::from_le_bytes(*number))
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    let text = text.trim();
    if text.len() % 2 != 0 || !text.is_ascii() {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
        .collect()
}

/// The issuer key in hex, as `--genesis-assemble`'s TOML takes it (`docs/04` T88-P1).
pub fn issuer_from_hex(text: &str) -> Result<IssuerPublic, Error> {
    unhex(text)
        .and_then(|bytes| IssuerPublic::decode(&bytes))
        .ok_or(Error::Malformed("issuer"))
}

fn random() -> Result<[u8; 32], Error> {
    let mut seed = [0u8; 32];
    getrandom::getrandom(&mut seed).map_err(|_| Error::Io("randomness", io::ErrorKind::Other))?;
    Ok(seed)
}

fn create(path: &Path, private: bool, what: &'static str) -> Result<File, Error> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    if private {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    #[cfg(not(unix))]
    let _ = private;
    options.open(path).map_err(|e| Error::Io(what, e.kind()))
}

fn fill(mut file: File, path: &Path, bytes: &[u8], what: &'static str) -> Result<(), Error> {
    let written = file.write_all(bytes).and_then(|()| file.sync_all());
    written.map_err(|e| {
        let _ = fs::remove_file(path);
        Error::Io(what, e.kind())
    })
}

/// Writes `bytes` to `path`, which must not exist; 0600 when `private`.
fn write_new(path: &Path, bytes: &[u8], private: bool, what: &'static str) -> Result<(), Error> {
    fill(create(path, private, what)?, path, bytes, what)
}

fn read(path: &Path, what: &'static str) -> Result<Vec<u8>, Error> {
    let io = |e: io::Error| Error::Io(what, e.kind());
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(io)?
        .take(MAX_FILE + 1)
        .read_to_end(&mut bytes)
        .map_err(io)?;
    match bytes.len() as u64 > MAX_FILE {
        true => Err(Error::Malformed(what)),
        false => Ok(bytes),
    }
}

fn decoded<T>(
    bytes: &[u8],
    decode: fn(&[u8]) -> Option<T>,
    what: &'static str,
) -> Result<T, Error> {
    decode(bytes).ok_or(Error::Malformed(what))
}

/// A new issuer key: its seed, 64 hex digits, in `key`, a new 0600 file.
pub fn new_key(key: &Path) -> Result<IssuerPublic, Error> {
    let seed = random()?;
    write_new(key, hex(&seed).as_bytes(), true, "issuer key")?;
    Ok(Issuer::new(seed).public())
}

fn load_issuer(key: &Path) -> Result<Issuer, Error> {
    let text = String::from_utf8(read(key, "issuer key")?);
    let seed = text
        .ok()
        .and_then(|t| unhex(&t))
        .and_then(|s| s.try_into().ok());
    seed.map(Issuer::new).ok_or(Error::Malformed("issuer key"))
}

/// The issuer's side: blind-signs a slot's request into `reply`, a new file; returns the slot.
/// Any valid request is signed, as often as it comes: no uniqueness, no limit.
pub fn issue(key: &Path, request: &Path, reply: &Path) -> Result<u64, Error> {
    let issuer = load_issuer(key)?;
    let request = decoded(
        &read(request, "request")?,
        IssuanceRequest::decode,
        "request",
    )?;
    let slot = slot_of(request.label()).ok_or(Error::NotASlot)?;
    let signature = issuer.issue(&request).map_err(Error::Issuance)?;
    write_new(reply, &signature.encode(), false, "reply")?;
    Ok(slot)
}

/// A person's side: a fresh secret's request for `slot` into `request`, its state into
/// `pending`, a 0600 file; both must be new, and neither is left if one cannot be made.
pub fn request(
    issuer: &IssuerPublic,
    slot: u64,
    pending: &Path,
    request: &Path,
) -> Result<(), Error> {
    let holder = Credential::from_secret(random()?);
    let (message, state) = holder.request_issuance(&slot_label(slot), issuer);
    let state_file = create(pending, true, "pending")?;
    let message_file = create(request, false, "request").inspect_err(|_| {
        let _ = fs::remove_file(pending);
    })?;
    fill(state_file, pending, &state.encode(), "pending")
        .and_then(|()| fill(message_file, request, &message.encode(), "request"))
        .inspect_err(|_| {
            let _ = fs::remove_file(pending);
            let _ = fs::remove_file(request);
        })
}

/// A person's side: the credential from `pending` and `reply` into `credential`, a new 0600
/// file, only if it verifies under `issuer`.
pub fn finalize(
    issuer: &IssuerPublic,
    pending: &Path,
    reply: &Path,
    credential: &Path,
) -> Result<(), Error> {
    let state = decoded(
        &read(pending, "pending")?,
        PendingIssuance::decode,
        "pending",
    )?;
    let signature = decoded(&read(reply, "reply")?, BlindSignature::decode, "reply")?;
    let finished = state.finalize(signature);
    if !finished.verify(issuer) {
        return Err(Error::Unverified);
    }
    write_new(credential, &finished.encode(), true, "credential")
}

/// A person's credential file, decoded strictly and verified under `issuer`.
pub fn read_credential(path: &Path, issuer: &IssuerPublic) -> Result<AnonymousCredential, Error> {
    let credential = decoded(
        &read(path, "credential")?,
        AnonymousCredential::decode,
        "credential",
    )?;
    match credential.verify(issuer) {
        true => Ok(credential),
        false => Err(Error::Unverified),
    }
}
