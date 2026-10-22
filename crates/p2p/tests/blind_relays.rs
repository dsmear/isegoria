//! Blind relays (`docs/04` §Blind relays, `docs/10` T85, first increment): the RFC 9458
//! vector, signed key configurations, and a synthetic event from a person through a relay.

use aes_gcm::aead::{Aead as _, KeyInit};
use bhttp::{Message as Http, Mode};
use futures::{AsyncWriteExt, StreamExt};
use hpke::Deserializable;
use libp2p::identity::Keypair;
use libp2p::request_response::{self, ProtocolSupport, ResponseChannel};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{noise, tcp, yamux, Multiaddr, PeerId, StreamProtocol};
use network::cid::cid;
use network::consortium::{Consortium, Member};
use network::log::TransparencyLog;
use network::replica::{EntryId, FeedWriter, Refused, Replica};
use network::tier::{Tier, Tiers};
use p2p::blind::{
    seal, submit, Answer, Carried, ClientError, Gateway, GatewayKey, KeyAnnouncement, Refusal,
    GATEWAY_PROTOCOL, MAX_REPLY, MAX_REQUEST, RELAY_PROTOCOL,
};
use p2p::member::MemberRole;
use p2p::{read_limited, Config, Handle, Own, StartError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;

const NET: [u8; 32] = [44; 32];
const MEMBER_SEEDS: [u8; 3] = [90, 91, 92];
const CIRCLE_SEEDS: [u8; 3] = [93, 94, 95];
const HPKE_SEEDS: [u8; 3] = [96, 97, 98];
const RELAY_SEED: u8 = 99;
const SPY_SEED: u8 = 100;

fn scratch(name: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("blind_relays")
        .join(format!("{name}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn hex(text: &str) -> Vec<u8> {
    let t: String = text.split_whitespace().collect();
    (0..t.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&t[i..i + 2], 16).unwrap())
        .collect()
}

fn members() -> Vec<Member> {
    MEMBER_SEEDS
        .iter()
        .map(|s| Member::from_seed([*s; 32]))
        .collect()
}

fn tiers_on(net: [u8; 32]) -> Tiers {
    let keys = members().iter().map(|m| m.public()).collect();
    let circle = CIRCLE_SEEDS.map(|s| FeedWriter::from_seed(net, [s; 32]).public());
    Tiers::new(net, Consortium::new(keys, 2), &circle).expect("a valid tiered set")
}

fn tiers() -> Tiers {
    tiers_on(NET)
}

fn gateway_key(i: usize) -> GatewayKey {
    GatewayKey::new(&tiers(), &members()[i], i, &[HPKE_SEEDS[i]; 32]).expect("a key")
}

fn announcements() -> Vec<KeyAnnouncement> {
    (0..3)
        .map(|i| gateway_key(i).announcement().clone())
        .collect()
}

fn transport(seed: u8) -> Keypair {
    Keypair::ed25519_from_bytes([seed; 32]).unwrap()
}

fn transport_bytes(seed: u8) -> [u8; 32] {
    transport(seed)
        .public()
        .try_into_ed25519()
        .unwrap()
        .to_bytes()
}

fn circle_key(i: usize) -> [u8; 32] {
    tiers().key(i, Tier::Circle).unwrap()
}

/// Member `i` under `root`; member 0 runs a gateway for the relay and the spy, admitting the
/// synthetic events this file sends.
fn member_config(i: usize, root: &Path) -> Config {
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
        store: Some(root.join("replica")),
        member: Some(MemberRole {
            member: Member::from_seed(seed),
            index: i,
            consortium: Consortium::new(members().iter().map(|m| m.public()).collect(), 2),
            every: Duration::from_millis(150),
            cuts_per_epoch: 2,
            patience: 10,
            view_patience: 20,
        }),
        consortium: None,
        tiers: Some(tiers()),
        relay: None,
        gateway: (i == 0).then(|| Gateway {
            key: gateway_key(0),
            relays: vec![transport_bytes(RELAY_SEED), transport_bytes(SPY_SEED)],
            admit: Box::new(|bytes| bytes.starts_with(b"synthetic-event")),
        }),
    }
}

fn relay_config(own: Option<Own>) -> Config {
    Config {
        writers: tiers().writers().clone(),
        sync_every: Duration::from_millis(300),
        own,
        circle: None,
        store: None,
        member: None,
        consortium: None,
        tiers: Some(tiers()),
        relay: Some(announcements()),
        gateway: None,
    }
}

async fn start(key: Keypair, config: Config) -> (Handle, Multiaddr) {
    let h = Handle::spawn(key, config).expect("a node starts");
    let addr = h
        .listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .await
        .unwrap();
    (h, addr)
}

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

/// Three members (member 0 with the gateway, `members` of them started) and a relay bound
/// to them; member 0's circle feed caught up when another member is there.
struct Net {
    members: Vec<Handle>,
    member0: Multiaddr,
    relay: Handle,
    relay_addr: Multiaddr,
}

async fn net(root: &Path, members: usize) -> Net {
    let (m0, member0) = start(transport(0), member_config(0, &root.join("0"))).await;
    let mut handles = vec![m0];
    for i in 1..members {
        let (h, _) = start(
            transport(i as u8),
            member_config(i, &root.join(i.to_string())),
        )
        .await;
        h.dial(member0.clone()).await.unwrap();
        handles.push(h);
    }
    let (relay, relay_addr) = start(transport(RELAY_SEED), relay_config(None)).await;
    relay.dial(member0.clone()).await.unwrap();
    until("the relay holds member 0's binding", || async {
        relay.status().await.bound >= 1
    })
    .await;
    if members > 1 {
        until("member 0's circle feed catches up", || async {
            handles[0].status().await.circle_caught_up
        })
        .await;
    }
    Net {
        members: handles,
        member0,
        relay,
        relay_addr,
    }
}

async fn circle_len(h: &Handle, i: usize) -> usize {
    h.replica().await.feed(&circle_key(i)).len()
}

/// Seals `event` to member `to`'s key, sends it through the relay at `relay`, opens the reply.
async fn send(relay: &Multiaddr, to: usize, event: &[u8]) -> Result<Answer, ClientError> {
    let sealed = seal(&tiers(), &announcements()[to], event)?;
    let reply = submit(relay.clone(), sealed.request.clone()).await?;
    sealed.open(&reply)
}

/// AT-NET-24: RFC 9458 Appendix A: HPKE open, exported secret, response key, nonce and reply.
#[test]
fn at_net_24_the_rfc_9458_example() {
    let sk = hex("3c168975674b2fa8e465970b79c8dcf09f1c741626480bd4c6162fc5b6a98e1a");
    let config = hex(
        "01002031e1f05a740102115220e9af918f738674aec95f54db6e04eb705aae8e79815500080001000100010003",
    );
    let request = hex("00034745540568747470730b6578616d706c652e636f6d012f");
    let info = hex("6d6573736167652f626874747020726571756573740001002000010001");
    let encapsulated = hex(
        "010020000100014b28f881333e7c164ffc499ad9796f877f4e1051ee6d31bad1
         9dec96c208b4726374e469135906992e1268c594d2a10c695d858c40a026e796
         5e7d86b83dd440b2c0185204b4d63525",
    );
    let response = hex("0140c8");
    let encapsulated_response =
        hex("c789e7151fcba46158ca84b04464910d86f9013e404feea014e7be4a441f234f857fbd");
    type Kem = hpke::kem::X25519HkdfSha256;
    let secret = <Kem as hpke::Kem>::PrivateKey::from_bytes(&sk).unwrap();
    let enc = &encapsulated[7..39];
    let encapped = <Kem as hpke::Kem>::EncappedKey::from_bytes(enc).unwrap();
    let mut ctx = hpke::setup_receiver::<hpke::aead::AesGcm128, hpke::kdf::HkdfSha256, Kem>(
        &hpke::OpModeR::Base,
        &secret,
        &encapped,
        &info,
    )
    .unwrap();
    assert_eq!(ctx.open(&encapsulated[39..], &[]).unwrap(), request);
    let mut exported = [0u8; 16];
    ctx.export(b"message/bhttp response", &mut exported)
        .unwrap();
    assert_eq!(exported.to_vec(), hex("62d87a6ba569ee81014c2641f52bea36"));
    let salt = [enc, &encapsulated_response[..16]].concat();
    let (prk, kdf) = hkdf::Hkdf::<sha2::Sha256>::extract(Some(&salt), &exported);
    assert_eq!(
        prk.to_vec(),
        hex("979aaeae066cf211ab407b31ae49767f344e1501e475c84e8aff547cc5a683db")
    );
    let (mut key, mut nonce) = ([0u8; 16], [0u8; 12]);
    kdf.expand(b"key", &mut key).unwrap();
    kdf.expand(b"nonce", &mut nonce).unwrap();
    assert_eq!(key.to_vec(), hex("5d0172a080e428b16d298c4ea0db620d"));
    assert_eq!(nonce.to_vec(), hex("f6bf1aeb88d6df87007fa263"));
    let aead = aes_gcm::Aes128Gcm::new_from_slice(&key).unwrap();
    let nonce = aes_gcm::Nonce::<aes_gcm::aead::consts::U12>::try_from(&nonce[..]).unwrap();
    let opened = aead.decrypt(&nonce, &encapsulated_response[16..]).unwrap();
    assert_eq!(opened, response);
    assert!(ohttp::KeyConfig::decode(&config).is_ok());
    let client = ohttp::ClientRequest::from_encoded_config(&config).unwrap();
    let (ours, _) = client.encapsulate(&request).unwrap();
    assert_eq!(
        (ours[..7].to_vec(), ours.len()),
        (encapsulated[..7].to_vec(), encapsulated.len())
    );
    let m = Http::read_bhttp(&mut std::io::Cursor::new(&request[..])).unwrap();
    let c = m.control();
    let parts = (c.method(), c.scheme(), c.authority(), c.path());
    let expected: (&[u8], &[u8], &[u8], &[u8]) = (b"GET", b"https", b"example.com", b"/");
    assert_eq!(
        parts,
        (
            Some(expected.0),
            Some(expected.1),
            Some(expected.2),
            Some(expected.3)
        )
    );
    let theirs = Http::read_bhttp(&mut std::io::Cursor::new(&response[..])).unwrap();
    assert_eq!(theirs.control().status().map(|s| s.code()), Some(200));
    assert!(theirs.content().is_empty());
    let mut ok = Vec::new();
    Http::response(bhttp::StatusCode::OK)
        .write_bhttp(Mode::KnownLength, &mut ok)
        .unwrap();
    assert_eq!(ok, [&response[..], &[0, 0, 0]].concat());
}

/// AT-NET-24: a key configuration is used only if its member signed it for this network.
#[test]
fn at_net_24_key_configurations_verify_only_as_signed() {
    let t = tiers();
    let good = announcements()[0].clone();
    assert!(good.verify(&t));
    assert_eq!(KeyAnnouncement::decode(&good.encode()), Some(good.clone()));
    let ms = members();
    let mut bad = vec![
        KeyAnnouncement {
            member: 1,
            ..good.clone()
        },
        KeyAnnouncement {
            signature: t.sign_key(&ms[1], 0, &good.config),
            ..good.clone()
        },
        KeyAnnouncement {
            signature: t.sign_key(&Member::from_seed([7; 32]), 0, &good.config),
            ..good.clone()
        },
        KeyAnnouncement {
            signature: tiers_on([45; 32]).sign_key(&ms[0], 0, &good.config),
            ..good.clone()
        },
    ];
    for at in 0..good.config.len() {
        let mut config = good.config.clone();
        config[at] ^= 1;
        bad.push(KeyAnnouncement {
            config,
            ..good.clone()
        });
    }
    for b in &bad {
        assert!(!b.verify(&t), "{b:?}");
        assert!(matches!(
            seal(&t, b, b"synthetic-event"),
            Err(ClientError::Unverified)
        ));
    }
    assert!(!good.verify(&tiers_on([45; 32])));
    for at in 0..good.encode().len() {
        assert_eq!(KeyAnnouncement::decode(&good.encode()[..at]), None);
    }
}

/// AT-NET-24: a synthetic event reaches member 0's circle feed through the relay, receipted.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_net_24_an_event_reaches_the_circle_feed_through_a_relay() {
    let n = net(&scratch("path"), 2).await;
    let event = b"synthetic-event-path".to_vec();
    let answer = send(&n.relay_addr, 0, &event).await.unwrap();
    let Answer::Logged(id) = answer else {
        panic!("{answer:?}")
    };
    assert_eq!(id.writer, circle_key(0));
    let held = n.members[0].replica().await;
    assert_eq!(held.get(&id).map(|(_, o)| o.to_vec()), Some(event.clone()));
    until("member 1 holds it", || async {
        n.members[1].replica().await.contains(&id)
    })
    .await;
    assert!(n
        .relay
        .replica()
        .await
        .ids()
        .all(|i| i.writer != circle_key(0)));
    assert_eq!(
        send(&n.relay_addr, 0, b"not synthetic").await,
        Ok(Answer::NotAdmitted)
    );
    assert_eq!(circle_len(&n.members[0], 0).await, 1);
}

