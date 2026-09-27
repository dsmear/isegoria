//! Replication between nodes (`docs/04` §Replication between nodes, `docs/08` NET-010): the
//! grow-only set of writers' signed entries, what it reads, the pull sync and its messages.

use crate::cid::{cid, Cid};
use crate::codec::{Reader, Writer};
use crate::hash::tagged;
use crate::log::{entry_hash, Entry};
use crate::store::{ObjectStore, StoreError, MAX_OBJECT};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use std::collections::BTreeMap;
use std::path::Path;

/// The most entry and object bytes one Entries message carries (at least one entry).
pub const MAX_RESPONSE: usize = 32 << 20;
const VERSION: u8 = 1;
const SUMMARY_RECORD: usize = 112;
const ID_LEN: usize = 72;

/// An entry's identity: another signature over the same entry is a duplicate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryId {
    pub writer: [u8; 32],
    pub seq: u64,
    pub hash: [u8; 32],
}

impl EntryId {
    fn write(&self, w: &mut Writer) {
        w.fixed(&self.writer).u64(self.seq).fixed(&self.hash);
    }

    fn read(r: &mut Reader) -> Option<Self> {
        Some(EntryId {
            writer: r.fixed().ok()?,
            seq: r.u64().ok()?,
            hash: r.fixed().ok()?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedEntry {
    pub writer: [u8; 32],
    pub entry: Entry,
    pub signature: [u8; 64],
}

fn entry_message(network_id: &[u8; 32], writer: &[u8; 32], hash: &[u8; 32]) -> [u8; 32] {
    tagged("isegoria/feed/entry/v1", &[network_id, writer, hash])
}

impl SignedEntry {
    /// The 168-byte encoding: writer, `seq`, `prev`, payload, signature.
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        self.write(&mut w);
        w.finish()
    }

    /// The entry of [`encode`](Self::encode)'s bytes, its hash recomputed.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader::new(bytes);
        let e = Self::read(&mut r)?;
        r.finish().ok()?;
        Some(e)
    }

    pub fn id(&self) -> EntryId {
        EntryId {
            writer: self.writer,
            seq: self.entry.seq,
            hash: self.entry.hash,
        }
    }

    fn write(&self, w: &mut Writer) {
        w.fixed(&self.writer)
            .u64(self.entry.seq)
            .fixed(&self.entry.prev)
            .fixed(&self.entry.payload.0)
            .fixed(&self.signature);
    }

    fn read(r: &mut Reader) -> Option<Self> {
        let writer = r.fixed().ok()?;
        let seq = r.u64().ok()?;
        let prev = r.fixed().ok()?;
        let payload = Cid(r.fixed().ok()?);
        let signature = r.fixed().ok()?;
        let hash = entry_hash(seq, &prev, &payload);
        Some(SignedEntry {
            writer,
            entry: Entry {
                seq,
                prev,
                payload,
                hash,
            },
            signature,
        })
    }
}

/// An infrastructure node's writer key (a consortium member or a relay).
pub struct FeedWriter {
    network_id: [u8; 32],
    key: SigningKey,
}

impl FeedWriter {
    pub fn from_seed(network_id: [u8; 32], seed: [u8; 32]) -> Self {
        FeedWriter {
            network_id,
            key: SigningKey::from_bytes(&seed),
        }
    }

    pub fn public(&self) -> VerifyingKey {
        self.key.verifying_key()
    }

    /// Signs `entry` as this writer's; the entry's hash is taken as recomputed by the log.
    pub fn sign(&self, entry: &Entry) -> SignedEntry {
        let writer = self.key.verifying_key().to_bytes();
        let message = entry_message(&self.network_id, &writer, &entry.hash);
        SignedEntry {
            writer,
            entry: entry.clone(),
            signature: self.key.sign(&message).to_bytes(),
        }
    }
}

/// The keys allowed to write on one network.
#[derive(Clone, Debug)]
pub struct WriterSet {
    network_id: [u8; 32],
    keys: BTreeMap<[u8; 32], VerifyingKey>,
}

impl WriterSet {
    pub fn new(network_id: [u8; 32], keys: &[VerifyingKey]) -> Self {
        WriterSet {
            network_id,
            keys: keys.iter().map(|k| (k.to_bytes(), *k)).collect(),
        }
    }

    pub fn contains(&self, key: &VerifyingKey) -> bool {
        self.keys.contains_key(&key.to_bytes())
    }

    pub fn network_id(&self) -> [u8; 32] {
        self.network_id
    }

    fn check(&self, e: &SignedEntry) -> Result<(), Refused> {
        let key = self.keys.get(&e.writer).ok_or(Refused::UnknownWriter)?;
        let message = entry_message(&self.network_id, &e.writer, &e.entry.hash);
        key.verify_strict(&message, &Signature::from_bytes(&e.signature))
            .map_err(|_| Refused::BadSignature)
    }
}

/// Two entries one writer signed at one `seq`: evidence anyone checks against the writer set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Equivocation {
    pub a: SignedEntry,
    pub b: SignedEntry,
}

impl Equivocation {
    pub fn verify(&self, writers: &WriterSet) -> bool {
        self.a.writer == self.b.writer
            && self.a.entry.seq == self.b.entry.seq
            && self.a.entry.hash != self.b.entry.hash
            && writers.check(&self.a).is_ok()
            && writers.check(&self.b).is_ok()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Accepted {
    New,
    Duplicate,
    /// New, at a `seq` where the writer already had another entry.
    Equivocation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    UnknownWriter,
    BadSignature,
    ObjectTooLarge,
    /// The object's CID is not the entry's payload.
    ObjectMismatch,
}

/// One writer's part of a [`Summary`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriterSummary {
    pub writer: [u8; 32],
    pub count: u64,
    pub digest: [u8; 32],
    pub feed_len: u64,
    pub head: [u8; 32],
}

/// Per writer held, sorted by writer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Summary(pub Vec<WriterSummary>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    Summary(Summary),
    Have(Vec<EntryId>),
    Want(Vec<EntryId>),
    Entries(Vec<(SignedEntry, Vec<u8>)>),
}

fn digest(domain: &str, ids: impl Iterator<Item = EntryId>) -> [u8; 32] {
    let mut w = Writer::new();
    for id in ids {
        id.write(&mut w);
    }
    tagged(domain, &[&w.finish()])
}

/// The set of accepted entries, each with its object.
#[derive(Clone, Debug)]
pub struct Replica {
    writers: WriterSet,
    entries: BTreeMap<EntryId, (SignedEntry, Vec<u8>)>,
}

impl Replica {
    pub fn new(writers: WriterSet) -> Self {
        Replica {
            writers,
            entries: BTreeMap::new(),
        }
    }

