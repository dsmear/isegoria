//! A tiered member's identity record (`docs/04` §Increments A and B, `docs/01` D58, AT-NET-36):
//! the states that start, the losses and inconsistencies that stop, and no entry once stopped.

use libp2p::identity::Keypair;
use libp2p::Multiaddr;
use network::cid::cid;
use network::consortium::{Consortium, Member};
use network::log::{Entry, TransparencyLog};
use network::replica::{FeedWriter, Replica, SignedEntry};
use network::store::faults::{self, Fault, On};
use network::store::{read_objects, DurableLog, ObjectStore};
use network::tier::{Tier, Tiers};
use p2p::member::MemberRole;
use p2p::record::{read, Found, Record, State, Stop};
use p2p::{Config, Handle, Own, PublishError, StartError};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

const NET: [u8; 32] = [36; 32];
const MEMBER_SEEDS: [u8; 3] = [130, 131, 132];
const CIRCLE_SEEDS: [u8; 3] = [140, 141, 142];
const TRANSPORT_SEEDS: [u8; 3] = [150, 151, 152];
const IDLE: Duration = Duration::from_secs(3600);
const BUSY: Duration = Duration::from_millis(150);

fn scratch(name: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("member_identity")
        .join(format!("{name}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn public(i: usize) -> [u8; 32] {
    Member::from_seed([MEMBER_SEEDS[i]; 32]).public().to_bytes()
}

fn circle(i: usize) -> [u8; 32] {
    FeedWriter::from_seed(NET, [CIRCLE_SEEDS[i]; 32])
        .public()
        .to_bytes()
}

fn consortium() -> Consortium {
    let keys = (0..3)
        .map(|i| Member::from_seed([MEMBER_SEEDS[i]; 32]).public())
        .collect();
    Consortium::new(keys, 2)
}

fn tiers() -> Tiers {
    let circle = CIRCLE_SEEDS.map(|s| FeedWriter::from_seed(NET, [s; 32]).public());
    Tiers::new(NET, consortium(), &circle).expect("a valid tiered set")
}

/// Member `i` keeping its feeds and record in `root`, its duties every `every`.
fn config(i: usize, root: &Path, every: Duration) -> Config {
    let seed = [MEMBER_SEEDS[i]; 32];
    Config {
        writers: tiers().writers().clone(),
        sync_every: Duration::from_millis(200),
        own: Some(Own {
            writer: FeedWriter::from_seed(NET, seed),
            dir: root.join("own"),
        }),
        circle: Some(Own {
            writer: FeedWriter::from_seed(NET, [CIRCLE_SEEDS[i]; 32]),
            dir: root.join("circle"),
        }),
        store: None,
        member: Some(MemberRole {
            member: Member::from_seed(seed),
            index: i,
            consortium: consortium(),
            every,
            cuts_per_epoch: 2,
            patience: 10,
            view_patience: 20,
        }),
        consortium: None,
        tiers: Some(tiers()),
        relay: None,
        gateway: None,
        identity: Some(root.join("identity")),
    }
}

/// Member `i`'s record in `root` with the feeds' `states`.
fn record(i: usize, root: &Path, states: [State; 2]) {
    let record = Record {
        states,
        ..Record::new(public(i), circle(i))
    };
    std::fs::write(root.join("identity"), record.encode()).unwrap();
}

fn on_disk(root: &Path) -> Record {
    match read(&root.join("identity")) {
        Found::Present(record) => record,
        other => panic!("no readable record: {other:?}"),
    }
}

/// Writes `objects` into the feed files in `dir`, as the writer's publications leave them.
fn append(dir: &Path, objects: &[&str]) -> Vec<Entry> {
    std::fs::create_dir_all(dir).unwrap();
    let (mut log, _) = DurableLog::open(&dir.join("log")).unwrap();
    let (mut store, _) = ObjectStore::open(&dir.join("objects")).unwrap();
    objects
        .iter()
        .map(|o| log.append(store.put(o.as_bytes()).unwrap()).unwrap())
        .collect()
}

fn transport(i: usize) -> Keypair {
    Keypair::ed25519_from_bytes([TRANSPORT_SEEDS[i]; 32]).unwrap()
}

async fn start(i: usize, config: Config) -> (Handle, Multiaddr) {
    let h = Handle::spawn(transport(i), config).expect("a node starts");
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

/// Member 1 with a new identity, to sync with.
async fn peer() -> (Handle, Multiaddr) {
    let root = scratch("peer");
    record(1, &root, [State::Unused; 2]);
    start(1, config(1, &root, IDLE)).await
}

/// Member 0 on `root`, connected to `peer` and caught up on both feeds.
async fn joined(root: &Path, peer: &Multiaddr) -> Handle {
    let (h, _) = start(0, config(0, root, IDLE)).await;
    h.dial(peer.clone()).await.unwrap();
    until("member 0 catches up", || async {
        let s = h.status().await;
        s.caught_up && s.circle_caught_up
    })
    .await;
    h
}

fn own_ids(r: &Replica) -> usize {
    r.ids()
        .filter(|id| id.writer == public(0) || id.writer == circle(0))
        .count()
}

async fn refused(h: &Handle, stop: Stop) {
    assert_eq!(h.status().await.stopped, Some(stop));
    let refused = Err(PublishError::Stopped(stop));
    assert_eq!(h.publish(b"refused".to_vec()).await, refused);
    assert_eq!(h.publish_circle(b"refused".to_vec()).await, refused);
}

/// Member 0's `seq`-th entry, signed under `net`, over objects `tag-0`..`tag-seq`.
fn signed(net: [u8; 32], seq: usize, tag: &str) -> (SignedEntry, Vec<u8>) {
    let mut log = TransparencyLog::new();
    let mut last = Vec::new();
    for k in 0..=seq {
        last = format!("{tag}-{k}").into_bytes();
        log.append(cid(&last));
    }
    let entry = log.entries()[seq].clone();
    let writer = FeedWriter::from_seed(net, [MEMBER_SEEDS[0]; 32]);
    (writer.sign(&entry), last)
}

// A new identity writes its public feed, then its circle after a restart: one history (AT-NET-36).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn feeds_start_independently_and_keep_one_history() {
    let (m1, a1) = peer().await;
    let root = scratch("fresh");
    record(0, &root, [State::Unused; 2]);
    let m0 = joined(&root, &a1).await;
    assert_eq!(m0.status().await.stopped, None);
    assert_eq!(m0.publish(b"p0".to_vec()).await.unwrap().entry.seq, 0);
    assert_eq!(on_disk(&root).states, [State::Written, State::Unused]);
    m0.finished().await;
    until("member 0 has left", || async {
        m1.status().await.peers == 0
    })
    .await;
    let m0 = joined(&root, &a1).await;
    assert_eq!(m0.status().await.stopped, None);
    assert_eq!(
        m0.publish_circle(b"c0".to_vec()).await.unwrap().entry.seq,
        0
    );
    assert_eq!(m0.publish(b"p1".to_vec()).await.unwrap().entry.seq, 1);
    assert_eq!(on_disk(&root).states, [State::Written, State::Written]);
    until("member 1 holds member 0's three entries", || async {
        own_ids(&m1.replica().await) == 3
    })
    .await;
    assert!(m1.replica().await.equivocations().is_empty());
}

// A crash before the first signature: nothing made anew, a complete entry kept (AT-NET-36).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_crash_before_the_first_signature_resumes_it() {
    let (_m1, a1) = peer().await;
    let root = scratch("opening-empty");
    record(0, &root, [State::Opening, State::Unused]);
    let m0 = joined(&root, &a1).await;
    assert_eq!(m0.status().await.stopped, None);
    assert_eq!(on_disk(&root).states, [State::Opening, State::Unused]);
    assert_eq!(m0.publish(b"first".to_vec()).await.unwrap().entry.seq, 0);
    assert_eq!(on_disk(&root).states, [State::Written, State::Unused]);

    let (m1, a1) = peer().await;
    let root = scratch("opening-one");
    record(0, &root, [State::Unused, State::Opening]);
    let kept = append(&root.join("circle"), &["kept"]);
    let (m0, _) = start(0, config(0, &root, IDLE)).await;
    assert_eq!(on_disk(&root).states, [State::Unused, State::Written]);
    let held = m0.replica().await;
    let feed = held.feed(&circle(0));
    assert_eq!(feed.len(), 1);
    assert_eq!(feed[0].entry, kept[0]);
    assert_eq!(held.get(&feed[0].id()).unwrap().1, b"kept");
    m0.dial(a1).await.unwrap();
    until("the circle catches up", || async {
        m0.status().await.circle_caught_up
    })
    .await;
    let next = m0.publish_circle(b"next".to_vec()).await.unwrap();
    assert_eq!((next.entry.seq, next.entry.prev), (1, kept[0].hash));
    until("member 1 holds both", || async {
        m1.replica().await.feed(&circle(0)).len() == 2
    })
    .await;
    assert!(m1.replica().await.equivocations().is_empty());
}

// A detected loss or inconsistency stops both feeds unsigned, across a restart (AT-NET-36).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn losses_and_inconsistencies_stop_durably() {
    type Case = (
        [State; 2],
        &'static [&'static str],
        &'static [&'static str],
        Stop,
    );
    let cases: [Case; 3] = [
        (
            [State::Written, State::Unused],
            &[],
            &[],
            Stop::Lost(Tier::Public),
        ),
        (
            [State::Written, State::Unused],
            &["a"],
            &["x"],
            Stop::Inconsistent(Tier::Circle),
        ),
        (
            [State::Opening, State::Written],
            &["a", "b"],
            &["x"],
            Stop::Inconsistent(Tier::Public),
        ),
    ];
    for (states, own, circle, stop) in cases {
        let root = scratch("stopped");
        record(0, &root, states);
        append(&root.join("own"), own);
        append(&root.join("circle"), circle);
        for _ in 0..2 {
            let (m0, _) = start(0, config(0, &root, IDLE)).await;
            refused(&m0, stop).await;
            assert_eq!(own_ids(&m0.replica().await), 0, "{stop:?} signed nothing");
            assert_eq!(on_disk(&root).stop, Some(stop));
            m0.finished().await;
        }
    }
}

