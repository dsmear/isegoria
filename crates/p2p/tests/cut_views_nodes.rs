//! The cuts' views and exclusions member nodes and a reader report (`docs/04` §The cuts' views
//! and exclusions in the status, AT-NET-39): `n = 4`, `t = 3`, member 3 never running.

use libp2p::identity::Keypair;
use libp2p::Multiaddr;
use network::beacon::RoundId;
use network::cid::cid;
use network::consortium::{Consortium, Member};
use network::cut::{collect_with, Collected, MemberObject};
use network::log::TransparencyLog;
use network::replica::{DurableReplica, FeedWriter, Replica};
use network::tier::Tiers;
use p2p::member::{CollectedCut, MemberRole};
use p2p::{Config, Handle, Own};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

const NET: [u8; 32] = [139; 32];
const SEEDS: [u8; 4] = [190, 191, 192, 193];
const CIRCLE_SEEDS: [u8; 4] = [200, 201, 202, 203];
const TRANSPORT_SEEDS: [u8; 4] = [210, 211, 212, 213];

fn scratch(name: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("cut_views_nodes")
        .join(format!("{name}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn member(i: usize) -> Member {
    Member::from_seed([SEEDS[i]; 32])
}

fn consortium() -> Consortium {
    Consortium::new((0..4).map(|i| member(i).public()).collect(), 3)
}

fn tiers() -> Tiers {
    let circle = CIRCLE_SEEDS.map(|s| FeedWriter::from_seed(NET, [s; 32]).public());
    Tiers::new(NET, consortium(), &circle).expect("a valid tiered set")
}

/// Member `i` keeping its feeds, record and replica in `root`; epochs of three cuts.
fn config(i: usize, root: &Path) -> Config {
    let record = root.join("identity");
    if !record.exists() {
        let public = member(i).public().to_bytes();
        let circle = FeedWriter::from_seed(NET, [CIRCLE_SEEDS[i]; 32]).public();
        p2p::record::create(&record, public, circle.to_bytes()).unwrap();
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
            cuts_per_epoch: 3,
            patience: 3,
            view_patience: 6,
        }),
        consortium: None,
        tiers: Some(tiers()),
        relay: None,
        gateway: None,
        identity: Some(record),
        hook: None,
    }
}

/// A tiered node without a role: a reader of the public tier, its replica in memory.
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

/// Member 3's commit for epoch 0, first on its feed: it commits and never reveals.
fn absent_commit(h: &Handle) -> impl std::future::Future<Output = ()> + '_ {
    let round = RoundId {
        network_id: NET,
        member_set_hash: consortium().member_set_hash(),
        epoch: 0,
    };
    let (commit, _) = member(3).beacon_commit(round, 3);
    let object = MemberObject::Commit(commit).encode();
    let entry = FeedWriter::from_seed(NET, [SEEDS[3]; 32])
        .sign(TransparencyLog::new().append(cid(&object)));
    async move {
        h.insert(entry, object).await.unwrap();
    }
}

/// Each reported cut is `collect_with`'s on `replica` with its exclusions, in its view.
fn oracle(replica: &Replica, reported: &[CollectedCut]) {
    for (k, r) in reported.iter().enumerate() {
        let excluded = r.excluded.clone();
        let got = collect_with(replica, &consortium(), NET, k as u64, &move |_| {
            excluded.clone()
        });
        match got {
            Collected::Ready(cut, view, _) => {
                assert_eq!((cut.digest(), view), (r.digest, r.view), "cut {k}")
            }
            other => panic!("cut {k}: {other:?}"),
        }
    }
}

/// The exclusions of cuts 0–6: epoch 0 is cuts 0–2, cut 3 finishes its round with member 3
/// withholding, cuts 4–5 are the rest of epoch 1.
fn expected_exclusions(reported: &[CollectedCut]) {
    for (k, r) in reported.iter().enumerate().take(7) {
        let withheld: &[usize] = if (4..6).contains(&k) { &[3] } else { &[] };
        assert_eq!(r.excluded, withheld, "cut {k}");
    }
}

// Members, a reader and a restart report the views and exclusions collected (AT-NET-39).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn members_and_a_reader_report_views_and_exclusions() {
    let roots: Vec<PathBuf> = (0..3).map(|i| scratch(&format!("member-{i}"))).collect();
    let (mut m, mut addrs) = (Vec::new(), Vec::new());
    for (i, root) in roots.iter().enumerate() {
        let (h, addr) = start(TRANSPORT_SEEDS[i], config(i, root)).await;
        if i == 0 {
            absent_commit(&h).await;
        }
        for a in &addrs {
            h.dial(Clone::clone(a)).await.unwrap();
        }
        m.push(h);
        addrs.push(addr);
    }
    let (reader, _) = start(TRANSPORT_SEEDS[3], reader_config()).await;
    reader.dial(addrs[1].clone()).await.unwrap();
    until("seven cuts and member 3 withheld", || async {
        let s = m[0].status().await;
        s.cuts.len() >= 7
            && s.beacons
                .first()
                .is_some_and(|b| b.0.is_some() && b.1 == [3])
    })
    .await;
    let s = m[0].status().await;
    assert!(
        s.cuts[3].view >= 1,
        "member 3 never proposes cut 3: {:?}",
        s.cuts[3]
    );
    expected_exclusions(&s.cuts);
    for h in [&m[1], &m[2], &reader] {
        until("the others report alike", || async {
            let other = h.status().await.cuts;
            other.len() >= 7 && other[..7] == s.cuts[..7]
        })
        .await;
    }
    oracle(&reader.replica().await, &reader.status().await.cuts);

    let before = m[0].status().await;
    m.remove(0).finished().await;
    let stored =
        DurableReplica::read(&roots[0].join("replica"), tiers().writers().clone()).unwrap();
    oracle(&stored, &before.cuts);
    let (again, _) = start(TRANSPORT_SEEDS[0], config(0, &roots[0])).await;
    let after = again.status().await;
    assert!(!after.caught_up && after.peers == 0, "{after:?}");
    assert_eq!(after.cuts[..before.cuts.len()], before.cuts[..]);
    oracle(&stored, &after.cuts);
    expected_exclusions(&after.cuts);
}
