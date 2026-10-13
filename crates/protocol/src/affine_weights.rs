//! Frozen A12 offline weight transfer; docs/16 §7.2 and sim/README.md.

use identity::nym::Nym;
use scoring::bridging::{Obs, Ratings};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AffineError(pub &'static str);

pub struct ExpectedAffineEpoch<'a> {
    pub artifact_sha256: &'a str,
    pub manifest_id: &'a str,
    pub snapshot_id: &'a str,
    pub epoch: &'a str,
    pub cut: u64,
}

#[derive(Clone, Copy)]
struct Entry {
    founder: bool,
    n: u64,
    bits: u64,
}

pub struct FrozenAffineEpoch {
    manifest_id: String,
    snapshot_id: String,
    epoch: String,
    cut: u64,
    artifact_sha256: String,
    roster: BTreeMap<Nym, Entry>,
}

fn hex32(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return None;
    }
    let mut result = [0; 32];
    for (index, byte) in result.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(result)
}

fn digits(value: &str) -> bool {
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|b| b.is_ascii_digit())
    {
        return false;
    }
    true
}

fn decimal(value: &str) -> Option<u64> {
    digits(value).then(|| value.parse().ok()).flatten()
}

fn rational(value: &str) -> bool {
    let parts: Vec<_> = value.split('/').collect();
    if parts.len() > 2 {
        return false;
    }
    let numerator = parts[0].strip_prefix('-').unwrap_or(parts[0]);
    if !digits(numerator) || (parts[0].starts_with('-') && numerator == "0") {
        return false;
    }
    parts.len() == 1 || (digits(parts[1]) && parts[1] != "0" && parts[1] != "1")
}

impl FrozenAffineEpoch {
    /// Digest and context must come from the trusted offline producer, outside this artifact.
    pub fn import(bytes: &[u8], expected: &ExpectedAffineEpoch<'_>) -> Result<Self, AffineError> {
        let hash = format!("{:x}", Sha256::digest(bytes));
        if hex32(expected.artifact_sha256).is_none() || hash != expected.artifact_sha256 {
            return Err(AffineError("artifact digest"));
        }
        let text = std::str::from_utf8(bytes).map_err(|_| AffineError("artifact ASCII"))?;
        if !text.is_ascii() || !text.ends_with('\n') || text.contains('\r') {
            return Err(AffineError("artifact ASCII"));
        }
        let mut lines = text.strip_suffix('\n').unwrap().split('\n');
        let header: Vec<_> = lines.next().unwrap().split('\t').collect();
        if header.len() != 9 || header[0] != "affine-frozen-v1" {
            return Err(AffineError("epoch header"));
        }
        let cut = decimal(header[4]).ok_or(AffineError("epoch cut"))?;
        if hex32(header[1]).is_none()
            || hex32(header[2]).is_none()
            || header[1] != expected.manifest_id
            || header[2] != expected.snapshot_id
            || header[3] != expected.epoch
            || cut != expected.cut
            || !rational(header[5])
            || !rational(header[6])
            || (header[7] != "null" && !rational(header[7]))
        {
            return Err(AffineError("epoch context or header"));
        }
        let count = usize::try_from(decimal(header[8]).ok_or(AffineError("roster count"))?)
            .map_err(|_| AffineError("roster count"))?;
        if count == 0 {
            return Err(AffineError("empty roster"));
        }
        let mut roster = BTreeMap::new();
        let mut prior = None;
        for _ in 0..count {
            let line = lines.next().ok_or(AffineError("row count"))?;
            let fields: Vec<_> = line.split('\t').collect();
            if fields.len() != 16 || fields[0] != "row" {
                return Err(AffineError("row fields"));
            }
            let nym = Nym(hex32(fields[1]).ok_or(AffineError("nym"))?);
            if prior.is_some_and(|last| last >= nym) {
                return Err(AffineError("roster order"));
            }
            prior = Some(nym);
            let founder = match fields[2] {
                "0" => false,
                "1" => true,
                _ => return Err(AffineError("founder")),
            };
            let n = decimal(fields[3]).ok_or(AffineError("count"))?;
            decimal(fields[4]).ok_or(AffineError("opportunities"))?;
            if ![5, 6, 7, 9, 10, 11, 12, 13, 14, 15]
                .iter()
                .all(|&i| rational(fields[i]))
            {
                return Err(AffineError("rational fields"));
            }
            if fields[6] != fields[7] {
                return Err(AffineError("non-slack cap"));
            }
            if fields[8].len() != 16
                || !fields[8]
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            {
                return Err(AffineError("weight bits"));
            }
            let bits =
                u64::from_str_radix(fields[8], 16).map_err(|_| AffineError("weight bits"))?;
            let weight = f64::from_bits(bits);
            if !weight.is_finite() || !(0.0..=1.2).contains(&weight) {
                return Err(AffineError("weight range"));
            }
            let bootstrap_bits = if founder {
                1.0f64.to_bits()
            } else {
                0.0f64.to_bits()
            };
            let bootstrap_exact = if founder { "1" } else { "0" };
            if n < 30 && (bits != bootstrap_bits || fields[7] != bootstrap_exact) {
                return Err(AffineError("probation/founder weight"));
            }
            if n >= 30 && (weight < 0.4 || fields[7] == "0") {
                return Err(AffineError("established weight"));
            }
            roster.insert(nym, Entry { founder, n, bits });
        }
        if lines.next().is_some() {
            return Err(AffineError("extra lines"));
        }
        Ok(Self {
            manifest_id: header[1].into(),
            snapshot_id: header[2].into(),
            epoch: header[3].into(),
            cut: expected.cut,
            artifact_sha256: hash,
            roster,
        })
    }

