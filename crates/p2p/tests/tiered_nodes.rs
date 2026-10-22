//! A tiered network over libp2p (`docs/04` §Transport, a network with tiers; `docs/10` T77):
//! what an outsider is sent, bindings, both feeds, the public reader, restart and recovery.

use futures::{AsyncWriteExt, StreamExt};
use libp2p::gossipsub::{self, IdentTopic, MessageAuthenticity, ValidationMode};
use libp2p::identity::Keypair;
use libp2p::request_response::{self, ProtocolSupport};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{noise, tcp, yamux, Multiaddr, PeerId, StreamProtocol};
use network::consortium::{Consortium, Member};
use network::cut::MemberObject;
use network::replica::{EntryId, FeedWriter, Message, Replica, Summary};
use network::tier::{Tier, Tiers, TransportBinding};
use p2p::member::MemberRole;
use p2p::tiered::Bind;
use p2p::{
    read_limited, topic, Config, Handle, Own, PublishError, Status, BIND_PROTOCOL, MESSAGE_LIMIT,
    SYNC_PROTOCOL,
};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;

const NET: [u8; 32] = [33; 32];
const MEMBER_SEEDS: [u8; 3] = [60, 61, 62];
const CIRCLE_SEEDS: [u8; 3] = [70, 71, 72];
const TRANSPORT_SEEDS: [u8; 4] = [80, 81, 82, 83];

fn scratch(name: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("tiered_nodes")
        .join(format!("{name}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn member_keys() -> Vec<Member> {
    MEMBER_SEEDS
        .iter()
        .map(|s| Member::from_seed([*s; 32]))
        .collect()
}

fn consortium() -> Consortium {
    Consortium::new(member_keys().iter().map(|m| m.public()).collect(), 2)
}

fn tiers_on(net: [u8; 32]) -> Tiers {
    let circle = CIRCLE_SEEDS.map(|s| FeedWriter::from_seed(net, [s; 32]).public());
    Tiers::new(net, consortium(), &circle).expect("a valid tiered set")
}

fn tiers() -> Tiers {
    tiers_on(NET)
}

fn is_circle(writer: &[u8; 32]) -> bool {
    tiers().tier(writer) == Some(Tier::Circle)
}

/// Node `i`'s libp2p key: 0–2 the members', 3 the public reader's.
fn transport(i: usize) -> Keypair {
    Keypair::ed25519_from_bytes([TRANSPORT_SEEDS[i]; 32]).expect("a 32-byte seed")
}

fn transport_bytes(key: &Keypair) -> [u8; 32] {
    key.public().try_into_ed25519().unwrap().to_bytes()
}

/// Member `i` keeping both feeds under `root`, its replica on disk there if `stored`.
fn member_config(i: usize, root: &std::path::Path, stored: bool) -> Config {
    let seed = [MEMBER_SEEDS[i]; 32];
    Config {
        writers: tiers().writers().clone(),
        sync_every: Duration::from_millis(300),
        own: Some(Own {
            writer: FeedWriter::from_seed(NET, seed),
            dir: root.join("own"),
        }),
        circle: Some(Own {
            writer: FeedWriter::from_seed(NET, [CIRCLE_SEEDS[i]; 32]),
            dir: root.join("circle"),
        }),
        store: stored.then(|| root.join("replica")),
        member: Some(MemberRole {
            member: Member::from_seed(seed),
            index: i,
            consortium: consortium(),
            every: Duration::from_millis(150),
            cuts_per_epoch: 2,
            patience: 10,
            view_patience: 20,
        }),
        consortium: None,
        tiers: Some(tiers()),
    }
}

fn reader_config() -> Config {
    Config {
        writers: tiers().writers().clone(),
        sync_every: Duration::from_millis(300),
        own: None,
        circle: None,
        store: None,
        member: None,
        consortium: None,
        tiers: Some(tiers()),
    }
}

async fn start(key: Keypair, config: Config) -> (Handle, Multiaddr) {
    let h = Handle::spawn(key, config).expect("a node starts");
    let addr = h
        .listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .await
        .expect("loopback listens");
    (h, addr)
}

/// Waits, at most 30 s, until `done` holds.
async fn until<F, Fut>(what: &str, done: F)
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    while !done().await {
        assert!(tokio::time::Instant::now() < deadline, "timed out: {what}");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn status(h: &Handle) -> Status {
    h.status().await
}

/// Publishes on the circle feed as soon as the member may, within 30 s.
async fn publish_circle(h: &Handle, object: Vec<u8>) {
    until("a circle publication", || async {
        h.publish_circle(object.clone()).await.is_ok()
    })
    .await;
}

/// Three members connected to member 0, each with `n` circle and `n` public objects.
async fn three_members(name: &str, n: usize) -> (Vec<Handle>, Multiaddr) {
    let root = scratch(name);
    let mut handles = Vec::new();
    let mut first = None;
    for i in 0..3 {
        let (h, addr) = start(
            transport(i),
            member_config(i, &root.join(i.to_string()), false),
        )
        .await;
        if let Some(a) = &first {
            h.dial(Clone::clone(a)).await.unwrap();
        } else {
            first = Some(addr);
        }
        handles.push(h);
    }
    for (i, h) in handles.iter().enumerate() {
        for k in 0..n {
            publish_circle(h, format!("circle-secret-{i}-{k}-{name}").into_bytes()).await;
            h.publish(format!("public-note-{i}-{k}").into_bytes())
                .await
                .unwrap();
        }
    }
    (handles, first.unwrap())
}

async fn converged(handles: &[&Handle]) {
    until("the members converge", || async {
        let mut digests = Vec::new();
        for h in handles {
            digests.push(h.replica().await.digest());
        }
        digests.windows(2).all(|d| d[0] == d[1])
    })
    .await;
}

#[derive(Clone, Default)]
struct Bytes;

#[async_trait::async_trait]
impl request_response::Codec for Bytes {
    type Protocol = StreamProtocol;
    type Request = Vec<u8>;
    type Response = Vec<u8>;

    async fn read_request<T>(&mut self, _: &StreamProtocol, io: &mut T) -> std::io::Result<Vec<u8>>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        read_limited(io, MESSAGE_LIMIT).await
    }

    async fn read_response<T>(&mut self, _: &StreamProtocol, io: &mut T) -> std::io::Result<Vec<u8>>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        read_limited(io, MESSAGE_LIMIT).await
    }

    async fn write_request<T>(
        &mut self,
        _: &StreamProtocol,
        io: &mut T,
        r: Vec<u8>,
    ) -> std::io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        io.write_all(&r).await?;
        io.close().await
    }

    async fn write_response<T>(
        &mut self,
        _: &StreamProtocol,
        io: &mut T,
        r: Vec<u8>,
    ) -> std::io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        io.write_all(&r).await?;
        io.close().await
    }
}