// Damaged files stop; a missing, unreadable or foreign record stops, never rewritten (AT-NET-36).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn damage_and_missing_or_foreign_records_stop() {
    let root = scratch("damaged");
    record(0, &root, [State::Written, State::Unused]);
    append(&root.join("own"), &["a", "b", "c"]);
    let path = root.join("own/log");
    let mut bytes = std::fs::read(&path).unwrap();
    bytes[16] ^= 1;
    std::fs::write(&path, bytes).unwrap();
    let (m0, _) = start(0, config(0, &root, IDLE)).await;
    let aside = [
        root.join("own/log.damaged-0"),
        root.join("own/objects.damaged-0"),
    ];
    assert_eq!(m0.set_aside(), aside);
    refused(&m0, Stop::SetAside(Tier::Public)).await;
    m0.finished().await;
    let (m0, _) = start(0, config(0, &root, IDLE)).await;
    refused(&m0, Stop::SetAside(Tier::Public)).await;
    m0.finished().await;

    let root = scratch("missing");
    append(&root.join("own"), &["a"]);
    let (m0, _) = start(0, config(0, &root, IDLE)).await;
    refused(&m0, Stop::Missing).await;
    assert_eq!(own_ids(&m0.replica().await), 0);
    m0.finished().await;
    assert!(!root.join("identity").exists());

    let root = scratch("unreadable");
    let mut bytes = Record::new(public(0), circle(0)).encode();
    bytes[80] = 7;
    std::fs::write(root.join("identity"), &bytes).unwrap();
    let (m0, _) = start(0, config(0, &root, IDLE)).await;
    refused(&m0, Stop::Unreadable).await;
    m0.finished().await;
    assert_eq!(std::fs::read(root.join("identity")).unwrap(), bytes);

    let root = scratch("foreign");
    record(1, &root, [State::Unused; 2]);
    let foreign = Handle::spawn(transport(0), config(0, &root, IDLE));
    assert!(matches!(foreign, Err(StartError::Identity)));
}

