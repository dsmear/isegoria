//! libp2p transport for replication (`docs/04` §Replication between nodes, `docs/08`
//! NET-010): gossipsub announces new entries, request-response pulls what a node lacks.

pub mod member;
pub mod tiered;

use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, StreamExt};
use libp2p::gossipsub::{self, IdentTopic, MessageAcceptance, MessageAuthenticity, ValidationMode};
use libp2p::request_response::{self, OutboundRequestId, ProtocolSupport};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{identity, noise, tcp, yamux, Multiaddr, PeerId, StreamProtocol, Swarm};
use member::{Collector, MemberRole, Progress};
use network::consortium::Consortium;
use network::replica::{
    Accepted, DiskError, DurableReplica, EntryId, FeedError, FeedWriter, Message, Replica,
    SignedEntry, Summary, WriterSet, MAX_RESPONSE,
};
use network::store::{DurableLog, ObjectStore, StoreError};
use network::tier::{Tier, Tiers};
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tiered::{Bind, Tiered};
use tokio::sync::{mpsc, oneshot};

/// The most bytes a sync request or response may carry.
pub const MESSAGE_LIMIT: u64 = 64 << 20;
pub const SYNC_PROTOCOL: &str = "/isegoria/sync/1";
/// A tiered network's binding exchange (`docs/04` §Transport).
pub const BIND_PROTOCOL: &str = "/isegoria/bind/1";
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
    bind: request_response::Behaviour<SyncCodec>,
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
    /// A tiered member's circle feed, under its circle key (`docs/04` §A writer's own feed).
    pub circle: Option<Own>,
    /// Where the replica is kept (`docs/04` §A replica on disk); in memory when `None`.
    pub store: Option<PathBuf>,
    /// A consortium member's duties; its writer key must be the member's.
    pub member: Option<MemberRole>,
    /// The consortium whose cuts a node that is no member collects for its status.
    pub consortium: Option<Consortium>,
    /// A tiered network (`docs/04` §Transport): `writers` and the consortium must be its own.
    pub tiers: Option<Tiers>,
}

/// What a node reports of itself (`docs/04` §A node process).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Status {
    pub caught_up: bool,
    /// Whether a tiered member's circle feed caught up from a member that granted it.
    pub circle_caught_up: bool,
    pub peers: usize,
    /// Connected peers whose binding verified.
    pub bound: usize,
    /// Pairs of entries one writer signed at one `seq`, in the replica.
    pub equivocations: usize,
    /// Whether the consortium signed two cuts for one number.
    pub conflict: bool,
    /// The collected cuts' digests, from number 0.
    pub cuts: Vec<[u8; 32]>,
    /// Each finished round, from epoch 0: its value and its withholders.
    pub beacons: Vec<(Option<[u8; 32]>, Vec<usize>)>,
}