#[derive(NetworkBehaviour)]
struct Spying {
    gossip: gossipsub::Behaviour,
    sync: request_response::Behaviour<Bytes>,
    bind: request_response::Behaviour<Bytes>,
}

/// Every byte a spy was sent: requests, responses and gossip, and the bind replies decoded.
#[derive(Default)]
struct Heard {
    sent: Vec<Vec<u8>>,
    gossip: usize,
    replies: Vec<Bind>,
    entries: Vec<EntryId>,
}

enum Order {
    Dial(Multiaddr),
    Sync(PeerId, Vec<u8>),
    Bind(PeerId, Vec<u8>),
    Gossip(Vec<u8>),
}

/// A raw libp2p peer outside the circle: it answers the node's requests with nothing, asks
/// what a test tells it to, listens to gossip and keeps all it receives.
struct Spy {
    orders: mpsc::UnboundedSender<Order>,
    heard: Arc<Mutex<Heard>>,
    peers: Arc<Mutex<HashSet<PeerId>>>,
}

impl Spy {
    fn start(key: Keypair, binding: Option<TransportBinding>) -> Spy {
        let gossip_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_millis(250))
            .validation_mode(ValidationMode::Strict)
            .build()
            .unwrap();
        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(key)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .unwrap()
            .with_behaviour(|key| {
                let protocol = |name| [(StreamProtocol::new(name), ProtocolSupport::Full)];
                Ok(Spying {
                    gossip: gossipsub::Behaviour::new(
                        MessageAuthenticity::Signed(key.clone()),
                        gossip_config,
                    )?,
                    sync: request_response::Behaviour::new(
                        protocol(SYNC_PROTOCOL),
                        Default::default(),
                    ),
                    bind: request_response::Behaviour::new(
                        protocol(BIND_PROTOCOL),
                        Default::default(),
                    ),
                })
            })
            .unwrap()
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(3600)))
            .build();
        swarm
            .behaviour_mut()
            .gossip
            .subscribe(&IdentTopic::new(topic(&NET)))
            .unwrap();
        let (orders, mut rx) = mpsc::unbounded_channel();
        let heard = Arc::new(Mutex::new(Heard::default()));
        let peers = Arc::new(Mutex::new(HashSet::new()));
        let (h, p) = (heard.clone(), peers.clone());
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    order = rx.recv() => match order {
                        None => return,
                        Some(Order::Dial(a)) => { let _ = swarm.dial(a); }
                        Some(Order::Sync(peer, b)) => { swarm.behaviour_mut().sync.send_request(&peer, b); }
                        Some(Order::Bind(peer, b)) => { swarm.behaviour_mut().bind.send_request(&peer, b); }
                        Some(Order::Gossip(b)) => {
                            let t = IdentTopic::new(topic(&NET));
                            let _ = swarm.behaviour_mut().gossip.publish(t, b);
                        }
                    },
                    event = swarm.select_next_some() => spy_event(&mut swarm, event, &h, &p, binding),
                }
            }
        });
        Spy {
            orders,
            heard,
            peers,
        }
    }

    fn order(&self, o: Order) {
        self.orders.send(o).unwrap();
    }

    async fn connect(&self, addr: Multiaddr) -> PeerId {
        self.order(Order::Dial(addr));
        until("the spy connects", || async {
            !self.peers.lock().unwrap().is_empty()
        })
        .await;
        *self.peers.lock().unwrap().iter().next().unwrap()
    }
}

