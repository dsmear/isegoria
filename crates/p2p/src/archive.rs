//! Assigned public archive shards and explicit-holder retrieval (`docs/04` §Public archive).

use crate::{read_limited, write_all};
use futures::{AsyncRead, AsyncWrite, StreamExt};
use libp2p::request_response::{self, OutboundRequestId, ProtocolSupport};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{identity, noise, tcp, yamux, Multiaddr, PeerId, StreamProtocol};
use network::archive::{BuiltArchive, Context, Recovered, Segment, SignedManifest, MAX_MANIFEST};
use network::codec::{Reader, Writer};
use network::erasure::shard_hash;
use network::store::MAX_OBJECT;
use network::tier::Tiers;
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

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

pub(crate) struct Prepared {
    context: [u8; 32],
    record: Record,
}
impl Prepared {
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

pub(crate) struct Store {
    context: [u8; 32],
    dir: Option<PathBuf>,
    records: BTreeMap<[u8; 32], Record>,
    temporary: u64,
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
        let store = Self {
            context: tiers.context(),
            dir,
            records: BTreeMap::new(),
            temporary: 0,
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
            store.usage(tiers)?.check()?;
        }
        Ok(store)
    }
    fn usage(&self, tiers: &Tiers) -> Result<Usage, InstallError> {
        let mut usage = Usage::default();
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
                    usage.shards += file_digest(&entry.path())
                        .and_then(|digest| read_record(&entry.path(), tiers, digest))
                        .map_or(256, |r| r.shards.len());
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
        Ok(usage)
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
        Ok(())
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

#[cfg(test)]
#[path = "archive_tests.rs"]
mod tests;
