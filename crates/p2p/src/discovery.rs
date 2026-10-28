//! Public CID discovery and checked object retrieval (`docs/04` §Public object discovery).

use crate::{read_limited, write_all};
use futures::{AsyncRead, AsyncWrite, StreamExt};
use libp2p::kad::{
    self,
    store::{MemoryStore, MemoryStoreConfig, RecordStore},
    ProviderRecord, Record, RecordKey,
};
use libp2p::request_response::{self, ProtocolSupport};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{identity, noise, tcp, yamux, Multiaddr, PeerId, StreamProtocol, Swarm};
use network::cid::Cid;
use network::replica::{Replica, SignedEntry};
use network::store::MAX_OBJECT;
use network::tier::{Tier, Tiers};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io;
use std::time::Duration;

pub const REQUEST_LIMIT: u64 = 33;
pub const REPLY_LIMIT: u64 = MAX_OBJECT as u64 + 170;
const QUERY_LIMIT: usize = 16;
const PROVIDER_LIMIT: usize = 32;

pub fn kad_protocol(network: &[u8; 32]) -> StreamProtocol {
    protocol(network, "kad")
}
pub fn object_protocol(network: &[u8; 32]) -> StreamProtocol {
    protocol(network, "object")
}
fn protocol(network: &[u8; 32], kind: &str) -> StreamProtocol {
    let hex: String = network.iter().map(|b| format!("{b:02x}")).collect();
    StreamProtocol::try_from_owned(format!("/isegoria/{hex}/{kind}/1")).expect("valid protocol")
}

