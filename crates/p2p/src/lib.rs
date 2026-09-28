//! libp2p transport for replication (`docs/04` §Replication between nodes, `docs/08`
//! NET-010): gossipsub announces new entries, request-response pulls what a node lacks.

pub mod member;

use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, StreamExt};
use libp2p::gossipsub::{self, IdentTopic, MessageAuthenticity, ValidationMode};
use libp2p::request_response::{self, ProtocolSupport};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{identity, noise, tcp, yamux, Multiaddr, PeerId, StreamProtocol, Swarm};
use member::MemberRole;
use network::replica::{
    Accepted, DiskError, DurableReplica, EntryId, FeedWriter, Message, Replica, SignedEntry,
    WriterSet, MAX_RESPONSE,
};
use network::store::{DurableLog, ObjectStore, StoreError};
use std::io;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

/// The most bytes a sync request or response may carry.
pub const MESSAGE_LIMIT: u64 = 64 << 20;
const PROTOCOL: StreamProtocol = StreamProtocol::new("/isegoria/sync/1");
const ANNOUNCE_CHUNK: usize = 512;

#[derive(Clone, Default)]
struct SyncCodec;

/// The whole stream, refused beyond `limit` bytes.
pub async fn read_limited<T: AsyncRead + Unpin + Send>(
    io: &mut T,
    limit: u64,
) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    io.take(limit + 1).read_to_end(&mut bytes).await?;
    if bytes.len() as u64 > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "message too large",
        ));
    }
    Ok(bytes)
}

async fn write_all<T: AsyncWrite + Unpin + Send>(io: &mut T, bytes: Vec<u8>) -> io::Result<()> {
    io.write_all(&bytes).await?;
    io.close().await
}

#[async_trait::async_trait]
impl request_response::Codec for SyncCodec {
    type Protocol = StreamProtocol;
    type Request = Vec<u8>;
    type Response = Vec<u8>;

    async fn read_request<T>(&mut self, _: &StreamProtocol, io: &mut T) -> io::Result<Vec<u8>>
    where
        T: AsyncRead + Unpin + Send,
    {
        read_limited(io, MESSAGE_LIMIT).await
    }

    async fn read_response<T>(&mut self, _: &StreamProtocol, io: &mut T) -> io::Result<Vec<u8>>
    where
        T: AsyncRead + Unpin + Send,
    {
        read_limited(io, MESSAGE_LIMIT).await
    }

    async fn write_request<T>(
        &mut self,
        _: &StreamProtocol,
        io: &mut T,
        r: Vec<u8>,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_all(io, r).await
    }

    async fn write_response<T>(
        &mut self,
        _: &StreamProtocol,
        io: &mut T,
        r: Vec<u8>,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_all(io, r).await
    }
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    gossip: gossipsub::Behaviour,
    sync: request_response::Behaviour<SyncCodec>,
}

/// A writer node's own feed: its key and the directory of its durable log (`docs/04` §A
/// node's own disk).
pub struct Own {
    pub writer: FeedWriter,
    pub dir: PathBuf,
}

pub struct Config {
    pub writers: WriterSet,
    /// How often the node pulls from every connected peer.
    pub sync_every: Duration,
    pub own: Option<Own>,
    /// Where the replica is kept (`docs/04` §A replica on disk); in memory when `None`.
    pub store: Option<PathBuf>,
    /// A consortium member's duties; its writer key must be the member's.
    pub member: Option<MemberRole>,
}

