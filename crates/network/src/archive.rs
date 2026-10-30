//! Bounded public archive segments and authenticated shard recovery (`docs/04` §Public archive
//! segments). Context and the prior cut are authenticated by the caller.

use crate::cid::{cid, Cid};
use crate::codec::{Reader, Writer};
use crate::consortium::Checkpoint;
use crate::cut::Cut;
use crate::erasure::{self, shard_hash};
use crate::hash::tagged;
use crate::log::entry_hash;
use crate::replica::{EntryId, Replica, SignedEntry};
use crate::state::{Origin, RootStatement, Shape};
use crate::store::MAX_OBJECT;
use crate::tier::{Tier, Tiers};
use ed25519_dalek::{Signature, VerifyingKey};
use std::collections::BTreeSet;

pub const MAX_SEGMENT: usize = 32 << 20;
pub const MAX_MANIFEST: usize = 64 << 10;
pub const MAX_ENCODED_SHARDS: usize = 64 << 20;

#[derive(Clone, Copy)]
pub struct Context<'a> {
    pub tiers: &'a Tiers,
    pub lineage: [u8; 32],
    pub previous: Option<&'a Cut>,
    pub target: &'a Cut,
    pub view: u64,
    pub excluded: &'a [usize],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Bounds,
    Codec,
    Context,
    Cut,
    Signatures,
    Delta,
    Snapshot,
    Layout,
    Shards,
    TooFewShards,
    Commitment,
    Roster,
}

type Result<T> = std::result::Result<T, Error>;

fn add(a: usize, b: usize) -> Result<usize> {
    a.checked_add(b).ok_or(Error::Bounds)
}

fn cut_len(cut: &Cut) -> Result<usize> {
    add(26, cut.marks.len().checked_mul(72).ok_or(Error::Bounds)?)
}

fn shape_len(shape: &Shape) -> Result<usize> {
    shape.entries().iter().try_fold(9, |size, entry| {
        add(
            size,
            if entry.origin == Origin::Live {
                65
            } else {
                105
            },
        )
    })
}

fn signatures_ordered(signatures: &[(usize, Signature)]) -> bool {
    signatures.windows(2).all(|p| p[0].0 < p[1].0)
}

fn signatures_verify(
    tiers: &Tiers,
    excluded: &[usize],
    cp: &Checkpoint,
    signatures: &[(usize, Signature)],
) -> bool {
    if !signatures_ordered(signatures) {
        return false;
    }
    let message = tagged(
        "isegoria/checkpoint/v2",
        &[
            &cp.network_id,
            &cp.member_set_hash,
            &cp.height.to_le_bytes(),
            &cp.head,
        ],
    );
    signatures
        .iter()
        .filter(|(index, signature)| {
            !excluded.contains(index)
                && tiers
                    .consortium()
                    .keys()
                    .get(*index)
                    .is_some_and(|key| key.verify_strict(&message, signature).is_ok())
        })
        .count()
        >= tiers.consortium().threshold()
}

/// Checks that `signatures` certify `cut` in `view`: `verify_strict` signatures in member order
/// from at least the threshold of distinct non-excluded members (`docs/04` §Public archive
/// segments).
pub fn verify_certificate(
    tiers: &Tiers,
    excluded: &[usize],
    cut: &Cut,
    view: u64,
    signatures: &[(usize, Signature)],
) -> Result<()> {
    let set = tiers.consortium().member_set_hash();
    let cp = cut.checkpoint_in(view, tiers.network_id(), set);
    match signatures_verify(tiers, excluded, &cp, signatures) {
        true => Ok(()),
        false => Err(Error::Signatures),
    }
}

