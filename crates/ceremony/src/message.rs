//! A ceremony message (`docs/03` T19-P3): one strict encoding, signed by the sender's consortium
//! key under a tag of its own over the id, step, sender and recipient, then sealed to the
//! recipient's HPKE key (RFC 9180 base mode). A **bespoke** composition of audited primitives.

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use hpke::{Deserializable, Kem as _, OpModeR, OpModeS, Serializable};
use network::codec::{Reader, Writer};
use network::hash::tagged;

pub const VERSION: u8 = 1;
pub const HEADER: usize = 36;
pub const SIGNED: &str = "isegoria/ceremony/message/v1";
pub const SEAL_INFO: &[u8] = b"isegoria/ceremony/seal/v1";
/// The steps that carry messages: 1 the fresh values to 13 the confirmations.
pub const STEPS: std::ops::RangeInclusive<u8> = 1..=13;

type Kem = hpke::kem::X25519HkdfSha256;
type Kdf = hpke::kdf::HkdfSha256;
type Aead = hpke::aead::AesGcm128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub id: [u8; 32],
    pub step: u8,
    pub sender: u8,
    pub recipient: u8,
}

impl Header {
    /// Version 1, the id, the step, the sender's and the recipient's member indices.
    pub fn encode(&self) -> [u8; HEADER] {
        let mut out = [0u8; HEADER];
        out[0] = VERSION;
        out[1..33].copy_from_slice(&self.id);
        out[33..].copy_from_slice(&[self.step, self.sender, self.recipient]);
        out
    }

    fn decode(bytes: &[u8; HEADER]) -> Result<Self, MessageError> {
        if bytes[0] != VERSION {
            return Err(MessageError::Version);
        }
        Ok(Header {
            id: bytes[1..33].try_into().expect("32 bytes"),
            step: bytes[33],
            sender: bytes[34],
            recipient: bytes[35],
        })
    }
}

/// Why bytes are not a message for this member; the caller checks the id and the step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageError {
    Malformed,
    Version,
    Recipient,
    Sender,
    Step,
    Seal,
    Signature,
}

/// The digest the sender signs: tagged under [`SIGNED`] over the header, then the payload.
pub fn signed_digest(header: &Header, payload: &[u8]) -> [u8; 32] {
    tagged(SIGNED, &[&header.encode(), payload])
}

/// A member's HPKE private key.
pub struct HpkeSecret(<Kem as hpke::Kem>::PrivateKey);

/// The pair the 32 bytes of `hpke.key` make (RFC 9180 `DeriveKeyPair`), as `--new-identity`'s.
pub fn hpke_pair(ikm: &[u8; 32]) -> (HpkeSecret, [u8; 32]) {
    let (secret, public) = Kem::derive_keypair(ikm);
    let bytes = public.to_bytes();
    let public = bytes
        .as_slice()
        .try_into()
        .expect("an X25519 key is 32 bytes");
    (HpkeSecret(secret), public)
}

/// The file for `payload`: the header, the encapsulated key (32 bytes), then the sealed signature
/// and payload as a field; `None` if `recipient` is not a key the library takes.
pub fn seal(
    header: &Header,
    payload: &[u8],
    key: &SigningKey,
    recipient: &[u8; 32],
) -> Option<Vec<u8>> {
    let public = <Kem as hpke::Kem>::PublicKey::from_bytes(recipient).ok()?;
    let signature = key.sign(&signed_digest(header, payload));
    let mut plaintext = signature.to_bytes().to_vec();
    plaintext.extend_from_slice(payload);
    let aad = header.encode();
    let (enc, sealed) = hpke::single_shot_seal::<Aead, Kdf, Kem>(
        &OpModeS::Base,
        &public,
        SEAL_INFO,
        &plaintext,
        &aad,
    )
    .ok()?;
    let mut w = Writer::new();
    w.fixed(&aad)
        .fixed(enc.to_bytes().as_slice())
        .field(&sealed);
    Some(w.finish())
}

/// The header and payload of `bytes` if they are one message to member `me` from another of
/// `members` (consortium keys), opening under `secret`, its signature verifying strictly.
pub fn open(
    bytes: &[u8],
    me: u8,
    secret: &HpkeSecret,
    members: &[[u8; 32]],
) -> Result<(Header, Vec<u8>), MessageError> {
    let malformed = |_| MessageError::Malformed;
    let mut r = Reader::new(bytes);
    let aad: [u8; HEADER] = r.fixed().map_err(malformed)?;
    let enc: [u8; 32] = r.fixed().map_err(malformed)?;
    let sealed = r.field().map_err(malformed)?;
    r.finish().map_err(malformed)?;
    let header = Header::decode(&aad)?;
    if header.recipient != me {
        return Err(MessageError::Recipient);
    }
    let sender = members
        .get(usize::from(header.sender))
        .filter(|_| header.sender != me)
        .ok_or(MessageError::Sender)?;
    if !STEPS.contains(&header.step) {
        return Err(MessageError::Step);
    }
    let encapped =
        <Kem as hpke::Kem>::EncappedKey::from_bytes(&enc).map_err(|_| MessageError::Seal)?;
    let plaintext = hpke::single_shot_open::<Aead, Kdf, Kem>(
        &OpModeR::Base,
        &secret.0,
        &encapped,
        SEAL_INFO,
        sealed,
        &aad,
    )
    .map_err(|_| MessageError::Seal)?;
    if plaintext.len() < Signature::BYTE_SIZE {
        return Err(MessageError::Signature);
    }
    let (signature, payload) = plaintext.split_at(Signature::BYTE_SIZE);
    let signature = Signature::from_bytes(signature.try_into().expect("64 bytes"));
    let key = VerifyingKey::from_bytes(sender).map_err(|_| MessageError::Signature)?;
    key.verify_strict(&signed_digest(&header, payload), &signature)
        .map_err(|_| MessageError::Signature)?;
    Ok((header, payload.to_vec()))
}