/// AT-NET-24: one event, sealed twice and sent together, then again after a restart, is
/// logged once and each time answered with the same receipt.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_net_24_duplicates_are_logged_once() {
    let root = scratch("duplicates");
    let n = net(&root, 2).await;
    let event = b"synthetic-event-twice".to_vec();
    let (a, b) = tokio::join!(
        send(&n.relay_addr, 0, &event),
        send(&n.relay_addr, 0, &event)
    );
    let (a, b) = (a.unwrap(), b.unwrap());
    assert!(matches!(a, Answer::Logged(_)));
    assert_eq!(a, b);
    assert_eq!(circle_len(&n.members[0], 0).await, 1);
    let Net {
        mut members, relay, ..
    } = n;
    drop(relay);
    drop(members.remove(0));
    until("member 0 has left", || async {
        members[0].status().await.peers == 0
    })
    .await;
    let n = {
        let (m0, member0) = start(transport(0), member_config(0, &root.join("0"))).await;
        members[0].dial(member0.clone()).await.unwrap();
        let (relay, relay_addr) = start(transport(RELAY_SEED), relay_config(None)).await;
        relay.dial(member0.clone()).await.unwrap();
        until("bound again", || async {
            relay.status().await.bound >= 1 && m0.status().await.circle_caught_up
        })
        .await;
        members.insert(0, m0);
        Net {
            members,
            member0,
            relay,
            relay_addr,
        }
    };
    assert_eq!(send(&n.relay_addr, 0, &event).await, Ok(a));
    assert_eq!(circle_len(&n.members[0], 0).await, 1);
}