/// Member 0 started on another thread: a start still blocked after 10 s fails the test.
async fn start_within(config: Config) -> Handle {
    let runtime = tokio::runtime::Handle::current();
    let (tx, rx) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let _entered = runtime.enter();
        let _ = tx.send(Handle::spawn(transport(0), config));
    });
    let started = tokio::time::timeout(Duration::from_secs(10), rx).await;
    started
        .expect("the start is not blocked")
        .unwrap()
        .expect("a node starts")
}

// A record that is a FIFO or a directory stops the start unblocked, left as found (AT-NET-36).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_non_regular_record_stops_without_blocking() {
    let mut kinds = vec!["directory"];
    if cfg!(unix) {
        kinds.push("fifo");
    }
    for kind in kinds {
        let root = scratch(kind);
        append(&root.join("own"), &["a"]);
        let path = root.join("identity");
        if kind == "directory" {
            std::fs::create_dir(&path).unwrap();
        } else {
            let made = std::process::Command::new("mkfifo").arg(&path).status();
            assert!(made.unwrap().success());
        }
        let m0 = start_within(config(0, &root, IDLE)).await;
        refused(&m0, Stop::Unreadable).await;
        assert_eq!(own_ids(&m0.replica().await), 0);
        m0.finished().await;
        let kind_now = std::fs::symlink_metadata(&path).unwrap().file_type();
        assert_eq!(kind_now.is_dir(), kind == "directory");
        assert!(!kind_now.is_file() && !kind_now.is_symlink());
        assert!(!root.join("identity.next").exists());
    }
}

