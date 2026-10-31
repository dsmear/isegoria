//! libp2p transport for replication (`docs/04` §Replication between nodes, `docs/08`
//! NET-010): gossipsub announces new entries, request-response pulls what a node lacks.

pub mod archive;
pub mod blind;
pub mod discovery;
pub mod member;
pub mod record;
pub mod tiered;

use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, StreamExt};
use libp2p::gossipsub::{self, IdentTopic, MessageAcceptance, MessageAuthenticity, ValidationMode};
use libp2p::request_response::{self, OutboundRequestId, ProtocolSupport, ResponseChannel};
use libp2p::swarm::{behaviour::toggle::Toggle, NetworkBehaviour, SwarmEvent};
use libp2p::{identity, noise, tcp, yamux, Multiaddr, PeerId, StreamProtocol, Swarm};
use member::{Collector, MemberRole, Progress};
use network::consortium::Consortium;
use network::replica::{
    Accepted, DiskError, DurableReplica, EntryId, FeedError, FeedWriter, Message, Replica,
    SignedEntry, Summary, WriterSet, MAX_RESPONSE,
};
use network::store::{DurableLog, ObjectStore, StoreError};
use network::tier::{Tier, Tiers};
use record::{Guard, State, Stop};
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

pub(crate) async fn write_all<T: AsyncWrite + Unpin + Send>(
    io: &mut T,
    bytes: Vec<u8>,
) -> io::Result<()> {
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
    archive: Toggle<request_response::Behaviour<archive::ShardCodec>>,
    kad: Toggle<libp2p::kad::Behaviour<discovery::ProviderStore>>,
    object: Toggle<request_response::Behaviour<discovery::ObjectCodec>>,
    gossip: gossipsub::Behaviour,
    sync: request_response::Behaviour<SyncCodec>,
    bind: request_response::Behaviour<SyncCodec>,
    relay: request_response::Behaviour<blind::Limited>,
    gateway: request_response::Behaviour<blind::Limited>,
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
    /// A tiered relay's routes (`docs/04` §Blind relays): the members' key configurations.
    pub relay: Option<Vec<blind::KeyAnnouncement>>,
    /// A tiered member's gateway (`docs/04` §Blind relays).
    pub gateway: Option<blind::Gateway>,
    /// The record governing a tiered node's feeds; without one they write nothing (`docs/04`
    /// §Increments A and B). Ignored without tiers.
    pub identity: Option<PathBuf>,
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
    /// Why a tiered writer signs nothing, if it does not (`docs/04` §Increments A and B).
    pub stopped: Option<Stop>,
}

#[derive(Debug)]
pub enum StartError {
    Archive(archive::InstallError),
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
    /// A relay or gateway outside a tiered network, a relay with a member role or a feed, a
    /// gateway of another member, or key configurations that do not verify (`docs/04`).
    Relays,
    /// An identity record naming another key for a configured feed.
    Identity,
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
    /// The tiered writer is stopped (`docs/04` §Increments A and B).
    Stopped(Stop),
}

enum Command {
    AuditArchive(
        [u8; 32],
        [u8; 32],
        Option<usize>,
        oneshot::Sender<Result<archive::Audit, archive::InstallError>>,
    ),
    MaintainArchive(
        archive::Prepared,
        archive::Deadline,
        oneshot::Sender<Result<(), archive::MaintainError>>,
    ),
    InstallArchive(
        archive::Prepared,
        oneshot::Sender<Result<(), archive::InstallError>>,
    ),
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

/// A running node; dropping its last handle or monitor ends the node's task.
pub struct Handle {
    peer_id: PeerId,
    commands: mpsc::UnboundedSender<Command>,
    set_aside: Vec<PathBuf>,
    task: tokio::task::JoinHandle<()>,
}

/// Status and peer reconnection only; cannot install or mutate stored archive records.
pub struct Monitor {
    commands: mpsc::UnboundedSender<Command>,
}

impl Monitor {
    pub async fn status(&self) -> Status {
        ask(&self.commands, Command::Status).await
    }