/// AT-NET-24: before its circle feed caught up a member answers "retry later", logging nothing.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_net_24_retry_later_before_the_circle_feed_caught_up() {
    let n = net(&scratch("retry"), 1).await;
    assert!(!n.members[0].status().await.circle_caught_up);
    assert_eq!(
        send(&n.relay_addr, 0, b"synthetic-event-early").await,
        Ok(Answer::RetryLater)
    );
    assert_eq!(circle_len(&n.members[0], 0).await, 0);
}

/// AT-NET-24: another network's path, another member's key and oversized requests are refused.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_net_24_wrong_network_recipient_and_size_are_refused() {
    let n = net(&scratch("wrong"), 2).await;
    let mut m = Http::request(
        b"POST".to_vec(),
        b"isegoria".to_vec(),
        Vec::new(),
        p2p::blind::event_path(&[45; 32]),
    );
    m.write_content(b"synthetic-event-elsewhere");
    let mut inner = Vec::new();
    m.write_bhttp(Mode::KnownLength, &mut inner).unwrap();
    let client = ohttp::ClientRequest::from_encoded_config(&announcements()[0].config).unwrap();
    let (request, response) = client.encapsulate(&inner).unwrap();
    let Ok(Carried::Sealed(reply)) = submit(n.relay_addr.clone(), request).await else {
        panic!("a sealed reply")
    };
    let plain = response.decapsulate(&reply).unwrap();
    let status = Http::read_bhttp(&mut std::io::Cursor::new(&plain[..])).unwrap();
    assert_eq!(status.control().status().map(|s| s.code()), Some(421));
    assert_eq!(
        send(&n.relay_addr, 1, b"synthetic-event-to-1").await,
        Err(ClientError::Refused(Refusal::Unreachable))
    );
    let big = vec![b's'; MAX_REQUEST];
    assert!(matches!(
        seal(&tiers(), &announcements()[0], &big),
        Err(ClientError::TooLarge)
    ));
    assert!(matches!(
        submit(n.relay_addr.clone(), vec![0; MAX_REQUEST + 1]).await,
        Err(ClientError::Transport(_))
    ));
    assert_eq!(circle_len(&n.members[0], 0).await, 0);
}

