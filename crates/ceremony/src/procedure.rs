//! The ceremony's procedure (`docs/03` T19-P1, P3, P5): the committee's keys in genesis order,
//! the threshold, the named signers and the slot list, read from a TOML; its digests and its id.

use crate::unhex;
use curve25519_dalek::montgomery::MontgomeryPoint;
use ed25519_dalek::VerifyingKey;
use identity::credential::ceremony::Setup;
use network::codec::{Reader, Writer};
use network::hash::tagged;
use serde::Deserialize;
use std::collections::BTreeSet;

const VERSION: u8 = 1;

/// A member's keys as `--new-identity` printed them: its consortium key and its HPKE key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Keys {
    pub consortium: [u8; 32],
    pub hpke: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Procedure {
    /// The operators' number for this ceremony of the committee, a new one after an abort.
    pub ceremony: u64,
    pub threshold: u16,
    pub members: Vec<Keys>,
    pub signers: Vec<u16>,
    pub slots: Vec<u32>,
}

/// Why a procedure is refused: its TOML, the field holding no valid key (`member[1].hpke`), a
/// key repeated (named at its later place), or a shape [`Setup::new`] refuses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProcedureError {
    Parse(String),
    Key(String),
    Repeated(String),
    Setup,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Text {
    ceremony: u64,
    threshold: u16,
    signers: Vec<u16>,
    slots: Vec<u32>,
    #[serde(default)]
    member: Vec<Printed>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Printed {
    consortium: String,
    hpke: String,
}

fn ed25519(bytes: &[u8; 32]) -> bool {
    VerifyingKey::from_bytes(bytes)
        .is_ok_and(|key| key.to_edwards().compress().as_bytes() == bytes && !key.is_weak())
}

fn x25519(bytes: &[u8; 32]) -> bool {
    let mut p = [0xff; 32];
    p[0] = 0xed;
    p[31] = 0x7f;
    let zero = MontgomeryPoint(*bytes).mul_clamped([1; 32]) == MontgomeryPoint([0; 32]);
    bytes.iter().rev().lt(p.iter().rev()) && !zero
}

impl Procedure {
    /// The procedure a TOML describes: `ceremony`, `threshold`, `signers` (member indices),
    /// `slots`, one `[[member]]` per member in genesis order with its `consortium` and `hpke` hex.
    pub fn parse(text: &str) -> Result<Self, ProcedureError> {
        let t: Text = toml::from_str(text).map_err(|e| ProcedureError::Parse(e.to_string()))?;
        let mut members = Vec::new();
        for (i, m) in t.member.iter().enumerate() {
            let key = |text: &str, role| {
                unhex(text).ok_or_else(|| ProcedureError::Key(format!("member[{i}].{role}")))
            };
            members.push(Keys {
                consortium: key(&m.consortium, "consortium")?,
                hpke: key(&m.hpke, "hpke")?,
            });
        }
        let procedure = Procedure {
            ceremony: t.ceremony,
            threshold: t.threshold,
            members,
            signers: t.signers,
            slots: t.slots,
        };
        procedure.check()?;
        Ok(procedure)
    }

    /// Every key valid and distinct, then the shape the ceremony runs.
    pub fn check(&self) -> Result<(), ProcedureError> {
        let mut seen = BTreeSet::new();
        let roles = |(i, m): (usize, &Keys)| [(m.consortium, i, "consortium"), (m.hpke, i, "hpke")];
        for (key, i, role) in self.members.iter().enumerate().flat_map(roles) {
            let valid = if role == "hpke" {
                x25519(&key)
            } else {
                ed25519(&key)
            };
            if !valid {
                return Err(ProcedureError::Key(format!("member[{i}].{role}")));
            }
            if !seen.insert(key) {
                return Err(ProcedureError::Repeated(format!("member[{i}].{role}")));
            }
        }
        self.setup().map(|_| ()).ok_or(ProcedureError::Setup)
    }

    pub fn setup(&self) -> Option<Setup> {
        let n = u16::try_from(self.members.len()).ok()?;
        Setup::new(n, self.threshold, self.signers.clone(), self.slots.clone())
    }

    /// The index of the member holding `keys`, if one does.
    pub fn member(&self, keys: &Keys) -> Option<u8> {
        let i = self.members.iter().position(|m| m == keys)?;
        u8::try_from(i).ok()
    }

    pub fn consortium_keys(&self) -> Vec<[u8; 32]> {
        self.members.iter().map(|m| m.consortium).collect()
    }

    /// Tagged under `isegoria/ceremony/slots/v1`: the count (8 bytes), then each slot (4 bytes).
    pub fn slots_digest(&self) -> [u8; 32] {
        let list: Vec<u8> = self.slots.iter().flat_map(|s| s.to_le_bytes()).collect();
        let count = (self.slots.len() as u64).to_le_bytes();
        tagged("isegoria/ceremony/slots/v1", &[&count, &list])
    }

    fn committee(&self) -> Vec<Vec<u8>> {
        let mut fields = vec![
            self.ceremony.to_le_bytes().to_vec(),
            u64::from(self.threshold).to_le_bytes().to_vec(),
            (self.members.len() as u64).to_le_bytes().to_vec(),
        ];
        for m in &self.members {
            fields.push(m.consortium.to_vec());
            fields.push(m.hpke.to_vec());
        }
        fields.push(self.slots_digest().to_vec());
        fields
    }

    /// What step 1's messages are bound to before the id exists: the id's fields but the fresh
    /// values, under `isegoria/ceremony/opening/v1`.
    pub fn opening(&self) -> [u8; 32] {
        let fields = self.committee();
        let refs: Vec<&[u8]> = fields.iter().map(Vec::as_slice).collect();
        tagged("isegoria/ceremony/opening/v1", &refs)
    }

    /// The ceremony's id (`docs/03` T19-P3), under `isegoria/ceremony/id/v1`: the number, `t`, `n`
    /// (8 bytes each), each member's consortium then HPKE key, the slots' digest, each fresh value.
    pub fn id(&self, fresh: &[[u8; 32]]) -> [u8; 32] {
        let mut fields = self.committee();
        fields.extend(fresh.iter().map(|f| f.to_vec()));
        let refs: Vec<&[u8]> = fields.iter().map(Vec::as_slice).collect();
        tagged("isegoria/ceremony/id/v1", &refs)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(VERSION)
            .u64(self.ceremony)
            .u64(u64::from(self.threshold))
            .u64(self.members.len() as u64);
        for m in &self.members {
            w.fixed(&m.consortium).fixed(&m.hpke);
        }
        w.u64(self.signers.len() as u64);
        for &s in &self.signers {
            w.u64(u64::from(s));
        }
        w.u64(self.slots.len() as u64);
        for &s in &self.slots {
            w.u64(u64::from(s));
        }
        w.finish()
    }

    /// `None` unless `bytes` are exactly one encoding of a procedure [`Procedure::check`] takes.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader::new(bytes);
        if r.u8().ok()? != VERSION {
            return None;
        }
        let ceremony = r.u64().ok()?;
        let threshold = u16::try_from(r.u64().ok()?).ok()?;
        let mut members = Vec::new();
        for _ in 0..r.u64().ok()?.min(256) {
            members.push(Keys {
                consortium: r.fixed().ok()?,
                hpke: r.fixed().ok()?,
            });
        }
        let mut signers = Vec::new();
        for _ in 0..r.u64().ok()?.min(256) {
            signers.push(u16::try_from(r.u64().ok()?).ok()?);
        }
        let mut slots = Vec::new();
        for _ in 0..r.u64().ok()?.min(bytes.len() as u64) {
            slots.push(u32::try_from(r.u64().ok()?).ok()?);
        }
        r.finish().ok()?;
        let procedure = Procedure {
            ceremony,
            threshold,
            members,
            signers,
            slots,
        };
        let canonical = procedure.check().is_ok() && procedure.encode() == bytes;
        canonical.then_some(procedure)
    }
}
