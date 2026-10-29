//! Assigned public archive shards and checked retrieval (`docs/04` §Public archive).

use crate::{read_limited, write_all};
use futures::{AsyncRead, AsyncWrite, StreamExt};
use libp2p::kad::{self, RecordKey};
use libp2p::request_response::{self, OutboundRequestId, ProtocolSupport};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{identity, noise, tcp, yamux, Multiaddr, PeerId, StreamProtocol};
use network::archive::{BuiltArchive, Context, Recovered, Segment, SignedManifest, MAX_MANIFEST};
use network::codec::{Reader, Writer};
use network::erasure::shard_hash;
use network::store::MAX_OBJECT;
use network::tier::Tiers;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[path = "archive_periodic.rs"]
mod periodic;
pub use periodic::{MaintenanceJob, MaintenanceReport, MaintenanceSchedule, ScheduleError};

pub const REQUEST_LIMIT: u64 = 41;
pub const REPLY_LIMIT: u64 = MAX_OBJECT as u64 + 42;
pub const MAX_RECORD: usize = (64 << 20) + MAX_MANIFEST + 49 + 256 * 16;
const MAX_ARCHIVES: usize = 256;
const MAX_SHARDS: usize = 4096;
const MAX_BYTES: u64 = 256 << 20;
const MAX_ENTRIES: usize = 512;

#[derive(Debug, PartialEq, Eq)]
pub enum InstallError {
    Archive(network::archive::Error),
    Indices,
    Configuration,
    Quota,
    Io(io::ErrorKind),
}
impl From<io::Error> for InstallError {
    fn from(error: io::Error) -> Self {
        Self::Io(error.kind())
    }
}
impl From<network::archive::Error> for InstallError {
    fn from(error: network::archive::Error) -> Self {
        Self::Archive(error)
    }
}

pub const MAX_ROSTER: usize = 1024;