/// AT-NET-24: a relay holds no feed: one given a member's key or its own refuses to start,
/// opening no file, and every replica refuses an entry its key signs.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_net_24_a_relay_cannot_write() {
    let relay_writer = FeedWriter::from_seed(NET, [RELAY_SEED; 32]);
    for seed in [MEMBER_SEEDS[0], RELAY_SEED] {
        let dir = scratch("relay-own");
        let own = Own {
            writer: FeedWriter::from_seed(NET, [seed; 32]),
            dir: dir.clone(),
        };
        let refused = Handle::spawn(transport(RELAY_SEED), relay_config(Some(own)));
        assert!(
            matches!(refused.err(), Some(StartError::Relays)),
            "seed {seed}"
        );
        assert!(!dir.exists(), "seed {seed}: no file opened");
    }
    let circle = Own {
        writer: FeedWriter::from_seed(NET, [CIRCLE_SEEDS[0]; 32]),
        dir: scratch("relay-circle"),
    };
    let dir = circle.dir.clone();
    let config = Config {
        circle: Some(circle),
        ..relay_config(None)
    };
    assert!(Handle::spawn(transport(RELAY_SEED), config).is_err());
    assert!(!dir.exists());
    assert!(Handle::spawn(transport(RELAY_SEED), relay_config(None)).is_ok());
    let object = b"a relay's entry".to_vec();
    let entry = TransparencyLog::new().append(cid(&object)).clone();
    let mut replica = Replica::new(tiers().writers().clone());
    assert_eq!(
        replica.insert(relay_writer.sign(&entry), object),
        Err(Refused::UnknownWriter)
    );
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
        read_limited(io, 4 << 20).await
    }

    async fn read_response<T>(&mut self, _: &StreamProtocol, io: &mut T) -> std::io::Result<Vec<u8>>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        read_limited(io, 4 << 20).await
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
struct Relaying {
    relay: request_response::Behaviour<Bytes>,
    gateway: request_response::Behaviour<Bytes>,
}

/// What an instrumented relay does to what it forwards.
#[derive(Clone, Copy, PartialEq)]
enum Tamper {
    None,
    Request(usize),
    Reply(usize),
    /// Holds the first reply until the second, then answers each request with the other's.
    Swap,
    /// Answers each request itself with a sealed reply of this many zero bytes.
    Answer(usize),
}

/// A relay of the test's own, listed by member 0's gateway: it forwards every request to
/// member 0, keeps every byte it receives, and tampers as told; or sends directly.
struct Spy {
    seen: Arc<Mutex<Vec<Vec<u8>>>>,
    addr: Multiaddr,
    direct: mpsc::UnboundedSender<(Vec<u8>, tokio::sync::oneshot::Sender<Vec<u8>>)>,
}