    pub async fn dial(&self, addr: Multiaddr) -> Result<(), String> {
        ask(&self.commands, |tx| Command::Dial(addr, tx)).await
    }
}

async fn ask<T>(
    commands: &mpsc::UnboundedSender<Command>,
    make: impl FnOnce(oneshot::Sender<T>) -> Command,
) -> T {
    let (tx, rx) = oneshot::channel();
    commands
        .send(make(tx))
        .expect("the node runs while a handle or monitor lives");
    rx.await.expect("the node answers every command")
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

impl Feed {
    fn tier(self) -> Tier {
        match self {
            Feed::Public => Tier::Public,
            Feed::Circle => Tier::Circle,
        }
    }
}

/// A governed feed's failed file write stops the member until a restart; an object refused
/// for its size does not.
fn stored<T>(result: Result<T, StoreError>, guard: Option<&mut Guard>) -> Result<T, PublishError> {
    result.map_err(|e| {
        if let (StoreError::Io(_), Some(g)) = (e, guard) {
            g.halt(Stop::Failed);
        }
        PublishError::Store(format!("{e:?}"))
    })
}

/// What an outbound sync request was, so its response is read for what it is; a pull keeps
/// the grant it was asked under.
#[derive(Clone, Copy)]
enum Sent {
    Pull { grant: Option<u64> },
    Want,
    Announce,
}

struct Node {
    archive: Option<archive::Store>,
    advertisements: discovery::Announcements,
    swarm: Swarm<Behaviour>,
    store: Store,
    topic: IdentTopic,
    writing: Option<Writing>,
    circle: Option<Writing>,
    /// A tiered node's record; `None` without tiers, where T80's rules hold.
    identity: Option<Guard>,
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
    routes: Option<Vec<blind::KeyAnnouncement>>,
    gateway: Option<blind::Gateway>,
    relaying: HashMap<OutboundRequestId, ResponseChannel<Vec<u8>>>,
    // Last field, so dropped after every store above.
    _held: Box<dyn Send>,
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

/// An opened feed, the object of each of its entries, and the paths set aside.
type Opened = (Writing, Vec<Vec<u8>>, Vec<PathBuf>);

/// A feed's files, damaged ones set aside (`docs/04` §A node's own disk, rule 2); nothing is
/// signed.
fn open_writing(own: Own, replica: &Store) -> Result<Opened, StartError> {
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
    let writing = Writing {
        writer: own.writer,
        log,
        objects,
    };
    Ok((writing, named, aside))
}

/// Signs the feed's entries again into the replica; the signatures are deterministic.
fn replay(w: &Writing, named: Vec<Vec<u8>>, replica: &mut Store) -> Result<(), StartError> {
    for (entry, object) in w.log.log().entries().iter().zip(named) {
        replica
            .insert(w.writer.sign(entry), object)
            .map_err(StartError::Replica)?;
    }
    Ok(())
}

/// Opens a writer's own feed under T80's rules (`docs/04` §A writer's own feed); returns it
/// and the paths set aside.
fn open_own(own: Own, replica: &mut Store) -> Result<(Writing, Vec<PathBuf>), StartError> {
    let (mut writing, named, aside) = open_writing(own, replica)?;
    let me = writing.writer.public().to_bytes();
    if let Err(e @ FeedError::Forked { .. }) = replica
        .replica()
        .feed_beyond(&me, writing.log.log().entries())
    {
        return Err(StartError::Feed(e));
    }
    replay(&writing, named, replica)?;
    let _ = writing.catch_up(replica.replica());
    Ok((writing, aside))
}

/// Whether the replica holds a verified entry of `w`'s key beyond its files or contradicting
/// them (`docs/04` §Increments A and B).
fn own_entry(replica: &Replica, w: &Writing) -> bool {
    let me = w.writer.public().to_bytes();
    !matches!(replica.feed_beyond(&me, w.log.log().entries()), Ok(beyond) if beyond.is_empty())
}

/// What a governed feed's files, with `entries` entries, show against its state at the start.
fn classify(state: State, entries: usize, aside: bool, tier: Tier) -> Option<Stop> {
    match (aside, state, entries) {
        (true, ..) => Some(Stop::SetAside(tier)),
        (_, State::Written, 0) => Some(Stop::Lost(tier)),
        (_, State::Unused, 1..) | (_, State::Opening, 2..) => Some(Stop::Inconsistent(tier)),
        _ => None,
    }
}

type Feeds = (Option<Writing>, Option<Writing>, Vec<PathBuf>);

/// A tiered node's feeds under its record: classified and checked before any signature, an
/// opening feed's one entry kept and the feed made written, then signed again unless stopped.
fn open_governed(
    own: Option<Own>,
    circle: Option<Own>,
    replica: &mut Store,
    guard: &mut Guard,
) -> Result<Feeds, StartError> {
    let mut aside = Vec::new();
    let mut opened = Vec::new();
    for (own, tier) in [(own, Tier::Public), (circle, Tier::Circle)] {
        let Some(own) = own else { continue };
        let (writing, named, set) = open_writing(own, replica)?;
        let entries = writing.log.log().len();
        if let Some(stop) = classify(guard.state(tier), entries, !set.is_empty(), tier) {
            guard.halt(stop);
        }
        if own_entry(replica.replica(), &writing) {
            guard.halt(Stop::OwnEntry(tier));
        }
        aside.extend(set);
        opened.push((tier, writing, named));
    }
    for (tier, writing, _) in &opened {
        if guard.state(*tier) == State::Opening && writing.log.log().len() == 1 {
            let _ = guard.advance(*tier, State::Written);
        }
    }
    let (mut public, mut circle) = (None, None);
    for (tier, writing, named) in opened {
        if guard.stop().is_none() {
            replay(&writing, named, replica)?;
        }
        match tier {
            Tier::Public => public = Some(writing),
            Tier::Circle => circle = Some(writing),
        }
    }
    Ok((public, circle, aside))
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

fn build_swarm(
    key: identity::Keypair,
    network: Option<[u8; 32]>,
) -> Result<Swarm<Behaviour>, String> {
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
            let relay = blind::behaviour(blind::RELAY_PROTOCOL);
            let gateway = blind::behaviour(blind::GATEWAY_PROTOCOL);
            Ok(Behaviour {
                archive: network.map(|n| archive::behaviour(&n)).into(),
                kad: network
                    .map(|n| discovery::kad_behaviour(key.public().to_peer_id(), &n, true))
                    .into(),
                object: network.map(|n| discovery::object_behaviour(&n)).into(),
                gossip,
                sync,
                bind,
                relay,
                gateway,
            })
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

/// A relay's routes and a gateway's key verify under the node's tiers; a relay is no member
/// and writes no feed, a gateway is its member's. Checked before any feed's file is opened.
fn blind_start(
    tiered: Option<&Tiered>,
    config: &Config,
    routes: Option<&[blind::KeyAnnouncement]>,
    gateway: Option<&blind::Gateway>,
) -> Result<(), StartError> {
    if routes.is_none() && gateway.is_none() {
        return Ok(());
    }
    let tiers = &tiered.ok_or(StartError::Relays)?.tiers;
    if let Some(routes) = routes {
        let mut members: Vec<usize> = routes.iter().map(|a| a.member).collect();
        members.sort_unstable();
        members.dedup();
        let valid = routes.iter().all(|a| a.verify(tiers)) && members.len() == routes.len();
        let feeds = config.member.is_some() || config.own.is_some() || config.circle.is_some();
        if feeds || !valid {
            return Err(StartError::Relays);
        }
    }
    if let Some(g) = gateway {
        let key = g.key.announcement();
        let index = config.member.as_ref().map(|m| m.index);
        if index != Some(key.member) || !key.verify(tiers) {
            return Err(StartError::Relays);
        }
    }
    Ok(())
}

impl Handle {
    /// Starts a node on the current tokio runtime; a writer's own feed is replayed first.
    pub fn spawn(key: identity::Keypair, config: Config) -> Result<Handle, StartError> {
        Self::spawn_holding(key, config, ())
    }

    /// As [`Self::spawn`]; the node's task owns `held` and drops it after its stores, whether
    /// the task ends or is canceled (`docs/04` §Initial archive installation on a holder).
    pub fn spawn_holding(
        key: identity::Keypair,
        mut config: Config,
        held: impl Send + 'static,
    ) -> Result<Handle, StartError> {
        let tiered = match config.tiers.take() {
            Some(tiers) => Some(tiered_start(&key, tiers, &config)?),
            None if config.circle.is_some() => return Err(StartError::NotTheCircle),
            None => None,
        };
        let (routes, gateway) = (config.relay.take(), config.gateway.take());
        blind_start(
            tiered.as_ref(),
            &config,
            routes.as_deref(),
            gateway.as_ref(),
        )?;
        let archive = tiered
            .as_ref()
            .map(|t| {
                archive::Store::open(
                    &t.tiers,
                    config.store.as_ref().map(|dir| dir.join("archive")),
                )
            })
            .transpose()
            .map_err(StartError::Archive)?;
        let mut replica = match config.store {
            Some(dir) => Store::Disk(
                DurableReplica::open(&dir, config.writers.clone()).map_err(StartError::Replica)?,
            ),
            None => Store::Memory(Replica::new(config.writers.clone())),
        };
        if let Some(role) = &config.member {
            let key = role.member.public();
            let mine = role.consortium.keys().get(role.index) == Some(&key);
            if !mine || config.own.as_ref().is_none_or(|w| w.writer.public() != key) {
                return Err(StartError::NotTheMember);
            }
        }
        let mut guard = match (&config.own, &tiered) {
            (Some(own), Some(_)) => {
                let circle = config.circle.as_ref().map(|c| c.writer.public().to_bytes());
                let public = own.writer.public().to_bytes();
                let path = config.identity.as_deref();
                Some(Guard::open(path, public, circle).map_err(|_| StartError::Identity)?)
            }
            _ => None,
        };
        let (writing, circle, set_aside) = match guard.as_mut() {
            Some(guard) => open_governed(config.own, config.circle, &mut replica, guard)?,
            None => {
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
                (writing, circle, set_aside)
            }
        };
        let network_id = config.writers.network_id();
        let consortium = config.member.as_ref().map(|m| &m.consortium);
        let collector = consortium
            .or(config.consortium.as_ref())
            .map_or_else(Collector::default, |c| Collector::new(c, network_id));
        let progress = consortium.map_or_else(Progress::default, |c| Progress::new(c, network_id));
        let discovery_network = tiered.as_ref().map(|t| t.tiers.network_id());
        let mut swarm = build_swarm(key, discovery_network).map_err(StartError::Transport)?;
        let topic = IdentTopic::new(topic(&config.writers.network_id()));
        swarm
            .behaviour_mut()
            .gossip
            .subscribe(&topic)
            .map_err(|e| StartError::Transport(e.to_string()))?;
        let peer_id = *swarm.local_peer_id();
        let (commands, rx) = mpsc::unbounded_channel();
        let mut node = Node {
            archive,
            advertisements: discovery::Announcements::default(),
            swarm,
            store: replica,
            topic,
            writing,
            circle,
            identity: guard,
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
            routes,
            gateway,
            relaying: HashMap::new(),
            _held: Box::new(held),
        };
        if let Some(t) = &node.tiered {
            for id in node.store.replica().ids().filter(|id| t.is_public(id)) {
                if let Some((entry, _)) = node.store.replica().get(&id) {
                    node.advertisements.add(entry.entry.payload);
                }
            }
        }
        if let Some(store) = &node.archive {
            for key in store.advertisements() {
                node.advertisements.add_key(key);
            }
        }
        node.advertise();
        let task = tokio::spawn(node.run(rx, config.sync_every));
        Ok(Handle {
            peer_id,
            commands,
            set_aside,
            task,
        })
    }

    /// Drops this handle and waits for the node's task: it first answers queued commands, and
    /// ends only once every monitor is dropped too.
    pub async fn finished(self) {
        drop(self.commands);
        let _ = self.task.await;
    }

    /// The own files found damaged at the start and renamed `<name>.damaged-<n>`.
    pub fn set_aside(&self) -> &[PathBuf] {
        &self.set_aside
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    async fn ask<T>(&self, make: impl FnOnce(oneshot::Sender<T>) -> Command) -> T {
        ask(&self.commands, make).await
    }

    pub fn monitor(&self) -> Monitor {
        Monitor {
            commands: self.commands.clone(),
        }
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

    /// Validates a public segment and atomically replaces its selected local shards.
    pub async fn install_archive(
        &self,
        ctx: &network::archive::Context<'_>,
        signed: &network::archive::SignedManifest,
        segment: &network::archive::Segment,
        indices: &[usize],
    ) -> Result<(), archive::InstallError> {
        let prepared = archive::Prepared::new(ctx, signed, segment, indices)?;
        self.ask(|tx| Command::InstallArchive(prepared, tx)).await
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
            stopped: self.identity.as_ref().and_then(|g| g.stop()),
        }
    }

    fn duties(&mut self) {
        let writes = self.identity.as_ref().is_none_or(|g| g.stop().is_none());
        let Some(role) = self.member.as_ref().filter(|_| self.caught_up && writes) else {
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
        let grant = self.tiered.as_ref().and_then(|t| t.grant(&peer));
        let summary = Message::Summary(self.summary_for(&peer));
        self.send(&peer, summary, Sent::Pull { grant });
    }

    fn advertise(&mut self) {
        if let Some(kad) = self.swarm.behaviour_mut().kad.as_mut() {
            self.advertisements.pump(kad);
        }
    }

    /// Public identifiers on gossip; in a tiered network, circle ones to each bound peer.
    fn announce(&mut self, ids: &[EntryId]) {
        if let Some(t) = &self.tiered {
            for id in ids.iter().filter(|id| t.is_public(id)) {
                if let Some((entry, _)) = self.store.replica().get(id) {
                    self.advertisements.add(entry.entry.payload);
                }
            }
            self.advertise();
        }
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

    /// Whether `feed` may sign now. Governed: the member not stopped, the feed caught up, no
    /// verified own entry outside its files. Otherwise caught up, taking its feed back (T80).
    fn ready(&mut self, feed: Feed) -> Result<(), PublishError> {
        let (w, caught) = match feed {
            Feed::Public => (self.writing.as_mut(), self.caught_up),
            Feed::Circle => (self.circle.as_mut(), self.circle_caught_up),
        };
        let w = w.ok_or(PublishError::NotAWriter)?;
        let Some(guard) = self.identity.as_mut() else {
            if !caught {
                return Err(PublishError::NotCaughtUp);
            }
            return w.catch_up(self.store.replica());
        };
        if let Some(stop) = guard.stop() {
            return Err(PublishError::Stopped(stop));
        }
        if !caught {
            return Err(PublishError::NotCaughtUp);
        }
        if own_entry(self.store.replica(), w) {
            return Err(PublishError::Stopped(
                guard.halt(Stop::OwnEntry(feed.tier())),
            ));
        }
        Ok(())
    }

    /// Signs `object` on `feed`; a governed feed's first entry follows the record's durable
    /// order (`docs/04` §Increments A and B).
    fn publish(&mut self, feed: Feed, object: Vec<u8>) -> Result<SignedEntry, PublishError> {
        self.ready(feed)?;
        let tier = feed.tier();
        let w = match feed {
            Feed::Public => self.writing.as_mut(),
            Feed::Circle => self.circle.as_mut(),
        }
        .ok_or(PublishError::NotAWriter)?;
        if let Some(g) = self.identity.as_mut() {
            if g.state(tier) == State::Unused {
                g.advance(tier, State::Opening)
                    .map_err(PublishError::Stopped)?;
            }
        }
        let payload = stored(w.objects.put(&object), self.identity.as_mut())?;
        let entry = stored(w.log.append(payload), self.identity.as_mut())?;
        if let Some(g) = self.identity.as_mut() {
            if g.state(tier) == State::Opening {
                g.advance(tier, State::Written)
                    .map_err(PublishError::Stopped)?;
            }
        }
        let signed = w.writer.sign(&entry);
        self.accept(signed.clone(), object)
            .map_err(PublishError::Replica)?;
        Ok(signed)
    }

    /// Stops a governed member whose replica now holds a verified own entry outside its files.
    fn watch(&mut self) {
        let Some(guard) = self.identity.as_mut() else {
            return;
        };
        for (w, tier) in [(&self.writing, Tier::Public), (&self.circle, Tier::Circle)] {
            if w.as_ref()
                .is_some_and(|w| own_entry(self.store.replica(), w))
            {
                guard.halt(Stop::OwnEntry(tier));
            }
        }
    }

    fn install_archive(
        &mut self,
        prepared: archive::Prepared,
    ) -> Result<(), archive::InstallError> {
        let result = match (&mut self.archive, &self.tiered) {
            (Some(store), Some(t)) => {
                let digest = prepared.digest();
                let old = store.keys(&digest);
                let result = store.install(&t.tiers, prepared);
                if result.is_ok() {
                    let new = store.keys(&digest);
                    if let Some(kad) = self.swarm.behaviour_mut().kad.as_mut() {
                        for key in old.iter().filter(|key| !new.contains(key)) {
                            self.advertisements.remove(key, kad);
                        }
                    }
                    for key in new {
                        self.advertisements.add_key(key);
                    }
                }
                result
            }
            _ => Err(archive::InstallError::Configuration),
        };
        self.advertise();
        result
    }

    fn command(&mut self, command: Command) {
        match command {
            Command::InstallArchive(prepared, tx) => {
                let result = self.install_archive(prepared);
                let _ = tx.send(result);
            }
            Command::AuditArchive(context, digest, index, tx) => {
                let result = match (&self.archive, &self.tiered) {
                    (Some(store), Some(t)) => store.audit(&t.tiers, context, digest, index),
                    _ => Err(archive::InstallError::Configuration),
                };
                let _ = tx.send(result);
            }
            Command::MaintainArchive(prepared, deadline, tx) => {
                let result = deadline.remaining().and_then(|_| {
                    self.install_archive(prepared)
                        .map_err(archive::MaintainError::Install)
                });
                let _ = tx.send(result);
            }
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
                let accepted = self.accept(entry, object);
                self.watch();
                let _ = tx.send(accepted);
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
                if self.tiered.is_some() {
                    self.swarm.add_external_address(address.clone());
                    self.advertisements.refresh();
                    self.advertise();
                }
                for tx in self.listening.drain(..) {
                    let _ = tx.send(Ok(address.clone()));
                }
            }
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                if let Some(kad) = self.swarm.behaviour_mut().kad.as_mut() {
                    if let libp2p::core::ConnectedPoint::Dialer { address, .. } = endpoint {
                        kad.add_address(&peer_id, address);
                    }
                }
                self.advertisements.refresh();
                self.advertise();
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
            SwarmEvent::Behaviour(BehaviourEvent::Kad(
                libp2p::kad::Event::OutboundQueryProgressed { id, step, .. },
            )) => {
                if step.last {
                    self.advertisements.completed(id);
                    self.advertise();
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Archive(request_response::Event::Message {
                message:
                    request_response::Message::Request {
                        request, channel, ..
                    },
                ..
            })) => {
                if let (Some(store), Some(t)) = (&self.archive, &self.tiered) {
                    let response = store.serve(&t.tiers, &request);
                    if let Some(archive) = self.swarm.behaviour_mut().archive.as_mut() {
                        let _ = archive.send_response(channel, response);
                    }
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Object(request_response::Event::Message {
                message:
                    request_response::Message::Request {
                        request, channel, ..
                    },
                ..
            })) => {
                if let Some(t) = &self.tiered {
                    let response = discovery::serve(&t.tiers, self.store.replica(), &request);
                    if let Some(object) = self.swarm.behaviour_mut().object.as_mut() {
                        let _ = object.send_response(channel, response);
                    }
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
            SwarmEvent::Behaviour(BehaviourEvent::Relay(request_response::Event::Message {
                message:
                    request_response::Message::Request {
                        request, channel, ..
                    },
                ..
            })) => self.relay_request(&request, channel),
            SwarmEvent::Behaviour(BehaviourEvent::Gateway(request_response::Event::Message {
                peer,
                message,
                ..
            })) => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => self.gateway_request(peer, &request, channel),
                request_response::Message::Response {
                    request_id,
                    response,
                } => self.relayed(request_id, response),
            },
            SwarmEvent::Behaviour(BehaviourEvent::Gateway(
                request_response::Event::OutboundFailure { request_id, .. },
            )) => {
                let refused = blind::Carried::Refused(blind::Refusal::Unreachable);
                self.relayed(request_id, refused.encode());
            }
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
                let granted = match Bind::decode(&response) {
                    Some(Bind::Reply { granted, binding }) => {
                        t.verify(&peer, binding.as_ref()) && granted
                    }
                    _ => t.verify(&peer, None),
                };
                t.granted(peer, granted);
                self.pull(peer);
            }
        }
    }

    /// A relay forwards a request unchanged to the member its key id names, if connected.
    fn relay_request(&mut self, request: &[u8], channel: ResponseChannel<Vec<u8>>) {
        let Some(routes) = self.routes.as_ref() else {
            return;
        };
        let named = request.first();
        let route = routes.iter().find(|a| a.config.first() == named);
        let peer = route.and_then(|a| self.tiered.as_ref()?.peer_of(a.member));
        let refusal = match (route, peer) {
            (Some(_), Some(peer)) => {
                let gateway = &mut self.swarm.behaviour_mut().gateway;
                let id = gateway.send_request(&peer, request.to_vec());
                self.relaying.insert(id, channel);
                return;
            }
            (None, _) => blind::Refusal::UnknownKey,
            (Some(_), None) => blind::Refusal::Unreachable,
        };
        let refused = blind::Carried::Refused(refusal).encode();
        let _ = self
            .swarm
            .behaviour_mut()
            .relay
            .send_response(channel, refused);
    }

    /// A relay returns the gateway's answer unchanged.
    fn relayed(&mut self, id: OutboundRequestId, response: Vec<u8>) {
        if let Some(channel) = self.relaying.remove(&id) {
            let _ = self
                .swarm
                .behaviour_mut()
                .relay
                .send_response(channel, response);
        }
    }

    /// A gateway answers a listed relay's request with its answer sealed in that request's
    /// context, or a refusal in the clear before anything is decapsulated or logged.
    fn gateway_request(&mut self, peer: PeerId, request: &[u8], channel: ResponseChannel<Vec<u8>>) {
        let Some(g) = self.gateway.as_ref() else {
            return;
        };
        let listed = tiered::transport_key(&peer).is_some_and(|k| g.relays.contains(&k));
        let opened = match listed {
            true => blind::open_request(&g.key, request),
            false => Err(blind::Refusal::NotARelay),
        };
        let carried = match opened {
            Err(refusal) => blind::Carried::Refused(refusal),
            Ok((inner, response)) => {
                let answer = self.answer(&inner);
                match answer.to_http().and_then(|b| response.encapsulate(&b).ok()) {
                    Some(sealed) => blind::Carried::Sealed(sealed),
                    None => blind::Carried::Refused(blind::Refusal::Undecryptable),
                }
            }
        };
        let _ = self
            .swarm
            .behaviour_mut()
            .gateway
            .send_response(channel, carried.encode());
    }

    /// What a gateway does with an opened request: bytes already on its circle feed answered
    /// with their entry, other admitted bytes logged there once the feed caught up.
    fn answer(&mut self, inner: &[u8]) -> blind::Answer {
        let event = match blind::read_request(inner, &self.network_id) {
            Ok(event) => event,
            Err(answer) => return answer,
        };
        if !self.gateway.as_ref().is_some_and(|g| (g.admit)(&event)) {
            return blind::Answer::NotAdmitted;
        }
        if !self.circle_caught_up {
            return blind::Answer::RetryLater;
        }
        let Some(w) = self.circle.as_mut() else {
            return blind::Answer::Failed;
        };
        if self.identity.is_none() && w.catch_up(self.store.replica()).is_err() {
            return blind::Answer::Failed;
        }
        let payload = network::cid::cid(&event);
        let writer = w.writer.public().to_bytes();
        if let Some(e) = w.log.log().entries().iter().find(|e| e.payload == payload) {
            let (seq, hash) = (e.seq, e.hash);
            return blind::Answer::Logged(EntryId { writer, seq, hash });
        }
        match self.publish(Feed::Circle, event) {
            Ok(signed) => blind::Answer::Logged(signed.id()),
            Err(PublishError::NotCaughtUp) => blind::Answer::RetryLater,
            Err(_) => blind::Answer::Failed,
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
            let _ = self.ready(Feed::Public);
        }
        if circle && !self.circle_caught_up && !lacks(&self.circle) {
            self.circle_caught_up = true;
            let _ = self.ready(Feed::Circle);
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
                (Some(Message::Have(ids)), Some(Sent::Pull { grant })) => {
                    let held = |t: &Tiered| grant.is_some_and(|g| t.holds(&peer, g));
                    let circle = self.tiered.as_ref().is_some_and(held);
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
                        self.watch();
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