#[derive(Debug)]
pub enum StartError {
    Store(StoreError),
    /// The own key is not in the writer set.
    NotAWriter,
    /// A member role without an own feed under the member's key.
    NotTheMember,
    /// The own log names an object the object store lacks.
    MissingObject(u64),
    /// The own feed was refused by the writer set (the key is not a writer).
    Replica(DiskError),
    Transport(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum PublishError {
    NotAWriter,
    Store(String),
    Replica(DiskError),
}

enum Command {
    Listen(Multiaddr, oneshot::Sender<Result<Multiaddr, String>>),
    Dial(Multiaddr, oneshot::Sender<Result<(), String>>),
    Publish(Vec<u8>, oneshot::Sender<Result<SignedEntry, PublishError>>),
    Insert(
        SignedEntry,
        Vec<u8>,
        oneshot::Sender<Result<Accepted, DiskError>>,
    ),
    Replica(oneshot::Sender<Replica>),
    Peers(oneshot::Sender<usize>),
}

/// A running node; dropping it closes its command channel, which ends the node's task.
pub struct Handle {
    peer_id: PeerId,
    commands: mpsc::UnboundedSender<Command>,
}

struct Writing {
    writer: FeedWriter,
    log: DurableLog,
    objects: ObjectStore,
}

struct Node {
    swarm: Swarm<Behaviour>,
    store: Store,
    topic: IdentTopic,
    writing: Option<Writing>,
    listening: Vec<oneshot::Sender<Result<Multiaddr, String>>>,
    member: Option<MemberRole>,
    network_id: [u8; 32],
    waited: u32,
}

/// The gossip topic of a network's announcements.
pub fn topic(network_id: &[u8; 32]) -> String {
    let hex: String = network_id.iter().map(|b| format!("{b:02x}")).collect();
    format!("isegoria/{hex}/entries")
}

fn open_own(own: Own, replica: &mut Store) -> Result<Writing, StartError> {
    if !replica.replica().writers().contains(&own.writer.public()) {
        return Err(StartError::NotAWriter);
    }
    std::fs::create_dir_all(&own.dir).map_err(|e| StartError::Store(StoreError::Io(e.kind())))?;
    let (log, _) = DurableLog::open(&own.dir.join("log")).map_err(StartError::Store)?;
    let (objects, _) = ObjectStore::open(&own.dir.join("objects")).map_err(StartError::Store)?;
    for entry in log.log().entries() {
        let object = objects
            .get(&entry.payload)
            .map_err(StartError::Store)?
            .ok_or(StartError::MissingObject(entry.seq))?;
        replica
            .insert(own.writer.sign(entry), object)
            .map_err(StartError::Replica)?;
    }
    Ok(Writing {
        writer: own.writer,
        log,
        objects,
    })
}

fn build_swarm(key: identity::Keypair) -> Result<Swarm<Behaviour>, String> {
    let gossip_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_millis(250))
        .validation_mode(ValidationMode::Strict)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(libp2p::SwarmBuilder::with_existing_identity(key)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .map_err(|e| e.to_string())?
        .with_behaviour(|key| {
            let gossip =
                gossipsub::Behaviour::new(MessageAuthenticity::Signed(key.clone()), gossip_config)?;
            let sync = request_response::Behaviour::new(
                [(PROTOCOL, ProtocolSupport::Full)],
                request_response::Config::default(),
            );
            Ok(Behaviour { gossip, sync })
        })
        .map_err(|e| e.to_string())?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(3600)))
        .build())
}