fn check_context(ctx: &Context<'_>) -> Result<()> {
    let mut context_len = add(131, cut_len(ctx.target)?)?;
    if let Some(previous) = ctx.previous {
        context_len = add(context_len, add(8, cut_len(previous)?)?)?;
    }
    if context_len > MAX_SEGMENT {
        return Err(Error::Bounds);
    }
    if !ctx.target.is_well_formed()
        || ctx.previous.is_some_and(|p| !p.is_well_formed())
        || ctx.previous.map_or(Some(0), |p| p.number.checked_add(1)) != Some(ctx.target.number)
        || ctx.previous.is_some_and(|p| {
            ctx.target.epoch < p.epoch || (p.closes && ctx.target.epoch == p.epoch)
        })
    {
        return Err(Error::Cut);
    }
    for cut in ctx.previous.into_iter().chain(std::iter::once(ctx.target)) {
        if cut
            .marks
            .iter()
            .any(|m| ctx.tiers.tier(&m.writer).is_none())
        {
            return Err(Error::Cut);
        }
    }
    for old in ctx.previous.into_iter().flat_map(|p| &p.marks) {
        let new = ctx
            .target
            .marks
            .binary_search_by_key(&old.writer, |m| m.writer)
            .ok()
            .map(|i| &ctx.target.marks[i])
            .ok_or(Error::Cut)?;
        if new.len < old.len || (new.len == old.len && new.head != old.head) {
            return Err(Error::Cut);
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub statement: RootStatement,
    pub shape: Shape,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    pub network_id: [u8; 32],
    pub lineage: [u8; 32],
    pub member_set_hash: [u8; 32],
    pub previous: Option<Cut>,
    pub target: Cut,
    pub view: u64,
    pub signatures: Vec<(usize, Signature)>,
    pub entries: Vec<(SignedEntry, Vec<u8>)>,
    pub snapshot: Option<Snapshot>,
}

impl Segment {
    pub fn build(
        ctx: &Context<'_>,
        replica: &Replica,
        signatures: Vec<(usize, Signature)>,
        snapshot: Option<Snapshot>,
    ) -> Result<Self> {
        check_context(ctx)?;
        let mut segment = Self {
            network_id: ctx.tiers.network_id(),
            lineage: ctx.lineage,
            member_set_hash: ctx.tiers.consortium().member_set_hash(),
            previous: None,
            target: Cut {
                number: 0,
                epoch: 0,
                closes: false,
                marks: Vec::new(),
            },
            view: ctx.view,
            signatures,
            entries: Vec::new(),
            snapshot,
        };
        let mut size = add(
            segment.encoded_len()?,
            cut_len(ctx.target)?.saturating_sub(26),
        )?;
        if let Some(previous) = ctx.previous {
            size = add(size, add(8, cut_len(previous)?)?)?;
        }
        if size > MAX_SEGMENT {
            return Err(Error::Bounds);
        }
        segment.previous = ctx.previous.cloned();
        segment.target = ctx.target.clone();
        for mark in &ctx.target.marks {
            if ctx.tiers.tier(&mark.writer) != Some(Tier::Public) {
                continue;
            }
            let from = ctx
                .previous
                .and_then(|p| p.marks.iter().find(|m| m.writer == mark.writer))
                .map_or(0, |m| m.len);
            let mut hash = mark.head;
            for seq in (from..mark.len).rev() {
                size = add(size, 176)?;
                if size > MAX_SEGMENT {
                    return Err(Error::Bounds);
                }
                let (entry, payload) = replica
                    .get(&EntryId {
                        writer: mark.writer,
                        seq,
                        hash,
                    })
                    .ok_or(Error::Delta)?;
                size = add(size, payload.len())?;
                if size > MAX_SEGMENT || payload.len() > MAX_OBJECT {
                    return Err(Error::Bounds);
                }
                hash = entry.entry.prev;
                segment.entries.push((entry.clone(), payload.to_vec()));
            }
        }
        segment.entries.sort_by_key(|(e, _)| e.id());
        segment.verify(ctx)?;
        Ok(segment)
    }

    fn encoded_len(&self) -> Result<usize> {
        let mut len = add(131, cut_len(&self.target)?)?;
        if let Some(previous) = &self.previous {
            len = add(len, add(8, cut_len(previous)?)?)?;
        }
        len = add(
            len,
            self.signatures.len().checked_mul(72).ok_or(Error::Bounds)?,
        )?;
        for (_, payload) in &self.entries {
            if payload.len() > MAX_OBJECT {
                return Err(Error::Bounds);
            }
            len = add(len, add(176, payload.len())?)?;
        }
        if let Some(snapshot) = &self.snapshot {
            len = add(len, add(257, shape_len(&snapshot.shape)?)?)?;
        }
        if len > MAX_SEGMENT {
            return Err(Error::Bounds);
        }
        Ok(len)
    }

    /// Checks the cut certificate, public delta and snapshot consistency. Snapshot bytes
    /// are authenticated by the signed archive manifest, not by the cut certificate alone.
    pub fn verify(&self, ctx: &Context<'_>) -> Result<()> {
        self.encoded_len()?;
        check_context(ctx)?;
        if self.network_id != ctx.tiers.network_id()
            || self.lineage != ctx.lineage
            || self.member_set_hash != ctx.tiers.consortium().member_set_hash()
            || self.previous.as_ref() != ctx.previous
            || self.target != *ctx.target
            || self.view != ctx.view
        {
            return Err(Error::Context);
        }
        verify_certificate(
            ctx.tiers,
            ctx.excluded,
            &self.target,
            self.view,
            &self.signatures,
        )?;
        if !self.entries.windows(2).all(|p| p[0].0.id() < p[1].0.id()) {
            return Err(Error::Delta);
        }
        let mut entries = self.entries.iter().peekable();
        for mark in &self.target.marks {
            if ctx.tiers.tier(&mark.writer) != Some(Tier::Public) {
                continue;
            }
            let old = ctx
                .previous
                .and_then(|p| p.marks.iter().find(|m| m.writer == mark.writer));
            let from = old.map_or(0, |m| m.len);
            let mut head = old.map_or([0; 32], |m| m.head);
            for seq in from..mark.len {
                let (entry, payload) = entries.next().ok_or(Error::Delta)?;
                if entry.writer != mark.writer
                    || entry.entry.seq != seq
                    || entry.entry.prev != head
                    || entry.entry.hash != entry_hash(seq, &head, &entry.entry.payload)
                    || entry.entry.payload != cid(payload)
                {
                    return Err(Error::Delta);
                }
                let message = tagged(
                    "isegoria/feed/entry/v1",
                    &[&self.network_id, &entry.writer, &entry.entry.hash],
                );
                let key = VerifyingKey::from_bytes(&entry.writer).map_err(|_| Error::Delta)?;
                key.verify_strict(&message, &Signature::from_bytes(&entry.signature))
                    .map_err(|_| Error::Delta)?;
                head = entry.entry.hash;
            }
            if head != mark.head {
                return Err(Error::Delta);
            }
        }
        if entries.next().is_some() {
            return Err(Error::Delta);
        }
        match (&self.snapshot, self.target.closes) {
            (None, false) => {}
            (Some(snapshot), true) => {
                let root = &snapshot.statement;
                if root.network_id != self.network_id
                    || root.lineage != self.lineage
                    || root.member_set_hash != self.member_set_hash
                    || root.epoch != self.target.epoch
                    || root.cut_number != self.target.number
                    || root.cut_digest != self.target.digest()
                {
                    return Err(Error::Snapshot);
                }
                snapshot.shape.check(root).map_err(|_| Error::Snapshot)?;
            }
            _ => return Err(Error::Snapshot),
        }
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        self.encoded_len()?;
        if !self.target.is_well_formed()
            || self.previous.as_ref().is_some_and(|p| !p.is_well_formed())
            || !signatures_ordered(&self.signatures)
            || !self.entries.windows(2).all(|p| p[0].0.id() < p[1].0.id())
            || self.entries.iter().any(|(e, _)| {
                e.entry.hash != entry_hash(e.entry.seq, &e.entry.prev, &e.entry.payload)
            })
        {
            return Err(Error::Codec);
        }
        let mut w = Writer::new();
        w.u8(1)
            .fixed(&self.network_id)
            .fixed(&self.lineage)
            .fixed(&self.member_set_hash);
        match &self.previous {
            None => {
                w.u8(0);
            }
            Some(p) => {
                w.u8(1).field(&p.encode());
            }
        }
        w.field(&self.target.encode()).u64(self.view);
        write_signatures(&mut w, &self.signatures);
        w.u64(self.entries.len() as u64);
        for (e, payload) in &self.entries {
            w.fixed(&e.encode()).field(payload);
        }
        match &self.snapshot {
            None => {
                w.u8(0);
            }
            Some(s) => {
                w.u8(1)
                    .field(&s.statement.encode())
                    .field(&s.shape.encode());
            }
        }
        Ok(w.finish())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_SEGMENT {
            return Err(Error::Bounds);
        }
        let mut r = Reader::new(bytes);
        let read = || -> Option<Self> {
            if r.u8().ok()? != 1 {
                return None;
            }
            let network_id = r.fixed().ok()?;
            let lineage = r.fixed().ok()?;
            let member_set_hash = r.fixed().ok()?;
            let previous = match r.u8().ok()? {
                0 => None,
                1 => Some(Cut::decode(r.field().ok()?)?),
                _ => return None,
            };
            let target = Cut::decode(r.field().ok()?)?;
            let view = r.u64().ok()?;
            let signatures = read_signatures(&mut r, bytes.len())?;
            let count = usize::try_from(r.u64().ok()?).ok()?;
            if count > bytes.len() / 176 {
                return None;
            }
            let mut entries = Vec::new();
            for _ in 0..count {
                let entry = SignedEntry::decode(&r.fixed::<168>().ok()?)?;
                let payload = r.field().ok()?;
                if payload.len() > MAX_OBJECT
                    || entries
                        .last()
                        .is_some_and(|(e, _): &(SignedEntry, Vec<u8>)| e.id() >= entry.id())
                {
                    return None;
                }
                entries.push((entry, payload.to_vec()));
            }
            let snapshot = match r.u8().ok()? {
                0 => None,
                1 => Some(Snapshot {
                    statement: RootStatement::decode(r.field().ok()?).ok()?,
                    shape: Shape::decode(r.field().ok()?).ok()?,
                }),
                _ => return None,
            };
            r.finish().ok()?;
            Some(Self {
                network_id,
                lineage,
                member_set_hash,
                previous,
                target,
                view,
                signatures,
                entries,
                snapshot,
            })
        };
        read().ok_or(Error::Codec)
    }
}

fn write_signatures(w: &mut Writer, signatures: &[(usize, Signature)]) {
    w.u64(signatures.len() as u64);
    for (index, signature) in signatures {
        w.u64(*index as u64).fixed(&signature.to_bytes());
    }
}

fn read_signatures(r: &mut Reader<'_>, limit: usize) -> Option<Vec<(usize, Signature)>> {
    let count = usize::try_from(r.u64().ok()?).ok()?;
    if count > limit / 72 {
        return None;
    }
    let mut signatures = Vec::new();
    for _ in 0..count {
        let index = usize::try_from(r.u64().ok()?).ok()?;
        let signature = Signature::from_bytes(&r.fixed().ok()?);
        if signatures.last().is_some_and(|(old, _)| *old >= index) {
            return None;
        }
        signatures.push((index, signature));
    }
    Some(signatures)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    pub data_shards: usize,
    pub parity_shards: usize,
}

impl Layout {
    fn lengths(self, orig_len: usize) -> Result<(usize, usize)> {
        let n = self
            .data_shards
            .checked_add(self.parity_shards)
            .ok_or(Error::Layout)?;
        if self.data_shards == 0
            || self.parity_shards == 0
            || n > 256
            || orig_len == 0
            || orig_len > MAX_SEGMENT
        {
            return Err(Error::Layout);
        }
        let width = orig_len.div_ceil(self.data_shards);
        if width > MAX_OBJECT || width.checked_mul(n).ok_or(Error::Bounds)? > MAX_ENCODED_SHARDS {
            return Err(Error::Bounds);
        }
        Ok((n, width))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Manifest {
    network_id: [u8; 32],
    lineage: [u8; 32],
    member_set_hash: [u8; 32],
    previous_digest: Option<[u8; 32]>,
    target_number: u64,
    target_digest: [u8; 32],
    view: u64,
    segment_cid: Cid,
    orig_len: usize,
    layout: Layout,
    shard_hashes: Vec<[u8; 32]>,
}

impl Manifest {
    pub fn network_id(&self) -> [u8; 32] {
        self.network_id
    }
    pub fn lineage(&self) -> [u8; 32] {
        self.lineage
    }
    pub fn member_set_hash(&self) -> [u8; 32] {
        self.member_set_hash
    }
    pub fn previous_digest(&self) -> Option<[u8; 32]> {
        self.previous_digest
    }
    pub fn target_number(&self) -> u64 {
        self.target_number
    }
    pub fn target_digest(&self) -> [u8; 32] {
        self.target_digest
    }
    pub fn view(&self) -> u64 {
        self.view
    }
    pub fn segment_cid(&self) -> Cid {
        self.segment_cid
    }
    pub fn orig_len(&self) -> usize {
        self.orig_len
    }
    pub fn layout(&self) -> Layout {
        self.layout
    }
    pub fn shard_hashes(&self) -> &[[u8; 32]] {
        &self.shard_hashes
    }

    fn check(&self) -> Result<()> {
        let (n, _) = self.layout.lengths(self.orig_len)?;
        if self.shard_hashes.len() != n {
            return Err(Error::Layout);
        }
        Ok(())
    }

    fn check_context(&self, ctx: &Context<'_>) -> Result<()> {
        self.check()?;
        check_context(ctx)?;
        if self.network_id != ctx.tiers.network_id()
            || self.lineage != ctx.lineage
            || self.member_set_hash != ctx.tiers.consortium().member_set_hash()
            || self.previous_digest != ctx.previous.map(Cut::digest)
            || self.target_number != ctx.target.number
            || self.target_digest != ctx.target.digest()
            || self.view != ctx.view
        {
            return Err(Error::Context);
        }
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        self.check()?;
        let mut w = Writer::new();
        w.u8(1)
            .fixed(&self.network_id)
            .fixed(&self.lineage)
            .fixed(&self.member_set_hash);
        match self.previous_digest {
            None => {
                w.u8(0);
            }
            Some(d) => {
                w.u8(1).fixed(&d);
            }
        }
        w.u64(self.target_number)
            .fixed(&self.target_digest)
            .u64(self.view)
            .fixed(&self.segment_cid.0)
            .u64(self.orig_len as u64)
            .u64(self.layout.data_shards as u64)
            .u64(self.layout.parity_shards as u64)
            .u64(self.shard_hashes.len() as u64);
        for hash in &self.shard_hashes {
            w.fixed(hash);
        }
        Ok(w.finish())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_MANIFEST {
            return Err(Error::Bounds);
        }
        let mut r = Reader::new(bytes);
        let read = || -> Option<Self> {
            if r.u8().ok()? != 1 {
                return None;
            }
            let network_id = r.fixed().ok()?;
            let lineage = r.fixed().ok()?;
            let member_set_hash = r.fixed().ok()?;
            let previous_digest = match r.u8().ok()? {
                0 => None,
                1 => Some(r.fixed().ok()?),
                _ => return None,
            };
            let target_number = r.u64().ok()?;
            let target_digest = r.fixed().ok()?;
            let view = r.u64().ok()?;
            let segment_cid = Cid(r.fixed().ok()?);
            let orig_len = usize::try_from(r.u64().ok()?).ok()?;
            let layout = Layout {
                data_shards: usize::try_from(r.u64().ok()?).ok()?,
                parity_shards: usize::try_from(r.u64().ok()?).ok()?,
            };
            let (n, _) = layout.lengths(orig_len).ok()?;
            if usize::try_from(r.u64().ok()?).ok()? != n {
                return None;
            }
            let shard_hashes = (0..n).map(|_| r.fixed().ok()).collect::<Option<Vec<_>>>()?;
            r.finish().ok()?;
            Some(Self {
                network_id,
                lineage,
                member_set_hash,
                previous_digest,
                target_number,
                target_digest,
                view,
                segment_cid,
                orig_len,
                layout,
                shard_hashes,
            })
        };
        read().ok_or(Error::Codec)
    }

    pub fn digest(&self) -> [u8; 32] {
        tagged(
            "isegoria/archive/manifest/v1",
            &[&self
                .encode()
                .expect("private fields preserve valid manifest")],
        )
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint::new(
            self.network_id,
            self.member_set_hash,
            self.target_number,
            self.digest(),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedManifest {
    pub manifest: Manifest,
    pub signatures: Vec<(usize, Signature)>,
}

impl SignedManifest {
    /// Checks canonical bounds and the network's certificate without beacon exclusions.
    /// Does not authenticate lineage, cut context, freshness, or segment semantics.
    pub fn verify_certificate(&self, tiers: &Tiers) -> Result<()> {
        self.encode()?;
        if self.manifest.network_id != tiers.network_id()
            || self.manifest.member_set_hash != tiers.consortium().member_set_hash()
        {
            return Err(Error::Context);
        }
        if !signatures_verify(tiers, &[], &self.manifest.checkpoint(), &self.signatures) {
            return Err(Error::Signatures);
        }
        Ok(())
    }

    pub fn verify(&self, ctx: &Context<'_>) -> Result<VerifiedManifest> {
        self.encode()?;
        self.manifest.check_context(ctx)?;
        if !signatures_verify(
            ctx.tiers,
            ctx.excluded,
            &self.manifest.checkpoint(),
            &self.signatures,
        ) {
            return Err(Error::Signatures);
        }
        Ok(VerifiedManifest {
            signed: self.clone(),
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        if !signatures_ordered(&self.signatures) {
            return Err(Error::Codec);
        }
        let manifest = self.manifest.encode()?;
        let size = add(
            add(17, manifest.len())?,
            self.signatures.len().checked_mul(72).ok_or(Error::Bounds)?,
        )?;
        if size > MAX_MANIFEST {
            return Err(Error::Bounds);
        }
        let mut w = Writer::new();
        w.u8(1).field(&manifest);
        write_signatures(&mut w, &self.signatures);
        Ok(w.finish())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_MANIFEST {
            return Err(Error::Bounds);
        }
        let mut r = Reader::new(bytes);
        if r.u8().map_err(|_| Error::Codec)? != 1 {
            return Err(Error::Codec);
        }
        let manifest = Manifest::decode(r.field().map_err(|_| Error::Codec)?)?;
        let signatures = read_signatures(&mut r, bytes.len()).ok_or(Error::Codec)?;
        r.finish().map_err(|_| Error::Codec)?;
        Ok(Self {
            manifest,
            signatures,
        })
    }
}

pub struct BuiltArchive {
    pub manifest: Manifest,
    pub shards: Vec<Vec<u8>>,
}

impl BuiltArchive {
    pub fn build(ctx: &Context<'_>, segment: &Segment, layout: Layout) -> Result<Self> {
        segment.verify(ctx)?;
        let len = segment.encoded_len()?;
        layout.lengths(len)?;
        let bytes = segment.encode()?;
        let encoded = erasure::encode(&bytes, layout.data_shards, layout.parity_shards);
        let manifest = Manifest {
            network_id: segment.network_id,
            lineage: segment.lineage,
            member_set_hash: segment.member_set_hash,
            previous_digest: segment.previous.as_ref().map(Cut::digest),
            target_number: segment.target.number,
            target_digest: segment.target.digest(),
            view: segment.view,
            segment_cid: cid(&bytes),
            orig_len: bytes.len(),
            layout,
            shard_hashes: encoded.manifest,
        };
        Ok(Self {
            manifest,
            shards: encoded.shards,
        })
    }
}

#[derive(Clone, Debug)]
pub struct VerifiedManifest {
    signed: SignedManifest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShardStatus {
    Missing,
    Corrupt,
    Valid,
}

pub struct Recovered {
    pub segment: Segment,
    pub repairs: Vec<(usize, Vec<u8>)>,
}

impl VerifiedManifest {
    pub fn manifest(&self) -> &Manifest {
        &self.signed.manifest
    }

    fn recheck(&self, ctx: &Context<'_>) -> Result<()> {
        self.signed.manifest.check_context(ctx)?;
        if !signatures_verify(
            ctx.tiers,
            ctx.excluded,
            &self.signed.manifest.checkpoint(),
            &self.signed.signatures,
        ) {
            return Err(Error::Signatures);
        }
        Ok(())
    }

    pub fn audit(&self, ctx: &Context<'_>, shards: &[Option<Vec<u8>>]) -> Result<Vec<ShardStatus>> {
        self.recheck(ctx)?;
        let m = self.manifest();
        let (n, width) = m.layout.lengths(m.orig_len)?;
        if shards.len() != n {
            return Err(Error::Shards);
        }
        Ok(shards
            .iter()
            .zip(&m.shard_hashes)
            .map(|(shard, hash)| match shard {
                None => ShardStatus::Missing,
                Some(bytes) if bytes.len() == width && shard_hash(bytes) == *hash => {
                    ShardStatus::Valid
                }
                Some(_) => ShardStatus::Corrupt,
            })
            .collect())
    }

    pub fn recover(&self, ctx: &Context<'_>, shards: &[Option<Vec<u8>>]) -> Result<Recovered> {
        let statuses = self.audit(ctx, shards)?;
        let m = self.manifest();
        if statuses
            .iter()
            .filter(|s| **s == ShardStatus::Valid)
            .count()
            < m.layout.data_shards
        {
            return Err(Error::TooFewShards);
        }
        let clean = shards
            .iter()
            .zip(&statuses)
            .map(|(bytes, status)| {
                if *status == ShardStatus::Valid {
                    bytes.clone()
                } else {
                    None
                }
            })
            .collect();
        let bytes = erasure::reconstruct(
            clean,
            m.layout.data_shards,
            m.layout.parity_shards,
            m.orig_len,
        )
        .ok_or(Error::Shards)?;
        let encoded = erasure::encode(&bytes, m.layout.data_shards, m.layout.parity_shards);
        if encoded.manifest != m.shard_hashes || cid(&bytes) != m.segment_cid {
            return Err(Error::Commitment);
        }
        let segment = Segment::decode(&bytes)?;
        segment.verify(ctx)?;
        let repairs = encoded
            .shards
            .into_iter()
            .enumerate()
            .filter(|(i, _)| statuses[*i] != ShardStatus::Valid)
            .collect();
        Ok(Recovered { segment, repairs })
    }

    pub fn placement(&self, ctx: &Context<'_>, roster: &[[u8; 32]]) -> Result<Vec<[u8; 32]>> {
        self.recheck(ctx)?;
        let mut unused: BTreeSet<_> = roster.iter().copied().collect();
        let n = self.manifest().shard_hashes.len();
        if unused.len() != roster.len() || unused.len() < n {
            return Err(Error::Roster);
        }
        let digest = self.manifest().digest();
        let mut holders = Vec::new();
        for index in 0..n {
            let chosen = unused
                .iter()
                .min_by_key(|id| {
                    (
                        tagged(
                            "isegoria/archive/place/v1",
                            &[&digest, &(index as u64).to_le_bytes(), *id],
                        ),
                        **id,
                    )
                })
                .copied()
                .ok_or(Error::Roster)?;
            unused.remove(&chosen);
            holders.push(chosen);
        }
        Ok(holders)
    }
}
