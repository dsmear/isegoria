//! Gate 2's rehearsal on one machine (`docs/05` §Gate 2's rehearsal): the processes of gate 1,
//! each node in a network namespace of its own, under delay, loss, partitions and a crash.
#![cfg(target_os = "linux")]

mod support;

use p2p::Status;
use std::fs;
use std::time::{Duration, Instant};
use support::*;

const NETEM: &str = "delay 30ms 10ms loss 1% duplicate 1%";
const ALL: [usize; N] = [0, 1, 2, 3];

fn cuts(g: &Gate, who: &[usize]) -> Vec<usize> {
    let count = |&i: &usize| g.status(i).expect("a status").cuts.len();
    who.iter().map(count).collect()
}

/// Whether the longest history among `who` gains no cut over four seconds, two seconds after
/// the call.
fn still(g: &Gate, who: &[usize]) -> bool {
    let longest = |g: &Gate| cuts(g, who).into_iter().max();
    std::thread::sleep(Duration::from_secs(2));
    let before = longest(g);
    std::thread::sleep(Duration::from_secs(4));
    before == longest(g)
}

fn agreed(s: &Status) -> bool {
    healthy(s) && s.stopped.is_none()
}

fn phase(q: &'static str) -> impl Fn(&Status) -> bool {
    move |s| agreed(s) && s.state.iter().any(|l| l == q)
}

