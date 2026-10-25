//! Public object discovery (`docs/04` §The four P2P structures, AT-NET-25).

use libp2p::identity::Keypair;
use network::cid::cid;
use network::consortium::{Consortium, Member};
use network::log::TransparencyLog;
use network::replica::{FeedWriter, SignedEntry};
use network::tier::Tiers;
use p2p::{discovery, Config, Handle};
use std::time::Duration;

const NET: [u8; 32] = [45; 32];
fn key(seed: u8) -> Keypair {
    Keypair::ed25519_from_bytes([seed; 32]).unwrap()
}
fn tiers() -> Tiers {
    Tiers::new(
        NET,
        Consortium::new(vec![Member::from_seed([1; 32]).public()], 1),
        &[FeedWriter::from_seed(NET, [2; 32]).public()],
    )
    .unwrap()
}
fn config(tiered: bool) -> Config {
    Config {
        writers: tiers().writers().clone(),
        sync_every: Duration::from_secs(3600),
        own: None,
        circle: None,
        store: None,
        member: None,
        consortium: None,
        tiers: tiered.then(tiers),
        relay: None,
        gateway: None,
    }
}
fn entry(seed: u8, object: &[u8]) -> SignedEntry {
    FeedWriter::from_seed(NET, [seed; 32]).sign(TransparencyLog::new().append(cid(object)))
}

// An empty bootstrap locates an unknown provider for a replica-free client (AT-NET-25).
#[tokio::test]
async fn unknown_provider_and_circle_refusal() {
    let bootstrap = fixture(10, NET, None, vec![], None, false).await;
    let addr = bootstrap.addr.clone();
    let provider = Handle::spawn(key(11), config(true)).unwrap();
    let provider_addr = provider
        .listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .await
        .unwrap();
    let public = b"public discovery".to_vec();
    provider
        .insert(entry(2, &public), public.clone())
        .await
        .unwrap();
    let signed = entry(1, &public);
    provider
        .insert(signed.clone(), public.clone())
        .await
        .unwrap();
    provider
        .insert(entry(2, b"circle discovery"), b"circle discovery".to_vec())
        .await
        .unwrap();
    provider.dial(addr.clone()).await.unwrap();
    tokio::time::sleep(Duration::from_millis(600)).await;
    let found = discovery::fetch(
        key(12),
        &tiers(),
        vec![addr.clone()],
        cid(&public),
        Duration::from_secs(10),
    )
    .await
    .unwrap();
    assert!(bootstrap
        .advertised
        .lock()
        .unwrap()
        .contains_key(&cid(&public).0.to_vec()));
    assert!(!bootstrap
        .advertised
        .lock()
        .unwrap()
        .contains_key(&cid(b"circle discovery").0.to_vec()));
    assert_eq!(found.provider, provider.peer_id());
    assert_eq!(found.entry, signed);
    assert_eq!(found.object, public);
    assert_eq!(
        raw(provider_addr, NET, cid(b"circle discovery")).await,
        vec![1, 0]
    );
    let refused = discovery::fetch(
        key(13),
        &tiers(),
        vec![addr],
        cid(b"circle discovery"),
        Duration::from_secs(5),
    )
    .await;
    assert!(matches!(
        refused,
        Err(discovery::FetchError::NotFound { .. })
    ));
}

// A light reader requires an explicit bootstrap (AT-NET-25).
#[tokio::test]
async fn requires_bootstrap() {
    assert!(matches!(
        discovery::fetch(key(14), &tiers(), vec![], cid(b"x"), Duration::from_secs(1)).await,
        Err(discovery::FetchError::NoBootstrap)
    ));
}

use futures::StreamExt;
use libp2p::kad::{self, store::MemoryStore, RecordKey};
use libp2p::request_response::{self, ProtocolSupport};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{noise, tcp, yamux, Multiaddr, PeerId};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(NetworkBehaviour)]
struct FixtureBehaviour {
    kad: kad::Behaviour<MemoryStore>,
    object: request_response::Behaviour<discovery::ObjectCodec>,
}
type Advertised =
    Arc<std::sync::Mutex<std::collections::HashMap<Vec<u8>, std::collections::HashSet<PeerId>>>>;