fn spy_event(
    swarm: &mut libp2p::Swarm<Spying>,
    event: SwarmEvent<SpyingEvent>,
    heard: &Mutex<Heard>,
    peers: &Mutex<HashSet<PeerId>>,
    binding: Option<TransportBinding>,
) {
    let mut heard = heard.lock().unwrap();
    match event {
        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
            peers.lock().unwrap().insert(peer_id);
        }
        SwarmEvent::Behaviour(SpyingEvent::Sync(request_response::Event::Message {
            message,
            ..
        })) => match message {
            request_response::Message::Request {
                request, channel, ..
            } => {
                let reply = match Message::decode(&request) {
                    Some(Message::Want(_)) => Message::Entries(Vec::new()),
                    _ => Message::Have(Vec::new()),
                };
                heard.sent.push(request);
                let _ = swarm
                    .behaviour_mut()
                    .sync
                    .send_response(channel, reply.encode());
            }
            request_response::Message::Response { response, .. } => {
                if let Some(Message::Entries(list)) = Message::decode(&response) {
                    heard.entries.extend(list.iter().map(|(e, _)| e.id()));
                }
                heard.sent.push(response);
            }
        },
        SwarmEvent::Behaviour(SpyingEvent::Bind(request_response::Event::Message {
            message,
            ..
        })) => match message {
            request_response::Message::Request {
                request, channel, ..
            } => {
                heard.sent.push(request);
                let reply = Bind::Reply {
                    granted: false,
                    binding,
                }
                .encode();
                let _ = swarm.behaviour_mut().bind.send_response(channel, reply);
            }
            request_response::Message::Response { response, .. } => {
                heard.replies.extend(Bind::decode(&response));
                heard.sent.push(response);
            }
        },
        SwarmEvent::Behaviour(SpyingEvent::Gossip(gossipsub::Event::Message {
            message, ..
        })) => {
            heard.gossip += 1;
            heard.sent.push(message.data);
        }
        _ => {}
    }
}

fn id_bytes(id: &EntryId) -> Vec<u8> {
    let mut out = id.writer.to_vec();
    out.extend_from_slice(&id.seq.to_le_bytes());
    out.extend_from_slice(&id.hash);
    out
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}