impl Spy {
    async fn start(key: Keypair, member0: Multiaddr, tamper: Tamper) -> Spy {
        let protocol = |name| [(StreamProtocol::new(name), ProtocolSupport::Full)];
        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(key)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .unwrap()
            .with_behaviour(|_| Relaying {
                relay: request_response::Behaviour::new(
                    protocol(RELAY_PROTOCOL),
                    Default::default(),
                ),
                gateway: request_response::Behaviour::new(
                    protocol(GATEWAY_PROTOCOL),
                    Default::default(),
                ),
            })
            .unwrap()
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();
        swarm
            .listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
            .unwrap();
        swarm.dial(member0).unwrap();
        let (addr_tx, addr_rx) = tokio::sync::oneshot::channel();
        let (direct, mut orders) =
            mpsc::unbounded_channel::<(Vec<u8>, tokio::sync::oneshot::Sender<Vec<u8>>)>();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let kept = seen.clone();
        tokio::spawn(async move {
            let (mut addr_tx, mut member, mut queued) = (Some(addr_tx), None::<PeerId>, Vec::new());
            let mut waiting: HashMap<
                request_response::OutboundRequestId,
                Option<ResponseChannel<Vec<u8>>>,
            > = HashMap::new();
            let mut answers: HashMap<
                request_response::OutboundRequestId,
                tokio::sync::oneshot::Sender<Vec<u8>>,
            > = HashMap::new();
            let mut held: Vec<(ResponseChannel<Vec<u8>>, Vec<u8>)> = Vec::new();
            let mut early = Vec::new();
            let send = |swarm: &mut libp2p::Swarm<Relaying>, member: PeerId, mut bytes: Vec<u8>| {
                if let Tamper::Request(at) = tamper {
                    bytes[at] ^= 1;
                }
                swarm.behaviour_mut().gateway.send_request(&member, bytes)
            };
            loop {
                tokio::select! {
                    order = orders.recv() => {
                        let Some((bytes, tx)) = order else { return };
                        match member {
                            Some(m) => {
                                let id = swarm.behaviour_mut().gateway.send_request(&m, bytes);
                                answers.insert(id, tx);
                            }
                            None => early.push((bytes, tx)),
                        }
                    }
                    event = swarm.select_next_some() => match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            if let Some(tx) = addr_tx.take() { let _ = tx.send(address); }
                        }
                        SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } if endpoint.is_dialer() => {
                            member = Some(peer_id);
                            for (bytes, tx) in early.drain(..) {
                                let id = swarm.behaviour_mut().gateway.send_request(&peer_id, bytes);
                                answers.insert(id, tx);
                            }
                            for (bytes, channel) in queued.drain(..) {
                                let id = send(&mut swarm, peer_id, bytes);
                                waiting.insert(id, Some(channel));
                            }
                        }
                        SwarmEvent::Behaviour(RelayingEvent::Relay(request_response::Event::Message {
                            message: request_response::Message::Request { request, channel, .. }, ..
                        })) => {
                            kept.lock().unwrap().push(request.clone());
                            if let Tamper::Answer(len) = tamper {
                                let reply = Carried::Sealed(vec![0; len]).encode();
                                let _ = swarm.behaviour_mut().relay.send_response(channel, reply);
                                continue;
                            }
                            match member {
                                Some(m) => { let id = send(&mut swarm, m, request); waiting.insert(id, Some(channel)); }
                                None => queued.push((request, channel)),
                            }
                        }
                        SwarmEvent::Behaviour(RelayingEvent::Gateway(request_response::Event::Message {
                            message: request_response::Message::Response { request_id, mut response }, ..
                        })) => {
                            kept.lock().unwrap().push(response.clone());
                            if let Some(tx) = answers.remove(&request_id) {
                                let _ = tx.send(response);
                                continue;
                            }
                            let Some(Some(channel)) = waiting.remove(&request_id) else { continue };
                            if let Tamper::Reply(at) = tamper {
                                let n = response.len();
                                response[n - 1 - at] ^= 1;
                            }
                            if tamper == Tamper::Swap {
                                held.push((channel, response));
                                if held.len() == 2 {
                                    let (b, a) = (held.pop().unwrap(), held.pop().unwrap());
                                    let _ = swarm.behaviour_mut().relay.send_response(a.0, b.1);
                                    let _ = swarm.behaviour_mut().relay.send_response(b.0, a.1);
                                }
                                continue;
                            }
                            let _ = swarm.behaviour_mut().relay.send_response(channel, response);
                        }
                        _ => {}
                    },
                }
            }
        });
        let addr = addr_rx.await.unwrap();
        Spy { seen, addr, direct }
    }

    /// Sends `request` straight to member 0's gateway, as a relay would.
    async fn direct(&self, request: Vec<u8>) -> Carried {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.direct.send((request, tx)).unwrap();
        let reply = tokio::time::timeout(Duration::from_secs(30), rx).await;
        Carried::decode(&reply.expect("a gateway reply").unwrap()).unwrap()
    }
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}