// Gate 2 rehearsed: a review under netem, partitions, a kill, the relay cut (AT-T79-66–70).
#[test]
#[ignore = "run by scripts/gate2-rehearsal.sh, in its namespaces"]
fn at_t79_66_gate_2_rehearsed_in_namespaces() {
    let mapped = fs::read_to_string("/proc/self/uid_map").unwrap_or_default();
    let host = mapped.split_whitespace().collect::<Vec<_>>() == ["0", "0", "4294967295"];
    let inside = std::env::var_os("ISEGORIA_REHEARSAL").is_some() && !host;
    assert!(inside, "run scripts/gate2-rehearsal.sh");
    let started = Instant::now();
    let mut g = Gate::new("gate2-rehearsal", Net::Namespaces);
    g.topology(NETEM);
    g.setup();
    g.members("members caught up", &ALL, |s| {
        s.caught_up && s.circle_caught_up && s.cuts.len() >= 2 && agreed(s)
    });
    g.members("the relay bound to every member", &[RELAY], |s| {
        s.bound == N
    });
    g.snapshot("started");
    g.credentials();
    let genesis = g.path("genesis.bin");

    let reviewers: Vec<usize> = (1..PERSONS).collect();
    for &k in &reviewers {
        let (cred, event) = (g.person(k, "credential"), g.person(k, "join"));
        g.ok(CLIENT, &["--join", &genesis, &cred, &event]);
        g.send(k, "join", k % N);
    }
    g.send(1, "join", 2);
    let question = g.person(0, "question.toml");
    let text = format!(
        "context = \"{CONTEXT}\"\nstem = \"{STEM}\"\nalternatives = {}\nkey = 0\n",
        quoted(&ALTERNATIVES.map(String::from))
    );
    fs::write(&question, text).unwrap();
    let (cred, event) = (g.person(0, "credential"), g.person(0, "deposit"));
    g.ok(
        CLIENT,
        &["--deposit", &genesis, &cred, "0", &question, &event],
    );
    g.send(0, "deposit", 0);
    for &k in &reviewers {
        let out = g.settled(k, "join", (k + 1) % N);
        let applied = out.lines().filter(|l| l.ends_with(" applied")).count();
        assert_eq!(applied, 1, "AT-T79-66: {out}");
    }
    let twice = g.settled(1, "join", 3);
    assert!(twice.starts_with("entries 2\n"), "AT-T79-66: {twice}");
    assert!(twice.contains("AlreadyInPool"), "AT-T79-66: {twice}");
    let deposited = g.settled(0, "deposit", 1);
    assert!(
        deposited.contains(" applied question 0\n"),
        "AT-T79-66: {deposited}"
    );
    for i in 0..T {
        g.request(i, ["close-epoch", "0"]);
    }
    let closed = g.members("epoch 0 closed, its round finished", &ALL, |s| {
        !s.digests.is_empty() && s.beacons.first().is_some_and(|b| b.0.is_some())
    });
    for s in &closed {
        assert_eq!(s.digests[0], closed[0].digests[0], "AT-T79-66");
        assert_eq!(s.beacons[0], closed[0].beacons[0], "AT-T79-66");
        assert!(s.beacons[0].1.is_empty() && agreed(s), "AT-T79-66");
    }
    g.members("the draw applied", &ALL, phase("question 0 committing"));
    let mut drawn = Vec::new();
    for &k in &reviewers {
        let out = g.until("a reviewer's read", |g| {
            g.read(k, k % N, &["reviewer", "0"]).ok()
        });
        if fact(&out, "question") == Some("0") {
            drawn.push(k);
        }
    }
    assert_eq!(drawn.len(), 12, "AT-T79-66: 12 of the pool's 13 drawn");
    g.snapshot("epoch-0-closed");

    g.fault("partition 3-1, member 3 apart", "ip link set v3 master br1");
    let base = cuts(&g, &[0])[0];
    std::thread::sleep(Duration::from_secs(2));
    let split = cuts(&g, &[3])[0];
    for (n, &k) in drawn.iter().enumerate() {
        let [cred, opened, event] = ["credential", "commitment", "commit"].map(|f| g.person(k, f));
        g.ok(
            CLIENT,
            &["--commit", &genesis, &cred, "0", "0", &opened, &event],
        );
        g.send(k, "commit", n % T);
    }
    for (n, &k) in drawn.iter().enumerate() {
        let out = g.settled(k, "commit", (n + 1) % T);
        assert!(
            out.lines().any(|l| l.ends_with(" applied")),
            "AT-T79-67: {out}"
        );
    }
    for i in 0..T {
        g.request(i, ["close-commits", "0"]);
    }
    let three = [0, 1, 2];
    g.members("three progress", &three, phase("question 0 revealing"));
    assert!(cuts(&g, &[0])[0] >= base + 2, "AT-T79-67");
    assert!(
        still(&g, &[3]),
        "AT-T79-67: the member apart collected cuts"
    );
    let apart = g.status(3).unwrap();
    assert!(
        apart.cuts.len() == split && split < cuts(&g, &[0])[0] && agreed(&apart),
        "AT-T79-67"
    );
    g.snapshot("partition-3-1");
    g.fault("rejoin", "ip link set v3 master br0");
    let caught = cuts(&g, &[0])[0];
    g.members("member 3 caught up", &[3], |s| {
        s.cuts.len() >= caught && phase("question 0 revealing")(s)
    });
    g.snapshot("rejoined-3-1");

    g.fault(
        "partition 2-2, members 2 and 3 apart",
        "ip link set v2 master br1 && ip link set v3 master br1",
    );
    let first = drawn[0];
    let [cred, opened, event] = ["credential", "commitment", "reveal"].map(|f| g.person(first, f));
    g.ok(CLIENT, &["--reveal", &genesis, &cred, &opened, &event]);
    g.send(first, "reveal", 0);
    assert!(
        still(&g, &ALL),
        "AT-T79-68: a cut made with two members on each side"
    );
    let waiting = g.until("the reveal's status", |g| {
        g.read(first, 0, &["status", &event]).ok()
    });
    assert!(waiting.contains("pending"), "AT-T79-68: {waiting}");
    assert!(g.statuses().iter().all(agreed), "AT-T79-68");
    g.snapshot("partition-2-2");
    g.fault(
        "rejoin",
        "ip link set v2 master br0 && ip link set v3 master br0",
    );
    let held = cuts(&g, &[0])[0];
    g.members("progress after the rejoin", &ALL, |s| {
        s.cuts.len() >= held + 2 && agreed(s)
    });
    let counted = g.settled(first, "reveal", 1);
    assert!(
        counted.lines().any(|l| l.ends_with(" applied")),
        "AT-T79-68: {counted}"
    );
    g.snapshot("rejoined-2-2");

    let killed = g.status(2).unwrap();
    g.kill(2);
    let others = [0, 1, 3];
    for (n, &k) in drawn.iter().enumerate().skip(1) {
        let [cred, opened, event] = ["credential", "commitment", "reveal"].map(|f| g.person(k, f));
        g.ok(CLIENT, &["--reveal", &genesis, &cred, &opened, &event]);
        g.send(k, "reveal", others[n % T]);
    }
    for (n, &k) in drawn.iter().enumerate().skip(1) {
        let out = g.settled(k, "reveal", others[(n + 1) % T]);
        assert!(
            out.lines().any(|l| l.ends_with(" applied")),
            "AT-T79-69: {out}"
        );
    }
    let caught = cuts(&g, &[0])[0];
    g.start(2);
    g.note("node 2 started");
    let back = g.members("member 2 rebuilt", &[2], |s| {
        let caught_up = s.caught_up && s.circle_caught_up && s.cuts.len() >= caught;
        caught_up && !s.digests.is_empty() && agreed(s)
    });
    assert_eq!(
        back[0].digests[..killed.digests.len()],
        killed.digests[..],
        "AT-T79-69"
    );
    g.snapshot("member-2-restarted");

    g.fault("the relay's link down", "ip link set v4 down");
    let cut_off = g
        .read(0, 0, &["author"])
        .expect_err("AT-T79-70: a read through a cut relay");
    assert!(
        cut_off.contains("Channel(Transport("),
        "AT-T79-70: {cut_off}"
    );
    g.fault("the relay's link up", "ip link set v4 up");
    g.until("a read through the relay again", |g| {
        g.read(0, 0, &["author"]).ok()
    });
    for i in 1..=T {
        g.request(i, ["close", "0"]);
    }
    g.members("the outcome applied", &ALL, phase("question 0 closed"));
    let outcome = g.until("the author's read", |g| g.read(0, 3, &["author"]).ok());
    assert!(
        outcome.contains("question 0 closed accepted observation success 1 1 "),
        "AT-T79-70: {outcome}"
    );
    for i in [0, 1, 3] {
        g.request(i, ["close-epoch", "1"]);
    }
    let end = g.members("epoch 1 closed, its round finished", &ALL, |s| {
        s.digests.len() >= 2 && s.beacons.get(1).is_some_and(|b| b.0.is_some())
    });
    g.snapshot("epoch-1-closed");

    let all = g.statuses();
    let common = all.iter().map(|s| s.cuts.len()).min().unwrap();
    let digests = |s: &Status| {
        s.cuts[..common]
            .iter()
            .map(|c| c.digest)
            .collect::<Vec<_>>()
    };
    for s in &all {
        assert_eq!(s.digests[..2], end[0].digests[..2], "AT-T79-66");
        assert!(agreed(s), "AT-T79-66");
        assert_eq!(digests(s), digests(&all[0]), "AT-T79-66: the same cuts");
    }
    for i in 0..=N {
        g.stop(i);
        let log = g.root.join(format!("node-{i}.log"));
        fs::copy(&log, g.private().join(format!("node-{i}.log"))).unwrap();
    }
    g.snapshot("stopped");
    g.no_secret_in_evidence();
    let transcript = fs::read_to_string(g.private().join("transcript.txt")).unwrap();
    let events = transcript.lines().filter_map(|l| l.strip_prefix("event "));
    let mut circle: Vec<String> = [CONTEXT, STEM].map(String::from).to_vec();
    circle.extend(ALTERNATIVES.map(String::from));
    circle.extend(events.map(String::from));
    g.no_circle_content_shared(&circle);
    g.log(&format!(
        "gate 2's rehearsal done in {:?}\n",
        started.elapsed()
    ));
    println!("gate 2 rehearsal evidence: {}", g.evidence.display());
}
