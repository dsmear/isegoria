//! Nodes as separate processes (`docs/04` §A node process, T78): three members and a relay,
//! each started from its configuration file, agree on the cuts and the beacons; a member
//! killed and started again rejoins without equivocating.

use network::consortium::Member;
use node::{hex, parse};
use p2p::Status;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const NET: [u8; 32] = [6; 32];
const SEEDS: [u8; 3] = [30, 31, 32];

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("processes")
        .join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn quoted(list: &[String]) -> String {
    let items: Vec<String> = list.iter().map(|s| format!("\"{s}\"")).collect();
    format!("[{}]", items.join(", "))
}

/// The configuration of node `i` (0–2 the members, 3 the relay) under `root`.
fn config(root: &Path, i: usize, addrs: &[String]) -> PathBuf {
    let keys: Vec<String> = SEEDS
        .iter()
        .map(|s| hex(&Member::from_seed([*s; 32]).public().to_bytes()))
        .collect();
    let peers: Vec<String> = (0..addrs.len())
        .filter(|j| *j != i)
        .map(|j| addrs[j].clone())
        .collect();
    let dir = root.join(format!("node{i}"));
    let mut text = format!(
        "network_id = \"{}\"\ndata_dir = \"{}\"\nlisten = [\"{}\"]\npeers = {}\n\
         writers = {}\nsync_every_ms = 300\nstatus_every_ms = 200\n",
        hex(&NET),
        dir.display(),
        addrs[i],
        quoted(&peers),
        quoted(&keys),
    );
    if i < 3 {
        let key = root.join(format!("key{i}"));
        std::fs::write(&key, hex(&[SEEDS[i]; 32])).unwrap();
        text += &format!("key_file = \"{}\"\n", key.display());
    }
    text += &format!(
        "\n[consortium]\nmembers = {}\nthreshold = 2\n",
        quoted(&keys)
    );
    if i < 3 {
        text += &format!(
            "\n[member]\nindex = {i}\nevery_ms = 100\ncuts_per_epoch = 2\npatience = 10\n\
             view_patience = 20\n"
        );
    }
    let path = root.join(format!("node{i}.toml"));
    std::fs::write(&path, text).unwrap();
    path
}

/// The node processes, killed when the test ends however it ends.
struct Nodes {
    root: PathBuf,
    configs: Vec<PathBuf>,
    children: Vec<Option<Child>>,
}

impl Nodes {
    fn start(name: &str) -> Nodes {
        let mut nodes = Nodes::configured(name);
        for i in 0..4 {
            nodes.spawn(i);
        }
        nodes
    }

    fn configured(name: &str) -> Nodes {
        let root = scratch(name);
        let addrs: Vec<String> = (0..4)
            .map(|_| format!("/ip4/127.0.0.1/tcp/{}", free_port()))
            .collect();
        let configs: Vec<PathBuf> = (0..4).map(|i| config(&root, i, &addrs)).collect();
        Nodes {
            root,
            configs,
            children: (0..4).map(|_| None).collect(),
        }
    }

    fn spawn(&mut self, i: usize) {
        let log = std::fs::File::create(self.root.join(format!("node{i}.log"))).unwrap();
        let child = Command::new(env!("CARGO_BIN_EXE_isegoria-node"))
            .arg(&self.configs[i])
            .stdout(Stdio::null())
            .stderr(log)
            .spawn()
            .expect("the node starts");
        self.children[i] = Some(child);
    }