impl Handle {
    /// Starts a node on the current tokio runtime; a writer's own feed is replayed first.
    pub fn spawn(key: identity::Keypair, config: Config) -> Result<Handle, StartError> {
        let mut replica = match config.store {
            Some(dir) => Store::Disk(
                DurableReplica::open(&dir, config.writers.clone()).map_err(StartError::Replica)?,
            ),
            None => Store::Memory(Replica::new(config.writers.clone())),
        };
        let writing = config
            .own
            .map(|own| open_own(own, &mut replica))
            .transpose()?;
        if let Some(role) = &config.member {
            let key = role.member.public();
            let mine = role.consortium.keys().get(role.index) == Some(&key);
            if !mine || writing.as_ref().is_none_or(|w| w.writer.public() != key) {
                return Err(StartError::NotTheMember);
            }
        }
        let mut swarm = build_swarm(key).map_err(StartError::Transport)?;
        let topic = IdentTopic::new(topic(&config.writers.network_id()));
        swarm
            .behaviour_mut()
            .gossip
            .subscribe(&topic)
            .map_err(|e| StartError::Transport(e.to_string()))?;
        let peer_id = *swarm.local_peer_id();
        let (commands, rx) = mpsc::unbounded_channel();
        let node = Node {
            swarm,
            store: replica,
            topic,
            writing,
            listening: Vec::new(),
            member: config.member,
            network_id: config.writers.network_id(),
            waited: 0,
        };
        tokio::spawn(node.run(rx, config.sync_every));
        Ok(Handle { peer_id, commands })
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    async fn ask<T>(&self, make: impl FnOnce(oneshot::Sender<T>) -> Command) -> T {
        let (tx, rx) = oneshot::channel();
        self.commands
            .send(make(tx))
            .expect("the node runs while its handle lives");
        rx.await.expect("the node answers every command")
    }

    /// Listens on `addr` and returns the address bound (`/ip4/127.0.0.1/tcp/0` picks a port).
    pub async fn listen(&self, addr: Multiaddr) -> Result<Multiaddr, String> {
        self.ask(|tx| Command::Listen(addr, tx)).await
    }

    pub async fn dial(&self, addr: Multiaddr) -> Result<(), String> {
        self.ask(|tx| Command::Dial(addr, tx)).await
    }

    /// Stores `object`, appends it to the own log, signs the entry and announces it.
    pub async fn publish(&self, object: Vec<u8>) -> Result<SignedEntry, PublishError> {
        self.ask(|tx| Command::Publish(object, tx)).await
    }

    /// Inserts an entry received out of band, announcing it if new.
    pub async fn insert(&self, entry: SignedEntry, object: Vec<u8>) -> Result<Accepted, DiskError> {
        self.ask(|tx| Command::Insert(entry, object, tx)).await
    }

    /// A copy of the node's replica.
    pub async fn replica(&self) -> Replica {
        self.ask(Command::Replica).await
    }

    pub async fn peers(&self) -> usize {
        self.ask(Command::Peers).await
    }
}

impl Node {
    async fn run(mut self, mut rx: mpsc::UnboundedReceiver<Command>, every: Duration) {
        let mut tick = tokio::time::interval(every);
        let duty_every = self
            .member
            .as_ref()
            .map_or(Duration::from_secs(3600), |m| m.every);
        let mut duty = tokio::time::interval(duty_every);
        loop {
            tokio::select! {
                command = rx.recv() => match command {
                    Some(c) => self.command(c),
                    None => return,
                },
                _ = duty.tick(), if self.member.is_some() => self.duties(),
                _ = tick.tick() => {
                    let peers: Vec<PeerId> = self.swarm.connected_peers().copied().collect();
                    for peer in peers {
                        self.pull(peer);
                    }
                }
                event = self.swarm.select_next_some() => self.event(event),
            }
        }
    }

    fn duties(&mut self) {
        let Some(role) = &self.member else {
            return;
        };
        let objects = role.duties(self.store.replica(), self.network_id, &mut self.waited);
        for object in objects {
            let _ = self.publish(object.encode());
        }
    }

    fn pull(&mut self, peer: PeerId) {
        let summary = Message::Summary(self.store.replica().summary()).encode();
        self.swarm.behaviour_mut().sync.send_request(&peer, summary);
    }

    fn announce(&mut self, ids: &[EntryId]) {
        for chunk in ids.chunks(ANNOUNCE_CHUNK) {
            let bytes = Message::Have(chunk.to_vec()).encode();
            let topic = self.topic.clone();
            let _ = self.swarm.behaviour_mut().gossip.publish(topic, bytes);
        }
    }

    fn accept(&mut self, entry: SignedEntry, object: Vec<u8>) -> Result<Accepted, DiskError> {
        let id = entry.id();
        let accepted = self.store.insert(entry, object)?;
        if accepted != Accepted::Duplicate {
            self.announce(&[id]);
        }
        Ok(accepted)
    }