#[derive(Debug)]
pub enum StartError {
    Store(StoreError),
    /// The own key is not in the writer set.
    NotAWriter,
    /// A member role without an own feed under the member's key.
    NotTheMember,
    /// The own log and the writer's entries in the stored replica fork.
    Feed(FeedError),
    /// The own feed was refused by the writer set (the key is not a writer).
    Replica(DiskError),
    Transport(String),
    /// Tiers whose writer set or consortium is not the one configured.
    Tiers,
    /// A circle feed outside a tiered member, a tiered member without one, or another key's.
    NotTheCircle,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PublishError {
    NotAWriter,
    /// No sync round with a peer has ended since the start (`docs/04` §A writer's own feed).
    NotCaughtUp,
    /// The replica holds an entry of this writer that its log cannot take.
    Feed(FeedError),
    Store(String),
    Replica(DiskError),
}

enum Command {
    Listen(Multiaddr, oneshot::Sender<Result<Multiaddr, String>>),
    Dial(Multiaddr, oneshot::Sender<Result<(), String>>),
    Publish(
        Feed,
        Vec<u8>,
        oneshot::Sender<Result<SignedEntry, PublishError>>,
    ),
    Insert(
        SignedEntry,
        Vec<u8>,
        oneshot::Sender<Result<Accepted, DiskError>>,
    ),
    Replica(oneshot::Sender<Replica>),
    Peers(oneshot::Sender<usize>),
    CaughtUp(oneshot::Sender<bool>),
    Status(oneshot::Sender<Status>),
}

/// A running node; dropping it closes its command channel, which ends the node's task.
pub struct Handle {
    peer_id: PeerId,
    commands: mpsc::UnboundedSender<Command>,
    set_aside: Vec<PathBuf>,
}

struct Writing {
    writer: FeedWriter,
    log: DurableLog,
    objects: ObjectStore,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Feed {
    Public,
    Circle,
}

/// What an outbound sync request was, so its response is read for what it is.
#[derive(Clone, Copy)]
enum Sent {
    Pull { circle: bool },
    Want,
    Announce,
}

struct Node {
    swarm: Swarm<Behaviour>,
    store: Store,
    topic: IdentTopic,
    writing: Option<Writing>,
    circle: Option<Writing>,
    listening: Vec<oneshot::Sender<Result<Multiaddr, String>>>,
    member: Option<MemberRole>,
    network_id: [u8; 32],
    progress: Progress,
    caught_up: bool,
    circle_caught_up: bool,
    consortium: Option<Consortium>,
    collector: Collector,
    tiered: Option<Tiered>,
    pending: HashMap<OutboundRequestId, Sent>,
}

/// The gossip topic of a network's announcements.
pub fn topic(network_id: &[u8; 32]) -> String {
    let hex: String = network_id.iter().map(|b| format!("{b:02x}")).collect();
    format!("isegoria/{hex}/entries")
}

fn io_error(e: io::Error) -> StartError {
    StartError::Store(StoreError::Io(e.kind()))
}

type Files = (DurableLog, ObjectStore, Vec<Vec<u8>>);

/// The own log, its objects and the object of each entry; `None` if either file is damaged.
fn open_files(dir: &Path) -> Result<Option<Files>, StartError> {
    let damaged = |e| match e {
        StoreError::Corrupt { .. } | StoreError::TooLarge { .. } => Ok(None),
        e => Err(StartError::Store(e)),
    };
    let log = match DurableLog::open(&dir.join("log")) {
        Ok((log, _)) => log,
        Err(e) => return damaged(e),
    };
    let objects = match ObjectStore::open(&dir.join("objects")) {
        Ok((objects, _)) => objects,
        Err(e) => return damaged(e),
    };
    let mut named = Vec::new();
    for entry in log.log().entries() {
        match objects.get(&entry.payload) {
            Ok(Some(object)) => named.push(object),
            Ok(None) | Err(StoreError::Corrupt { .. }) => return Ok(None),
            Err(e) => return Err(StartError::Store(e)),
        }
    }
    Ok(Some((log, objects, named)))
}

/// Renames `path` to the first free `<name>.damaged-<n>`.
fn set_aside(path: &Path) -> Result<Option<PathBuf>, StartError> {
    if !path.exists() {
        return Ok(None);
    }
    let to = (0u32..)
        .map(|n| path.with_extension(format!("damaged-{n}")))
        .find(|p| !p.exists())
        .ok_or(StartError::Store(StoreError::Io(
            io::ErrorKind::AlreadyExists,
        )))?;
    std::fs::rename(path, &to).map_err(io_error)?;
    Ok(Some(to))
}

/// Opens the own feed, setting damaged files aside (`docs/04` §A node's own disk, rule 2);
/// returns it and the paths set aside.
fn open_own(own: Own, replica: &mut Store) -> Result<(Writing, Vec<PathBuf>), StartError> {
    if !replica.replica().writers().contains(&own.writer.public()) {
        return Err(StartError::NotAWriter);
    }
    std::fs::create_dir_all(&own.dir).map_err(io_error)?;
    let mut aside = Vec::new();
    let (log, objects, named) = match open_files(&own.dir)? {
        Some(files) => files,
        None => {
            for name in ["log", "objects"] {
                aside.extend(set_aside(&own.dir.join(name))?);
            }
            let (log, _) = DurableLog::open(&own.dir.join("log")).map_err(StartError::Store)?;
            let (objects, _) =
                ObjectStore::open(&own.dir.join("objects")).map_err(StartError::Store)?;
            (log, objects, Vec::new())
        }
    };
    let me = own.writer.public().to_bytes();
    if let Err(e @ FeedError::Forked { .. }) =
        replica.replica().feed_beyond(&me, log.log().entries())
    {
        return Err(StartError::Feed(e));
    }
    for (entry, object) in log.log().entries().iter().zip(named) {
        replica
            .insert(own.writer.sign(entry), object)
            .map_err(StartError::Replica)?;
    }
    let mut writing = Writing {
        writer: own.writer,
        log,
        objects,
    };
    let _ = writing.catch_up(replica.replica());
    Ok((writing, aside))
}

impl Writing {
    /// Appends to the own files the writer's entries the replica holds past its log.
    fn catch_up(&mut self, replica: &Replica) -> Result<(), PublishError> {
        let me = self.writer.public().to_bytes();
        let store = |e: StoreError| PublishError::Store(format!("{e:?}"));
        let beyond = replica
            .feed_beyond(&me, self.log.log().entries())
            .map_err(PublishError::Feed)?;
        for (entry, object) in beyond {
            self.objects.put(object).map_err(store)?;
            self.log.append(entry.entry.payload).map_err(store)?;
        }
        Ok(())
    }
}

fn build_swarm(key: identity::Keypair) -> Result<Swarm<Behaviour>, String> {
    let gossip_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_millis(250))
        .validation_mode(ValidationMode::Strict)
        .validate_messages()
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
            let protocol = |name| [(StreamProtocol::new(name), ProtocolSupport::Full)];
            let sync =
                request_response::Behaviour::new(protocol(SYNC_PROTOCOL), Default::default());
            let bind =
                request_response::Behaviour::new(protocol(BIND_PROTOCOL), Default::default());
            Ok(Behaviour { gossip, sync, bind })
        })
        .map_err(|e| e.to_string())?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(3600)))
        .build())
}