// Only a verified own entry past or against the files stops, outliving its evidence (AT-NET-36).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn verified_own_entries_stop_the_member() {
    let root = scratch("own-entry");
    record(0, &root, [State::Written, State::Unused]);
    append(&root.join("own"), &["e-0"]);
    let (m0, _) = start(0, config(0, &root, IDLE)).await;
    assert_eq!(m0.replica().await.feed(&public(0)).len(), 1);
    let (elsewhere, object) = signed([37; 32], 1, "e");
    assert!(m0.insert(elsewhere, object).await.is_err());
    assert_eq!(m0.status().await.stopped, None);
    let (beyond, object) = signed(NET, 1, "e");
    assert!(m0.insert(beyond, object).await.is_ok());
    refused(&m0, Stop::OwnEntry(Tier::Public)).await;
    assert_eq!(on_disk(&root).stop, Some(Stop::OwnEntry(Tier::Public)));
    m0.finished().await;
    let (m0, _) = start(0, config(0, &root, IDLE)).await;
    assert_eq!(m0.replica().await.len(), 0);
    refused(&m0, Stop::OwnEntry(Tier::Public)).await;

    let root = scratch("own-fork");
    record(0, &root, [State::Written, State::Unused]);
    append(&root.join("own"), &["e-0"]);
    let (m0, _) = start(0, config(0, &root, IDLE)).await;
    let (against, object) = signed(NET, 0, "z");
    assert!(m0.insert(against, object).await.is_ok());
    refused(&m0, Stop::OwnEntry(Tier::Public)).await;
}