struct Fixture {
    addr: Multiaddr,
    peer: PeerId,
    requests: Arc<AtomicUsize>,
    task: tokio::task::JoinHandle<()>,
    advertised: Advertised,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.task.abort();
    }
}
async fn fixture(
    seed: u8,
    net: [u8; 32],
    bootstrap: Option<Multiaddr>,
    advertise: Vec<network::cid::Cid>,
    answer: Option<Vec<u8>>,
    silent: bool,
) -> Fixture {
    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(key(seed))
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .unwrap()
        .with_behaviour(|key| {
            let peer = key.public().to_peer_id();
            let mut config = kad::Config::new(discovery::kad_protocol(&net));
            config.set_query_timeout(Duration::from_secs(3));
            config.set_record_filtering(kad::StoreInserts::FilterBoth);
            let mut kad = kad::Behaviour::with_config(peer, MemoryStore::new(peer), config);
            kad.set_mode(Some(kad::Mode::Server));
            FixtureBehaviour {
                kad,
                object: discovery::object_behaviour(&net),
            }
        })
        .unwrap()
        .build();
    let peer = *swarm.local_peer_id();
    swarm
        .listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .unwrap();
    let addr = loop {
        if let SwarmEvent::NewListenAddr { address, .. } = swarm.select_next_some().await {
            swarm.add_external_address(address.clone());
            break address;
        }
    };
    if let Some(addr) = bootstrap {
        swarm.dial(addr).unwrap();
    }
    let requests = Arc::new(AtomicUsize::new(0));
    let count = requests.clone();
    let advertised = Arc::new(std::sync::Mutex::new(std::collections::HashMap::<
        Vec<u8>,
        std::collections::HashSet<PeerId>,
    >::new()));
    let received = advertised.clone();
    let task = tokio::spawn(async move {
        let mut silent_channels = Vec::new();
        loop {
            match swarm.select_next_some().await {
                SwarmEvent::Behaviour(FixtureBehaviourEvent::Kad(kad::Event::InboundRequest {
                    request:
                        kad::InboundRequest::AddProvider {
                            record: Some(record),
                        },
                })) => {
                    use kad::store::RecordStore;
                    received
                        .lock()
                        .unwrap()
                        .entry(record.key.to_vec())
                        .or_default()
                        .insert(record.provider);
                    swarm
                        .behaviour_mut()
                        .kad
                        .store_mut()
                        .add_provider(record)
                        .unwrap();
                }
                SwarmEvent::ConnectionEstablished {
                    peer_id, endpoint, ..
                } => {
                    if let libp2p::core::ConnectedPoint::Dialer { address, .. } = endpoint {
                        swarm.behaviour_mut().kad.add_address(&peer_id, address);
                    }
                    for cid in &advertise {
                        let _ = swarm
                            .behaviour_mut()
                            .kad
                            .start_providing(RecordKey::new(&cid.0));
                    }
                }
                SwarmEvent::Behaviour(FixtureBehaviourEvent::Object(
                    request_response::Event::Message {
                        message: request_response::Message::Request { channel, .. },
                        ..
                    },
                )) => {
                    count.fetch_add(1, Ordering::SeqCst);
                    if silent {
                        silent_channels.push(channel);
                    } else {
                        let _ = swarm
                            .behaviour_mut()
                            .object
                            .send_response(channel, answer.clone().unwrap_or_else(|| vec![1, 0]));
                    }
                }
                _ => {}
            }
        }
    });
    Fixture {
        addr,
        peer,
        requests,
        task,
        advertised,
    }
}
fn response(signed: &SignedEntry, object: &[u8]) -> Vec<u8> {
    let mut bytes = vec![1, 1];
    bytes.extend(signed.encode());
    bytes.extend(object);
    bytes
}
async fn raw(addr: Multiaddr, net: [u8; 32], cid: network::cid::Cid) -> Vec<u8> {
    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(key(100))
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .unwrap()
        .with_behaviour(|_| {
            request_response::Behaviour::<discovery::ObjectCodec>::new(
                [(discovery::object_protocol(&net), ProtocolSupport::Full)],
                Default::default(),
            )
        })
        .unwrap()
        .build();
    swarm.dial(addr).unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match swarm.select_next_some().await {
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    let mut req = vec![1];
                    req.extend(cid.0);
                    swarm.behaviour_mut().send_request(&peer_id, req);
                }
                SwarmEvent::Behaviour(request_response::Event::Message {
                    message: request_response::Message::Response { response, .. },
                    ..
                }) => return response,
                other => {
                    let _ = other;
                }
            }
        }
    })
    .await
    .unwrap()
}

