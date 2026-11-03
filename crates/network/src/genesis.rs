//! A network's genesis (`docs/04` §Operating contract, T88-P1): the configuration the operators
//! confirm, its one encoding and strict decoding, its digest and the network id it derives.

use crate::codec::{DecodeError, Reader, Writer};
use crate::consortium::Consortium;
use crate::hash::tagged;
use crate::tier::Tiers;
use ed25519_dalek::VerifyingKey;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub const VERSION: u8 = 1;
/// The answer review v1, the one event family (`docs/05` T79-P1).
pub const ANSWER_REVIEW_V1: u8 = 1;
/// A member's HPKE key id is its index in one byte (T85-P3).
pub const MAX_MEMBERS: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GenesisMember {
    pub consortium: [u8; 32],
    pub circle: [u8; 32],
    /// Its X25519 public key; its signed key configuration follows from it and its index.
    pub hpke: [u8; 32],
}

/// The members in order; the family's parts as opaque bytes their embedding validates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Genesis {
    pub threshold: usize,
    pub members: Vec<GenesisMember>,
    pub answer_config: Vec<u8>,
    pub issuer: Vec<u8>,
    /// The relays' transport keys, strictly increasing.
    pub relays: Vec<[u8; 32]>,
}

/// Why bytes are not one genesis document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenesisError {
    /// Truncated, or bytes after the document.
    Malformed,
    Version,
    /// No member, or more than [`MAX_MEMBERS`].
    Members,
    Threshold,
    Family,
    /// A key that is not one canonical encoding of an Ed25519 point or an X25519 coordinate.
    Key,
    /// A key twice among the consortium and circle keys, the HPKE keys or the relays.
    Repeated,
    /// Relays not in increasing order.
    Order,
}

impl From<DecodeError> for GenesisError {
    fn from(_: DecodeError) -> Self {
        GenesisError::Malformed
    }
}

/// The key, if `bytes` are the canonical encoding of an Ed25519 point.
fn ed25519(bytes: &[u8; 32]) -> Option<VerifyingKey> {
    let key = VerifyingKey::from_bytes(bytes).ok()?;
    (key.to_edwards().compress().as_bytes() == bytes).then_some(key)
}

/// Whether little-endian `bytes` are below `2^255 − 19`.
fn x25519(bytes: &[u8; 32]) -> bool {
    let mut p = [0xff; 32];
    p[0] = 0xed;
    p[31] = 0x7f;
    bytes.iter().rev().lt(p.iter().rev())
}

impl Genesis {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(VERSION)
            .u64(self.threshold as u64)
            .u64(self.members.len() as u64);
        for m in &self.members {
            w.fixed(&m.consortium).fixed(&m.circle).fixed(&m.hpke);
        }
        w.u8(ANSWER_REVIEW_V1)
            .field(&self.answer_config)
            .field(&self.issuer)
            .u64(self.relays.len() as u64);
        for relay in &self.relays {
            w.fixed(relay);
        }
        w.finish()
    }

    /// The document `bytes` are exactly the encoding of, checked whole.
    pub fn decode(bytes: &[u8]) -> Result<Genesis, GenesisError> {
        let mut r = Reader::new(bytes);
        if r.u8()? != VERSION {
            return Err(GenesisError::Version);
        }
        let threshold = r.u64()?;
        let n = r.u64()?;
        if n == 0 || n > MAX_MEMBERS as u64 {
            return Err(GenesisError::Members);
        }
        if threshold == 0 || threshold > n {
            return Err(GenesisError::Threshold);
        }
        let mut members = Vec::new();
        for _ in 0..n {
            members.push(GenesisMember {
                consortium: r.fixed()?,
                circle: r.fixed()?,
                hpke: r.fixed()?,
            });
        }
        if r.u8()? != ANSWER_REVIEW_V1 {
            return Err(GenesisError::Family);
        }
        let answer_config = r.field()?.to_vec();
        let issuer = r.field()?.to_vec();
        let mut relays = Vec::new();
        for _ in 0..r.u64()? {
            relays.push(r.fixed()?);
        }
        r.finish()?;
        let genesis = Genesis {
            threshold: threshold as usize,
            members,
            answer_config,
            issuer,
            relays,
        };
        genesis.check()?;
        Ok(genesis)
    }

    pub fn check(&self) -> Result<(), GenesisError> {
        let n = self.members.len();
        if n == 0 || n > MAX_MEMBERS {
            return Err(GenesisError::Members);
        }
        if !(1..=n).contains(&self.threshold) {
            return Err(GenesisError::Threshold);
        }
        let mut signing = BTreeSet::new();
        let mut hpke = BTreeSet::new();
        for m in &self.members {
            for key in [&m.consortium, &m.circle] {
                ed25519(key).ok_or(GenesisError::Key)?;
                if !signing.insert(*key) {
                    return Err(GenesisError::Repeated);
                }
            }
            if !x25519(&m.hpke) {
                return Err(GenesisError::Key);
            }
            if !hpke.insert(m.hpke) {
                return Err(GenesisError::Repeated);
            }
        }
        for relay in &self.relays {
            ed25519(relay).ok_or(GenesisError::Key)?;
        }
        for pair in self.relays.windows(2) {
            if pair[0] == pair[1] {
                return Err(GenesisError::Repeated);
            }
            if pair[0] > pair[1] {
                return Err(GenesisError::Order);
            }
        }
        Ok(())
    }

    /// The SHA-256 of the encoding: the file's digest, which identifies it without the code.
    pub fn digest(&self) -> [u8; 32] {
        Sha256::digest(self.encode()).into()
    }

    /// The network id: the tagged SHA-256 of the encoding under `isegoria/genesis/v1`.
    pub fn id(&self) -> [u8; 32] {
        tagged("isegoria/genesis/v1", &[&self.encode()])
    }

    /// The members' consortium and circle keys as the tiered set under [`Genesis::id`].
    pub fn tiers(&self) -> Result<Tiers, GenesisError> {
        self.check()?;
        let key = |k: &[u8; 32]| ed25519(k).ok_or(GenesisError::Key);
        let members = self.members.iter().map(|m| key(&m.consortium));
        let circle = self.members.iter().map(|m| key(&m.circle));
        let consortium = Consortium::new(members.collect::<Result<_, _>>()?, self.threshold);
        let circle = circle.collect::<Result<Vec<_>, _>>()?;
        Tiers::new(self.id(), consortium, &circle).map_err(|_| GenesisError::Repeated)
    }
}
