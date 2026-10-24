//! Sparse state commitments and fork recovery (`docs/04` §The state tree and the fork).
//! This is a bespoke composition of tagged SHA-256; records remain opaque.

use crate::codec::{DecodeError, Reader, Writer};
use crate::hash::tagged;
use std::collections::BTreeMap;
use std::sync::OnceLock;

pub type Hash = [u8; 32];
pub type Position = [u8; 32];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Propose,
    Judge,
    Respond,
}

impl Role {
    fn tag(self) -> &'static [u8] {
        match self {
            Self::Propose => b"propose",
            Self::Judge => b"judge",
            Self::Respond => b"respond",
        }
    }

    fn byte(self) -> u8 {
        match self {
            Self::Propose => 0,
            Self::Judge => 1,
            Self::Respond => 2,
        }
    }

    fn read(r: &mut Reader<'_>) -> Result<Self, DecodeError> {
        match r.u8()? {
            0 => Ok(Self::Propose),
            1 => Ok(Self::Judge),
            2 => Ok(Self::Respond),
            _ => Err(DecodeError),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Context {
    pub lineage: Hash,
    pub network_id: Hash,
    pub epoch: u64,
}

pub fn position(lineage: &Hash, role: Role, nym: &Hash) -> Position {
    tagged("isegoria/state/position/v1", &[lineage, role.tag(), nym])
}

pub fn salt(key: &Hash, context: &Context) -> Hash {
    tagged(
        "isegoria/state/salt/v1",
        &[
            key,
            &context.lineage,
            &context.network_id,
            &context.epoch.to_le_bytes(),
        ],
    )
}

#[derive(Clone, PartialEq, Eq)]
pub struct Records {
    role: Role,
    payload: Vec<u8>,
}

impl Records {
    pub fn new(role: Role, payload: Vec<u8>) -> Self {
        Self { role, payload }
    }

    pub fn role(&self) -> Role {
        self.role
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn encode(&self) -> Vec<u8> {
        Writer::new()
            .u8(1)
            .u8(self.role.byte())
            .field(&self.payload)
            .finish()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut r = Reader::new(bytes);
        version(&mut r)?;
        let records = Self::new(Role::read(&mut r)?, r.field()?.to_vec());
        r.finish()?;
        Ok(records)
    }
}

pub fn commitment(context: &Context, records: &Records, salt: &Hash) -> Hash {
    tagged(
        "isegoria/state/commit/v1",
        &[
            &context.lineage,
            &context.network_id,
            &context.epoch.to_le_bytes(),
            records.role.tag(),
            salt,
            &records.encode(),
        ],
    )
}

pub fn leaf_hash(position: &Position, commitment: &Hash) -> Hash {
    tagged("isegoria/state/leaf/v1", &[position, commitment])
}

fn node(left: &Hash, right: &Hash) -> Hash {
    tagged("isegoria/state/node/v1", &[left, right])
}

fn empties() -> &'static [Hash; 257] {
    static EMPTY: OnceLock<[Hash; 257]> = OnceLock::new();
    EMPTY.get_or_init(|| {
        let mut hashes = [[0; 32]; 257];
        hashes[0] = tagged("isegoria/state/empty/v1", &[]);
        for height in 1..=256 {
            hashes[height] = node(&hashes[height - 1], &hashes[height - 1]);
        }
        hashes
    })
}

/// The empty subtree of height 0..=256, or `None` for an invalid height.
pub fn empty_hash(height: usize) -> Option<Hash> {
    empties().get(height).copied()
}

fn bit(position: &Position, depth: usize) -> bool {
    position[depth / 8] & (128 >> (depth % 8)) != 0
}

fn set_bit(position: &mut Position, depth: usize) {
    position[depth / 8] |= 128 >> (depth % 8);
}

fn version(r: &mut Reader<'_>) -> Result<(), DecodeError> {
    if r.u8()? == 1 {
        Ok(())
    } else {
        Err(DecodeError)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Live,
    Frozen { network_id: Hash, epoch: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShapeEntry {
    pub position: Position,
    pub leaf_hash: Hash,
    pub origin: Origin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shape {
    entries: Vec<ShapeEntry>,
}

impl Shape {
    pub fn new(entries: Vec<ShapeEntry>) -> Result<Self, StateError> {
        if entries.iter().any(|entry| entry.leaf_hash == empties()[0])
            || entries
                .windows(2)
                .any(|pair| pair[0].position >= pair[1].position)
        {
            return Err(StateError::Shape);
        }
        Ok(Self { entries })
    }

    pub fn entries(&self) -> &[ShapeEntry] {
        &self.entries
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(1).u64(self.entries.len() as u64);
        for entry in &self.entries {
            w.fixed(&entry.position).fixed(&entry.leaf_hash);
            match entry.origin {
                Origin::Live => {
                    w.u8(0);
                }
                Origin::Frozen { network_id, epoch } => {
                    w.u8(1).fixed(&network_id).u64(epoch);
                }
            }
        }
        w.finish()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut r = Reader::new(bytes);
        version(&mut r)?;
        let count = usize::try_from(r.u64()?).map_err(|_| DecodeError)?;
        if count > bytes.len().saturating_sub(9) / 65 {
            return Err(DecodeError);
        }
        let mut entries = Vec::new();
        for _ in 0..count {
            let position = r.fixed()?;
            let leaf_hash = r.fixed()?;
            let origin = match r.u8()? {
                0 => Origin::Live,
                1 => Origin::Frozen {
                    network_id: r.fixed()?,
                    epoch: r.u64()?,
                },
                _ => return Err(DecodeError),
            };
            entries.push(ShapeEntry {
                position,
                leaf_hash,
                origin,
            });
        }
        r.finish()?;
        Self::new(entries).map_err(|_| DecodeError)
    }

    pub fn digest(&self) -> Hash {
        tagged("isegoria/state/shape/v1", &[&self.encode()])
    }

    pub fn root(&self) -> Hash {
        subtree(&self.entries, 0)
    }

    pub fn path(&self, position: Position) -> Path {
        let mut bitmap = [0; 32];
        let mut siblings = Vec::new();
        let mut entries = &self.entries[..];
        for depth in 0..256 {
            let split = entries.partition_point(|e| !bit(&e.position, depth));
            let (left, right) = entries.split_at(split);
            let sibling = if bit(&position, depth) {
                entries = right;
                subtree(left, depth + 1)
            } else {
                entries = left;
                subtree(right, depth + 1)
            };
            if sibling != empties()[255 - depth] {
                set_bit(&mut bitmap, depth);
                siblings.push(sibling);
            }
        }
        siblings.reverse();
        Path {
            position,
            bitmap,
            siblings,
        }
    }

    pub fn check(&self, trusted: &RootStatement) -> Result<(), StateError> {
        if self.root() != trusted.state_root || self.digest() != trusted.shape_digest {
            return Err(StateError::Shape);
        }
        Ok(())
    }
}

fn subtree(entries: &[ShapeEntry], depth: usize) -> Hash {
    if entries.is_empty() {
        return empties()[256 - depth];
    }
    if depth == 256 {
        return entries[0].leaf_hash;
    }
    let split = entries.partition_point(|e| !bit(&e.position, depth));
    node(
        &subtree(&entries[..split], depth + 1),
        &subtree(&entries[split..], depth + 1),
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Path {
    pub position: Position,
    bitmap: Hash,
    siblings: Vec<Hash>,
}

impl Path {
    pub fn bitmap(&self) -> &Hash {
        &self.bitmap
    }
    pub fn siblings(&self) -> &[Hash] {
        &self.siblings
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(1).fixed(&self.position).fixed(&self.bitmap);
        for sibling in &self.siblings {
            w.fixed(sibling);
        }
        w.finish()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut r = Reader::new(bytes);
        version(&mut r)?;
        let position = r.fixed()?;
        let bitmap = r.fixed()?;
        let mut siblings = Vec::new();
        for depth in (0..256).rev() {
            if bit(&bitmap, depth) {
                let sibling = r.fixed()?;
                if sibling == empties()[255 - depth] {
                    return Err(DecodeError);
                }
                siblings.push(sibling);
            }
        }
        r.finish()?;
        Ok(Self {
            position,
            bitmap,
            siblings,
        })
    }

    pub fn verify(&self, leaf: &Hash, root: &Hash) -> bool {
        let mut value = *leaf;
        let mut siblings = self.siblings.iter();
        for depth in (0..256).rev() {
            let sibling = if bit(&self.bitmap, depth) {
                match siblings.next() {
                    Some(sibling) if *sibling != empties()[255 - depth] => *sibling,
                    _ => return false,
                }
            } else {
                empties()[255 - depth]
            };
            value = if bit(&self.position, depth) {
                node(&sibling, &value)
            } else {
                node(&value, &sibling)
            };
        }
        siblings.next().is_none() && value == *root
    }

    pub fn verify_absence(&self, root: &Hash) -> bool {
        self.verify(&empties()[0], root)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Opening {
    pub path: Path,
    pub records: Records,
    pub salt: Hash,
}

impl Opening {
    pub fn encode(&self) -> Vec<u8> {
        Writer::new()
            .u8(1)
            .field(&self.path.encode())
            .field(&self.records.encode())
            .fixed(&self.salt)
            .finish()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut r = Reader::new(bytes);
        version(&mut r)?;
        let opening = Self {
            path: Path::decode(r.field()?)?,
            records: Records::decode(r.field()?)?,
            salt: r.fixed()?,
        };
        r.finish()?;
        Ok(opening)
    }

    pub fn verify(&self, context: &Context, role: Role, root: &Hash) -> bool {
        self.records.role == role && self.path.verify(&self.leaf(context), root)
    }

    fn leaf(&self, context: &Context) -> Hash {
        leaf_hash(
            &self.path.position,
            &commitment(context, &self.records, &self.salt),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootStatement {
    pub network_id: Hash,
    pub lineage: Hash,
    pub member_set_hash: Hash,
    pub epoch: u64,
    pub cut_number: u64,
    pub cut_digest: Hash,
    pub inputs_root: Hash,
    pub state_root: Hash,
    pub shape_digest: Hash,
}

impl RootStatement {
    pub fn context(&self) -> Context {
        Context {
            lineage: self.lineage,
            network_id: self.network_id,
            epoch: self.epoch,
        }
    }

    pub fn digest(&self) -> Hash {
        tagged(
            "isegoria/state/roots/v1",
            &[
                &self.network_id,
                &self.lineage,
                &self.member_set_hash,
                &self.epoch.to_le_bytes(),
                &self.cut_number.to_le_bytes(),
                &self.cut_digest,
                &self.inputs_root,
                &self.state_root,
                &self.shape_digest,
            ],
        )
    }

    pub fn encode(&self) -> Vec<u8> {
        Writer::new()
            .u8(1)
            .fixed(&self.network_id)
            .fixed(&self.lineage)
            .fixed(&self.member_set_hash)
            .u64(self.epoch)
            .u64(self.cut_number)
            .fixed(&self.cut_digest)
            .fixed(&self.inputs_root)
            .fixed(&self.state_root)
            .fixed(&self.shape_digest)
            .finish()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut r = Reader::new(bytes);
        version(&mut r)?;
        let statement = Self {
            network_id: r.fixed()?,
            lineage: r.fixed()?,
            member_set_hash: r.fixed()?,
            epoch: r.u64()?,
            cut_number: r.u64()?,
            cut_digest: r.fixed()?,
            inputs_root: r.fixed()?,
            state_root: r.fixed()?,
            shape_digest: r.fixed()?,
        };
        r.finish()?;
        Ok(statement)
    }
}

/// The caller must have verified the nullifier proof and accepted its issuer.
/// Construction performs no authentication; `network` has no identity verifier.
#[derive(Clone, Copy)]
pub struct CallerAuthenticatedNym {
    nym: Hash,
    role: Role,
}

impl CallerAuthenticatedNym {
    pub fn from_verified_caller(nym: Hash, role: Role) -> Self {
        Self { nym, role }
    }

    fn position(&self, lineage: &Hash) -> Position {
        position(lineage, self.role, &self.nym)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateError {
    Authentication,
    Position,
    Opening,
    Shape,
    Occupied,
    Frozen,
    NotFrozen,
    Role,
    Epoch,
    Context,
    MissingRecords,
}

pub fn verify_new(
    context: &Context,
    root: &Hash,
    caller: Option<&CallerAuthenticatedNym>,
    path: &Path,
) -> Result<(), StateError> {
    let caller = caller.ok_or(StateError::Authentication)?;
    if path.position != caller.position(&context.lineage) {
        return Err(StateError::Position);
    }
    if !path.verify_absence(root) {
        return Err(StateError::Occupied);
    }
    Ok(())
}

pub fn verify_reentry(
    context: &Context,
    root: &Hash,
    caller: Option<&CallerAuthenticatedNym>,
    opening: &Opening,
) -> Result<(), StateError> {
    let caller = caller.ok_or(StateError::Authentication)?;
    if opening.path.position != caller.position(&context.lineage) {
        return Err(StateError::Position);
    }
    if !opening.verify(context, caller.role, root) {
        return Err(StateError::Opening);
    }
    Ok(())
}

#[derive(Clone)]
struct Live {
    records: Records,
    key: Hash,
}

#[derive(Clone)]
pub struct State {
    context: Context,
    shape: Shape,
    live: BTreeMap<Position, Live>,
}

impl State {
    pub fn new(context: Context) -> Self {
        Self {
            context,
            shape: Shape {
                entries: Vec::new(),
            },
            live: BTreeMap::new(),
        }
    }

    pub fn context(&self) -> Context {
        self.context
    }
    pub fn shape(&self) -> Shape {
        self.shape.clone()
    }
    pub fn root(&self) -> Hash {
        self.shape.root()
    }
    pub fn path(&self, position: Position) -> Path {
        self.shape.path(position)
    }

    pub fn statement(
        &self,
        member_set_hash: Hash,
        cut_number: u64,
        cut_digest: Hash,
        inputs_root: Hash,
    ) -> RootStatement {
        RootStatement {
            network_id: self.context.network_id,
            lineage: self.context.lineage,
            epoch: self.context.epoch,
            member_set_hash,
            cut_number,
            cut_digest,
            inputs_root,
            state_root: self.root(),
            shape_digest: self.shape.digest(),
        }
    }

    pub fn insert(
        &mut self,
        caller: &CallerAuthenticatedNym,
        records: Records,
        key: Hash,
    ) -> Result<(), StateError> {
        if records.role != caller.role {
            return Err(StateError::Role);
        }
        let position = caller.position(&self.context.lineage);
        let index = match self
            .shape
            .entries
            .binary_search_by_key(&position, |e| e.position)
        {
            Ok(_) => return Err(StateError::Occupied),
            Err(index) => index,
        };
        let leaf_hash = live_leaf(&self.context, &position, &records, &key);
        self.shape.entries.insert(
            index,
            ShapeEntry {
                position,
                leaf_hash,
                origin: Origin::Live,
            },
        );
        self.live.insert(position, Live { records, key });
        Ok(())
    }

    pub fn set_records(
        &mut self,
        caller: &CallerAuthenticatedNym,
        records: Records,
    ) -> Result<(), StateError> {
        if records.role != caller.role {
            return Err(StateError::Role);
        }
        let position = caller.position(&self.context.lineage);
        let live = self.live.get_mut(&position).ok_or(StateError::Frozen)?;
        live.records = records;
        let entry = self
            .shape
            .entries
            .iter_mut()
            .find(|e| e.position == position)
            .ok_or(StateError::MissingRecords)?;
        if entry.origin == Origin::Live {
            entry.leaf_hash = live_leaf(&self.context, &position, &live.records, &live.key);
        }
        Ok(())
    }

    pub fn opening(&self, caller: &CallerAuthenticatedNym) -> Option<Opening> {
        let position = caller.position(&self.context.lineage);
        let live = self.live.get(&position)?;
        let entry = self.shape.entries.iter().find(|e| e.position == position)?;
        let context = origin_context(&self.context, entry.origin);
        let opening = Opening {
            path: self.path(position),
            records: live.records.clone(),
            salt: salt(&live.key, &context),
        };
        (opening.leaf(&context) == entry.leaf_hash).then_some(opening)
    }

    pub fn advance_epoch(&mut self, epoch: u64) -> Result<(), StateError> {
        if self.context.epoch.checked_add(1) != Some(epoch) {
            return Err(StateError::Epoch);
        }
        self.context.epoch = epoch;
        for entry in &mut self.shape.entries {
            if let Some(live) = self.live.get(&entry.position) {
                entry.leaf_hash =
                    live_leaf(&self.context, &entry.position, &live.records, &live.key);
                entry.origin = Origin::Live;
            }
        }
        Ok(())
    }

    /// `trusted` must already be authenticated and anchored by the caller.
    pub fn fork_without_member(
        trusted: &RootStatement,
        shape: &Shape,
        network_id: Hash,
    ) -> Result<Self, StateError> {
        shape.check(trusted)?;
        if network_id == trusted.network_id {
            return Err(StateError::Context);
        }
        let mut state = Self::new(Context {
            network_id,
            ..trusted.context()
        });
        state.shape = shape.clone();
        for entry in &mut state.shape.entries {
            if entry.origin == Origin::Live {
                entry.origin = Origin::Frozen {
                    network_id: trusted.network_id,
                    epoch: trusted.epoch,
                };
            }
        }
        Ok(state)
    }

    /// Checks the member's state against the caller's authenticated anchored statement.
    pub fn fork_with_member(
        &self,
        trusted: &RootStatement,
        network_id: Hash,
    ) -> Result<Self, StateError> {
        self.shape.check(trusted)?;
        if self.context != trusted.context() || network_id == trusted.network_id {
            return Err(StateError::Context);
        }
        for entry in &self.shape.entries {
            if entry.origin == Origin::Live {
                let live = self
                    .live
                    .get(&entry.position)
                    .ok_or(StateError::MissingRecords)?;
                if live_leaf(&self.context, &entry.position, &live.records, &live.key)
                    != entry.leaf_hash
                {
                    return Err(StateError::Opening);
                }
            }
        }
        let mut state = self.clone();
        state.context.network_id = network_id;
        for entry in &mut state.shape.entries {
            if entry.origin == Origin::Live {
                let live = &state.live[&entry.position];
                entry.leaf_hash =
                    live_leaf(&state.context, &entry.position, &live.records, &live.key);
            }
        }
        Ok(state)
    }

    pub fn reopen(
        &mut self,
        caller: Option<&CallerAuthenticatedNym>,
        opening: &Opening,
        key: Hash,
    ) -> Result<(), StateError> {
        let caller = caller.ok_or(StateError::Authentication)?;
        let position = caller.position(&self.context.lineage);
        if opening.path.position != position {
            return Err(StateError::Position);
        }
        if self.live.contains_key(&position) {
            return Err(StateError::NotFrozen);
        }
        let entry = self
            .shape
            .entries
            .iter()
            .find(|e| e.position == position)
            .ok_or(StateError::NotFrozen)?;
        if entry.origin == Origin::Live {
            return Err(StateError::NotFrozen);
        }
        let context = origin_context(&self.context, entry.origin);
        verify_reentry(&context, &self.root(), Some(caller), opening)?;
        if salt(&key, &context) != opening.salt || opening.leaf(&context) != entry.leaf_hash {
            return Err(StateError::Opening);
        }
        let entry = self
            .shape
            .entries
            .iter_mut()
            .find(|e| e.position == position)
            .ok_or(StateError::NotFrozen)?;
        entry.leaf_hash = live_leaf(&self.context, &position, &opening.records, &key);
        entry.origin = Origin::Live;
        self.live.insert(
            position,
            Live {
                records: opening.records.clone(),
                key,
            },
        );
        Ok(())
    }
}

fn live_leaf(context: &Context, position: &Position, records: &Records, key: &Hash) -> Hash {
    leaf_hash(position, &commitment(context, records, &salt(key, context)))
}

fn origin_context(current: &Context, origin: Origin) -> Context {
    match origin {
        Origin::Live => *current,
        Origin::Frozen { network_id, epoch } => Context {
            network_id,
            epoch,
            ..*current
        },
    }
}
