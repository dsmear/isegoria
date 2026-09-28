//! The consortium's cuts and beacon between separate nodes over libp2p (`docs/04` §Proposing
//! and signing, §The beacon on cuts, `docs/08` PROTO-015, T74 step 2).

use identity::credential::Issuer;
use libp2p::identity::Keypair;
use network::consortium::{Consortium, Member};
use network::cut::{collect, Collected};
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
            patience: 40,
        }),
    }
}

async fn start(i: usize, role: bool) -> Handle {
    let h = Handle::spawn(Keypair::generate_ed25519(), config(i, role)).expect("a node starts");
    h.listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .await
        .expect("loopback listens");
    h
}

/// Applies the first `n` collected cuts of `replica` to a fresh ledger.
fn ledger(replica: &Replica, n: u64) -> Ledger {
    let mut l = Ledger::new(NET, consortium(), Issuer::new([1; 32]).public());
    for number in 0..n {
        let Collected::Ready(cut, sigs) = collect(replica, &consortium(), NET, number) else {
            panic!("cut {number} is collected")
        };
        l.apply(replica, &cut, &sigs)
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
