//! The consortium's cuts and beacon between separate nodes over libp2p (`docs/04` §Proposing
//! and signing, §The beacon on cuts, `docs/08` PROTO-015, T74 step 2).

use identity::credential::Issuer;
use libp2p::identity::Keypair;
use network::consortium::{Consortium, Member};
use network::cut::{collect_with, Collected};
use network::replica::{FeedWriter, Replica, WriterSet};
use p2p::member::{collected, MemberRole};
use p2p::{Config, Handle, Own, StartError};
use protocol::ledger::Ledger;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

const NET: [u8; 32] = [9; 32];
const SEEDS: [u8; 4] = [10, 11, 12, 20];

fn scratch(name: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("beacon_between_nodes")
        .join(format!("{name}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn consortium() -> Consortium {
    let keys = SEEDS[..3]
        .iter()
        .map(|s| Member::from_seed([*s; 32]).public())
        .collect();
    Consortium::new(keys, 2)
}

fn writers() -> WriterSet {
    let keys: Vec<_> = SEEDS
        .iter()
        .map(|s| FeedWriter::from_seed(NET, [*s; 32]).public())
        .collect();
    WriterSet::new(NET, &keys)
}

fn config(i: usize, role: bool) -> Config {
    config_waiting(i, role, 40, 60)
}

/// As [`config`], a member waiting `patience` looks for beacon messages and `view_patience`
/// looks before it changes view.
fn config_waiting(i: usize, role: bool, patience: u32, view_patience: u32) -> Config {
    let seed = [SEEDS[i]; 32];
    Config {
        writers: writers(),
        sync_every: Duration::from_millis(300),
        own: Some(Own {
            writer: FeedWriter::from_seed(NET, seed),
            dir: scratch(&format!("w{i}")),
        }),
        store: None,
        member: role.then(|| MemberRole {
            member: Member::from_seed(seed),
            index: i,
            consortium: consortium(),
            every: Duration::from_millis(150),
            cuts_per_epoch: 2,
            patience,
            view_patience,
        }),
        consortium: (!role).then(consortium),
        circle: None,
        tiers: None,
        relay: None,
        gateway: None,
    }
}

async fn start(i: usize, role: bool) -> Handle {
    start_with(config(i, role)).await
}

/// A member that waits 10 looks for beacon messages and 20 before it changes view.
async fn quick(i: usize) -> Handle {
    start_with(config_waiting(i, true, 10, 20)).await
}

async fn start_with(config: Config) -> Handle {
    let h = Handle::spawn(Keypair::generate_ed25519(), config).expect("a node starts");
    h.listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .await
        .expect("loopback listens");
    h
}

/// Applies the first `n` collected cuts of `replica` to a fresh ledger.
fn ledger(replica: &Replica, n: u64) -> Ledger {
    let mut l = Ledger::new(NET, consortium(), Issuer::new([1; 32]).public());
    for number in 0..n {
        let ready = collect_with(replica, &consortium(), NET, number, &|c| l.excluded(c));
        let Collected::Ready(cut, view, sigs) = ready else {
            panic!("cut {number} is collected")
        };
        l.apply_in_view(replica, &cut, view, &sigs)
            .expect("a collected cut applies");
    }
    l
}

/// AT-NET-17: three member nodes and a relay, over loopback TCP, propose and sign cuts in
/// turn and run two epochs' beacon; every node computes the same cuts and beacons.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_net_17_members_cut_and_run_the_beacon_between_nodes() {
    let (m0, m1, m2, relay) = (
        start(0, true).await,
        start(1, true).await,
        start(2, true).await,
        start(3, false).await,
    );
    let addr = m0
        .listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .await
        .unwrap();
    for h in [&m1, &m2, &relay] {
        h.dial(addr.clone()).await.unwrap();
    }
    while !relay.caught_up().await {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    for i in 0..3 {
        relay
            .publish(format!("relayed-{i}").into_bytes())
            .await
            .unwrap();
    }
    let nodes = [&m0, &m1, &m2, &relay];
    let deadline = tokio::time::Instant::now() + Duration::from_secs(40);
    loop {
        let mut done = true;
        for h in nodes {
            done &= collected(&h.replica().await, &consortium(), NET).len() >= 5;
        }
        if done {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "cuts did not advance"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let mut outcomes = Vec::new();
    for h in nodes {
        let r = h.replica().await;
        let cuts = collected(&r, &consortium(), NET);
        assert!(cuts[1].closes && cuts[3].closes, "two cuts per epoch");
        let l = ledger(&r, 5);
        let pair = (l.beacon(0).cloned(), l.beacon(1).cloned());
        assert!(pair.0.as_ref().is_some_and(|o| o.value().is_some()));
        assert!(pair.1.as_ref().is_some_and(|o| o.value().is_some()));
        outcomes.push((cuts[..5].to_vec(), pair));
    }
    assert!(
        outcomes.windows(2).all(|w| w[0] == w[1]),
        "every node agrees"
    );
    for h in nodes {
        let status = h.status().await;
        let digests: Vec<_> = outcomes[0].0.iter().map(|c| c.digest()).collect();
        assert_eq!(status.cuts[..5], digests[..], "the status reports the cuts");
        let rounds: Vec<_> = [&outcomes[0].1 .0, &outcomes[0].1 .1]
            .iter()
            .map(|o| {
                o.as_ref()
                    .map(|o| (o.value(), o.withheld().to_vec()))
                    .unwrap()
            })
            .collect();
        assert_eq!(status.beacons[..2], rounds[..]);
        assert!(status.caught_up && status.peers > 0 && !status.conflict);
        assert_eq!(status.equivocations, 0);
    }
    assert_ne!(
        outcomes[0].1 .0.as_ref().unwrap().value(),
        outcomes[0].1 .1.as_ref().unwrap().value()
    );
}

/// AT-NET-17: a member role needs its own feed under the member's key.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn at_net_17_a_member_writes_under_its_own_key() {
    let mut wrong = config(0, true);
    wrong.own = Some(Own {
        writer: FeedWriter::from_seed(NET, [SEEDS[3]; 32]),
        dir: scratch("wrong"),
    });
    assert!(matches!(
        Handle::spawn(Keypair::generate_ed25519(), wrong).err(),
        Some(StartError::NotTheMember)
    ));
    let mut index = config(0, true);
    if let Some(role) = &mut index.member {
        role.index = 1;
    }
    assert!(matches!(
        Handle::spawn(Keypair::generate_ed25519(), index).err(),
        Some(StartError::NotTheMember)
    ));
    let mut none = config(0, true);
    none.own = None;
    assert!(matches!(
        Handle::spawn(Keypair::generate_ed25519(), none).err(),
        Some(StartError::NotTheMember)
    ));
}

/// Waits until every node in `nodes` has collected at least `n` cuts, within `secs`.
async fn cut_at_least(nodes: &[&Handle], n: usize, secs: u64) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(secs);
    loop {
        let mut done = true;
        for h in nodes {
            done &= collected(&h.replica().await, &consortium(), NET).len() >= n;
        }
        if done {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "cuts did not advance"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// The first `n` cuts and the beacons of epochs 0 and 1, as each node computes them.
async fn agreed(nodes: &[&Handle], n: usize) {
    let mut seen = Vec::new();
    for h in nodes {
        let r = h.replica().await;
        let l = ledger(&r, n as u64);
        let cuts = collected(&r, &consortium(), NET);
        seen.push((
            cuts[..n].to_vec(),
            l.beacon(0).cloned(),
            l.beacon(1).cloned(),
        ));
    }
    assert!(seen.windows(2).all(|w| w[0] == w[1]), "every node agrees");
    assert!(seen[0].1.as_ref().is_some_and(|o| o.value().is_some()));
}

/// AT-NET-19: with the proposer of cut 0 never started, the other two members replace it and
/// keep cutting; every node computes the same cuts and beacons.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_net_19_a_silent_proposer_is_replaced() {
    let (m1, m2) = (quick(1).await, quick(2).await);
    let addr = m1
        .listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .await
        .unwrap();
    m2.dial(addr).await.unwrap();
    cut_at_least(&[&m1, &m2], 5, 60).await;
    agreed(&[&m1, &m2], 5).await;
}

/// AT-NET-19: a member stopped after the first cuts leaves the others cutting.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_net_19_a_member_stopped_midway_is_replaced() {
    let (m0, m1, m2) = (quick(0).await, quick(1).await, quick(2).await);
    let addr = m0
        .listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .await
        .unwrap();
    for h in [&m1, &m2] {
        h.dial(addr.clone()).await.unwrap();
    }
    cut_at_least(&[&m0, &m1, &m2], 2, 40).await;
    let before = collected(&m1.replica().await, &consortium(), NET).len();
    drop(m0);
    let addr = m1
        .listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .await
        .unwrap();
    m2.dial(addr).await.unwrap();
    cut_at_least(&[&m1, &m2], before + 6, 60).await;
    agreed(&[&m1, &m2], before + 6).await;
}