    pub fn writers(&self) -> &WriterSet {
        &self.writers
    }

    /// What [`insert`](Self::insert) would answer, without inserting.
    pub fn check(&self, entry: &SignedEntry, object: &[u8]) -> Result<Accepted, Refused> {
        self.writers.check(entry)?;
        if object.len() > MAX_OBJECT {
            return Err(Refused::ObjectTooLarge);
        }
        if cid(object) != entry.entry.payload {
            return Err(Refused::ObjectMismatch);
        }
        let id = entry.id();
        Ok(if self.entries.contains_key(&id) {
            Accepted::Duplicate
        } else if self.at(&id.writer, id.seq).next().is_some() {
            Accepted::Equivocation
        } else {
            Accepted::New
        })
    }

    pub fn insert(&mut self, entry: SignedEntry, object: Vec<u8>) -> Result<Accepted, Refused> {
        let accepted = self.check(&entry, &object)?;
        if accepted != Accepted::Duplicate {
            self.entries.insert(entry.id(), (entry, object));
        }
        Ok(accepted)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn contains(&self, id: &EntryId) -> bool {
        self.entries.contains_key(id)
    }

    pub fn get(&self, id: &EntryId) -> Option<(&SignedEntry, &[u8])> {
        self.entries.get(id).map(|(e, o)| (e, o.as_slice()))
    }

    pub fn ids(&self) -> impl Iterator<Item = EntryId> + '_ {
        self.entries.keys().copied()
    }

    fn of(&self, writer: &[u8; 32]) -> impl Iterator<Item = &SignedEntry> + '_ {
        let lo = EntryId {
            writer: *writer,
            seq: 0,
            hash: [0; 32],
        };
        let hi = EntryId {
            writer: *writer,
            seq: u64::MAX,
            hash: [0xff; 32],
        };
        self.entries.range(lo..=hi).map(|(_, (e, _))| e)
    }