/// AT-NET-24: an instrumented relay sees nothing of the event, its path or the receipt.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_net_24_the_relay_sees_no_plaintext() {
    let n = net(&scratch("plaintext"), 2).await;
    let spy = Spy::start(transport(SPY_SEED), n.member0.clone(), Tamper::None).await;
    let event = b"synthetic-event-secret-content".to_vec();
    let Ok(Answer::Logged(id)) = send(&spy.addr, 0, &event).await else {
        panic!("a receipt")
    };
    let mut receipt = id.writer.to_vec();
    receipt.extend_from_slice(&id.seq.to_le_bytes());
    receipt.extend_from_slice(&id.hash);
    let seen = spy.seen.lock().unwrap().clone();
    assert_eq!(seen.len(), 2);
    for bytes in &seen {
        for secret in [
            &event[..],
            b"/isegoria/v1",
            b"synthetic",
            &receipt,
            &id.hash,
        ] {
            assert!(!contains(bytes, secret));
        }
    }
}

/// AT-NET-24: a request or reply with a byte changed, or a reply swapped between two
/// requests, is refused; nothing is logged for the refused requests.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_net_24_alterations_and_swapped_replies_are_refused() {
    let n = net(&scratch("alter"), 2).await;
    let cases = [
        (
            Tamper::Request(0),
            Err(ClientError::Refused(Refusal::UnknownKey)),
        ),
        (
            Tamper::Request(10),
            Err(ClientError::Refused(Refusal::Undecryptable)),
        ),
        (
            Tamper::Request(60),
            Err(ClientError::Refused(Refusal::Undecryptable)),
        ),
        (Tamper::Reply(0), Err(ClientError::Open)),
        (Tamper::Reply(20), Err(ClientError::Open)),
    ];
    let mut logged = 0;
    for (k, (tamper, expected)) in cases.into_iter().enumerate() {
        let spy = Spy::start(transport(SPY_SEED), n.member0.clone(), tamper).await;
        let event = format!("synthetic-event-altered-{k}").into_bytes();
        assert_eq!(send(&spy.addr, 0, &event).await, expected, "{k}");
        if matches!(tamper, Tamper::Reply(_)) {
            logged += 1;
        }
        assert_eq!(circle_len(&n.members[0], 0).await, logged, "{k}");
    }
    let spy = Spy::start(transport(SPY_SEED), n.member0.clone(), Tamper::Swap).await;
    let (a, b) = tokio::join!(
        send(&spy.addr, 0, b"synthetic-event-swap-a"),
        send(&spy.addr, 0, b"synthetic-event-swap-b")
    );
    assert_eq!((a, b), (Err(ClientError::Open), Err(ClientError::Open)));
}

/// AT-NET-24: member 0's gateway answers only listed relays, before decapsulating.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_net_24_the_gateway_answers_only_listed_relays() {
    let n = net(&scratch("listed"), 2).await;
    let sealed = seal(&tiers(), &announcements()[0], b"synthetic-event-direct").unwrap();
    let outsider = Spy::start(Keypair::generate_ed25519(), n.member0.clone(), Tamper::None).await;
    assert_eq!(
        outsider.direct(sealed.request.clone()).await,
        Carried::Refused(Refusal::NotARelay)
    );
    let listed = Spy::start(transport(SPY_SEED), n.member0.clone(), Tamper::None).await;
    let reply = listed.direct(sealed.request.clone()).await;
    assert!(matches!(sealed.open(&reply), Ok(Answer::Logged(_))));
    let other = seal(&tiers(), &announcements()[1], b"synthetic-event-to-1").unwrap();
    assert_eq!(
        listed.direct(other.request.clone()).await,
        Carried::Refused(Refusal::UnknownKey)
    );
    assert_eq!(circle_len(&n.members[0], 0).await, 1);
}

proptest::proptest! {
    /// AT-NET-24: carried replies and key configurations decode only as their one encoding.
    #[test]
    fn at_net_24_carried_and_announced_bytes_have_one_encoding(bytes in proptest::collection::vec(proptest::prelude::any::<u8>(), 0..120)) {
        for c in [Carried::Sealed(bytes.clone()), Carried::Refused(Refusal::Unreachable)] {
            proptest::prop_assert_eq!(Carried::decode(&c.encode()), Some(c));
        }
        if let Some(c) = Carried::decode(&bytes) {
            proptest::prop_assert_eq!(c.encode(), bytes.clone());
        }
        if let Some(a) = KeyAnnouncement::decode(&bytes) {
            proptest::prop_assert_eq!(a.encode(), bytes);
        }
    }
}

#[derive(NetworkBehaviour)]
struct Gatewaying {
    bind: request_response::Behaviour<Bytes>,
    gateway: request_response::Behaviour<Bytes>,
}

/// A gateway of the test's own, bound as member 0 to the relay it dials, answering every
/// request with a sealed reply of `len` zero bytes; it stops when dropped.
struct FakeGateway(tokio::task::JoinHandle<()>);

impl Drop for FakeGateway {
    fn drop(&mut self) {
        self.0.abort();
    }
}

