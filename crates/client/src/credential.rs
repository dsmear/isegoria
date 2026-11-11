//! A person's credential by files: from the provisional test issuer (`docs/05` credentials
//! before T19, `docs/01` D65) or from the committee's three signers (`docs/03` T19-P5).

use crate::{decoded, random, read, write_new, Error, Output, SMALL};
use identity::credential::ceremony::{self, SignatureShare, SlotRequest};
use identity::credential::{
    AnonymousCredential, BlindSignature, Credential, IssuerPublic, PendingIssuance,
};
use identity::enrollment::Label;
use std::path::{Path, PathBuf};

/// The test issuer's label of `slot`: the number in 8 little-endian bytes, then 24 zero bytes.
pub fn slot_label(slot: u64) -> Label {
    let mut label = [0u8; 32];
    label[..8].copy_from_slice(&slot.to_le_bytes());
    Label(label)
}

fn private(path: &Path, bytes: &[u8], what: &'static str) -> Result<(), Error> {
    write_new(&[Output(path, bytes, true, what)])
}

/// A fresh secret's request to the test issuer for `slot` into `request`, its state into
/// `pending`, a 0600 file; both must be new, and neither is left if one cannot be made.
pub fn request(
    issuer: &IssuerPublic,
    slot: u64,
    pending: &Path,
    request: &Path,
) -> Result<(), Error> {
    let holder = Credential::from_secret(random()?);
    let (message, state) = holder.request_issuance(&slot_label(slot), issuer);
    write_new(&[
        Output(pending, &state.encode(), true, "pending"),
        Output(request, &message.encode(), false, "request"),
    ])
}

fn pending_state(pending: &Path) -> Result<PendingIssuance, Error> {
    let bytes = read(pending, SMALL, "pending")?;
    decoded(&bytes, PendingIssuance::decode, "pending")
}

/// The credential from `pending` and the test issuer's `reply` into `credential`, a new 0600
/// file, only if it verifies under `issuer`.
pub fn finalize(
    issuer: &IssuerPublic,
    pending: &Path,
    reply: &Path,
    credential: &Path,
) -> Result<(), Error> {
    let state = pending_state(pending)?;
    let reply = read(reply, SMALL, "reply")?;
    let signature = decoded(&reply, BlindSignature::decode, "reply")?;
    let finished = state.finalize(signature);
    if !finished.verify(issuer) {
        return Err(Error::Unverified);
    }
    private(credential, &finished.encode(), "credential")
}

/// A fresh secret's request for listed `slot`, the one file each of the three signers answers,
/// and its state into `pending`, a 0600 file; as [`request`] for the files.
pub fn slot_request(
    issuer: &IssuerPublic,
    slot: u32,
    pending: &Path,
    request: &Path,
) -> Result<(), Error> {
    let holder = Credential::from_secret(random()?);
    let (message, state) = SlotRequest::new(&holder, slot, issuer);
    write_new(&[
        Output(pending, &state.encode(), true, "pending"),
        Output(request, &message.encode(), false, "request"),
    ])
}

/// The credential the signers' `shares` of `pending`'s slot make into `credential`, a new 0600
/// file, only once it verifies under `issuer`.
pub fn combine(
    issuer: &IssuerPublic,
    pending: &Path,
    shares: &[PathBuf],
    credential: &Path,
) -> Result<(), Error> {
    let state = pending_state(pending)?;
    let shares = (shares.iter())
        .map(|path| {
            decoded(
                &read(path, SMALL, "share")?,
                SignatureShare::decode,
                "share",
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let finished = ceremony::combine(state, &shares, issuer).map_err(Error::Combine)?;
    private(credential, &finished.encode(), "credential")
}

/// A person's credential file, decoded strictly and verified under `issuer`.
pub fn read_credential(path: &Path, issuer: &IssuerPublic) -> Result<AnonymousCredential, Error> {
    let bytes = read(path, SMALL, "credential")?;
    let credential = decoded(&bytes, AnonymousCredential::decode, "credential")?;
    match credential.verify(issuer) {
        true => Ok(credential),
        false => Err(Error::Unverified),
    }
}