    fn at(&self, writer: &[u8; 32], seq: u64) -> impl Iterator<Item = &SignedEntry> + '_ {
        self.of(writer).filter(move |e| e.entry.seq == seq)
    }

    /// `writer`'s chain of `len` entries ending with `head`, from `seq` 0; `None` while an
    /// entry of it is missing.
    pub fn chain(&self, writer: &[u8; 32], len: u64, head: [u8; 32]) -> Option<Vec<&SignedEntry>> {
        let mut chain = Vec::new();
        let mut hash = head;
        for seq in (0..len).rev() {
            let id = EntryId {
                writer: *writer,
                seq,
                hash,
            };
            let (e, _) = self.entries.get(&id)?;
            hash = e.entry.prev;
            chain.push(e);
        }
        (hash == [0; 32]).then(|| {
            chain.reverse();
            chain
        })
    }

    /// The digest of the whole set: equal on two replicas exactly when their sets are.
    pub fn digest(&self) -> [u8; 32] {
        digest("isegoria/replica/set/v1", self.ids())
    }

    /// `writer`'s entries from `seq` 0, up to the first gap, fork or broken link.
    pub fn feed(&self, writer: &[u8; 32]) -> Vec<&SignedEntry> {
        let mut feed: Vec<&SignedEntry> = Vec::new();
        let mut entries = self.of(writer).peekable();
        while let Some(e) = entries.next() {
            let seq = feed.len() as u64;
            let prev = feed.last().map_or([0; 32], |p| p.entry.hash);
            let forked = entries.peek().is_some_and(|n| n.entry.seq == seq);
            if e.entry.seq != seq || e.entry.prev != prev || forked {
                break;
            }
            feed.push(e);
        }
        feed
    }

    /// Per writer and `seq` holding several entries, the two with the smallest hashes.
    pub fn equivocations(&self) -> Vec<Equivocation> {
        let mut found = Vec::new();
        let mut last: Option<&SignedEntry> = None;
        for (e, _) in self.entries.values() {
            if let Some(p) = last.filter(|p| p.writer == e.writer && p.entry.seq == e.entry.seq) {
                if found.last().is_none_or(|q: &Equivocation| q.a != *p) {
                    found.push(Equivocation {
                        a: p.clone(),
                        b: e.clone(),
                    });
                }
                continue;
            }
            last = Some(e);
        }
        found
    }

    pub fn summary(&self) -> Summary {
        let mut writers: Vec<[u8; 32]> = self.entries.keys().map(|id| id.writer).collect();
        writers.dedup();
        Summary(
            writers
                .into_iter()
                .map(|writer| {
                    let feed = self.feed(&writer);
                    WriterSummary {
                        writer,
                        count: self.of(&writer).count() as u64,
                        digest: digest(
                            "isegoria/replica/writer/v1",
                            self.of(&writer).map(|e| e.id()),
                        ),
                        feed_len: feed.len() as u64,
                        head: feed.last().map_or([0; 32], |e| e.entry.hash),
                    }
                })
                .collect(),
        )
    }

    /// The identifiers to offer a peer that sent `peer`: every entry of a writer whose
    /// digest differs, less the prefix of this feed the peer's head commits it to.
    pub fn have_for(&self, peer: &Summary) -> Vec<EntryId> {
        let mine = self.summary();
        let mut have = Vec::new();
        for w in &mine.0 {
            let theirs = peer.0.iter().find(|p| p.writer == w.writer);
            if theirs.is_some_and(|p| p.digest == w.digest) {
                continue;
            }
            let feed = self.feed(&w.writer);
            let skip = theirs.map_or(0, |p| {
                let n = usize::try_from(p.feed_len).unwrap_or(usize::MAX);
                match n.checked_sub(1).and_then(|i| feed.get(i)) {
                    Some(e) if e.entry.hash == p.head => p.feed_len,
                    _ => 0,
                }
            });
            have.extend(
                self.of(&w.writer)
                    .map(|e| e.id())
                    .filter(|id| id.seq >= skip),
            );
        }
        have
    }

    /// Of `offered`, those this replica lacks.
    pub fn want(&self, offered: &[EntryId]) -> Vec<EntryId> {
        offered
            .iter()
            .filter(|id| !self.contains(id))
            .copied()
            .collect()
    }

    /// The entries of `wanted` held here, in identifier order, within `cap` bytes but at
    /// least one.
    pub fn entries_for(&self, wanted: &[EntryId], cap: usize) -> Vec<(SignedEntry, Vec<u8>)> {
        let mut ids: Vec<&EntryId> = wanted.iter().filter(|id| self.contains(id)).collect();
        ids.sort();
        ids.dedup();
        let mut out = Vec::new();
        let mut size = 0usize;
        for id in ids {
            let (e, o) = &self.entries[id];
            size = size.saturating_add(o.len() + 168);
            if size > cap && !out.is_empty() {
                break;
            }
            out.push((e.clone(), o.clone()));
        }
        out
    }
}

fn write_ids(w: &mut Writer, ids: &[EntryId]) {
    let mut f = Writer::new();
    for id in ids {
        id.write(&mut f);
    }
    w.field(&f.finish());
}

fn read_ids(r: &mut Reader) -> Option<Vec<EntryId>> {
    let bytes = r.field().ok()?;
    if bytes.len() % ID_LEN != 0 {
        return None;
    }
    let mut f = Reader::new(bytes);
    let ids: Vec<EntryId> = (0..bytes.len() / ID_LEN)
        .map(|_| EntryId::read(&mut f))
        .collect::<Option<_>>()?;
    ids.windows(2).all(|p| p[0] < p[1]).then_some(ids)
}

