//! A writer's recovery (`docs/04` §A writer's own feed, `docs/08` G-23, T80): restarted with
//! lost, damaged or old files, it rebuilds its feed from a peer and never forks it.

use libp2p::identity::Keypair;
use libp2p::Multiaddr;
use network::cid::cid;
use network::log::TransparencyLog;
use network::replica::{FeedError, FeedWriter, Replica, SignedEntry, WriterSet};
use network::store::DurableLog;
use p2p::{Config, Handle, Own, PublishError};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

const NET: [u8; 32] = [4; 32];

fn scratch(name: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("writer_recovery")
        .join(format!("{name}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn writer() -> FeedWriter {
    FeedWriter::from_seed(NET, [1; 32])
}

fn me() -> [u8; 32] {
    writer().public().to_bytes()
}

/// A node; with `own`, writer 1 keeps its feed there.
async fn node(own: Option<&Path>) -> (Handle, Multiaddr) {
    let config = Config {
        writers: WriterSet::new(NET, &[writer().public()]),
        sync_every: Duration::from_millis(200),
        own: own.map(|dir| Own {
            writer: writer(),
            dir: dir.to_path_buf(),
        }),
        store: None,
        member: None,
        consortium: None,
        circle: None,
        tiers: None,
        relay: None,
        gateway: None,
    };
    let handle = Handle::spawn(Keypair::generate_ed25519(), config).expect("a node starts");
    let addr = handle
        .listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .await
        .expect("loopback listens");
    (handle, addr)
}

async fn until<F, Fut>(done: F)
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    while !done().await {
        assert!(tokio::time::Instant::now() < deadline, "timed out");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// Publishes as soon as the writer may, within 15 s.
async fn publish(h: &Handle, object: &[u8]) -> SignedEntry {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    loop {
        if let Ok(e) = h.publish(object.to_vec()).await {
            return e;
        }
        assert!(tokio::time::Instant::now() < deadline, "never published");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn converged(nodes: &[&Handle], n: usize) {
    until(|| async {
        let mut digests = Vec::new();
        for h in nodes {
            let r = h.replica().await;
            if r.len() != n {
                return false;
            }
            digests.push(r.digest());
        }
        digests.windows(2).all(|d| d[0] == d[1])
    })
    .await;
}

fn whole(r: &Replica, n: usize) {
    assert!(r.equivocations().is_empty(), "the writer forked");
    assert_eq!(r.feed(&me()).len(), n);
}

/// A relay holding `n` entries the writer published under `dir`, the writer stopped.
async fn history(n: usize) -> (Handle, Multiaddr, PathBuf) {
    let dir = scratch("own");
    let (relay, addr) = node(None).await;
    {
        let (w, _) = node(Some(&dir)).await;
        w.dial(addr.clone()).await.unwrap();
        for i in 0..n {
            publish(&w, format!("e-{i}").as_bytes()).await;
        }
        converged(&[&w, &relay], n).await;
    }
    until(|| async { relay.peers().await == 0 }).await;
    (relay, addr, dir)
}

/// `n` entries of the writer, signed out of band, with their objects.
fn signed(n: usize, tag: &str) -> Vec<(SignedEntry, Vec<u8>)> {
    let mut log = TransparencyLog::new();
    (0..n)
        .map(|i| {
            let object = format!("{tag}-{i}").into_bytes();
            let entry = log.append(cid(&object)).clone();
            (writer().sign(&entry), object)
        })
        .collect()
}

/// Restarts the writer on `dir`, connects it to the relay and publishes one entry; returns
/// the files set aside.
async fn resume(relay: &Handle, addr: Multiaddr, dir: &Path, n: u64) -> Vec<PathBuf> {
    let (w, _) = node(Some(dir)).await;
    let aside = w.set_aside().to_vec();
    w.dial(addr).await.unwrap();
    assert_eq!(publish(&w, b"after").await.entry.seq, n);
    converged(&[&w, relay], n as usize + 1).await;
    whole(&relay.replica().await, n as usize + 1);
    drop(w);
    until(|| async { relay.peers().await == 0 }).await;
    let (log, _) = DurableLog::open(&dir.join("log")).unwrap();
    assert_eq!(log.log().len() as u64, n + 1, "the own log is rebuilt");
    aside
}

/// AT-NET-18: a writer that lost its files rebuilds its feed from a peer and continues.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn at_net_18_a_writer_that_lost_its_files_rebuilds_its_feed() {
    let (relay, addr, dir) = history(3).await;
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(resume(&relay, addr, &dir, 3).await.is_empty());
}

/// AT-NET-18: a log damaged before its tail is set aside and the feed rebuilt from a peer.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn at_net_18_a_damaged_log_is_set_aside_and_rebuilt() {
    let (relay, addr, dir) = history(3).await;
    let path = dir.join("log");
    let mut bytes = std::fs::read(&path).unwrap();
    bytes[16] ^= 1;
    std::fs::write(&path, bytes).unwrap();
    let aside = resume(&relay, addr, &dir, 3).await;
    assert_eq!(
        aside,
        [dir.join("log.damaged-0"), dir.join("objects.damaged-0")]
    );
    assert!(
        aside.iter().all(|p| p.exists()),
        "the damaged files are kept"
    );
}

/// AT-NET-18: a writer whose objects are lost rebuilds them from a peer.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn at_net_18_lost_objects_are_rebuilt() {
    let (relay, addr, dir) = history(3).await;
    std::fs::remove_file(dir.join("objects")).unwrap();
    let aside = resume(&relay, addr, &dir, 3).await;
    assert_eq!(
        aside,
        [dir.join("log.damaged-0"), dir.join("objects.damaged-0")]
    );
}

/// AT-NET-18: a writer restored from older files takes the rest of its feed from a peer.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn at_net_18_a_writer_restored_from_old_files_catches_up() {
    let dir = scratch("restored");
    let old = scratch("backup");
    let (relay, addr) = node(None).await;
    {
        let (w, _) = node(Some(&dir)).await;
        w.dial(addr.clone()).await.unwrap();
        publish(&w, b"one").await;
        publish(&w, b"two").await;
        converged(&[&w, &relay], 2).await;
        std::fs::create_dir_all(&old).unwrap();
        for name in ["log", "objects"] {
            std::fs::copy(dir.join(name), old.join(name)).unwrap();
        }
        publish(&w, b"three").await;
        publish(&w, b"four").await;
        converged(&[&w, &relay], 4).await;
    }
    until(|| async { relay.peers().await == 0 }).await;
    assert!(resume(&relay, addr, &old, 4).await.is_empty());
}

/// AT-NET-18: restarted without its files, a writer does not write before it hears a peer.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn at_net_18_no_write_before_a_peer_is_heard() {
    let (relay, addr, dir) = history(2).await;
    std::fs::remove_dir_all(&dir).unwrap();
    let (w, _) = node(Some(&dir)).await;
    assert!(!w.caught_up().await);
    assert_eq!(
        w.publish(b"too early".to_vec()).await,
        Err(PublishError::NotCaughtUp)
    );
    w.dial(addr).await.unwrap();
    assert_eq!(publish(&w, b"later").await.entry.seq, 2);
    converged(&[&w, &relay], 3).await;
    whole(&relay.replica().await, 3);
}