#[derive(Debug)]
pub struct Found {
    pub provider: PeerId,
    pub entry: SignedEntry,
    pub object: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FetchError {
    NoBootstrap,
    Timeout,
    NotFound { rejected: usize },
    Transport(String),
}

#[derive(Clone, Default)]
pub struct ObjectCodec;
#[async_trait::async_trait]
impl request_response::Codec for ObjectCodec {
    type Protocol = StreamProtocol;
    type Request = Vec<u8>;
    type Response = Vec<u8>;
    async fn read_request<T: AsyncRead + Unpin + Send>(
        &mut self,
        _: &StreamProtocol,
        io: &mut T,
    ) -> io::Result<Vec<u8>> {
        let bytes = read_limited(io, REQUEST_LIMIT).await?;
        request_cid(&bytes)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid object request"))?;
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
        r: Vec<u8>,
    ) -> io::Result<()> {
        write_all(io, r).await
    }
    async fn write_response<T: AsyncWrite + Unpin + Send>(
        &mut self,
        _: &StreamProtocol,
        io: &mut T,
        r: Vec<u8>,
    ) -> io::Result<()> {
        write_all(io, r).await
    }
}

pub fn object_behaviour(network: &[u8; 32]) -> request_response::Behaviour<ObjectCodec> {
    request_response::Behaviour::new(
        [(object_protocol(network), ProtocolSupport::Full)],
        request_response::Config::default()
            .with_request_timeout(Duration::from_secs(3))
            .with_max_concurrent_streams(8),
    )
}
fn request(cid: Cid) -> Vec<u8> {
    let mut out = vec![1];
    out.extend(cid.0);
    out
}
fn request_cid(bytes: &[u8]) -> Option<Cid> {
    (bytes.len() == 33 && bytes[0] == 1).then(|| Cid(bytes[1..].try_into().unwrap()))
}
fn reply(entry: &SignedEntry, object: &[u8]) -> Vec<u8> {
    let mut out = vec![1, 1];
    out.extend(entry.encode());
    out.extend(object);
    out
}
pub(crate) fn serve(tiers: &Tiers, replica: &Replica, bytes: &[u8]) -> Vec<u8> {
    let Some(cid) = request_cid(bytes) else {
        return vec![1, 0];
    };
    replica
        .ids()
        .filter(|id| tiers.tier(&id.writer) == Some(Tier::Public))
        .find_map(|id| {
            let (entry, object) = replica.get(&id)?;
            (entry.entry.payload == cid && replica.check(entry, object).is_ok())
                .then(|| reply(entry, object))
        })
        .unwrap_or_else(|| vec![1, 0])
}
fn checked(tiers: &Tiers, cid: Cid, bytes: &[u8]) -> Option<(SignedEntry, Vec<u8>)> {
    if bytes.len() < 170 || bytes.len() as u64 > REPLY_LIMIT || bytes[..2] != [1, 1] {
        return None;
    }
    let entry = SignedEntry::decode(&bytes[2..170])?;
    if tiers.tier(&entry.writer) != Some(Tier::Public) || entry.entry.payload != cid {
        return None;
    }
    let object = &bytes[170..];
    Replica::new(tiers.writers().clone())
        .check(&entry, object)
        .ok()?;
    Some((entry, object.to_vec()))
}

pub struct ProviderStore {
    local_id: PeerId,
    local: MemoryStore,
    remote: MemoryStore,
}
impl ProviderStore {
    fn new(local_id: PeerId) -> Self {
        let local = MemoryStore::with_config(
            local_id,
            MemoryStoreConfig {
                max_provided_keys: usize::MAX,
                ..Default::default()
            },
        );
        Self {
            local_id,
            local,
            remote: MemoryStore::new(local_id),
        }
    }
}
impl RecordStore for ProviderStore {
    type RecordsIter<'a> = std::iter::Empty<Cow<'a, Record>>;
    type ProvidedIter<'a> = <MemoryStore as RecordStore>::ProvidedIter<'a>;
    fn get(&self, _: &RecordKey) -> Option<Cow<'_, Record>> {
        None
    }
    fn put(&mut self, _: Record) -> kad::store::Result<()> {
        Err(kad::store::Error::MaxRecords)
    }
    fn remove(&mut self, _: &RecordKey) {}
    fn records(&self) -> Self::RecordsIter<'_> {
        std::iter::empty()
    }
    fn add_provider(&mut self, record: ProviderRecord) -> kad::store::Result<()> {
        if record.provider == self.local_id {
            self.local.add_provider(record)
        } else {
            self.remote.add_provider(record)
        }
    }
    fn providers(&self, key: &RecordKey) -> Vec<ProviderRecord> {
        let mut found = self.local.providers(key);
        found.extend(self.remote.providers(key));
        found
    }
    fn provided(&self) -> Self::ProvidedIter<'_> {
        self.local.provided()
    }
    fn remove_provider(&mut self, key: &RecordKey, provider: &PeerId) {
        if *provider == self.local_id {
            self.local.remove_provider(key, provider)
        } else {
            self.remote.remove_provider(key, provider)
        }
    }
}
pub(crate) fn kad_behaviour(
    peer: PeerId,
    network: &[u8; 32],
    server: bool,
) -> kad::Behaviour<ProviderStore> {
    let mut config = kad::Config::new(kad_protocol(network));
    config.set_query_timeout(Duration::from_secs(3));
    config.set_max_packet_size(64 << 10);
    let mut kad = kad::Behaviour::with_config(peer, ProviderStore::new(peer), config);
    kad.set_mode(Some(if server {
        kad::Mode::Server
    } else {
        kad::Mode::Client
    }));
    kad
}

#[derive(Default)]
pub(crate) struct Announcements {
    known: HashSet<RecordKey>,
    queued: HashSet<RecordKey>,
    queue: VecDeque<RecordKey>,
    active: HashMap<kad::QueryId, RecordKey>,
}
impl Announcements {
    pub(crate) fn add(&mut self, cid: Cid) {
        self.add_key(RecordKey::new(&cid.0));
    }
    pub(crate) fn add_key(&mut self, key: RecordKey) {
        self.known.insert(key.clone());
        if self.queued.insert(key.clone()) {
            self.queue.push_back(key);
        }
    }
    pub(crate) fn remove(&mut self, key: &RecordKey, kad: &mut kad::Behaviour<ProviderStore>) {
        self.known.remove(key);
        self.queued.remove(key);
        self.queue.retain(|queued| queued != key);
        kad.stop_providing(key);
    }
    pub(crate) fn refresh(&mut self) {
        for key in self.known.clone() {
            self.add_key(key);
        }
    }
    pub(crate) fn completed(&mut self, id: kad::QueryId) {
        self.active.remove(&id);
    }
    pub(crate) fn pump(&mut self, kad: &mut kad::Behaviour<ProviderStore>) {
        let mut remaining = self.queue.len();
        while self.active.len() < QUERY_LIMIT && remaining > 0 {
            remaining -= 1;
            let Some(key) = self.queue.pop_front() else {
                break;
            };
            if self.active.values().any(|active| active == &key) {
                self.queue.push_back(key);
                continue;
            }
            self.queued.remove(&key);
            if let Ok(id) = kad.start_providing(key.clone()) {
                self.active.insert(id, key);
            }
        }
    }
}