/// A tiered node's start: its writer set and consortium those of `tiers`, a member's circle
/// feed its own, and a member's binding of its transport key.
fn tiered_start(
    key: &identity::Keypair,
    tiers: Tiers,
    config: &Config,
) -> Result<Tiered, StartError> {
    let same = |c: &Consortium| {
        c.member_set_hash() == tiers.consortium().member_set_hash()
            && c.threshold() == tiers.consortium().threshold()
    };
    let member = config.member.as_ref();
    if config.writers != *tiers.writers()
        || member.is_some_and(|m| !same(&m.consortium))
        || config.consortium.as_ref().is_some_and(|c| !same(c))
    {
        return Err(StartError::Tiers);
    }
    let circle = config
        .circle
        .as_ref()
        .map(|own| own.writer.public().to_bytes());
    if member.map(|m| (m.index, Tier::Circle)) != circle.and_then(|c| tiers.place(&c)) {
        return Err(StartError::NotTheCircle);
    }
    let binding = match member {
        Some(role) => {
            let ed = key.public().try_into_ed25519();
            let transport = ed.map_err(|e| StartError::Transport(e.to_string()))?;
            Some(tiers.bind(&role.member, role.index, transport.to_bytes()))
        }
        None => None,
    };
    Ok(Tiered::new(tiers, binding))
}