#[derive(Debug, PartialEq, Eq)]
pub enum Maintenance {
    Healthy { index: usize },
    Repaired { index: usize },
    Unassigned,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MaintainError {
    Archive(network::archive::Error),
    Roster,
    Identity,
    Install(InstallError),
    Fetch(FetchError),
    Timeout,
}

#[derive(Clone, Copy)]
pub(crate) struct Deadline {
    started: Instant,
    budget: Duration,
}
impl Deadline {
    pub(crate) fn remaining(self) -> Result<Duration, MaintainError> {
        self.budget
            .checked_sub(self.started.elapsed())
            .filter(|d| !d.is_zero())
            .ok_or(MaintainError::Timeout)
    }
}

pub(crate) struct Audit {
    indices: Vec<usize>,
    healthy: bool,
}

impl crate::Handle {
    /// Audits this node's roster assignment and pulls a verified replacement if needed.
    /// The deadline ends at commit acceptance; accepted durable writes are not canceled.
    pub async fn maintain_archive(
        &mut self,
        ctx: &Context<'_>,
        signed: &SignedManifest,
        roster: &[[u8; 32]],
        bootstrap: Vec<Multiaddr>,
        deadline: Duration,
    ) -> Result<Maintenance, MaintainError> {
        let deadline = Deadline {
            started: Instant::now(),
            budget: deadline,
        };
        let local = crate::tiered::transport_key(&self.peer_id).ok_or(MaintainError::Identity)?;
        if roster.len() > MAX_ROSTER
            || roster.len() < signed.manifest.shard_hashes().len()
            || roster.iter().copied().collect::<BTreeSet<_>>().len() != roster.len()
            || roster
                .iter()
                .any(|key| ed25519_dalek::VerifyingKey::from_bytes(key).is_err())
        {
            return Err(MaintainError::Roster);
        }
        let verified = signed.verify(ctx).map_err(MaintainError::Archive)?;
        let placement = verified
            .placement(ctx, roster)
            .map_err(MaintainError::Archive)?;
        let index = placement.iter().position(|key| *key == local);
        deadline.remaining()?;
        let audit = self
            .ask(|tx| {
                crate::Command::AuditArchive(
                    ctx.tiers.context(),
                    signed.manifest.digest(),
                    index,
                    tx,
                )
            })
            .await
            .map_err(MaintainError::Install)?;
        deadline.remaining()?;
        let Some(index) = index else {
            return Ok(Maintenance::Unassigned);
        };
        if audit.healthy {
            return Ok(Maintenance::Healthy { index });
        }
        let recovered = fetch_discovered(ctx, signed, bootstrap, deadline.remaining()?)
            .await
            .map_err(|error| match error {
                FetchError::Timeout => MaintainError::Timeout,
                error => MaintainError::Fetch(error),
            })?;
        deadline.remaining()?;
        let mut indices = audit.indices;
        if !indices.contains(&index) {
            indices.push(index);
        }
        let prepared = Prepared::new(ctx, signed, &recovered.segment, &indices)
            .map_err(MaintainError::Install)?;
        deadline.remaining()?;
        self.ask(|tx| crate::Command::MaintainArchive(prepared, deadline, tx))
            .await?;
        Ok(Maintenance::Repaired { index })
    }
}

pub(crate) struct Prepared {
    context: [u8; 32],
    record: Record,
}
impl Prepared {
    pub(crate) fn digest(&self) -> [u8; 32] {
        self.record.signed.manifest.digest()
    }
    pub(crate) fn new(
        ctx: &Context<'_>,
        signed: &SignedManifest,
        segment: &Segment,
        indices: &[usize],
    ) -> Result<Self, InstallError> {
        signed.verify(ctx)?;
        let n = signed.manifest.shard_hashes().len();
        if indices.len() > n {
            return Err(InstallError::Indices);
        }
        let selected: BTreeSet<_> = indices.iter().copied().collect();
        if selected.is_empty()
            || selected.len() != indices.len()
            || selected.iter().any(|i| *i >= n)
        {
            return Err(InstallError::Indices);
        }
        let built = BuiltArchive::build(ctx, segment, signed.manifest.layout())?;
        if built.manifest != signed.manifest {
            return Err(network::archive::Error::Commitment.into());
        }
        Ok(Self {
            context: ctx.tiers.context(),
            record: Record {
                signed: signed.clone(),
                shards: built
                    .shards
                    .into_iter()
                    .enumerate()
                    .filter(|(i, _)| selected.contains(i))
                    .collect(),
            },
        })
    }
}
struct Record {
    signed: SignedManifest,
    shards: BTreeMap<usize, Vec<u8>>,
}
impl Record {
    fn size(&self) -> Result<usize, InstallError> {
        Ok(49
            + self.signed.encode()?.len()
            + self.shards.values().map(|s| 16 + s.len()).sum::<usize>())
    }
    fn encode(&self, context: &[u8; 32]) -> Result<Vec<u8>, InstallError> {
        let mut w = Writer::new();
        w.u8(1)
            .fixed(context)
            .field(&self.signed.encode()?)
            .u64(self.shards.len() as u64);
        for (index, shard) in &self.shards {
            w.u64(*index as u64).field(shard);
        }
        Ok(w.finish())
    }
    fn decode(bytes: &[u8], tiers: &Tiers, digest: [u8; 32]) -> Option<Self> {
        if bytes.len() > MAX_RECORD {
            return None;
        }
        let mut r = Reader::new(bytes);
        if r.u8().ok()? != 1 || r.fixed::<32>().ok()? != tiers.context() {
            return None;
        }
        let signed = SignedManifest::decode(r.field().ok()?).ok()?;
        signed.verify_certificate(tiers).ok()?;
        if signed.manifest.digest() != digest {
            return None;
        }
        let count = usize::try_from(r.u64().ok()?).ok()?;
        let n = signed.manifest.shard_hashes().len();
        if count == 0 || count > n {
            return None;
        }
        let width = signed
            .manifest
            .orig_len()
            .div_ceil(signed.manifest.layout().data_shards);
        let mut shards = BTreeMap::new();
        for _ in 0..count {
            let index = usize::try_from(r.u64().ok()?).ok()?;
            if index >= n
                || shards
                    .last_key_value()
                    .is_some_and(|(old, _)| *old >= index)
            {
                return None;
            }
            let shard = r.field().ok()?;
            if shard.len() != width || shard_hash(shard) != signed.manifest.shard_hashes()[index] {
                return None;
            }
            shards.insert(index, shard.to_vec());
        }
        r.finish().ok()?;
        Some(Self { signed, shards })
    }
}

type Retained = BTreeMap<[u8; 32], Vec<usize>>;

pub(crate) struct Store {
    context: [u8; 32],
    dir: Option<PathBuf>,
    records: BTreeMap<[u8; 32], Record>,
    temporary: u64,
    retained: Retained,
}
#[derive(Default)]
struct Usage {
    entries: usize,
    archives: usize,
    shards: usize,
    bytes: u64,
}
impl Usage {
    fn check(&self) -> Result<(), InstallError> {
        if self.entries > MAX_ENTRIES
            || self.archives > MAX_ARCHIVES
            || self.shards > MAX_SHARDS
            || self.bytes > MAX_BYTES
        {
            Err(InstallError::Quota)
        } else {
            Ok(())
        }
    }
}
fn hex(digest: &[u8; 32]) -> String {
    digest.iter().map(|b| format!("{b:02x}")).collect()
}
fn file_digest(path: &Path) -> Option<[u8; 32]> {
    let name = path.file_name()?.to_str()?.strip_suffix(".arc")?;
    if name.len() != 64
        || !name
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return None;
    }
    let mut digest = [0; 32];
    for (i, byte) in digest.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&name[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(digest)
}
fn read_record(path: &Path, tiers: &Tiers, digest: [u8; 32]) -> Option<Record> {
    let metadata = fs::symlink_metadata(path).ok()?;
    if !metadata.file_type().is_file() || metadata.len() > MAX_RECORD as u64 {
        return None;
    }
    let mut bytes = Vec::new();
    File::open(path)
        .ok()?
        .take(MAX_RECORD as u64 + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    Record::decode(&bytes, tiers, digest)
}
impl Store {
    pub(crate) fn open(tiers: &Tiers, dir: Option<PathBuf>) -> Result<Self, InstallError> {
        let mut store = Self {
            context: tiers.context(),
            dir,
            records: BTreeMap::new(),
            temporary: 0,
            retained: BTreeMap::new(),
        };
        if let Some(dir) = &store.dir {
            fs::create_dir_all(dir)?;
            let header = dir.join("config");
            let mut expected = vec![1];
            expected.extend(store.context);
            match File::open(&header) {
                Ok(file) => {
                    let mut bytes = Vec::new();
                    file.take(34).read_to_end(&mut bytes)?;
                    if bytes != expected {
                        return Err(InstallError::Configuration);
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    if fs::read_dir(dir)?.next().is_some() {
                        return Err(InstallError::Configuration);
                    }
                    let mut file = OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&header)?;
                    file.write_all(&expected)?;
                    file.sync_all()?;
                    File::open(dir)?.sync_all()?;
                }
                Err(error) => return Err(error.into()),
            }
            let (usage, retained) = store.inspect(tiers)?;
            usage.check()?;
            store.retained = retained;
        }
        Ok(store)
    }
    pub(crate) fn advertisements(&self) -> Vec<RecordKey> {
        self.retained
            .iter()
            .flat_map(|(digest, indices)| indices.iter().map(|index| provider_key(digest, *index)))
            .collect()
    }
    pub(crate) fn keys(&self, digest: &[u8; 32]) -> Vec<RecordKey> {
        self.retained
            .get(digest)
            .into_iter()
            .flatten()
            .map(|index| provider_key(digest, *index))
            .collect()
    }
    fn usage(&self, tiers: &Tiers) -> Result<Usage, InstallError> {
        self.inspect(tiers).map(|(usage, _)| usage)
    }
    fn inspect(&self, tiers: &Tiers) -> Result<(Usage, Retained), InstallError> {
        let mut usage = Usage::default();
        let mut retained = BTreeMap::new();
        if let Some(dir) = &self.dir {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                usage.entries += 1;
                if usage.entries > MAX_ENTRIES {
                    return Err(InstallError::Quota);
                }
                let metadata = fs::symlink_metadata(entry.path())?;
                usage.bytes = usage
                    .bytes
                    .checked_add(metadata.len())
                    .ok_or(InstallError::Quota)?;
                if entry.path().extension().is_some_and(|e| e == "arc") {
                    usage.archives += 1;
                    let record = file_digest(&entry.path()).and_then(|digest| {
                        let record = read_record(&entry.path(), tiers, digest)?;
                        retained.insert(digest, record.shards.keys().copied().collect());
                        Some(record)
                    });
                    usage.shards += record.map_or(256, |r| r.shards.len());
                }
                usage.check()?;
            }
        } else {
            usage.archives = self.records.len();
            for record in self.records.values() {
                usage.shards += record.shards.len();
                usage.bytes += record.size()? as u64;
            }
        }
        Ok((usage, retained))
    }
    pub(crate) fn install(
        &mut self,
        tiers: &Tiers,
        prepared: Prepared,
    ) -> Result<(), InstallError> {
        if self.context != tiers.context() || prepared.context != self.context {
            return Err(InstallError::Configuration);
        }
        let record = prepared.record;
        let digest = record.signed.manifest.digest();
        let indices = record.shards.keys().copied().collect();
        let mut usage = self.usage(tiers)?;
        if let Some(dir) = &self.dir {
            let old_path = dir.join(format!("{}.arc", hex(&digest)));
            match fs::symlink_metadata(&old_path) {
                Ok(_) => {
                    usage.archives -= 1;
                    usage.shards -=
                        read_record(&old_path, tiers, digest).map_or(256, |r| r.shards.len());
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        } else if let Some(old) = self.records.get(&digest) {
            usage.archives -= 1;
            usage.shards -= old.shards.len();
            usage.bytes -= old.size()? as u64;
        }
        usage.archives += 1;
        usage.shards += record.shards.len();
        usage.bytes += record.size()? as u64;
        if self.dir.is_some() {
            usage.entries += 1;
        }
        usage.check()?;
        if let Some(dir) = &self.dir {
            let bytes = record.encode(&self.context)?;
            let (path, mut file) = loop {
                self.temporary = self.temporary.wrapping_add(1);
                let path = dir.join(format!(".{}-{}.tmp", hex(&digest), self.temporary));
                match OpenOptions::new().write(true).create_new(true).open(&path) {
                    Ok(file) => break (path, file),
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                    Err(error) => return Err(error.into()),
                }
            };
            let result = (|| -> io::Result<()> {
                file.write_all(&bytes)?;
                file.sync_all()?;
                fs::rename(&path, dir.join(format!("{}.arc", hex(&digest))))?;
                File::open(dir)?.sync_all()
            })();
            if result.is_err() {
                let _ = fs::remove_file(&path);
            }
            result?;
        } else {
            self.records.insert(digest, record);
        }
        self.retained.insert(digest, indices);
        Ok(())
    }
    pub(crate) fn audit(
        &self,
        tiers: &Tiers,
        context: [u8; 32],
        digest: [u8; 32],
        index: Option<usize>,
    ) -> Result<Audit, InstallError> {
        if context != self.context || tiers.context() != self.context {
            return Err(InstallError::Configuration);
        }
        let Some(index) = index else {
            return Ok(Audit {
                indices: Vec::new(),
                healthy: false,
            });
        };
        let disk;
        let record = if let Some(dir) = &self.dir {
            disk = read_record(&dir.join(format!("{}.arc", hex(&digest))), tiers, digest);
            disk.as_ref()
        } else {
            self.records.get(&digest)
        };
        let mut indices: BTreeSet<_> = self
            .retained
            .get(&digest)
            .into_iter()
            .flatten()
            .copied()
            .collect();
        if let Some(record) = record {
            indices.extend(record.shards.keys().copied());
        }
        Ok(Audit {
            indices: indices.into_iter().collect(),
            healthy: record.is_some_and(|r| r.shards.contains_key(&index)),
        })
    }
    pub(crate) fn serve(&self, tiers: &Tiers, request: &[u8]) -> Vec<u8> {
        let Some((digest, index)) = parse_request(request) else {
            return vec![1, 0];
        };
        let disk;
        let record = if let Some(dir) = &self.dir {
            disk = read_record(&dir.join(format!("{}.arc", hex(&digest))), tiers, digest);
            disk.as_ref()
        } else {
            self.records.get(&digest)
        };
        record
            .and_then(|r| r.shards.get(&index))
            .map_or_else(|| vec![1, 0], |shard| reply(digest, index, shard))
    }
}

pub fn provider_key(digest: &[u8; 32], index: usize) -> RecordKey {
    let mut key = b"isegoria/archive/shard/1/".to_vec();
    key.extend(digest);
    key.extend((index as u64).to_le_bytes());
    RecordKey::new(&key)
}

pub fn protocol(network: &[u8; 32]) -> StreamProtocol {
    StreamProtocol::try_from_owned(format!("/isegoria/{}/archive/1", hex(network)))
        .expect("valid protocol")
}
#[derive(Clone, Default)]
pub struct ShardCodec;
#[async_trait::async_trait]
impl request_response::Codec for ShardCodec {
    type Protocol = StreamProtocol;
    type Request = Vec<u8>;
    type Response = Vec<u8>;
    async fn read_request<T: AsyncRead + Unpin + Send>(
        &mut self,
        _: &StreamProtocol,
        io: &mut T,
    ) -> io::Result<Vec<u8>> {
        let bytes = read_limited(io, REQUEST_LIMIT).await?;
        parse_request(&bytes)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid shard request"))?;
        Ok(bytes)
    }
    async fn read_response<T: AsyncRead + Unpin + Send>(
        &mut self,
        _: &StreamProtocol,
        io: &mut T,
    ) -> io::Result<Vec<u8>> {
        read_limited(io, REPLY_LIMIT).await
    }
    async fn write_request<T: AsyncWrite + Unpin + Send>(
        &mut self,
        _: &StreamProtocol,
        io: &mut T,
        bytes: Vec<u8>,
    ) -> io::Result<()> {
        write_all(io, bytes).await
    }
    async fn write_response<T: AsyncWrite + Unpin + Send>(
        &mut self,
        _: &StreamProtocol,
        io: &mut T,
        bytes: Vec<u8>,
    ) -> io::Result<()> {
        write_all(io, bytes).await
    }
}
pub(crate) fn behaviour(network: &[u8; 32]) -> request_response::Behaviour<ShardCodec> {
    configured_behaviour(network, ProtocolSupport::Inbound)
}
fn configured_behaviour(
    network: &[u8; 32],
    support: ProtocolSupport,
) -> request_response::Behaviour<ShardCodec> {
    request_response::Behaviour::new(
        [(protocol(network), support)],
        request_response::Config::default()
            .with_request_timeout(Duration::from_secs(3))
            .with_max_concurrent_streams(8),
    )
}
fn request(digest: [u8; 32], index: usize) -> Vec<u8> {
    Writer::new()
        .u8(1)
        .fixed(&digest)
        .u64(index as u64)
        .finish()
}
fn parse_request(bytes: &[u8]) -> Option<([u8; 32], usize)> {
    if bytes.len() != REQUEST_LIMIT as usize {
        return None;
    }
    let mut r = Reader::new(bytes);
    if r.u8().ok()? != 1 {
        return None;
    }
    Some((r.fixed().ok()?, usize::try_from(r.u64().ok()?).ok()?))
}
fn reply(digest: [u8; 32], index: usize, shard: &[u8]) -> Vec<u8> {
    Writer::new()
        .u8(1)
        .u8(1)
        .fixed(&digest)
        .u64(index as u64)
        .fixed(shard)
        .finish()
}
fn checked(signed: &SignedManifest, index: usize, bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.len() < 42 || bytes.len() as u64 > REPLY_LIMIT || bytes[..2] != [1, 1] {
        return None;
    }
    let (digest, echoed) = parse_request(&[&[1][..], &bytes[2..42]].concat())?;
    let m = &signed.manifest;
    let shard = &bytes[42..];
    (digest == m.digest()
        && echoed == index
        && shard.len() == m.orig_len().div_ceil(m.layout().data_shards)
        && shard_hash(shard) == *m.shard_hashes().get(index)?)
    .then(|| shard.to_vec())
}

pub struct Holder {
    pub index: usize,
    pub peer: PeerId,
    pub address: Multiaddr,
}
#[derive(Debug, PartialEq, Eq)]
pub enum FetchError {
    Archive(network::archive::Error),
    Holders,
    Bootstrap,
    Timeout,
    Unavailable,
    Transport(String),
}
#[derive(NetworkBehaviour)]
struct Client {
    archive: request_response::Behaviour<ShardCodec>,
}

/// `deadline` bounds verification and I/O; synchronous verification cannot be preempted.
pub async fn fetch(
    ctx: &Context<'_>,
    signed: &SignedManifest,
    holders: Vec<Holder>,
    deadline: Duration,
) -> Result<Recovered, FetchError> {
    let started = Instant::now();
    let remaining = || {
        deadline
            .checked_sub(started.elapsed())
            .filter(|d| !d.is_zero())
            .ok_or(FetchError::Timeout)
    };
    let verified = signed.verify(ctx).map_err(FetchError::Archive)?;
    remaining()?;
    let n = signed.manifest.shard_hashes().len();
    if holders.is_empty() || holders.len() > n {
        return Err(FetchError::Holders);
    }
    let indices: BTreeSet<_> = holders.iter().map(|h| h.index).collect();
    if indices.len() != holders.len()
        || holders
            .iter()
            .any(|h| h.index >= n || h.address.len() > 1024 || h.address.iter().count() > 16)
    {
        return Err(FetchError::Holders);
    }
    let mut swarm =
        libp2p::SwarmBuilder::with_existing_identity(identity::Keypair::generate_ed25519())
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| FetchError::Transport(e.to_string()))?
            .with_behaviour(|_| Client {
                archive: configured_behaviour(&ctx.tiers.network_id(), ProtocolSupport::Outbound),
            })
            .map_err(|e| FetchError::Transport(e.to_string()))?
            .build();
    let mut queue: VecDeque<_> = holders.into();
    let mut active: HashMap<OutboundRequestId, (PeerId, usize)> = HashMap::new();
    let mut shards = vec![None; n];
    let mut authentic = 0;
    loop {
        remaining()?;
        while active.len() < 8 {
            let Some(holder) = queue.pop_front() else {
                break;
            };
            swarm.add_peer_address(holder.peer, holder.address);
            let id = swarm.behaviour_mut().archive.send_request(
                &holder.peer,
                request(signed.manifest.digest(), holder.index),
            );
            active.insert(id, (holder.peer, holder.index));
        }
        if active.is_empty() {
            return Err(FetchError::Unavailable);
        }
        let event = tokio::time::timeout(remaining()?, swarm.select_next_some())
            .await
            .map_err(|_| FetchError::Timeout)?;
        match event {
            SwarmEvent::Behaviour(ClientEvent::Archive(request_response::Event::Message {
                peer,
                message:
                    request_response::Message::Response {
                        request_id,
                        response,
                    },
                ..
            })) => {
                if let Some((expected, index)) = active.remove(&request_id) {
                    if peer == expected {
                        if let Some(shard) = checked(signed, index, &response) {
                            shards[index] = Some(shard);
                            authentic += 1;
                        }
                    }
                    if authentic >= signed.manifest.layout().data_shards {
                        let recovered = verified
                            .recover(ctx, &shards)
                            .map_err(FetchError::Archive)?;
                        remaining()?;
                        return Ok(recovered);
                    }
                }
            }
            SwarmEvent::Behaviour(ClientEvent::Archive(
                request_response::Event::OutboundFailure { request_id, .. },
            )) => {
                active.remove(&request_id);
            }
            _ => {}
        }
    }
}

const DISCOVERY_CONCURRENCY: usize = 8;
const CANDIDATES_PER_SHARD: usize = 32;
const CANDIDATES_TOTAL: usize = 256;

#[derive(NetworkBehaviour)]
struct DiscoveryClient {
    kad: kad::Behaviour<crate::discovery::ProviderStore>,
    archive: request_response::Behaviour<ShardCodec>,
}
struct Candidates {
    seen: Vec<HashSet<PeerId>>,
    queues: Vec<VecDeque<PeerId>>,
    total: usize,
    addresses: HashMap<PeerId, Vec<Multiaddr>>,
}
impl Candidates {
    fn new(n: usize) -> Self {
        Self {
            seen: vec![HashSet::new(); n],
            queues: vec![VecDeque::new(); n],
            total: 0,
            addresses: HashMap::new(),
        }
    }
    fn send(
        &self,
        swarm: &mut libp2p::Swarm<DiscoveryClient>,
        peer: PeerId,
        digest: [u8; 32],
        index: usize,
    ) -> OutboundRequestId {
        for address in self.addresses.get(&peer).into_iter().flatten() {
            swarm.add_peer_address(peer, address.clone());
        }
        swarm
            .behaviour_mut()
            .archive
            .send_request(&peer, request(digest, index))
    }
    fn add(&mut self, index: usize, providers: HashSet<PeerId>) -> Vec<PeerId> {
        let mut providers: Vec<_> = providers.into_iter().collect();
        providers.sort();
        let mut added = Vec::new();
        for peer in providers {
            if self.total < CANDIDATES_TOTAL
                && self.seen[index].len() < CANDIDATES_PER_SHARD
                && self.seen[index].insert(peer)
            {
                self.total += 1;
                self.queues[index].push_back(peer);
                added.push(peer);
            }
        }
        added
    }
}

/// Provider hints locate candidates only; verified recovery establishes the returned bytes.
/// `deadline` includes synchronous verification and recovery, which cannot be preempted.
pub async fn fetch_discovered(
    ctx: &Context<'_>,
    signed: &SignedManifest,
    bootstrap: Vec<Multiaddr>,
    deadline: Duration,
) -> Result<Recovered, FetchError> {
    let started = Instant::now();
    let remaining = || {
        deadline
            .checked_sub(started.elapsed())
            .filter(|d| !d.is_zero())
            .ok_or(FetchError::Timeout)
    };
    let verified = signed.verify(ctx).map_err(FetchError::Archive)?;
    remaining()?;
    if bootstrap.is_empty()
        || bootstrap.len() > 32
        || bootstrap
            .iter()
            .any(|a| a.len() > 1024 || a.iter().count() > 16)
    {
        return Err(FetchError::Bootstrap);
    }
    let network = ctx.tiers.network_id();
    let mut swarm =
        libp2p::SwarmBuilder::with_existing_identity(identity::Keypair::generate_ed25519())
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| FetchError::Transport(e.to_string()))?
            .with_behaviour(|key| DiscoveryClient {
                kad: crate::discovery::kad_behaviour(key.public().to_peer_id(), &network, false),
                archive: configured_behaviour(&network, ProtocolSupport::Outbound),
            })
            .map_err(|e| FetchError::Transport(e.to_string()))?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(10)))
            .build();
    let mut dialed = 0usize;
    for address in bootstrap {
        if swarm.dial(address).is_ok() {
            dialed += 1;
        }
    }
    if dialed == 0 {
        return Err(FetchError::Bootstrap);
    }
    let n = signed.manifest.shard_hashes().len();
    let digest = signed.manifest.digest();
    let mut started_queries = false;
    let mut next_query = 0;
    let mut queries = HashMap::new();
    let mut candidates = Candidates::new(n);
    let mut active: HashMap<OutboundRequestId, (PeerId, usize)> = HashMap::new();
    let mut shards = vec![None; n];
    let mut authentic = 0;
    loop {
        remaining()?;
        if started_queries {
            while queries.len() < DISCOVERY_CONCURRENCY && next_query < n {
                let index = next_query;
                next_query += 1;
                let id = swarm
                    .behaviour_mut()
                    .kad
                    .get_providers(provider_key(&digest, index));
                queries.insert(id, index);
            }
        }
        for (index, shard) in shards.iter().enumerate() {
            if active.len() >= DISCOVERY_CONCURRENCY {
                break;
            }
            if shard.is_some() || active.values().any(|(_, i)| *i == index) {
                continue;
            }
            if let Some(peer) = candidates.queues[index].pop_front() {
                let id = candidates.send(&mut swarm, peer, digest, index);
                active.insert(id, (peer, index));
            }
        }
        if started_queries && next_query == n && queries.is_empty() && active.is_empty() {
            return Err(FetchError::Unavailable);
        }
        match tokio::time::timeout(remaining()?, swarm.select_next_some())
            .await
            .map_err(|_| FetchError::Timeout)?
        {
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                if let libp2p::core::ConnectedPoint::Dialer { address, .. } = endpoint {
                    swarm.behaviour_mut().kad.add_address(&peer_id, address);
                }
                started_queries = true;
            }
            SwarmEvent::Behaviour(DiscoveryClientEvent::Kad(
                kad::Event::OutboundQueryProgressed {
                    id,
                    result: kad::QueryResult::GetProviders(result),
                    step,
                    ..
                },
            )) => {
                if let Some(index) = queries.get(&id).copied() {
                    if shards[index].is_none() {
                        if let Ok(kad::GetProvidersOk::FoundProviders { providers, .. }) = result {
                            for peer in candidates.add(index, providers) {
                                let dial =
                                    libp2p::swarm::dial_opts::DialOpts::peer_id(peer).build();
                                let addresses = swarm
                                    .behaviour_mut()
                                    .kad
                                    .handle_pending_outbound_connection(
                                        dial.connection_id(),
                                        Some(peer),
                                        &[],
                                        libp2p::core::Endpoint::Dialer,
                                    )
                                    .unwrap_or_default();
                                let retained = candidates.addresses.entry(peer).or_default();
                                for address in addresses
                                    .into_iter()
                                    .filter(|a| a.len() <= 1024 && a.iter().count() <= 16)
                                {
                                    if retained.len() == 8 {
                                        break;
                                    }
                                    if !retained.contains(&address) {
                                        retained.push(address);
                                    }
                                }
                            }
                        }
                    }
                    if step.last {
                        queries.remove(&id);
                    }
                }
            }
            SwarmEvent::Behaviour(DiscoveryClientEvent::Archive(
                request_response::Event::Message {
                    peer,
                    message:
                        request_response::Message::Response {
                            request_id,
                            response,
                        },
                    ..
                },
            )) => {
                if let Some((expected, index)) = active.remove(&request_id) {
                    if peer == expected && shards[index].is_none() {
                        if let Some(shard) = checked(signed, index, &response) {
                            shards[index] = Some(shard);
                            candidates.queues[index].clear();
                            authentic += 1;
                        }
                    }
                    if authentic >= signed.manifest.layout().data_shards {
                        remaining()?;
                        let recovered = verified
                            .recover(ctx, &shards)
                            .map_err(FetchError::Archive)?;
                        remaining()?;
                        return Ok(recovered);
                    }
                }
            }
            SwarmEvent::Behaviour(DiscoveryClientEvent::Archive(
                request_response::Event::OutboundFailure { request_id, .. },
            )) => {
                active.remove(&request_id);
            }
            SwarmEvent::OutgoingConnectionError { .. } if !started_queries => {
                dialed = dialed.saturating_sub(1);
                if dialed == 0 {
                    return Err(FetchError::Unavailable);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
#[path = "archive_tests.rs"]
mod tests;