/// AT-NET-18: a writer whose reachable feed is forked or has a gap signs nothing until the
/// gap is filled; a fork stops it for good.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn at_net_18_a_fork_or_a_gap_stops_the_writer() {
    let (forked, forked_addr) = node(None).await;
    let x = signed(1, "x");
    let y = signed(1, "y");
    for (e, o) in [&x[0], &y[0]] {
        forked.insert(e.clone(), o.clone()).await.unwrap();
    }
    let (w, _) = node(Some(&scratch("forked"))).await;
    w.dial(forked_addr).await.unwrap();
    until(|| async { w.replica().await.len() == 2 && w.caught_up().await }).await;
    assert_eq!(
        w.publish(b"over a fork".to_vec()).await,
        Err(PublishError::Feed(FeedError::Forked { seq: 0 }))
    );

    let (gapped, gapped_addr) = node(None).await;
    let chain = signed(3, "c");
    for i in [0, 2] {
        gapped
            .insert(chain[i].0.clone(), chain[i].1.clone())
            .await
            .unwrap();
    }
    let (w, _) = node(Some(&scratch("gapped"))).await;
    w.dial(gapped_addr).await.unwrap();
    until(|| async { w.replica().await.len() == 2 && w.caught_up().await }).await;
    assert_eq!(
        w.publish(b"over a gap".to_vec()).await,
        Err(PublishError::Feed(FeedError::Gap { seq: 1 }))
    );
    w.insert(chain[1].0.clone(), chain[1].1.clone())
        .await
        .unwrap();
    assert_eq!(publish(&w, b"filled").await.entry.seq, 3);
    whole(&w.replica().await, 4);
}

/// AT-NET-18: a writer counts as caught up only once it holds every entry of its key the
/// peer offered.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn at_net_18_caught_up_only_with_the_whole_feed() {
    let dir = scratch("whole");
    let (relay, addr) = node(None).await;
    {
        let (w, _) = node(Some(&dir)).await;
        w.dial(addr.clone()).await.unwrap();
        for i in 0..3u8 {
            publish(&w, &vec![i; 1 << 20]).await;
        }
        converged(&[&w, &relay], 3).await;
    }
    until(|| async { relay.peers().await == 0 }).await;
    std::fs::remove_dir_all(&dir).unwrap();
    let (w, _) = node(Some(&dir)).await;
    w.dial(addr).await.unwrap();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    while !w.caught_up().await {
        assert!(tokio::time::Instant::now() < deadline, "never caught up");
        tokio::task::yield_now().await;
    }
    whole(&w.replica().await, 3);
}