impl Message {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(VERSION);
        match self {
            Message::Summary(s) => {
                let mut f = Writer::new();
                for r in &s.0 {
                    f.fixed(&r.writer)
                        .u64(r.count)
                        .fixed(&r.digest)
                        .u64(r.feed_len)
                        .fixed(&r.head);
                }
                w.u8(1).field(&f.finish());
            }
            Message::Have(ids) => write_ids(w.u8(2), ids),
            Message::Want(ids) => write_ids(w.u8(3), ids),
            Message::Entries(list) => {
                w.u8(4).u64(list.len() as u64);
                for (e, o) in list {
                    e.write(&mut w);
                    w.field(o);
                }
            }
        }
        w.finish()
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader::new(bytes);
        if r.u8().ok()? != VERSION {
            return None;
        }
        let message = match r.u8().ok()? {
            1 => {
                let bytes = r.field().ok()?;
                if bytes.len() % SUMMARY_RECORD != 0 {
                    return None;
                }
                let mut f = Reader::new(bytes);
                let records: Vec<WriterSummary> = (0..bytes.len() / SUMMARY_RECORD)
                    .map(|_| {
                        Some(WriterSummary {
                            writer: f.fixed().ok()?,
                            count: f.u64().ok()?,
                            digest: f.fixed().ok()?,
                            feed_len: f.u64().ok()?,
                            head: f.fixed().ok()?,
                        })
                    })
                    .collect::<Option<_>>()?;
                if !records.windows(2).all(|p| p[0].writer < p[1].writer) {
                    return None;
                }
                Message::Summary(Summary(records))
            }
            2 => Message::Have(read_ids(&mut r)?),
            3 => Message::Want(read_ids(&mut r)?),
            4 => {
                let n = r.u64().ok()?;
                let mut list: Vec<(SignedEntry, Vec<u8>)> = Vec::new();
                for _ in 0..n {
                    let e = SignedEntry::read(&mut r)?;
                    let o = r.field().ok()?.to_vec();
                    if list.last().is_some_and(|(p, _)| p.id() >= e.id()) {
                        return None;
                    }
                    list.push((e, o));
                }
                Message::Entries(list)
            }
            _ => return None,
        };
        r.finish().ok()?;
        Some(message)
    }
}

/// Why a replica on disk did not take an entry, or did not open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiskError {
    Refused(Refused),
    Store(StoreError),
    /// A stored entry does not decode or its object is missing: the files were damaged.
    Damaged,
}

/// A replica kept in `entries` and `objects` under one directory (`docs/04` §A replica on
/// disk).
pub struct DurableReplica {
    replica: Replica,
    entries: ObjectStore,
    objects: ObjectStore,
}

impl DurableReplica {
    /// Opens or creates the files and re-inserts every stored entry.
    pub fn open(dir: &Path, writers: WriterSet) -> Result<Self, DiskError> {
        let store = |e| DiskError::Store(e);
        std::fs::create_dir_all(dir).map_err(|e| DiskError::Store(StoreError::Io(e.kind())))?;
        let (entries, _) = ObjectStore::open(&dir.join("entries")).map_err(store)?;
        let (objects, _) = ObjectStore::open(&dir.join("objects")).map_err(store)?;
        let mut replica = Replica::new(writers);
        for id in entries.cids() {
            let bytes = entries.get(&id).map_err(store)?.ok_or(DiskError::Damaged)?;
            let entry = SignedEntry::decode(&bytes).ok_or(DiskError::Damaged)?;
            let object = objects
                .get(&entry.entry.payload)
                .map_err(store)?
                .ok_or(DiskError::Damaged)?;
            replica.insert(entry, object).map_err(DiskError::Refused)?;
        }
        Ok(DurableReplica {
            replica,
            entries,
            objects,
        })
    }

    /// Checks the entry, writes its object then the entry, then inserts it.
    pub fn insert(&mut self, entry: SignedEntry, object: Vec<u8>) -> Result<Accepted, DiskError> {
        let accepted = self
            .replica
            .check(&entry, &object)
            .map_err(DiskError::Refused)?;
        if accepted != Accepted::Duplicate {
            self.objects.put(&object).map_err(DiskError::Store)?;
            self.entries
                .put(&entry.encode())
                .map_err(DiskError::Store)?;
            self.replica
                .insert(entry, object)
                .map_err(DiskError::Refused)?;
        }
        Ok(accepted)
    }

    pub fn replica(&self) -> &Replica {
        &self.replica
    }
}