/// What of the circle a spy was sent: decoded identifiers, entries and records of a circle
/// writer, and in the raw bytes a circle object, identifier, or a hash no cut's mark shows.
fn circle_leaks(heard: &Heard, member: &Replica) -> Vec<String> {
    let circle: Vec<EntryId> = member.ids().filter(|id| is_circle(&id.writer)).collect();
    let mut marked = HashSet::new();
    for id in member.ids() {
        let object = member.get(&id).and_then(|(_, o)| MemberObject::decode(o));
        if let Some(MemberObject::CutSignature { cut, .. }) = object {
            marked.extend(cut.marks.iter().map(|m| m.head));
        }
    }
    let mut leaks = Vec::new();
    for bytes in &heard.sent {
        let named: Vec<[u8; 32]> = match Message::decode(bytes) {
            Some(Message::Summary(s)) => s.0.iter().map(|w| w.writer).collect(),
            Some(Message::Have(ids) | Message::Want(ids)) => ids.iter().map(|i| i.writer).collect(),
            Some(Message::Entries(list)) => list.iter().map(|(e, _)| e.writer).collect(),
            None => Vec::new(),
        };
        if named.iter().any(is_circle) {
            leaks.push("a message naming a circle writer".into());
        }
        for id in &circle {
            let (_, object) = member.get(id).unwrap();
            if contains(bytes, object) {
                leaks.push(format!("the object of {}", id.seq));
            }
            if contains(bytes, &id_bytes(id)) {
                leaks.push(format!("the identifier of {}", id.seq));
            }
            if !marked.contains(&id.hash) && contains(bytes, &id.hash) {
                leaks.push(format!("the unmarked hash of {}", id.seq));
            }
        }
    }
    leaks
}

/// Has the spy ask `peer` for everything it can name: a Summary, every circle identifier and
/// head, and a few public entries; then claims to hold circle entries.
fn interrogate(spy: &Spy, peer: PeerId, member: &Replica) {
    let circle: Vec<EntryId> = member.ids().filter(|id| is_circle(&id.writer)).collect();
    let public: Vec<EntryId> = member
        .ids()
        .filter(|id| !is_circle(&id.writer))
        .take(4)
        .collect();
    spy.order(Order::Sync(
        peer,
        Message::Summary(Summary::default()).encode(),
    ));
    spy.order(Order::Sync(peer, Message::Want(circle.clone()).encode()));
    spy.order(Order::Sync(peer, Message::Want(public).encode()));
    spy.order(Order::Sync(peer, Message::Have(circle).encode()));
}

/// AT-NET-23: an outsider asking for every circle identifier and listening is sent public only.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_net_23_an_outsider_is_sent_the_public_tier_only() {
    let (members, addr) = three_members("outsider", 2).await;
    converged(&[&members[0], &members[1], &members[2]]).await;
    let spy = Spy::start(Keypair::generate_ed25519(), None);
    let peer = spy.connect(addr).await;
    spy.order(Order::Bind(peer, Bind::Hello(None).encode()));
    interrogate(&spy, peer, &members[0].replica().await);
    for k in 0..3 {
        publish_circle(&members[0], format!("circle-late-{k}").into_bytes()).await;
        members[0]
            .publish(format!("public-late-{k}").into_bytes())
            .await
            .unwrap();
    }
    until("gossip and the answers reach the spy", || async {
        let h = spy.heard.lock().unwrap();
        h.gossip > 0 && !h.entries.is_empty() && !h.replies.is_empty()
    })
    .await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    let member = members[0].replica().await;
    let heard = spy.heard.lock().unwrap();
    assert_eq!(circle_leaks(&heard, &member), Vec::<String>::new());
    assert!(heard.entries.iter().all(|id| !is_circle(&id.writer)));
    assert!(matches!(
        heard.replies[..],
        [Bind::Reply {
            granted: false,
            binding: Some(_)
        }]
    ));
}