impl Handle {
    /// Starts a node on the current tokio runtime; a writer's own feed is replayed first.
    pub fn spawn(key: identity::Keypair, mut config: Config) -> Result<Handle, StartError> {
        let tiered = match config.tiers.take() {
            Some(tiers) => Some(tiered_start(&key, tiers, &config)?),
            None if config.circle.is_some() => return Err(StartError::NotTheCircle),
            None => None,
        };
        let mut replica = match config.store {
            Some(dir) => Store::Disk(
                DurableReplica::open(&dir, config.writers.clone()).map_err(StartError::Replica)?,
            ),
            None => Store::Memory(Replica::new(config.writers.clone())),
        };
        let (writing, mut set_aside) = match config.own {
            Some(own) => {
                let (writing, aside) = open_own(own, &mut replica)?;
                (Some(writing), aside)
            }
            None => (None, Vec::new()),
        };
        let circle = match config.circle {
            Some(own) => {
                let (circle, aside) = open_own(own, &mut replica)?;
                set_aside.extend(aside);
                Some(circle)
            }
            None => None,
        };
        if let Some(role) = &config.member {
            let key = role.member.public();
            let mine = role.consortium.keys().get(role.index) == Some(&key);
            if !mine || writing.as_ref().is_none_or(|w| w.writer.public() != key) {
                return Err(StartError::NotTheMember);
            }
        }
        let network_id = config.writers.network_id();
        let consortium = config.member.as_ref().map(|m| &m.consortium);
        let collector = consortium
            .or(config.consortium.as_ref())
            .map_or_else(Collector::default, |c| Collector::new(c, network_id));
        let progress = consortium.map_or_else(Progress::default, |c| Progress::new(c, network_id));
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
            circle,
            listening: Vec::new(),
            member: config.member,
            network_id,
            progress,
            caught_up: false,
            circle_caught_up: false,
            consortium: config.consortium,
            collector,
            tiered,
            pending: HashMap::new(),
        };
        tokio::spawn(node.run(rx, config.sync_every));
        Ok(Handle {
            peer_id,
            commands,
            set_aside,
        })
    }

    /// The own files found damaged at the start and renamed `<name>.damaged-<n>`.
    pub fn set_aside(&self) -> &[PathBuf] {
        &self.set_aside
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
        self.ask(|tx| Command::Publish(Feed::Public, object, tx))
            .await
    }

    /// As [`Self::publish`], on a tiered member's circle feed.
    pub async fn publish_circle(&self, object: Vec<u8>) -> Result<SignedEntry, PublishError> {
        self.ask(|tx| Command::Publish(Feed::Circle, object, tx))
            .await
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

    /// Whether a sync round with a peer has ended since the start, so a writer may write.
    pub async fn caught_up(&self) -> bool {
        self.ask(Command::CaughtUp).await
    }

    /// The node's status; its cuts and rounds only with a consortium, its own or configured.
    pub async fn status(&self) -> Status {
        self.ask(Command::Status).await
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

    fn status(&mut self) -> Status {
        let replica = self.store.replica();
        let digests = |cuts: &[network::cut::Cut]| cuts.iter().map(|c| c.digest()).collect();
        let rounds = |outcomes: Vec<&network::beacon::BeaconOutcome>| {
            outcomes
                .iter()
                .map(|o| (o.value(), o.withheld().to_vec()))
                .collect()
        };
        let (cuts, conflict, beacons) = match (&mut self.tiered, &self.member) {
            (Some(t), None) => {
                let _ = t.public.update(replica, &t.tiers);
                let p = &t.public;
                (digests(p.cuts()), p.conflict(), rounds(p.beacons()))
            }
            _ => {
                let consortium = self.member.as_ref().map(|m| &m.consortium);
                if let Some(c) = consortium.or(self.consortium.as_ref()) {
                    let _ = self.collector.update(replica, c, self.network_id);
                }
                let c = &self.collector;
                (digests(c.cuts()), c.conflict(), rounds(c.beacons()))
            }
        };
        Status {
            caught_up: self.caught_up,
            circle_caught_up: self.circle_caught_up,
            peers: self.swarm.connected_peers().count(),
            bound: self.tiered.as_ref().map_or(0, |t| t.bound().len()),
            equivocations: replica.equivocations().len(),
            conflict,
            cuts,
            beacons,
        }
    }

    fn duties(&mut self) {
        let Some(role) = self.member.as_ref().filter(|_| self.caught_up) else {
            return;
        };
        let objects = role.duties(self.store.replica(), self.network_id, &mut self.progress);
        for object in objects {
            let _ = self.publish(Feed::Public, object.encode());
        }
    }

    /// Whether `peer` may read `writer`'s entries: every writer in a network without tiers.
    fn reads(&self, peer: &PeerId, writer: &[u8; 32]) -> bool {
        self.tiered.as_ref().is_none_or(|t| t.reads(peer, writer))
    }

    fn summary_for(&self, peer: &PeerId) -> Summary {
        let replica = self.store.replica();
        match &self.tiered {
            None => replica.summary(),
            Some(t) => t.summary(replica, peer),
        }
    }

    fn send(&mut self, peer: &PeerId, message: Message, sent: Sent) {
        let id = self
            .swarm
            .behaviour_mut()
            .sync
            .send_request(peer, message.encode());
        self.pending.insert(id, sent);
    }

    fn pull(&mut self, peer: PeerId) {
        let circle = self.tiered.as_ref().is_some_and(|t| t.circle_pull(&peer));
        let summary = Message::Summary(self.summary_for(&peer));
        self.send(&peer, summary, Sent::Pull { circle });
    }

    /// Public identifiers on gossip; in a tiered network, circle ones to each bound peer.
    fn announce(&mut self, ids: &[EntryId]) {
        let (public, circle): (Vec<EntryId>, Vec<EntryId>) = match &self.tiered {
            None => (ids.to_vec(), Vec::new()),
            Some(t) => ids.iter().partition(|id| t.is_public(id)),
        };
        for chunk in public.chunks(ANNOUNCE_CHUNK) {
            let bytes = Message::Have(chunk.to_vec()).encode();
            let topic = self.topic.clone();
            let _ = self.swarm.behaviour_mut().gossip.publish(topic, bytes);
        }
        let bound = self.tiered.as_ref().map_or(Vec::new(), |t| t.bound());
        for peer in bound {
            for chunk in circle.chunks(ANNOUNCE_CHUNK) {
                self.send(&peer, Message::Have(chunk.to_vec()), Sent::Announce);
            }
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

    fn publish(&mut self, feed: Feed, object: Vec<u8>) -> Result<SignedEntry, PublishError> {
        let (w, ready) = match feed {
            Feed::Public => (self.writing.as_mut(), self.caught_up),
            Feed::Circle => (self.circle.as_mut(), self.circle_caught_up),
        };
        let w = w.ok_or(PublishError::NotAWriter)?;
        if !ready {
            return Err(PublishError::NotCaughtUp);
        }
        w.catch_up(self.store.replica())?;
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
            Command::Publish(feed, object, tx) => {
                let _ = tx.send(self.publish(feed, object));
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
            Command::CaughtUp(tx) => {
                let _ = tx.send(self.caught_up);
            }
            Command::Status(tx) => {
                let _ = tx.send(self.status());
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
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                if let Some(t) = &self.tiered {
                    let hello = Bind::Hello(t.binding).encode();
                    self.swarm
                        .behaviour_mut()
                        .bind
                        .send_request(&peer_id, hello);
                }
                self.pull(peer_id);
            }
            SwarmEvent::ConnectionClosed {
                peer_id,
                num_established: 0,
                ..
            } => {
                if let Some(t) = self.tiered.as_mut() {
                    t.forget(&peer_id);
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Sync(request_response::Event::Message {
                peer,
                message,
                ..
            })) => self.sync_message(peer, message),
            SwarmEvent::Behaviour(BehaviourEvent::Sync(
                request_response::Event::OutboundFailure { request_id, .. },
            )) => {
                self.pending.remove(&request_id);
            }
            SwarmEvent::Behaviour(BehaviourEvent::Bind(request_response::Event::Message {
                peer,
                message,
                ..
            })) => self.bind_message(peer, message),
            SwarmEvent::Behaviour(BehaviourEvent::Gossip(gossipsub::Event::Message {
                propagation_source,
                message_id,
                message,
            })) => {
                let ids = match Message::decode(&message.data) {
                    Some(Message::Have(ids)) => ids,
                    _ => Vec::new(),
                };
                let public = self
                    .tiered
                    .as_ref()
                    .is_none_or(|t| ids.iter().all(|id| t.is_public(id)));
                let accept = public && !ids.is_empty();
                let acceptance = if accept {
                    MessageAcceptance::Accept
                } else {
                    MessageAcceptance::Reject
                };
                let gossip = &mut self.swarm.behaviour_mut().gossip;
                gossip.report_message_validation_result(
                    &message_id,
                    &propagation_source,
                    acceptance,
                );
                if accept {
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

    /// A tiered node checks each hello or reply's binding against the peer's authenticated
    /// key; after a reply, it pulls again, now as that peer reads it.
    fn bind_message(&mut self, peer: PeerId, message: request_response::Message<Vec<u8>, Vec<u8>>) {
        let Some(t) = self.tiered.as_mut() else {
            return;
        };
        match message {
            request_response::Message::Request {
                request, channel, ..
            } => {
                let granted = match Bind::decode(&request) {
                    Some(Bind::Hello(binding)) => t.verify(&peer, binding.as_ref()),
                    _ => t.verify(&peer, None),
                };
                let reply = Bind::Reply {
                    granted,
                    binding: t.binding,
                }
                .encode();
                let _ = self
                    .swarm
                    .behaviour_mut()
                    .bind
                    .send_response(channel, reply);
            }
            request_response::Message::Response { response, .. } => {
                if let Some(Bind::Reply { granted, binding }) = Bind::decode(&response) {
                    if t.verify(&peer, binding.as_ref()) && granted {
                        t.granted(peer);
                    }
                }
                self.pull(peer);
            }
        }
    }

    /// Asks `peer` for what of `offered` this node lacks and `peer` may hear it ask for.
    fn want(&mut self, peer: PeerId, offered: &[EntryId]) {
        let want: Vec<EntryId> = self
            .store
            .replica()
            .want(offered)
            .into_iter()
            .filter(|id| self.reads(&peer, &id.writer))
            .collect();
        self.ask_for(peer, want);
    }

    fn ask_for(&mut self, peer: PeerId, want: Vec<EntryId>) {
        if !want.is_empty() {
            self.send(&peer, Message::Want(want), Sent::Want);
        }
    }

    /// Takes the catch-up a pull's answer proves: a feed whose key it offers nothing more of.
    fn caught_up_by(&mut self, want: &[EntryId], circle: bool) {
        let lacks = |w: &Option<Writing>| {
            let key = w.as_ref().map(|w| w.writer.public().to_bytes());
            want.iter().any(|id| Some(id.writer) == key)
        };
        if !self.caught_up && !lacks(&self.writing) {
            self.caught_up = true;
            if let Some(w) = self.writing.as_mut() {
                let _ = w.catch_up(self.store.replica());
            }
        }
        if circle && !self.circle_caught_up && !lacks(&self.circle) {
            self.circle_caught_up = true;
            if let Some(w) = self.circle.as_mut() {
                let _ = w.catch_up(self.store.replica());
            }
        }
    }

    fn sync_message(&mut self, peer: PeerId, message: request_response::Message<Vec<u8>, Vec<u8>>) {
        match message {
            request_response::Message::Request {
                request, channel, ..
            } => {
                let replica = self.store.replica();
                let reply = match (Message::decode(&request), &self.tiered) {
                    (Some(Message::Summary(s)), None) => Message::Have(replica.have_for(&s)),
                    (Some(Message::Summary(s)), Some(t)) => {
                        Message::Have(t.have_for(replica, &peer, &s))
                    }
                    (Some(Message::Want(ids)), None) => {
                        Message::Entries(replica.entries_for(&ids, MAX_RESPONSE))
                    }
                    (Some(Message::Want(ids)), Some(t)) => {
                        Message::Entries(t.entries_for(replica, &peer, &ids))
                    }
                    (Some(Message::Have(ids)), Some(t)) => {
                        let bound = t.is_bound(&peer);
                        let ack = Message::Have(Vec::new()).encode();
                        let _ = self.swarm.behaviour_mut().sync.send_response(channel, ack);
                        if bound {
                            self.want(peer, &ids);
                        }
                        return;
                    }
                    _ => return,
                };
                let _ = self
                    .swarm
                    .behaviour_mut()
                    .sync
                    .send_response(channel, reply.encode());
            }
            request_response::Message::Response {
                request_id,
                response,
            } => match (Message::decode(&response), self.pending.remove(&request_id)) {
                (Some(Message::Have(ids)), Some(Sent::Pull { circle })) => {
                    let want: Vec<EntryId> = self
                        .store
                        .replica()
                        .want(&ids)
                        .into_iter()
                        .filter(|id| self.reads(&peer, &id.writer))
                        .collect();
                    self.caught_up_by(&want, circle);
                    self.ask_for(peer, want);
                }
                (Some(Message::Entries(list)), Some(Sent::Want)) => {
                    let mut new = Vec::new();
                    for (entry, object) in list {
                        let id = entry.id();
                        if !self.reads(&peer, &id.writer) {
                            continue;
                        }
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
            },
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