impl FakeGateway {
    fn start(len: usize, relay: Multiaddr) -> FakeGateway {
        let key = Keypair::generate_ed25519();
        let mine = key.public().try_into_ed25519().unwrap().to_bytes();
        let binding = Some(tiers().bind(&members()[0], 0, mine));
        let protocol = |name| [(StreamProtocol::new(name), ProtocolSupport::Full)];
        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(key)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .unwrap()
            .with_behaviour(|_| Gatewaying {
                bind: request_response::Behaviour::new(
                    protocol(p2p::BIND_PROTOCOL),
                    Default::default(),
                ),
                gateway: request_response::Behaviour::new(
                    protocol(GATEWAY_PROTOCOL),
                    Default::default(),
                ),
            })
            .unwrap()
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();
        swarm.dial(relay).unwrap();
        FakeGateway(tokio::spawn(async move {
            loop {
                match swarm.select_next_some().await {
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        let hello = p2p::tiered::Bind::Hello(binding).encode();
                        swarm.behaviour_mut().bind.send_request(&peer_id, hello);
                    }
                    SwarmEvent::Behaviour(GatewayingEvent::Bind(
                        request_response::Event::Message {
                            message: request_response::Message::Request { channel, .. },
                            ..
                        },
                    )) => {
                        let reply = p2p::tiered::Bind::Reply {
                            granted: false,
                            binding,
                        }
                        .encode();
                        let _ = swarm.behaviour_mut().bind.send_response(channel, reply);
                    }
                    SwarmEvent::Behaviour(GatewayingEvent::Gateway(
                        request_response::Event::Message {
                            message: request_response::Message::Request { channel, .. },
                            ..
                        },
                    )) => {
                        let reply = Carried::Sealed(vec![0; len]).encode();
                        let _ = swarm.behaviour_mut().gateway.send_response(channel, reply);
                    }
                    _ => {}
                }
            }
        }))
    }
}

/// AT-NET-24: the reply limit counts the encapsulation, MAX_REPLY bytes, not its envelope.
#[test]
fn at_net_24_the_reply_limit_counts_the_encapsulation() {
    assert_eq!(
        Carried::Sealed(vec![0; MAX_REPLY]).encode().len(),
        MAX_REPLY + 10
    );
    let fresh = || seal(&tiers(), &announcements()[0], b"synthetic-event-limit").unwrap();
    assert_eq!(
        fresh().open(&Carried::Sealed(vec![0; MAX_REPLY + 1])),
        Err(ClientError::TooLarge)
    );
    assert_eq!(
        fresh().open(&Carried::Sealed(vec![0; MAX_REPLY])),
        Err(ClientError::Open)
    );
}

/// AT-NET-24: a reply over MAX_REPLY bytes is refused by the relay and by the client.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_net_24_oversized_replies_are_refused() {
    let (relay, relay_addr) = start(transport(RELAY_SEED), relay_config(None)).await;
    let request = || {
        seal(&tiers(), &announcements()[0], b"synthetic-event-big")
            .unwrap()
            .request
    };
    for (len, passes) in [
        (MAX_REPLY, true),
        (MAX_REPLY + 1, false),
        (MAX_REPLY + 6, false),
    ] {
        let fake = FakeGateway::start(len, relay_addr.clone());
        until("the fake gateway is bound", || async {
            relay.status().await.bound == 1
        })
        .await;
        let expected = match passes {
            true => Carried::Sealed(vec![0; len]),
            false => Carried::Refused(Refusal::Unreachable),
        };
        assert_eq!(
            submit(relay_addr.clone(), request()).await,
            Ok(expected),
            "relay, {len}"
        );
        drop(fake);
        until("the fake gateway leaves", || async {
            relay.status().await.bound == 0
        })
        .await;
    }
    for (len, passes) in [
        (MAX_REPLY, true),
        (MAX_REPLY + 1, false),
        (MAX_REPLY + 6, false),
    ] {
        let spy = Spy::start(transport(SPY_SEED), relay_addr.clone(), Tamper::Answer(len)).await;
        let got = submit(spy.addr.clone(), request()).await;
        match passes {
            true => assert_eq!(got, Ok(Carried::Sealed(vec![0; len])), "client, {len}"),
            false => assert!(
                matches!(got, Err(ClientError::Transport(_))),
                "client, {len}: {got:?}"
            ),
        }
    }
}

type ReferenceKem = hpke::kem::X25519HkdfSha256;
type ReferenceAead = hpke::aead::AesGcm128;
type ReferenceKdf = hpke::kdf::HkdfSha256;
const HEADER: [u8; 7] = [0, 0, 0x20, 0, 1, 0, 1];

/// RFC 9458 §4.3's HPKE info for a request with `header`.
fn request_info(header: &[u8]) -> Vec<u8> {
    [&b"message/bhttp request"[..], &[0], header].concat()
}

/// RFC 9458 §4.4's reply key and nonce from the exported secret, `enc` and the reply's nonce.
fn reply_keys(secret: &[u8], enc: &[u8], nonce: &[u8]) -> (aes_gcm::Aes128Gcm, [u8; 12]) {
    let salt = [enc, nonce].concat();
    let (_, kdf) = hkdf::Hkdf::<sha2::Sha256>::extract(Some(&salt), secret);
    let (mut key, mut iv) = ([0u8; 16], [0u8; 12]);
    kdf.expand(b"key", &mut key).unwrap();
    kdf.expand(b"nonce", &mut iv).unwrap();
    (aes_gcm::Aes128Gcm::new_from_slice(&key).unwrap(), iv)
}