/// AT-NET-23: a copied or other network's binding grants nothing; one of the peer's key does.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_net_23_only_a_binding_of_the_authenticated_key_grants() {
    let (members, addr) = three_members("bindings", 1).await;
    converged(&[&members[0], &members[1], &members[2]]).await;
    let ms = member_keys();
    let key = Keypair::generate_ed25519();
    let own = transport_bytes(&key);
    let cases = [
        (
            "copied",
            tiers().bind(&ms[1], 1, transport_bytes(&transport(1))),
            false,
        ),
        (
            "other network",
            tiers_on([34; 32]).bind(&ms[1], 1, own),
            false,
        ),
        ("own key", tiers().bind(&ms[1], 1, own), true),
    ];
    let cases = cases.into_iter().map(|(n, b, g)| (n, Some(b), g)).chain([(
        "own key again, no binding",
        None,
        false,
    )]);
    for (name, binding, grants) in cases {
        let spy = Spy::start(key.clone(), binding);
        let peer = spy.connect(addr.clone()).await;
        spy.order(Order::Bind(peer, Bind::Hello(binding).encode()));
        until("the bind reply", || async {
            !spy.heard.lock().unwrap().replies.is_empty()
        })
        .await;
        interrogate(&spy, peer, &members[0].replica().await);
        until("the entries", || async {
            !spy.heard.lock().unwrap().entries.is_empty()
        })
        .await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let member = members[0].replica().await;
        {
            let heard = spy.heard.lock().unwrap();
            let got_circle = heard.entries.iter().any(|id| is_circle(&id.writer));
            assert_eq!(got_circle, grants, "{name}");
            assert!(
                matches!(heard.replies[0], Bind::Reply { granted, .. } if granted == grants),
                "{name}"
            );
            if !grants {
                let leaks = circle_leaks(&heard, &member);
                assert_eq!(leaks, Vec::<String>::new(), "{name}");
            }
        }
        drop(spy);
        until("the spy leaves", || async {
            status(&members[0]).await.peers == 2
        })
        .await;
    }
}

/// AT-NET-23: members fill both feeds; a public reader holds public only, agreeing on cuts.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_net_23_members_and_a_public_reader_agree() {
    let (members, addr) = three_members("agree", 2).await;
    let (reader, _) = start(transport(3), reader_config()).await;
    reader.dial(addr).await.unwrap();
    converged(&[&members[0], &members[1], &members[2]]).await;
    until("five cuts and two beacons", || async {
        let s = status(&reader).await;
        s.cuts.len() >= 5 && s.beacons.len() >= 2
    })
    .await;
    let full = members[0].replica().await;
    for (i, (member, seed)) in member_keys().iter().zip(CIRCLE_SEEDS).enumerate() {
        let feeds = [Tier::Public, Tier::Circle].map(|tier| {
            let key = match tier {
                Tier::Public => member.public().to_bytes(),
                Tier::Circle => FeedWriter::from_seed(NET, [seed; 32]).public().to_bytes(),
            };
            full.feed(&key).len()
        });
        assert!(
            feeds.iter().all(|n| *n >= 2),
            "member {i}'s feeds: {feeds:?}"
        );
    }
    let held = reader.replica().await;
    assert!(held.ids().all(|id| !is_circle(&id.writer)));
    let (r, m) = (status(&reader).await, status(&members[0]).await);
    let n = r.cuts.len().min(m.cuts.len());
    assert!(n >= 5);
    assert_eq!(r.cuts[..n], m.cuts[..n]);
    let b = r.beacons.len().min(m.beacons.len());
    assert!(b >= 2);
    assert_eq!(r.beacons[..b], m.beacons[..b]);
    assert!(!r.conflict && !m.conflict);
    assert_eq!((m.bound, r.bound), (2, 1));
}

/// AT-NET-23: a member restarted on its files is bound again and continues without forking.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_net_23_a_restart_keeps_the_grant_and_the_history() {
    let root = scratch("restart");
    let (m1, a1) = start(transport(1), member_config(1, &root.join("1"), false)).await;
    let (m2, _) = start(transport(2), member_config(2, &root.join("2"), false)).await;
    m2.dial(a1.clone()).await.unwrap();
    let digest = {
        let (m0, _) = start(transport(0), member_config(0, &root.join("0"), true)).await;
        m0.dial(a1.clone()).await.unwrap();
        for k in 0..3 {
            publish_circle(&m0, format!("circle-before-{k}").into_bytes()).await;
        }
        converged(&[&m0, &m1]).await;
        assert_eq!(status(&m0).await.bound, 1);
        m0.replica().await.digest()
    };
    until("the old connection closes", || async {
        status(&m1).await.peers == 1
    })
    .await;
    let (m0, _) = start(transport(0), member_config(0, &root.join("0"), true)).await;
    assert_eq!(m0.replica().await.digest(), digest);
    m0.dial(a1).await.unwrap();
    until("bound again", || async {
        status(&m0).await.bound == 1 && status(&m1).await.bound == 2
    })
    .await;
    publish_circle(&m0, b"circle-after".to_vec()).await;
    converged(&[&m0, &m1, &m2]).await;
    let r = m1.replica().await;
    let circle0 = FeedWriter::from_seed(NET, [CIRCLE_SEEDS[0]; 32])
        .public()
        .to_bytes();
    assert_eq!(r.feed(&circle0).len(), 4);
    assert!(r.equivocations().is_empty());
}