    pub fn weight_bits(&self, nym: Nym) -> Option<u64> {
        self.roster.get(&nym).map(|row| row.bits)
    }
    pub fn context(&self) -> (&str, &str, &str, u64, &str) {
        (
            &self.manifest_id,
            &self.snapshot_id,
            &self.epoch,
            self.cut,
            &self.artifact_sha256,
        )
    }
    fn axis(&self, nym: Nym) -> Option<bool> {
        self.roster.get(&nym).map(|row| row.founder || row.n >= 30)
    }
}

pub struct AffineRatings {
    ratings: Ratings,
    rows: Vec<Nym>,
    context: (String, String, String, u64, String),
}

impl AffineRatings {
    pub fn ratings(&self) -> &Ratings {
        &self.ratings
    }
    pub fn rows(&self) -> &[Nym] {
        &self.rows
    }
    pub fn context(&self) -> (&str, &str, &str, u64, &str) {
        (
            &self.context.0,
            &self.context.1,
            &self.context.2,
            self.context.3,
            &self.context.4,
        )
    }

    pub fn first(
        r: &[Vec<f64>],
        mask: &[Vec<bool>],
        rows: &[Nym],
        epoch: &FrozenAffineEpoch,
    ) -> Result<Self, AffineError> {
        if r.len() != rows.len() || mask.len() != rows.len() || rows.is_empty() {
            return Err(AffineError("matrix rows"));
        }
        let m = r[0].len();
        if r.iter().any(|row| row.len() != m) || mask.iter().any(|row| row.len() != m) {
            return Err(AffineError("matrix shape"));
        }
        let unique: BTreeSet<_> = rows.iter().copied().collect();
        if unique.len() != rows.len() {
            return Err(AffineError("duplicate row nym"));
        }
        let mut weights = Vec::with_capacity(rows.len());
        let mut axis = Vec::with_capacity(rows.len());
        for &nym in rows {
            let bits = epoch
                .weight_bits(nym)
                .ok_or(AffineError("unknown row nym"))?;
            weights.push(f64::from_bits(bits));
            axis.push(epoch.axis(nym).unwrap());
        }
        if r.iter()
            .flatten()
            .any(|prob| !prob.is_finite() || !(0.0..=1.0).contains(prob))
        {
            return Err(AffineError("invalid probability"));
        }
        let ratings = Ratings::from_dense(r, mask)
            .with_weights(weights)
            .with_axis(axis);
        ratings
            .validate()
            .map_err(|_| AffineError("invalid ratings"))?;
        let (manifest, snapshot, id, cut, digest) = epoch.context();
        Ok(Self {
            ratings,
            rows: rows.to_vec(),
            context: (
                manifest.into(),
                snapshot.into(),
                id.into(),
                cut,
                digest.into(),
            ),
        })
    }

    pub fn supplementary(
        &self,
        epoch: &FrozenAffineEpoch,
        item: usize,
        reveals: &[(Nym, f64)],
    ) -> Result<Self, AffineError> {
        let (manifest, snapshot, id, cut, digest) = epoch.context();
        if self.context() != (manifest, snapshot, id, cut, digest) {
            return Err(AffineError("mixed epoch context"));
        }
        if item >= self.ratings.m {
            return Err(AffineError("item index"));
        }
        let mut seen = BTreeSet::new();
        for &(nym, prob) in reveals {
            if !seen.insert(nym) {
                return Err(AffineError("duplicate reveal"));
            }
            if !prob.is_finite() || !(0.0..=1.0).contains(&prob) {
                return Err(AffineError("invalid probability"));
            }
            if epoch.weight_bits(nym).is_none() {
                return Err(AffineError("unknown row nym"));
            }
            if let Some(u) = self.rows.iter().position(|&row| row == nym) {
                if self
                    .ratings
                    .obs
                    .iter()
                    .any(|obs| obs.u == u && obs.j == item)
                {
                    return Err(AffineError("duplicate observation"));
                }
            }
        }
        let mut ratings = self.ratings.clone();
        let mut rows = self.rows.clone();
        for &(nym, prob) in reveals {
            let u = match rows.iter().position(|&row| row == nym) {
                Some(u) => u,
                None => {
                    rows.push(nym);
                    ratings.n += 1;
                    ratings
                        .weights
                        .push(f64::from_bits(epoch.weight_bits(nym).unwrap()));
                    ratings.axis.push(epoch.axis(nym).unwrap());
                    ratings.n - 1
                }
            };
            ratings.obs.push(Obs {
                u,
                j: item,
                r: prob,
            });
        }
        ratings
            .validate()
            .map_err(|_| AffineError("invalid ratings"))?;
        Ok(Self {
            ratings,
            rows,
            context: self.context.clone(),
        })
    }
}