/// Three busy members; member 0 stops by a verified own entry, or by a failed feed write.
async fn busy_member_stops(by_failed_write: bool) {
    let roots: Vec<PathBuf> = (0..3).map(|_| scratch("busy")).collect();
    let mut members = Vec::new();
    for (i, root) in roots.iter().enumerate() {
        record(i, root, [State::Unused; 2]);
        members.push(start(i, config(i, root, BUSY)).await);
    }
    for (h, _) in &members[1..] {
        h.dial(members[0].1.clone()).await.unwrap();
    }
    let (m0, m1) = (&members[0].0, &members[1].0);
    until("two cuts", || async { m0.status().await.cuts.len() >= 2 }).await;
    let stop = if by_failed_write {
        let lost = b"busy-lost".to_vec();
        faults::arm(On::Object, cid(&lost), Fault::Write);
        assert!(matches!(
            m0.publish(lost).await,
            Err(PublishError::Store(_))
        ));
        Stop::Failed
    } else {
        let len = m0.replica().await.feed(&public(0)).len();
        let (beyond, object) = signed(NET, len + 3, "gap");
        assert!(m0.insert(beyond, object).await.is_ok());
        Stop::OwnEntry(Tier::Public)
    };
    refused(m0, stop).await;
    let mine = own_ids(&m0.replica().await);
    let cuts = m1.status().await.cuts.len();
    until("the others collect three more cuts", || async {
        m1.status().await.cuts.len() >= cuts + 3
    })
    .await;
    until("the stopped member reads them", || async {
        m0.status().await.cuts.len() >= cuts + 3
    })
    .await;
    assert_eq!(own_ids(&m0.replica().await), mine);
    assert!(own_ids(&m1.replica().await) <= mine);
    refused(m0, stop).await;
}

// A stopped member adds no entry, by publication or duty, while it reads on (AT-NET-36).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_stopped_member_adds_no_entry() {
    busy_member_stops(false).await;
}

// A failed feed write stops a busy member's duties while the others collect cuts (AT-NET-36).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_failed_feed_write_stops_the_duties() {
    busy_member_stops(true).await;
}

fn feed_key(tier: Tier) -> [u8; 32] {
    match tier {
        Tier::Public => public(0),
        Tier::Circle => circle(0),
    }
}

async fn publish_on(h: &Handle, tier: Tier, object: &[u8]) -> Result<SignedEntry, PublishError> {
    match tier {
        Tier::Public => h.publish(object.to_vec()).await,
        Tier::Circle => h.publish_circle(object.to_vec()).await,
    }
}

/// The log's length in bytes and the objects of member 0's `tier` files, read unchanged.
fn held(root: &Path, tier: Tier) -> (u64, Vec<Vec<u8>>) {
    let dir = root.join(if tier == Tier::Public {
        "own"
    } else {
        "circle"
    });
    let log = std::fs::metadata(dir.join("log")).unwrap().len();
    let objects = read_objects(&dir.join("objects")).unwrap();
    (log, objects.into_iter().map(|(_, o)| o).collect())
}