#[derive(NetworkBehaviour)]
struct Client {
    kad: kad::Behaviour<ProviderStore>,
    object: request_response::Behaviour<ObjectCodec>,
}
fn client(key: identity::Keypair, network: &[u8; 32]) -> Result<Swarm<Client>, FetchError> {
    Ok(libp2p::SwarmBuilder::with_existing_identity(key)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .map_err(|e| FetchError::Transport(e.to_string()))?
        .with_behaviour(|key| Client {
            kad: kad_behaviour(key.public().to_peer_id(), network, false),
            object: request_response::Behaviour::new(
                [(object_protocol(network), ProtocolSupport::Outbound)],
                request_response::Config::default()
                    .with_request_timeout(Duration::from_secs(3))
                    .with_max_concurrent_streams(8),
            ),
        })
        .map_err(|e| FetchError::Transport(e.to_string()))?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(10)))
        .build())
}

/// Returns authenticated public bytes; does not establish feed continuity or freshness.
pub async fn fetch(
    key: identity::Keypair,
    tiers: &Tiers,
    bootstrap: Vec<Multiaddr>,
    cid: Cid,
    timeout: Duration,
) -> Result<Found, FetchError> {
    if bootstrap.is_empty() {
        return Err(FetchError::NoBootstrap);
    }
    tokio::time::timeout(timeout, fetch_inner(key, tiers, bootstrap, cid))
        .await
        .map_err(|_| FetchError::Timeout)?
}
async fn fetch_inner(
    key: identity::Keypair,
    tiers: &Tiers,
    bootstrap: Vec<Multiaddr>,
    cid: Cid,
) -> Result<Found, FetchError> {
    let mut swarm = client(key, &tiers.network_id())?;
    let mut dialed = 0;
    for addr in bootstrap {
        if swarm.dial(addr).is_ok() {
            dialed += 1;
        }
    }
    if dialed == 0 {
        return Err(FetchError::Transport(
            "no bootstrap address could be dialed".into(),
        ));
    }
    let mut started = false;
    let mut finished = false;
    let mut seen = HashSet::new();
    let mut queue = VecDeque::new();
    let mut active = None;
    let mut rejected = 0;
    loop {
        if active.is_none() {
            if let Some(peer) = queue.pop_front() {
                if !swarm.is_connected(&peer) {
                    let _ = swarm.dial(peer);
                }
                let id = swarm
                    .behaviour_mut()
                    .object
                    .send_request(&peer, request(cid));
                active = Some((id, peer));
            } else if finished {
                return Err(FetchError::NotFound { rejected });
            }
        }
        match swarm.select_next_some().await {
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                if let libp2p::core::ConnectedPoint::Dialer { address, .. } = endpoint {
                    swarm.behaviour_mut().kad.add_address(&peer_id, address);
                }
                if !started {
                    swarm
                        .behaviour_mut()
                        .kad
                        .get_providers(RecordKey::new(&cid.0));
                    started = true;
                }
            }
            SwarmEvent::Behaviour(ClientEvent::Kad(kad::Event::OutboundQueryProgressed {
                result: kad::QueryResult::GetProviders(result),
                step,
                ..
            })) => {
                if let Ok(kad::GetProvidersOk::FoundProviders { providers, .. }) = result {
                    let mut providers: Vec<_> = providers.into_iter().collect();
                    providers.sort();
                    for peer in providers {
                        if seen.len() < PROVIDER_LIMIT && seen.insert(peer) {
                            queue.push_back(peer);
                        }
                    }
                }
                finished |= step.last;
            }
            SwarmEvent::Behaviour(ClientEvent::Object(request_response::Event::Message {
                message:
                    request_response::Message::Response {
                        request_id,
                        response,
                    },
                ..
            })) => {
                if let Some((id, peer)) = active {
                    if id == request_id {
                        if let Some((entry, object)) = checked(tiers, cid, &response) {
                            return Ok(Found {
                                provider: peer,
                                entry,
                                object,
                            });
                        }
                        if response != [1, 0] {
                            rejected += 1;
                        }
                        active = None;
                    }
                }
            }
            SwarmEvent::Behaviour(ClientEvent::Object(
                request_response::Event::OutboundFailure { request_id, .. },
            )) => {
                if active.is_some_and(|(id, _)| id == request_id) {
                    rejected += 1;
                    active = None;
                }
            }
            SwarmEvent::OutgoingConnectionError { .. } if !started => {
                dialed -= 1;
                if dialed == 0 {
                    return Err(FetchError::Transport("bootstrap connection failed".into()));
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::io::Cursor;
    use libp2p::request_response::Codec;
    use network::{
        cid::cid,
        consortium::{Consortium, Member},
        log::TransparencyLog,
        replica::FeedWriter,
    };
    fn tiers() -> Tiers {
        Tiers::new(
            [3; 32],
            Consortium::new(vec![Member::from_seed([1; 32]).public()], 1),
            &[FeedWriter::from_seed([3; 32], [2; 32]).public()],
        )
        .unwrap()
    }
    // Exact requests and inclusive reply bounds reject an extra byte (AT-NET-25).
    #[tokio::test]
    async fn wire_bounds() {
        let protocol = object_protocol(&[3; 32]);
        let mut codec = ObjectCodec;
        let req = request(cid(b"x"));
        assert_eq!(
            codec
                .read_request(&protocol, &mut Cursor::new(req.clone()))
                .await
                .unwrap(),
            req
        );
        let mut extra = req;
        extra.push(0);
        assert!(codec
            .read_request(&protocol, &mut Cursor::new(extra))
            .await
            .is_err());
        assert!(codec
            .read_request(&protocol, &mut Cursor::new(vec![1; 32]))
            .await
            .is_err());
        assert_eq!(
            codec
                .read_response(&protocol, &mut Cursor::new(vec![0; REPLY_LIMIT as usize]))
                .await
                .unwrap()
                .len(),
            REPLY_LIMIT as usize
        );
        assert!(codec
            .read_response(
                &protocol,
                &mut Cursor::new(vec![0; REPLY_LIMIT as usize + 1])
            )
            .await
            .is_err());
    }
    // A full-size public object verifies, including the signed provenance (AT-NET-25).
    #[test]
    fn maximum_verified_object() {
        let object = vec![5; MAX_OBJECT];
        let cid = cid(&object);
        let entry =
            FeedWriter::from_seed([3; 32], [1; 32]).sign(TransparencyLog::new().append(cid));
        let bytes = reply(&entry, &object);
        assert_eq!(bytes.len(), REPLY_LIMIT as usize);
        assert_eq!(checked(&tiers(), cid, &bytes), Some((entry, object)));
    }
    // Local verified catalogues and bounded remote hints have separate quotas (AT-NET-25).
    #[test]
    fn provider_quotas() {
        let local = PeerId::random();
        let remote = PeerId::random();
        let mut store = ProviderStore::new(local);
        for i in 0..1025u64 {
            let key = RecordKey::new(&i.to_le_bytes());
            assert!(store
                .add_provider(ProviderRecord::new(key.clone(), local, vec![]))
                .is_ok());
            let added = store.add_provider(ProviderRecord::new(key, remote, vec![]));
            assert_eq!(added.is_ok(), i < 1024);
        }
        assert_eq!(store.provided().count(), 1025);
        let key = RecordKey::new(&0u64.to_le_bytes());
        for _ in 0..30 {
            store
                .add_provider(ProviderRecord::new(key.clone(), PeerId::random(), vec![]))
                .unwrap();
        }
        assert_eq!(store.providers(&key).len(), 21);
        assert!(matches!(
            store.put(Record::new(vec![0], vec![1])),
            Err(kad::store::Error::MaxRecords)
        ));
    }
    // Application-started announcements never exceed sixteen active queries (AT-NET-25).
    #[test]
    fn announcements_are_bounded() {
        let mut advertisements = Announcements::default();
        let mut kad = kad_behaviour(PeerId::random(), &[3; 32], true);
        for i in 0..1025u64 {
            advertisements.add(cid(&i.to_le_bytes()));
        }
        advertisements.pump(&mut kad);
        assert_eq!(advertisements.active.len(), QUERY_LIMIT);
        assert_eq!(advertisements.queue.len(), 1025 - QUERY_LIMIT);
        let query = *advertisements.active.keys().next().unwrap();
        advertisements.completed(query);
        advertisements.pump(&mut kad);
        assert_eq!(advertisements.active.len(), QUERY_LIMIT);
        assert_eq!(advertisements.queue.len(), 1025 - QUERY_LIMIT - 1);
    }
    // Withdrawal removes queued work; in-flight hints stay counted until completion (AT-NET-28).
    #[tokio::test]
    async fn withdrawal_and_readdition_keep_query_accounting() {
        use libp2p::swarm::ToSwarm;
        use std::task::{Context, Poll};
        let local = PeerId::random();
        let mut kad = kad_behaviour(local, &[3; 32], true);
        let mut advertisements = Announcements::default();
        let key = crate::archive::provider_key(&[7; 32], 0);
        advertisements.add_key(key.clone());
        advertisements.pump(&mut kad);
        assert_eq!(advertisements.active.len(), 1);
        advertisements.refresh();
        advertisements.refresh();
        assert_eq!(advertisements.queue.len(), 1);
        advertisements.pump(&mut kad);
        assert_eq!(advertisements.active.len(), 1);
        advertisements.remove(&key, &mut kad);
        assert!(!advertisements.known.contains(&key));
        assert!(advertisements.queue.is_empty());
        assert_eq!(advertisements.active.len(), 1);
        assert!(!kad
            .store_mut()
            .providers(&key)
            .iter()
            .any(|r| r.provider == local));
        advertisements.add_key(key.clone());
        advertisements.pump(&mut kad);
        assert_eq!(advertisements.active.len(), 1);
        assert_eq!(advertisements.queue.len(), 1);
        let waker = futures::task::noop_waker();
        let mut context = Context::from_waker(&waker);
        for _ in 0..100 {
            if let Poll::Ready(ToSwarm::GenerateEvent(kad::Event::OutboundQueryProgressed {
                id,
                step,
                ..
            })) = kad.poll(&mut context)
            {
                if step.last {
                    advertisements.completed(id);
                }
            }
            if advertisements.active.is_empty() {
                break;
            }
            tokio::task::yield_now().await;
        }
        assert!(advertisements.active.is_empty());
        advertisements.pump(&mut kad);
        assert_eq!(advertisements.active.len(), 1);
        assert!(advertisements.queue.is_empty());
        assert!(kad
            .store_mut()
            .providers(&key)
            .iter()
            .any(|r| r.provider == local));
        advertisements.remove(&key, &mut kad);
        for _ in 0..100 {
            if let Poll::Ready(ToSwarm::GenerateEvent(kad::Event::OutboundQueryProgressed {
                id,
                step,
                ..
            })) = kad.poll(&mut context)
            {
                if step.last {
                    advertisements.completed(id);
                }
            }
            if advertisements.active.is_empty() {
                break;
            }
            tokio::task::yield_now().await;
        }
        advertisements.refresh();
        advertisements.pump(&mut kad);
        assert!(advertisements.active.is_empty());
        assert!(kad.store_mut().providers(&key).is_empty());
    }
}