    fn kill(&mut self, i: usize) {
        if let Some(mut child) = self.children[i].take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    fn status(&self, i: usize) -> Option<Status> {
        let path = self.root.join(format!("node{i}")).join("status");
        parse(&std::fs::read_to_string(path).ok()?)
    }

    /// Waits, at most 15 s, until node `i` reports `peers` peers.
    fn until_peers(&self, i: usize, peers: usize) {
        let deadline = Instant::now() + Duration::from_secs(15);
        while self.status(i).is_none_or(|s| s.peers != peers) {
            assert!(Instant::now() < deadline, "{:?}", self.status(i));
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Waits until every node in `who` reports at least `cuts` cuts and `beacons` rounds.
    fn until(&self, who: &[usize], cuts: usize, beacons: usize, secs: u64) -> Vec<Status> {
        let deadline = Instant::now() + Duration::from_secs(secs);
        loop {
            let all: Vec<Option<Status>> = who.iter().map(|&i| self.status(i)).collect();
            let done = all.iter().all(|s| {
                s.as_ref()
                    .is_some_and(|s| s.cuts.len() >= cuts && s.beacons.len() >= beacons)
            });
            if done {
                return all.into_iter().flatten().collect();
            }
            assert!(Instant::now() < deadline, "no progress: {all:?}");
            std::thread::sleep(Duration::from_millis(200));
        }
    }
}

impl Drop for Nodes {
    fn drop(&mut self) {
        for i in 0..self.children.len() {
            self.kill(i);
        }
    }
}

fn agree(statuses: &[Status], cuts: usize, beacons: usize) {
    for s in statuses {
        assert_eq!(s.equivocations, 0, "a writer equivocated");
        assert!(!s.conflict);
        let digests = |s: &Status| s.cuts[..cuts].iter().map(|c| c.digest).collect::<Vec<_>>();
        assert_eq!(digests(s), digests(&statuses[0]), "the cuts differ");
        assert_eq!(s.beacons[..beacons], statuses[0].beacons[..beacons]);
    }
}

/// AT-NET-21: three member processes and a relay reach the same cuts and beacon values over
/// two epochs; a member killed and started again rejoins without equivocating.
#[test]
fn at_net_21_node_processes_agree_and_a_restart_rejoins() {
    let mut nodes = Nodes::start("agree");
    let all = [0, 1, 2, 3];
    let first = nodes.until(&all, 5, 2, 90);
    agree(&first, 5, 2);
    let values: Vec<_> = first[0].beacons[..2].iter().map(|(v, _)| *v).collect();
    assert!(values.iter().all(|v| v.is_some()) && values[0] != values[1]);

    nodes.kill(1);
    let before = nodes.status(0).unwrap().cuts.len();
    nodes.until(&[0, 2, 3], before + 2, 0, 90);
    nodes.spawn(1);
    let after = nodes.until(&all, before + 6, 4, 120);
    agree(&after, before + 6, 4);
}

/// AT-NET-45: a node with a peer still reaches a configured one that starts later and dials no one.
#[test]
fn at_net_44_a_node_with_a_peer_reaches_a_later_one() {
    let mut nodes = Nodes::configured("late-peer");
    let text = std::fs::read_to_string(&nodes.configs[3]).unwrap();
    let dials = text.lines().find(|l| l.starts_with("peers = ")).unwrap();
    std::fs::write(&nodes.configs[3], text.replace(dials, "peers = []")).unwrap();
    nodes.spawn(1);
    nodes.spawn(0);
    nodes.until_peers(0, 1);
    nodes.spawn(3);
    nodes.until_peers(0, 2);
}

/// AT-NET-21: a SIGTERM that arrives once the data directory is locked stops the node cleanly.
#[cfg(unix)]
#[test]
fn at_net_21_a_sigterm_during_the_start_stops_the_node() {
    for round in 0..10 {
        let root = scratch(&format!("early-term-{round}"));
        let addrs: Vec<String> = (0..4)
            .map(|_| format!("/ip4/127.0.0.1/tcp/{}", free_port()))
            .collect();
        let log = std::fs::File::create(root.join("node3.log")).unwrap();
        let mut child = Command::new(env!("CARGO_BIN_EXE_isegoria-node"))
            .arg(config(&root, 3, &addrs))
            .stdout(Stdio::null())
            .stderr(log)
            .spawn()
            .expect("the node starts");
        let deadline = Instant::now() + Duration::from_secs(20);
        while !root.join("node3").join("lock").exists() {
            assert!(
                Instant::now() < deadline,
                "the node never locked its directory"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        let sent = Command::new("kill")
            .args(["-TERM", &child.id().to_string()])
            .status()
            .unwrap();
        assert!(sent.success());
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                panic!("round {round}: the node did not stop");
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        assert!(status.success(), "round {round}: node exited {status}");
    }
}