// Untrusted bytes, writer tiers and network signatures never authorize a response (AT-NET-25).
#[tokio::test]
async fn rejects_untrusted_providers() {
    let object = b"checked bytes";
    let signed = entry(1, object);
    let mut bad_signature = signed.clone();
    bad_signature.signature[0] ^= 1;
    let wrong_net =
        FeedWriter::from_seed([99; 32], [1; 32]).sign(TransparencyLog::new().append(cid(object)));
    let cases = [
        response(&signed, b"changed bytes"),
        response(&bad_signature, object),
        response(&entry(2, object), object),
        response(&wrong_net, object),
        response(&entry(9, object), object),
        response(&entry(1, b"different CID"), b"different CID"),
    ];
    for (i, answer) in cases.into_iter().enumerate() {
        let bootstrap = fixture(30, NET, None, vec![], None, false).await;
        let provider = fixture(
            31 + i as u8,
            NET,
            Some(bootstrap.addr.clone()),
            vec![cid(object)],
            Some(answer),
            false,
        )
        .await;
        tokio::time::sleep(Duration::from_millis(250)).await;
        let result = discovery::fetch(
            key(40 + i as u8),
            &tiers(),
            vec![bootstrap.addr.clone()],
            cid(object),
            Duration::from_secs(8),
        )
        .await;
        assert!(
            matches!(result, Err(discovery::FetchError::NotFound { rejected }) if rejected >= 1),
            "{result:?}"
        );
        assert!(provider.requests.load(Ordering::SeqCst) > 0);
    }
}

// A silent provider consumes only the single caller deadline (AT-NET-25).
#[tokio::test]
async fn silent_provider_deadline() {
    let bootstrap = fixture(50, NET, None, vec![], None, false).await;
    let provider = fixture(
        51,
        NET,
        Some(bootstrap.addr.clone()),
        vec![cid(b"silent")],
        None,
        true,
    )
    .await;
    tokio::time::sleep(Duration::from_millis(250)).await;
    let start = std::time::Instant::now();
    let result = discovery::fetch(
        key(52),
        &tiers(),
        vec![bootstrap.addr.clone()],
        cid(b"silent"),
        Duration::from_secs(2),
    )
    .await;
    assert_eq!(result.unwrap_err(), discovery::FetchError::Timeout);
    assert!(start.elapsed() < Duration::from_secs(3));
    assert!(provider.requests.load(Ordering::SeqCst) > 0);
}

// Bad or stale first providers do not prevent a valid later result (AT-NET-25).
#[tokio::test]
async fn invalid_and_stale_provider_fallback() {
    for stale in [false, true] {
        let bootstrap = fixture(60, NET, None, vec![], None, false).await;
        let object = b"fallback";
        let signed = entry(1, object);
        let mut seeds = [61u8, 62];
        seeds.sort_by_key(|seed| key(*seed).public().to_peer_id());
        let bad = if stale {
            vec![1, 0]
        } else {
            response(&signed, b"corrupt")
        };
        let first = fixture(
            seeds[0],
            NET,
            Some(bootstrap.addr.clone()),
            vec![cid(object)],
            Some(bad),
            false,
        )
        .await;
        let second = fixture(
            seeds[1],
            NET,
            Some(bootstrap.addr.clone()),
            vec![cid(object)],
            Some(response(&signed, object)),
            false,
        )
        .await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        let found = discovery::fetch(
            key(63),
            &tiers(),
            vec![bootstrap.addr.clone()],
            cid(object),
            Duration::from_secs(10),
        )
        .await
        .unwrap();
        assert_eq!(found.provider, second.peer);
        assert!(first.requests.load(Ordering::SeqCst) > 0);
    }
}

// Network protocol separation has a known-CID successful control (AT-NET-25).
#[tokio::test]
async fn network_separation_and_untiered_nodes() {
    let bootstrap = fixture(70, NET, None, vec![], None, false).await;
    let public = b"network scoped";
    let provider = fixture(
        71,
        NET,
        Some(bootstrap.addr.clone()),
        vec![cid(public)],
        Some(response(&entry(1, public), public)),
        false,
    )
    .await;
    tokio::time::sleep(Duration::from_millis(250)).await;
    assert!(discovery::fetch(
        key(72),
        &tiers(),
        vec![bootstrap.addr.clone()],
        cid(public),
        Duration::from_secs(8)
    )
    .await
    .is_ok());
    let other = Tiers::new(
        [90; 32],
        Consortium::new(vec![Member::from_seed([1; 32]).public()], 1),
        &[FeedWriter::from_seed(NET, [2; 32]).public()],
    )
    .unwrap();
    let result = discovery::fetch(
        key(73),
        &other,
        vec![bootstrap.addr.clone()],
        cid(public),
        Duration::from_secs(5),
    )
    .await;
    assert!(
        matches!(result, Err(discovery::FetchError::NotFound { .. })),
        "{result:?}"
    );
    assert!(provider.requests.load(Ordering::SeqCst) > 0);
    let unscoped = Handle::spawn(key(74), config(false)).unwrap();
    unscoped
        .listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .await
        .unwrap();
    unscoped
        .insert(entry(1, b"untiered"), b"untiered".to_vec())
        .await
        .unwrap();
    unscoped.dial(bootstrap.addr.clone()).await.unwrap();
    tokio::time::sleep(Duration::from_millis(250)).await;
    assert!(matches!(
        discovery::fetch(
            key(75),
            &tiers(),
            vec![bootstrap.addr.clone()],
            cid(b"untiered"),
            Duration::from_secs(5)
        )
        .await,
        Err(discovery::FetchError::NotFound { .. })
    ));
}

