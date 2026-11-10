//! A consortium conflict between member nodes (`docs/04` §The duties' stop at a conflict,
//! AT-NET-38): the duties stop, the evidence is served, a restart keeps them stopped.

use libp2p::identity::Keypair;
use libp2p::Multiaddr;
use network::cid::cid;
use network::consortium::{Consortium, Member};
use network::cut::{sign_cut, Cut};
use network::log::TransparencyLog;
use network::replica::{DurableReplica, FeedWriter, Replica, SignedEntry};
use network::tier::{Tier, Tiers};
use p2p::member::{collected, MemberRole};
use p2p::record::Stop;
use p2p::{Config, Handle, Own};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

const NET: [u8; 32] = [38; 32];
const SEEDS: [u8; 3] = [160, 161, 162];
const CIRCLE_SEEDS: [u8; 3] = [170, 171, 172];
const TRANSPORT_SEEDS: [u8; 4] = [180, 181, 182, 183];

fn scratch(name: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("cut_conflict")
        .join(format!("{name}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn member(i: usize) -> Member {
    Member::from_seed([SEEDS[i]; 32])
}

fn consortium() -> Consortium {
    Consortium::new((0..3).map(|i| member(i).public()).collect(), 2)
}

fn tiers() -> Tiers {
    let circle = CIRCLE_SEEDS.map(|s| FeedWriter::from_seed(NET, [s; 32]).public());
    Tiers::new(NET, consortium(), &circle).expect("a valid tiered set")
}

fn public(i: usize) -> [u8; 32] {
    member(i).public().to_bytes()
}

/// Member `i` keeping its feeds, record and replica in `root`.
fn config(i: usize, root: &Path) -> Config {
    let record = root.join("identity");
    if !record.exists() {
        let circle = FeedWriter::from_seed(NET, [CIRCLE_SEEDS[i]; 32]).public();
        p2p::record::create(&record, public(i), circle.to_bytes()).unwrap();
    }
    Config {
        writers: tiers().writers().clone(),
        sync_every: Duration::from_millis(200),
        own: Some(Own {
            writer: FeedWriter::from_seed(NET, [SEEDS[i]; 32]),
            dir: root.join("own"),
        }),
        circle: Some(Own {
            writer: FeedWriter::from_seed(NET, [CIRCLE_SEEDS[i]; 32]),
            dir: root.join("circle"),
        }),
        store: Some(root.join("replica")),
        member: Some(MemberRole {
            member: member(i),
            index: i,
            consortium: consortium(),
            every: Duration::from_millis(100),
            cuts_per_epoch: 1000,
            patience: 2,
            view_patience: 3,
        }),
        consortium: None,
        tiers: Some(tiers()),
        relay: None,
        gateway: None,
        identity: Some(record),
        hook: None,
    }
}

fn reader_config() -> Config {
    Config {
        own: None,
        circle: None,
        store: None,
        member: None,
        identity: None,
        hook: None,
        ..config(0, &scratch("reader"))
    }
}

async fn start(transport: u8, config: Config) -> (Handle, Multiaddr) {
    let key = Keypair::ed25519_from_bytes([transport; 32]).unwrap();
    let h = Handle::spawn(key, config).expect("a node starts");
    let addr = h
        .listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .await
        .unwrap();
    (h, addr)
}

/// Waits, at most 60 s, until `done` holds.
async fn until<F, Fut>(what: &str, done: F)
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    while !done().await {
        assert!(tokio::time::Instant::now() < deadline, "timed out: {what}");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// Members 0–2 running, each connected to the others, once each collected two cuts.
async fn members(roots: &[PathBuf]) -> (Vec<Handle>, Vec<Multiaddr>) {
    let (mut handles, mut addrs) = (Vec::new(), Vec::new());
    for (i, root) in roots.iter().enumerate() {
        let (h, addr) = start(TRANSPORT_SEEDS[i], config(i, root)).await;
        for a in &addrs {
            h.dial(Clone::clone(a)).await.unwrap();
        }
        handles.push(h);
        addrs.push(addr);
    }
    for h in &handles {
        until("two cuts", || async { h.status().await.cuts.len() >= 2 }).await;
    }
    (handles, addrs)
}

/// Members 1 and 2's keys sign a second cut for number 1, on entries far past their feeds.
fn second_cut(replica: &Replica) -> Vec<(SignedEntry, Vec<u8>)> {
    let cuts = collected(replica, &consortium(), NET);
    let other = Cut::next(replica, Some(&cuts[0]), cuts[1].epoch, cuts[1].closes);
    assert_ne!(other, cuts[1]);
    [1, 2]
        .map(|m| {
            let object = sign_cut(&member(m), m, NET, &consortium(), &other).encode();
            let mut log = TransparencyLog::new();
            for k in 0..1000u32 {
                log.append(cid(&k.to_le_bytes()));
            }
            let entry = log.append(cid(&object)).clone();
            (
                FeedWriter::from_seed(NET, [SEEDS[m]; 32]).sign(&entry),
                object,
            )
        })
        .to_vec()
}

async fn own_len(h: &Handle, i: usize) -> usize {
    h.replica().await.feed(&public(i)).len()
}

/// Member `i`'s feed keeps `len` entries over ten duty periods, more than its view patience.
async fn steady(h: &Handle, i: usize, len: usize) {
    for _ in 0..10 {
        assert_eq!(own_len(h, i).await, len, "a duty added an entry");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// The status of `h` once its duties halted at the conflict.
async fn halted(h: &Handle) -> p2p::Status {
    until("the duties halt", || async {
        let s = h.status().await;
        s.conflict && s.duties_halted
    })
    .await;
    h.status().await
}

// At a second cut for a collected number the duties halt apart from A, the evidence is served
// and publication keeps its own rules (AT-NET-38).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_duties_halt_at_a_conflict_and_the_evidence_is_served() {
    let roots: Vec<PathBuf> = (0..3).map(|i| scratch(&format!("live-{i}"))).collect();
    let (m, addrs) = members(&roots).await;
    for (entry, object) in second_cut(&m[0].replica().await) {
        m[0].insert(entry, object).await.unwrap();
    }
    let s = halted(&m[0]).await;
    assert_eq!((s.stopped, s.cuts.len()), (None, 1));
    let len = own_len(&m[0], 0).await;
    let (reader, _) = start(TRANSPORT_SEEDS[3], reader_config()).await;
    reader.dial(addrs[0].clone()).await.unwrap();
    until(
        "the reader holds the evidence and sees the conflict",
        || async { reader.status().await.conflict },
    )
    .await;
    for h in &m[1..] {
        until(
            "A stops the members whose keys signed past their files",
            || async {
                let s = h.status().await;
                s.conflict && s.stopped == Some(Stop::OwnEntry(Tier::Public))
            },
        )
        .await;
        assert!(!h.status().await.duties_halted);
    }
    steady(&m[0], 0, len).await;
    let published = m[0].publish(b"a note after the conflict".to_vec()).await;
    assert_eq!(published.unwrap().entry.seq as usize, len);
    assert!(m[0].publish_circle(b"a circle note".to_vec()).await.is_ok());
    assert_eq!(m[0].status().await.cuts.len(), 1);
}

// Restarted on a stored replica holding a conflict, the duties halt at their first look and
// add nothing; the condition is not A's stop (AT-NET-38).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_restart_from_the_stored_replica_keeps_the_duties_halted() {
    let roots: Vec<PathBuf> = (0..3).map(|i| scratch(&format!("restart-{i}"))).collect();
    let (mut m, addrs) = members(&roots).await;
    let evidence = second_cut(&m[0].replica().await);
    m.remove(0).finished().await;
    until("member 0 has left", || async {
        m[0].status().await.peers == 1 && m[1].status().await.peers == 1
    })
    .await;
    let writers = tiers().writers().clone();
    let mut stored = DurableReplica::open(&roots[0].join("replica"), writers).unwrap();
    for (entry, object) in evidence {
        stored.insert(entry, object).unwrap();
    }
    drop(stored);
    let (m0, _) = start(TRANSPORT_SEEDS[0], config(0, &roots[0])).await;
    let s = m0.status().await;
    assert!(s.conflict && !s.caught_up && !s.duties_halted, "{s:?}");
    let len = own_len(&m0, 0).await;
    for a in &addrs[1..] {
        m0.dial(a.clone()).await.unwrap();
    }
    until("member 0's check completes", || async {
        m0.status().await.caught_up
    })
    .await;
    let s = halted(&m0).await;
    assert_eq!((s.stopped, s.cuts.len()), (None, 1));
    steady(&m0, 0, len).await;
}
