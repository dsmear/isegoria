//! Gate 1 of Milestone 2 on one machine (`docs/05` §Gate 1 on one machine, `docs/10` §2.3): four
//! member processes and a relay under one operator on loopback, every step through the binaries.
#![cfg(unix)]

mod support;

use p2p::Status;
use protocol::answer_author::QuotaBands;
use std::fs;
use std::time::Instant;
use support::*;

// Gate 1: identities, genesis, issuance, a question through the review over the relay, digests,
// a restart, a duplicate, the evidence split into shared and private (AT-T79-50–59).
#[test]
fn at_t79_50_gate_1_on_one_machine() {
    let started = Instant::now();
    let mut g = Gate::new("gate1", Net::Loopback);

    g.setup();
    g.members("members caught up", &[0, 1, 2, 3], |s| {
        s.caught_up && s.circle_caught_up && s.cuts.len() >= 2 && healthy(s)
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
    for &k in &reviewers[1..] {
        let out = g.settled(k, "join", (k + 1) % N);
        assert_eq!(
            out.lines().filter(|l| l.ends_with(" applied")).count(),
            1,
            "AT-T79-52"
        );
    }
    let twice = g.settled(1, "join", 3);
    assert!(twice.starts_with("entries 2\n"), "AT-T79-53: {twice}");
    assert_eq!(twice.lines().filter(|l| l.ends_with(" applied")).count(), 1);
    let refused = twice.lines().filter(|l| l.contains(" refused "));
    assert_eq!(refused.filter(|l| l.contains("AlreadyInPool")).count(), 1);
    let deposited = g.settled(0, "deposit", 1);
    assert!(
        deposited.contains(" applied question 0\n"),
        "AT-T79-52: {deposited}"
    );
    g.snapshot("joined-and-deposited");

    for i in 0..T {
        g.request(i, ["close-epoch", "0"]);
    }
    let closed = g.members("epoch 0 closed, its round finished", &[0, 1, 2, 3], |s| {
        !s.digests.is_empty() && s.beacons.first().is_some_and(|b| b.0.is_some())
    });
    for s in &closed {
        assert_eq!(s.digests[0], closed[0].digests[0], "AT-T79-57");
        assert_eq!(s.beacons[0], closed[0].beacons[0]);
        assert!(s.beacons[0].1.is_empty() && healthy(s));
    }
    g.snapshot("epoch-0-closed");
    let all = [0, 1, 2, 3];
    let phase = |q: &'static str| move |s: &Status| s.state.iter().any(|l| l == q);
    g.members("the draw applied", &all, phase("question 0 committing"));
    let mut drawn = Vec::new();
    for &k in &reviewers {
        let out = g.read(k, k % N, &["reviewer", "0"]).unwrap();
        match fact(&out, "question") {
            Some("0") => drawn.push(k),
            Some("none") => {}
            other => panic!("AT-T79-54: {other:?} in {out}"),
        }
        if drawn.last() == Some(&k) {
            assert_eq!(fact(&out, "phase"), Some("commits-open"), "AT-T79-54");
            assert_eq!(fact(&out, "stem"), Some(format!("{STEM:?}").as_str()));
            assert_eq!(
                out.lines()
                    .filter(|l| l.starts_with("alternative "))
                    .count(),
                3
            );
            assert!(
                !out.lines().any(|l| l.starts_with("key")),
                "no key in the view"
            );
        }
    }
    assert_eq!(drawn.len(), 12, "AT-T79-54: 12 of the pool's 13 drawn");
    let before = g.read(0, 2, &["author"]).unwrap();
    assert_eq!(
        fact(&before, "quota"),
        Some("2"),
        "AT-T79-56: a new author's"
    );
    assert!(before.contains("question 0 committing\n"), "{before}");

    g.stop(3);
    let stopped_text = fs::read_to_string(g.dir(3).join("status")).unwrap();
    let stopped = node::parse(&stopped_text).unwrap();
    let progress = g.status(0).unwrap().cuts.len();
    g.snapshot("member-3-stopped");
    let answers = (0..12).map(|n| match n {
        10 => "1",
        11 => "dont-know",
        _ => "0",
    });
    for (n, (&k, answer)) in drawn.iter().zip(answers).enumerate() {
        let [cred, opened, event] = ["credential", "commitment", "commit"].map(|f| g.person(k, f));
        g.ok(
            CLIENT,
            &["--commit", &genesis, &cred, "0", answer, &opened, &event],
        );
        g.send(k, "commit", n % T);
    }
    for (n, &k) in drawn.iter().enumerate() {
        let out = g.settled(k, "commit", (n + 1) % T);
        assert!(
            out.lines().any(|l| l.ends_with(" applied")),
            "AT-T79-55: {out}"
        );
    }
    for i in 0..T {
        g.request(i, ["close-commits", "0"]);
    }
    g.until("reveals open, member 3 down", |g| {
        let out = g.read(drawn[0], 0, &["reviewer", "0"]).ok()?;
        (fact(&out, "phase") == Some("reveals-open")).then_some(())
    });
    let meanwhile = g.members("3 of 4 progress", &[0, 1, 2], |s| {
        s.cuts.len() >= progress + 2
    });
    assert!(meanwhile.iter().all(healthy), "AT-T79-58");
    let unchanged = fs::read_to_string(g.dir(3).join("status")).unwrap();
    assert_eq!(
        unchanged, stopped_text,
        "AT-T79-58: a stopped member writes nothing"
    );
    g.snapshot("reveals-open-member-3-down");

    let caught = g.status(0).unwrap().cuts.len();
    g.start(3);
    let back = g.members("member 3 rebuilt", &[3], |s| {
        s.circle_caught_up && s.cuts.len() >= caught && !s.digests.is_empty()
    });
    assert_eq!(
        back[0].digests[..stopped.digests.len()],
        stopped.digests[..],
        "AT-T79-58"
    );
    assert_eq!(back[0].digests[0], closed[0].digests[0]);
    g.snapshot("member-3-restarted");
    for (n, &k) in drawn.iter().enumerate() {
        let [cred, opened, event] = ["credential", "commitment", "reveal"].map(|f| g.person(k, f));
        g.ok(CLIENT, &["--reveal", &genesis, &cred, &opened, &event]);
        g.send(k, "reveal", 1 + n % T);
    }
    for (n, &k) in drawn.iter().enumerate() {
        let out = g.settled(k, "reveal", 1 + (n + 1) % T);
        assert!(
            out.lines().any(|l| l.ends_with(" applied")),
            "AT-T79-55: {out}"
        );
    }
    for i in 1..=T {
        g.request(i, ["close", "0"]);
    }
    let outcome = g.until("the outcome", |g| {
        let out = g.read(0, 3, &["author"]).ok()?;
        out.contains("question 0 closed").then_some(out)
    });
    let line = outcome
        .lines()
        .find(|l| l.starts_with("question 0 "))
        .unwrap();
    assert!(
        line.starts_with("question 0 closed accepted observation success 1 1 "),
        "AT-T79-55: {line}"
    );
    g.members("the outcome applied", &all, phase("question 0 closed"));
    for (n, &k) in drawn.iter().enumerate() {
        let out = g.read(k, n % N, &["reviewer", "0"]).unwrap();
        assert_eq!(fact(&out, "phase"), Some("closed"), "AT-T79-55");
        assert_eq!(fact(&out, "committed"), Some("true"));
        assert_eq!(fact(&out, "revealed"), Some("true"));
    }
    g.snapshot("question-0-closed");

    for i in [0, 1, 3] {
        g.request(i, ["close-epoch", "1"]);
    }
    let end = g.members("epoch 1 closed, its round finished", &[0, 1, 2, 3], |s| {
        s.digests.len() >= 2 && s.beacons.get(1).is_some_and(|b| b.0.is_some())
    });
    let after = g.read(0, 1, &["author"]).unwrap();
    let reputation = |out: &str| fact(out, "reputation").unwrap().parse::<f64>().unwrap();
    let (was, is) = (reputation(&before), reputation(&after));
    assert!(is > was, "AT-T79-56: {was} then {is}");
    let quota = fact(&after, "quota").unwrap().parse::<u32>().unwrap();
    assert_eq!(quota, QuotaBands::default().quota(is), "AT-T79-56");
    g.snapshot("epoch-1-closed");

    let all = g.statuses();
    for s in &all {
        assert_eq!(s.digests[..2], end[0].digests[..2], "AT-T79-57");
        assert!(healthy(s));
        let common = all.iter().map(|s| s.cuts.len()).min().unwrap();
        let cuts = |s: &Status| {
            s.cuts[..common]
                .iter()
                .map(|c| c.digest)
                .collect::<Vec<_>>()
        };
        assert_eq!(cuts(s), cuts(&all[0]), "the same cuts");
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
    let last = g.shared().join(format!("{:02}-stopped/m0.status", g.stage));
    let last = fs::read_to_string(last).unwrap();
    assert_eq!(last.lines().filter(|l| l.starts_with("digest ")).count(), 2);
    g.no_circle_content_shared(&circle);
    g.log(&format!("gate 1 done in {:?}\n", started.elapsed()));
    println!("gate 1 evidence: {}", g.evidence.display());
}