// A synchronized public entry and a restored replica are advertised (AT-NET-25).
#[tokio::test]
async fn synchronized_and_restored_catalogues() {
    let bootstrap = fixture(80, NET, None, vec![], None, false).await;
    let source = Handle::spawn(key(81), config(false)).unwrap();
    let source_addr = source
        .listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .await
        .unwrap();
    let object = b"stored public";
    source
        .insert(entry(1, object), object.to_vec())
        .await
        .unwrap();
    let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("discovery-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut stored = config(true);
    stored.store = Some(root.clone());
    let provider = Handle::spawn(key(82), stored).unwrap();
    provider
        .listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .await
        .unwrap();
    provider.dial(bootstrap.addr.clone()).await.unwrap();
    provider.dial(source_addr).await.unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        while provider.replica().await.is_empty() {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(
        discovery::fetch(
            key(83),
            &tiers(),
            vec![bootstrap.addr.clone()],
            cid(object),
            Duration::from_secs(8)
        )
        .await
        .unwrap()
        .provider,
        provider.peer_id()
    );
    provider
        .insert(entry(2, b"stored circle"), b"stored circle".to_vec())
        .await
        .unwrap();
    drop(provider);
    tokio::time::sleep(Duration::from_millis(150)).await;
    let bootstrap2 = fixture(84, NET, None, vec![], None, false).await;
    let mut stored = config(true);
    stored.store = Some(root.clone());
    let restored = Handle::spawn(key(85), stored).unwrap();
    let restored_addr = restored
        .listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .await
        .unwrap();
    restored.dial(bootstrap2.addr.clone()).await.unwrap();
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert_eq!(
        discovery::fetch(
            key(86),
            &tiers(),
            vec![bootstrap2.addr.clone()],
            cid(object),
            Duration::from_secs(8)
        )
        .await
        .unwrap()
        .provider,
        restored.peer_id()
    );
    assert_eq!(
        raw(restored_addr, NET, cid(b"stored circle")).await,
        vec![1, 0]
    );
    assert!(bootstrap2
        .advertised
        .lock()
        .unwrap()
        .contains_key(&cid(object).0.to_vec()));
    assert!(!bootstrap2
        .advertised
        .lock()
        .unwrap()
        .contains_key(&cid(b"stored circle").0.to_vec()));
    let _ = std::fs::remove_dir_all(root);
}

// An offline advertised provider does not prevent finding a live holder (AT-NET-25).
#[tokio::test]
async fn offline_provider_fallback() {
    let bootstrap = fixture(90, NET, None, vec![], None, false).await;
    let object = b"offline fallback";
    let signed = entry(1, object);
    let mut seeds = [91u8, 92];
    seeds.sort_by_key(|seed| key(*seed).public().to_peer_id());
    let offline = fixture(
        seeds[0],
        NET,
        Some(bootstrap.addr.clone()),
        vec![cid(object)],
        Some(response(&signed, object)),
        false,
    )
    .await;
    let live = fixture(
        seeds[1],
        NET,
        Some(bootstrap.addr.clone()),
        vec![cid(object)],
        Some(response(&signed, object)),
        false,
    )
    .await;
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert!(bootstrap
        .advertised
        .lock()
        .unwrap()
        .contains_key(&cid(object).0.to_vec()));
    assert!(bootstrap
        .advertised
        .lock()
        .unwrap()
        .get(&cid(object).0.to_vec())
        .unwrap()
        .contains(&offline.peer));
    offline.task.abort();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let found = discovery::fetch(
        key(93),
        &tiers(),
        vec![bootstrap.addr.clone()],
        cid(object),
        Duration::from_secs(8),
    )
    .await
    .unwrap();
    assert_eq!(found.provider, live.peer);
}