    fn publish(&mut self, object: Vec<u8>) -> Result<SignedEntry, PublishError> {
        let w = self.writing.as_mut().ok_or(PublishError::NotAWriter)?;
        let store = |e: StoreError| PublishError::Store(format!("{e:?}"));
        let payload = w.objects.put(&object).map_err(store)?;
        let entry = w.log.append(payload).map_err(store)?;
        let signed = w.writer.sign(&entry);
        self.accept(signed.clone(), object)
            .map_err(PublishError::Replica)?;
        Ok(signed)
    }

    fn command(&mut self, command: Command) {
        match command {
            Command::Listen(addr, tx) => match self.swarm.listen_on(addr) {
                Ok(_) => self.listening.push(tx),
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                }
            },
            Command::Dial(addr, tx) => {
                let _ = tx.send(self.swarm.dial(addr).map_err(|e| e.to_string()));
            }
            Command::Publish(object, tx) => {
                let _ = tx.send(self.publish(object));
            }
            Command::Insert(entry, object, tx) => {
                let _ = tx.send(self.accept(entry, object));
            }
            Command::Replica(tx) => {
                let _ = tx.send(self.store.replica().clone());
            }
            Command::Peers(tx) => {
                let _ = tx.send(self.swarm.connected_peers().count());
            }
        }
    }

    fn event(&mut self, event: SwarmEvent<BehaviourEvent>) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                for tx in self.listening.drain(..) {
                    let _ = tx.send(Ok(address.clone()));
                }
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => self.pull(peer_id),
            SwarmEvent::Behaviour(BehaviourEvent::Sync(request_response::Event::Message {
                peer,
                message,
                ..
            })) => self.sync_message(peer, message),
            SwarmEvent::Behaviour(BehaviourEvent::Gossip(gossipsub::Event::Message {
                propagation_source,
                message,
                ..
            })) => {
                if let Some(Message::Have(ids)) = Message::decode(&message.data) {
                    let from = message
                        .source
                        .filter(|s| self.swarm.is_connected(s))
                        .unwrap_or(propagation_source);
                    self.want(from, &ids);
                }
            }
            _ => {}
        }
    }

    fn want(&mut self, peer: PeerId, offered: &[EntryId]) {
        let want = self.store.replica().want(offered);
        if !want.is_empty() {
            let bytes = Message::Want(want).encode();
            self.swarm.behaviour_mut().sync.send_request(&peer, bytes);
        }
    }

    fn sync_message(&mut self, peer: PeerId, message: request_response::Message<Vec<u8>, Vec<u8>>) {
        match message {
            request_response::Message::Request {
                request, channel, ..
            } => {
                let reply = match Message::decode(&request) {
                    Some(Message::Summary(s)) => Message::Have(self.store.replica().have_for(&s)),
                    Some(Message::Want(ids)) => {
                        Message::Entries(self.store.replica().entries_for(&ids, MAX_RESPONSE))
                    }
                    _ => return,
                };
                let _ = self
                    .swarm
                    .behaviour_mut()
                    .sync
                    .send_response(channel, reply.encode());
            }
            request_response::Message::Response { response, .. } => {
                match Message::decode(&response) {
                    Some(Message::Have(ids)) => self.want(peer, &ids),
                    Some(Message::Entries(list)) => {
                        let mut new = Vec::new();
                        for (entry, object) in list {
                            let id = entry.id();
                            if matches!(
                                self.store.insert(entry, object),
                                Ok(Accepted::New | Accepted::Equivocation)
                            ) {
                                new.push(id);
                            }
                        }
                        if !new.is_empty() {
                            self.announce(&new);
                            self.pull(peer);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

enum Store {
    Memory(Replica),
    Disk(DurableReplica),
}

impl Store {
    fn replica(&self) -> &Replica {
        match self {
            Store::Memory(r) => r,
            Store::Disk(d) => d.replica(),
        }
    }

    fn insert(&mut self, entry: SignedEntry, object: Vec<u8>) -> Result<Accepted, DiskError> {
        match self {
            Store::Memory(r) => r.insert(entry, object).map_err(DiskError::Refused),
            Store::Disk(d) => d.insert(entry, object),
        }
    }
}