/// AT-NET-23: a member without files writes no circle entry until a member has served it.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_net_23_the_circle_feed_recovers_only_from_a_member() {
    let root = scratch("recover");
    let (m1, a1) = start(transport(1), member_config(1, &root.join("1"), false)).await;
    let (m2, _) = start(transport(2), member_config(2, &root.join("2"), false)).await;
    m2.dial(a1.clone()).await.unwrap();
    let (reader, ra) = start(transport(3), reader_config()).await;
    reader.dial(a1.clone()).await.unwrap();
    {
        let (m0, _) = start(transport(0), member_config(0, &root.join("0"), false)).await;
        m0.dial(a1.clone()).await.unwrap();
        for k in 0..3 {
            publish_circle(&m0, format!("circle-old-{k}").into_bytes()).await;
            m0.publish(format!("public-old-{k}").into_bytes())
                .await
                .unwrap();
        }
        converged(&[&m0, &m1]).await;
        let public0 = member_keys()[0].public().to_bytes();
        let n = m1.replica().await.feed(&public0).len();
        until("the reader holds member 0's public feed", || async {
            reader.replica().await.feed(&public0).len() >= n
        })
        .await;
    }
    until("member 0 has left", || async {
        status(&m1).await.peers == 2
    })
    .await;
    let (m0, _) = start(
        transport(0),
        member_config(0, &scratch("recover-lost"), false),
    )
    .await;
    m0.dial(ra).await.unwrap();
    until("the public feed catches up", || async {
        status(&m0).await.caught_up
    })
    .await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    let s = status(&m0).await;
    assert!(!s.circle_caught_up);
    assert_eq!(s.bound, 0);
    assert_eq!(
        m0.publish_circle(b"too-early".to_vec()).await,
        Err(PublishError::NotCaughtUp)
    );
    m0.dial(a1).await.unwrap();
    until("the circle feed catches up", || async {
        status(&m0).await.circle_caught_up
    })
    .await;
    publish_circle(&m0, b"circle-next".to_vec()).await;
    converged(&[&m0, &m1]).await;
    let r = m1.replica().await;
    let circle0 = FeedWriter::from_seed(NET, [CIRCLE_SEEDS[0]; 32])
        .public()
        .to_bytes();
    assert_eq!(r.feed(&circle0).len(), 4);
    assert!(r.equivocations().iter().all(|e| e.a.writer != circle0));
}

/// AT-NET-23: gossip naming a circle entry is not forwarded, nor answered with a Want.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_net_23_circle_gossip_is_not_forwarded() {
    let (members, addr) = three_members("forward", 1).await;
    converged(&[&members[0], &members[1], &members[2]]).await;
    let member = members[0].replica().await;
    let circle: Vec<EntryId> = member.ids().filter(|id| is_circle(&id.writer)).collect();
    let (talker, listener) = (
        Spy::start(Keypair::generate_ed25519(), None),
        Spy::start(Keypair::generate_ed25519(), None),
    );
    let peer = talker.connect(addr.clone()).await;
    listener.connect(addr).await;
    until("the listener hears gossip", || async {
        listener.heard.lock().unwrap().gossip > 0
    })
    .await;
    let claim = Message::Have(circle).encode();
    for _ in 0..20 {
        talker.order(Order::Gossip(claim.clone()));
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    tokio::time::sleep(Duration::from_secs(1)).await;
    let member = members[0].replica().await;
    for spy in [&talker, &listener] {
        let heard = spy.heard.lock().unwrap();
        assert_eq!(circle_leaks(&heard, &member), Vec::<String>::new());
        assert!(!heard.sent.contains(&claim));
    }
    let _ = peer;
}
