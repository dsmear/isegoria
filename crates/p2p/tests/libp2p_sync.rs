//! Replication over libp2p (`docs/04` §Replication between nodes, `docs/08` NET-010, T18):
//! real nodes on loopback TCP converge on the same signed set.

use libp2p::identity::Keypair;
use libp2p::Multiaddr;
use network::cid::cid;
use network::log::TransparencyLog;
use network::replica::{Accepted, FeedWriter, WriterSet};
use p2p::{Config, Handle, Own, PublishError, StartError};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

const NET: [u8; 32] = [3; 32];

fn scratch(name: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("libp2p_sync")
        .join(format!("{name}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn writer(i: u8) -> FeedWriter {
    FeedWriter::from_seed(NET, [i; 32])
}

fn writer_set() -> WriterSet {
    let keys: Vec<_> = (1..=3).map(|i| writer(i).public()).collect();
    WriterSet::new(NET, &keys)
}

/// A node; a writer (`Some(i)`) keeps its own feed under `dir`.
async fn node(own: Option<(u8, PathBuf)>) -> (Handle, Multiaddr) {
    node_every(own, Duration::from_millis(300)).await
}

async fn node_every(own: Option<(u8, PathBuf)>, every: Duration) -> (Handle, Multiaddr) {
    let config = Config {
        writers: writer_set(),
        sync_every: every,
        own: own.map(|(i, dir)| Own {
            writer: writer(i),
            dir,
        }),
    };
    let handle = Handle::spawn(Keypair::generate_ed25519(), config).expect("a node starts");
    let addr = handle
        .listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .await
        .expect("loopback listens");
    (handle, addr)
}

/// Waits until every node has the same digest and holds `n` entries.
async fn converged(nodes: &[&Handle], n: usize) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        let mut digests = Vec::new();
        let mut lens = Vec::new();
        for h in nodes {
            let r = h.replica().await;
            digests.push(r.digest());
            lens.push(r.len());
        }
        if lens.iter().all(|l| *l == n) && digests.windows(2).all(|d| d[0] == d[1]) {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "no convergence: {lens:?}"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// AT-NET-14: writers publishing on different nodes, relayed through a third, converge.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn at_net_14_nodes_converge_over_libp2p() {
    let (a, _) = node(Some((1, scratch("a")))).await;
    let (b, b_addr) = node(Some((2, scratch("b")))).await;
    let (c, _) = node(None).await;
    a.dial(b_addr.clone()).await.unwrap();
    c.dial(b_addr).await.unwrap();
    for i in 0..3 {
        a.publish(format!("a-{i}").into_bytes()).await.unwrap();
    }
    for i in 0..2 {
        b.publish(format!("b-{i}").into_bytes()).await.unwrap();
    }
    converged(&[&a, &b, &c], 5).await;
    let r = c.replica().await;
    assert_eq!(r.feed(&writer(1).public().to_bytes()).len(), 3);
    assert_eq!(r.feed(&writer(2).public().to_bytes()).len(), 2);
    assert!(r.equivocations().is_empty());
    assert_eq!(
        c.publish(b"c".to_vec()).await,
        Err(PublishError::NotAWriter)
    );
}

/// AT-NET-14: a node that wrote while partitioned converges once it reconnects.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn at_net_14_a_partition_heals() {
    let (a, a_addr) = node(Some((1, scratch("a")))).await;
    let (b, _) = node(None).await;
    let (c, _) = node(Some((3, scratch("c")))).await;
    b.dial(a_addr.clone()).await.unwrap();
    for i in 0..4 {
        a.publish(format!("a-{i}").into_bytes()).await.unwrap();
    }
    for i in 0..3 {
        c.publish(format!("c-{i}").into_bytes()).await.unwrap();
    }
    converged(&[&a, &b], 4).await;
    assert_eq!(c.replica().await.len(), 3);
    c.dial(a_addr).await.unwrap();
    converged(&[&a, &b, &c], 7).await;
}

/// AT-NET-14: a writer's fork, shown to two sides of the network, reaches every node as
/// evidence.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn at_net_14_a_fork_reaches_everyone_as_evidence() {
    let (a, a_addr) = node(None).await;
    let (b, _) = node(None).await;
    let (c, _) = node(None).await;
    let w = writer(2);
    for (h, tag) in [(&a, "x"), (&c, "y")] {
        let mut log = TransparencyLog::new();
        for i in 0..2 {
            let object = format!("{tag}-{i}").into_bytes();
            let entry = log.append(cid(&object)).clone();
            assert_eq!(h.insert(w.sign(&entry), object).await, Ok(Accepted::New));
        }
    }
    b.dial(a_addr.clone()).await.unwrap();
    c.dial(a_addr).await.unwrap();
    converged(&[&a, &b, &c], 4).await;
    let set = writer_set();
    for h in [&a, &b, &c] {
        let found = h.replica().await.equivocations();
        assert_eq!(found.len(), 2);
        assert!(found.iter().all(|e| e.verify(&set)));
    }
}

/// AT-NET-14: a writer restarted on its own files signs the same entries again and
/// continues its feed without forking it.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn at_net_14_a_restarted_writer_does_not_fork() {
    let dir = scratch("restart");
    let (b, b_addr) = node(None).await;
    {
        let (a, _) = node(Some((1, dir.clone()))).await;
        a.dial(b_addr.clone()).await.unwrap();
        a.publish(b"one".to_vec()).await.unwrap();
        a.publish(b"two".to_vec()).await.unwrap();
        converged(&[&a, &b], 2).await;
    }
    let (a, _) = node(Some((1, dir))).await;
    assert_eq!(a.replica().await.digest(), b.replica().await.digest());
    let third = a.publish(b"three".to_vec()).await.unwrap();
    assert_eq!(third.entry.seq, 2);
    a.dial(b_addr).await.unwrap();
    converged(&[&a, &b], 3).await;
    let r = b.replica().await;
    assert!(r.equivocations().is_empty());
    assert_eq!(r.feed(&writer(1).public().to_bytes()).len(), 3);
}

/// AT-NET-14: a key outside the writer set cannot start a writer node.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn at_net_14_only_writers_publish() {
    let config = Config {
        writers: writer_set(),
        sync_every: Duration::from_millis(300),
        own: Some(Own {
            writer: writer(9),
            dir: scratch("outsider"),
        }),
    };
    let outsider = Handle::spawn(Keypair::generate_ed25519(), config);
    assert!(matches!(outsider.err(), Some(StartError::NotAWriter)));
}

/// AT-NET-14: with no periodic pull, an entry published after two nodes connected reaches
/// the other through its gossip announcement.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn at_net_14_gossip_carries_new_entries() {
    let hour = Duration::from_secs(3600);
    let (a, a_addr) = node_every(Some((1, scratch("gossip"))), hour).await;
    let (b, _) = node_every(None, hour).await;
    b.dial(a_addr).await.unwrap();
    while a.peers().await == 0 {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert_eq!(b.replica().await.len(), 0);
    a.publish(b"news".to_vec()).await.unwrap();
    converged(&[&a, &b], 1).await;
}