/// Member 0 fails, by `fault`, the write `on` of `tier`'s entry after `before` others, then
/// restarts on what its files hold.
async fn failed_write(tier: Tier, before: usize, on: On, fault: Fault) {
    let case = format!("fault-{tier:?}-{before}-{on:?}-{fault:?}");
    let object = |k: &str| format!("{case}-{k}").into_bytes();
    let (m1, a1) = peer().await;
    let root = scratch("feed-fault");
    record(0, &root, [State::Unused; 2]);
    let m0 = joined(&root, &a1).await;
    let mut objects: Vec<Vec<u8>> = (0..before).map(|k| object(&k.to_string())).collect();
    for o in &objects {
        publish_on(&m0, tier, o).await.unwrap();
    }
    let lost = object("lost");
    faults::arm(on, cid(&lost), fault);
    let failed = publish_on(&m0, tier, &lost).await;
    assert!(
        matches!(failed, Err(PublishError::Store(_))),
        "{case}: {failed:?}"
    );
    refused(&m0, Stop::Failed).await;
    let len = |r: &Replica| r.feed(&feed_key(tier)).len();
    assert_eq!(len(&m0.replica().await), before, "{case}");
    let i = usize::from(tier == Tier::Circle);
    let mut states = [State::Unused; 2];
    states[i] = if before == 0 {
        State::Opening
    } else {
        State::Written
    };
    assert_eq!(on_disk(&root).states, states, "{case}");
    until("member 1 holds the earlier entries", || async {
        len(&m1.replica().await) == before
    })
    .await;
    m0.finished().await;
    until("member 0 has left", || async {
        m1.status().await.peers == 0
    })
    .await;
    assert_eq!(len(&m1.replica().await), before, "{case}");

    let kept = on == On::Log && fault == Fault::Sync;
    if on == On::Log || fault == Fault::Sync {
        objects.push(lost.clone());
    }
    let entries = before + usize::from(kept);
    assert_eq!(
        held(&root, tier),
        (16 + 104 * entries as u64, objects),
        "{case}"
    );
    let m0 = joined(&root, &a1).await;
    assert_eq!(m0.status().await.stopped, None, "{case}");
    if kept {
        states[i] = State::Written;
    }
    assert_eq!(on_disk(&root).states, states, "{case}");
    assert_eq!(len(&m0.replica().await), entries, "{case}");
    let next = publish_on(&m0, tier, &object("next")).await.unwrap();
    assert_eq!(next.entry.seq as usize, entries, "{case}");
    states[i] = State::Written;
    assert_eq!(on_disk(&root).states, states, "{case}");
    until("member 1 holds the feed", || async {
        len(&m1.replica().await) == entries + 1
    })
    .await;
    let replica = m1.replica().await;
    assert!(replica.equivocations().is_empty(), "{case}");
    let at = replica.feed(&feed_key(tier))[before].id();
    let expected = if kept { lost } else { object("next") };
    assert_eq!(replica.get(&at).unwrap().1.to_vec(), expected, "{case}");
}

/// Every write and sync fault of the object and the log, first entry and later, on `tier`.
async fn failed_writes(tier: Tier) {
    for before in [0, 1] {
        for on in [On::Object, On::Log] {
            for fault in [Fault::Write, Fault::Sync] {
                failed_write(tier, before, on, fault).await;
            }
        }
    }
}

// A failed public-feed write signs nothing and stops both feeds; a restart resumes (AT-NET-36).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failed_public_feed_writes_stop_until_a_restart() {
    failed_writes(Tier::Public).await;
}

// A failed circle-feed write signs nothing and stops both feeds; a restart resumes (AT-NET-36).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failed_circle_feed_writes_stop_until_a_restart() {
    failed_writes(Tier::Circle).await;
}

// A record write failing after its rename signs nothing; a restart resumes one history (AT-NET-36).
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_record_write_failing_after_its_rename_signs_nothing() {
    use std::os::unix::fs::PermissionsExt;
    let mode = |root: &Path, m| {
        std::fs::set_permissions(root, std::fs::Permissions::from_mode(m)).unwrap()
    };
    let (m1, a1) = peer().await;
    let root = scratch("unsynced");
    record(0, &root, [State::Unused; 2]);
    let m0 = joined(&root, &a1).await;
    mode(&root, 0o300);
    if std::fs::File::open(&root).is_ok() {
        mode(&root, 0o700);
        eprintln!("skipped: this user opens a directory without read permission");
        return;
    }
    let failed = m0.publish(b"lost".to_vec()).await;
    mode(&root, 0o700);
    assert_eq!(failed, Err(PublishError::Stopped(Stop::Failed)));
    refused(&m0, Stop::Failed).await;
    assert_eq!(own_ids(&m0.replica().await), 0);
    assert_eq!(on_disk(&root).states, [State::Opening, State::Unused]);
    m0.finished().await;
    until("member 0 has left", || async {
        m1.status().await.peers == 0
    })
    .await;
    let m0 = joined(&root, &a1).await;
    assert_eq!(m0.publish(b"first".to_vec()).await.unwrap().entry.seq, 0);
    assert_eq!(on_disk(&root).states, [State::Written, State::Unused]);
    until("member 1 holds it", || async {
        m1.replica().await.feed(&public(0)).len() == 1
    })
    .await;
    assert_eq!(own_ids(&m1.replica().await), 1);
}