fn nonce12(iv: &[u8; 12]) -> aes_gcm::Nonce<aes_gcm::aead::consts::U12> {
    aes_gcm::Nonce::<aes_gcm::aead::consts::U12>::try_from(&iv[..]).unwrap()
}

fn event_request(event: &[u8]) -> Vec<u8> {
    let path = p2p::blind::event_path(&NET);
    let mut m = Http::request(b"POST".to_vec(), b"isegoria".to_vec(), Vec::new(), path);
    m.write_content(event);
    let mut out = Vec::new();
    m.write_bhttp(Mode::KnownLength, &mut out).unwrap();
    out
}

fn receipt_of(plain: &[u8]) -> EntryId {
    let m = Http::read_bhttp(&mut std::io::Cursor::new(plain)).unwrap();
    assert_eq!(m.control().status().map(|s| s.code()), Some(200));
    let b = m.content();
    EntryId {
        writer: b[..32].try_into().unwrap(),
        seq: u64::from_le_bytes(b[32..40].try_into().unwrap()),
        hash: b[40..72].try_into().unwrap(),
    }
}

/// AT-NET-24: what `seal` sends opens with the reference HPKE path, and `open` reads a reply
/// the reference path seals: the client's OHTTP against RFC 9458 §4.3–4.4.
#[test]
fn at_net_24_the_client_matches_the_reference_hpke() {
    use hpke::{Deserializable, Kem as _, Serializable};
    let (sk, pk) = ReferenceKem::derive_keypair(&[HPKE_SEEDS[0]; 32]);
    let key = announcements()[0].clone();
    assert_eq!(pk.to_bytes().to_vec(), key.config[3..35].to_vec());
    let event = b"synthetic-event-reference".to_vec();
    let sealed = seal(&tiers(), &key, &event).unwrap();
    let request = sealed.request.clone();
    let (header, enc, ct) = (&request[..7], &request[7..39], &request[39..]);
    assert_eq!(header, HEADER);
    let encapped = <ReferenceKem as hpke::Kem>::EncappedKey::from_bytes(enc).unwrap();
    let mut receiver = hpke::setup_receiver::<ReferenceAead, ReferenceKdf, ReferenceKem>(
        &hpke::OpModeR::Base,
        &sk,
        &encapped,
        &request_info(header),
    )
    .unwrap();
    assert_eq!(receiver.open(ct, &[]).unwrap(), event_request(&event));
    let mut secret = [0u8; 16];
    receiver
        .export(b"message/bhttp response", &mut secret)
        .unwrap();
    let nonce = [7u8; 16];
    let (aead, iv) = reply_keys(&secret, enc, &nonce);
    let receipt = EntryId {
        writer: circle_key(0),
        seq: 5,
        hash: [9; 32],
    };
    let mut answer = Http::response(bhttp::StatusCode::OK);
    answer.write_content([&receipt.writer[..], &5u64.to_le_bytes(), &[9; 32]].concat());
    let mut plain = Vec::new();
    answer.write_bhttp(Mode::KnownLength, &mut plain).unwrap();
    let reply = [
        &nonce[..],
        &aead.encrypt(&nonce12(&iv), &plain[..]).unwrap(),
    ]
    .concat();
    assert_eq!(
        sealed.open(&Carried::Sealed(reply)),
        Ok(Answer::Logged(receipt))
    );
}

/// AT-NET-24: the gateway opens a request the reference HPKE path seals, and seals its answer
/// so the reference path opens it: the server's OHTTP against RFC 9458 §4.3–4.4.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_net_24_the_gateway_matches_the_reference_hpke() {
    use hpke::{Kem as _, Serializable};
    let n = net(&scratch("reference"), 2).await;
    let relay = Spy::start(transport(SPY_SEED), n.member0.clone(), Tamper::None).await;
    let (_, pk) = ReferenceKem::derive_keypair(&[HPKE_SEEDS[0]; 32]);
    let (encapped, mut sender) = hpke::setup_sender::<ReferenceAead, ReferenceKdf, ReferenceKem>(
        &hpke::OpModeS::Base,
        &pk,
        &request_info(&HEADER),
    )
    .unwrap();
    let event = b"synthetic-event-from-the-reference".to_vec();
    let ct = sender.seal(&event_request(&event), &[]).unwrap();
    let enc = encapped.to_bytes().to_vec();
    let request = [&HEADER[..], &enc, &ct].concat();
    let Carried::Sealed(reply) = relay.direct(request).await else {
        panic!("a sealed reply")
    };
    let mut secret = [0u8; 16];
    sender
        .export(b"message/bhttp response", &mut secret)
        .unwrap();
    let (aead, iv) = reply_keys(&secret, &enc, &reply[..16]);
    let plain = aead.decrypt(&nonce12(&iv), &reply[16..]).unwrap();
    let id = receipt_of(&plain);
    assert_eq!(id.writer, circle_key(0));
    let held = n.members[0].replica().await;
    assert_eq!(held.get(&id).map(|(_, o)| o.to_vec()), Some(event));
}
