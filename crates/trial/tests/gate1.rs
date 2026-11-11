//! Gate 1 of Milestone 2 on one machine (`docs/05` §Gate 1 on one machine, `docs/10` §2.3): four
//! member processes and a relay under one operator on loopback, every step through the binaries.
#![cfg(unix)]

use p2p::Status;
use protocol::answer_author::QuotaBands;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const NODE: &str = env!("CARGO_BIN_EXE_trial-node");
const CLIENT: &str = env!("CARGO_BIN_EXE_trial-client");
const ISSUER: &str = env!("CARGO_BIN_EXE_trial-test-issuer");
const N: usize = 4;
const T: usize = 3;
const RELAY: usize = N;
const PERSONS: usize = 14;
const LIMIT: Duration = Duration::from_secs(90);

/// A key `--new-identity` printed.
fn printed(text: &str, word: &str) -> String {
    let value = text
        .lines()
        .find_map(|l| l.strip_prefix(word)?.strip_prefix(' '));
    value
        .unwrap_or_else(|| panic!("no {word} in {text}"))
        .to_string()
}

fn quoted(list: &[String]) -> String {
    let items: Vec<String> = list.iter().map(|s| format!("\"{s}\"")).collect();
    format!("[{}]", items.join(", "))
}

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

fn peer_id(transport: &str) -> String {
    let bytes: Vec<u8> = (0..transport.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&transport[i..i + 2], 16).unwrap())
        .collect();
    let key = libp2p::identity::ed25519::PublicKey::try_from_bytes(&bytes).unwrap();
    libp2p::identity::PublicKey::from(key)
        .to_peer_id()
        .to_string()
}

struct Keys {
    consortium: String,
    circle: String,
    hpke: String,
    transport: String,
}

const CONTEXT: &str = "gate 1, a synthetic question";
const STEM: &str = "Which of these numbers is even?";
const ALTERNATIVES: [&str; 3] = ["two, synthetic", "three, synthetic", "five, synthetic"];

/// The first words of the lines the client prints for events and reads.
const PRINTED: [&str; 14] = [
    "entries ",
    "entry ",
    "question ",
    "phase ",
    "committed ",
    "revealed ",
    "context ",
    "stem ",
    "alternative ",
    "reputation ",
    "quota ",
    "event ",
    "logged ",
    "request ",
];

/// The status lines the trial's shared evidence may hold: public facts and digests.
const PUBLIC: [&str; 14] = [
    "caught_up",
    "circle_caught_up",
    "peers",
    "bound",
    "equivocations",
    "conflict",
    "duties_halted",
    "stopped",
    "synced",
    "circle_synced",
    "cut",
    "beacon",
    "digest",
    "refused_cut",
];

fn public_lines(status: &str) -> String {
    let public = |l: &&str| PUBLIC.contains(&l.split(' ').next().unwrap_or_default());
    status
        .lines()
        .filter(public)
        .map(|l| format!("{l}\n"))
        .collect()
}

/// One gate-1 run: its files under `root`, its evidence under `root/evidence/shared` and
/// `root/evidence/private`, the processes killed when it ends however it ends.
struct Gate {
    root: PathBuf,
    evidence: PathBuf,
    stage: usize,
    keys: Vec<Keys>,
    ports: Vec<u16>,
    issuer: String,
    lines: String,
    announcements: Vec<String>,
    children: Vec<Option<Child>>,
}

impl Gate {
    fn new() -> Gate {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join("gate1")
            .join(format!("run-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("persons")).unwrap();
        let evidence = root.join("evidence");
        for part in ["shared", "private"] {
            fs::create_dir_all(evidence.join(part)).unwrap();
        }
        Gate {
            root,
            evidence,
            stage: 0,
            keys: Vec::new(),
            ports: (0..=N).map(|_| free_port()).collect(),
            issuer: String::new(),
            lines: String::new(),
            announcements: Vec::new(),
            children: (0..=N).map(|_| None).collect(),
        }
    }

    fn path(&self, name: &str) -> String {
        self.root.join(name).display().to_string()
    }

    fn dir(&self, i: usize) -> PathBuf {
        match i {
            RELAY => self.root.join("relay"),
            _ => self.root.join(format!("m{i}")),
        }
    }

    fn config(&self, i: usize) -> String {
        match i {
            RELAY => self.path("relay.toml"),
            _ => self.path(&format!("m{i}.toml")),
        }
    }

    fn person(&self, k: usize, file: &str) -> String {
        self.path(&format!("persons/p{k}/{file}"))
    }

    fn shared(&self) -> PathBuf {
        self.evidence.join("shared")
    }

    fn private(&self) -> PathBuf {
        self.evidence.join("private")
    }

    fn log(&self, text: &str) {
        let text = text.replace(&self.root.display().to_string(), "$RUN");
        let mut file = (OpenOptions::new().create(true).append(true))
            .open(self.private().join("transcript.txt"))
            .unwrap();
        file.write_all(text.as_bytes()).unwrap();
    }

    /// Runs `bin` with `args`, its command line and output added to the transcript: its standard
    /// output, or its standard error when it exits otherwise than 0.
    fn call(&self, bin: &str, args: &[&str]) -> Result<String, String> {
        let output = Command::new(bin).args(args).output().unwrap();
        let name = Path::new(bin).file_name().unwrap().to_string_lossy();
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let mut text = format!("$ {name} {}\n{stdout}", args.join(" "));
        if !output.status.success() {
            text += &format!("[{}] {stderr}", output.status);
        }
        self.log(&text);
        match output.status.success() {
            true => Ok(stdout),
            false => Err(stderr),
        }
    }

    fn ok(&self, bin: &str, args: &[&str]) -> String {
        let result = self.call(bin, args);
        result.unwrap_or_else(|e| panic!("{bin} {args:?}: {e}"))
    }

    fn status(&self, i: usize) -> Option<Status> {
        node::parse(&fs::read_to_string(self.dir(i).join("status")).ok()?)
    }

    fn statuses(&self) -> Vec<Status> {
        (0..N).map(|i| self.status(i).expect("a status")).collect()
    }

    /// Copies every node's status file into the private evidence and its public lines into the
    /// shared one, under the stage's name.
    fn snapshot(&mut self, name: &str) {
        self.stage += 1;
        let stage = format!("{:02}-{name}", self.stage);
        let (private, shared) = (self.private().join(&stage), self.shared().join(&stage));
        fs::create_dir_all(&private).unwrap();
        fs::create_dir_all(&shared).unwrap();
        for i in 0..=N {
            let name = match i {
                RELAY => "relay.status".to_string(),
                _ => format!("m{i}.status"),
            };
            let text = fs::read_to_string(self.dir(i).join("status")).unwrap_or_default();
            fs::write(shared.join(&name), public_lines(&text)).unwrap();
            fs::write(private.join(&name), text).unwrap();
        }
    }

    /// Waits for `f` to give a value; on a timeout every status is kept as evidence.
    fn until<V>(&mut self, what: &str, mut f: impl FnMut(&Gate) -> Option<V>) -> V {
        let end = Instant::now() + LIMIT;
        loop {
            if let Some(value) = f(self) {
                return value;
            }
            if Instant::now() > end {
                self.snapshot("timeout");
                panic!("{what}: timed out; evidence in {}", self.evidence.display());
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    fn start(&mut self, i: usize) {
        let log = fs::File::create(self.root.join(format!("node-{i}.log"))).unwrap();
        let child = Command::new(NODE)
            .arg(self.config(i))
            .stdout(Stdio::null())
            .stderr(log)
            .spawn()
            .unwrap();
        self.children[i] = Some(child);
    }

    fn stop(&mut self, i: usize) {
        let mut child = self.children[i].take().expect("running");
        let pid = child.id().to_string();
        assert!(Command::new("kill")
            .args(["-TERM", &pid])
            .status()
            .unwrap()
            .success());
        let end = Instant::now() + Duration::from_secs(10);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            assert!(Instant::now() < end, "member {i} did not stop");
            std::thread::sleep(Duration::from_millis(25));
        };
        assert!(status.success(), "member {i} exited {status}");
    }

    fn listen(&self, i: usize) -> String {
        format!("/ip4/127.0.0.1/tcp/{}", self.ports[i])
    }

    fn relay_address(&self) -> String {
        let peer = peer_id(&self.keys[RELAY].transport);
        format!("{}/p2p/{peer}", self.listen(RELAY))
    }

    /// The lines every node's TOML shares: the genesis's values, the writers, the consortium.
    fn common(&self, data_dir: &Path) -> String {
        let members: Vec<String> = self.keys[..N]
            .iter()
            .map(|k| k.consortium.clone())
            .collect();
        let circle: Vec<String> = self.keys[..N].iter().map(|k| k.circle.clone()).collect();
        let writers: Vec<String> = members.iter().chain(&circle).cloned().collect();
        format!(
            "{}data_dir = \"{}\"\nwriters = {}\ndiscovery_and_archive = false\n\
             sync_every_ms = 100\nstatus_every_ms = 100\n",
            self.lines,
            data_dir.display(),
            quoted(&writers)
        ) + &format!(
            "[consortium]\nmembers = {}\nthreshold = {T}\ncircle = {}\nhpke = {}\n",
            quoted(&members),
            quoted(&circle),
            quoted(&self.announcements)
        )
    }

    /// Member `i`'s TOML: its own files, the other members as peers, its gateway for the relay.
    fn write_member(&self, i: usize) {
        let dir = self.dir(i);
        let peers: Vec<String> = (0..N).filter(|j| *j != i).map(|j| self.listen(j)).collect();
        let text = format!(
            "listen = [\"{}\"]\npeers = {}\nkey_file = \"{}\"\ncircle_key_file = \"{}\"\n\
             hpke_key_file = \"{}\"\nrelays = [\"{}\"]\n",
            self.listen(i),
            quoted(&peers),
            dir.join("member.key").display(),
            dir.join("circle.key").display(),
            dir.join("hpke.key").display(),
            self.keys[RELAY].transport
        ) + &self.common(&dir)
            + &format!(
                "[member]\nindex = {i}\nevery_ms = 100\npatience = 10\nview_patience = 20\n"
            );
        fs::write(self.config(i), text).unwrap();
    }

    fn write_relay(&self) {
        let peers: Vec<String> = (0..N).map(|j| self.listen(j)).collect();
        let text = format!(
            "listen = [\"{}\"]\npeers = {}\nrelay = true\n",
            self.listen(RELAY),
            quoted(&peers)
        ) + &self.common(&self.dir(RELAY));
        fs::write(self.config(RELAY), text).unwrap();
    }

    /// Person `k`'s `--send` of `event` to member `i`'s gateway through the relay, repeated while
    /// the relay or the gateway cannot take it yet; the receipt's line.
    fn send(&mut self, k: usize, event: &str, i: usize) -> String {
        let (genesis, relay) = (self.path("genesis.bin"), self.relay_address());
        let event = self.person(k, event);
        let announcement = self.announcements[i].clone();
        let args = ["--send", &genesis, &announcement, &relay, &event];
        let receipt = self.until("a receipt", |g| g.call(CLIENT, &args).ok());
        assert!(receipt.starts_with(&format!("logged {} ", self.keys[i].circle)));
        receipt
    }

    /// Person `k`'s `--read` of `ask` from member `i` through the relay.
    fn read(&self, k: usize, i: usize, ask: &[&str]) -> Result<String, String> {
        let (genesis, cred, relay) = (
            self.path("genesis.bin"),
            self.person(k, "credential"),
            self.relay_address(),
        );
        let mut args = vec!["--read", &genesis, &cred, &self.announcements[i], &relay];
        args.extend_from_slice(ask);
        self.call(CLIENT, &args)
    }

    /// Person `k`'s status of its `event` from member `i`, once no entry of it is pending.
    fn settled(&mut self, k: usize, event: &str, i: usize) -> String {
        let event = self.person(k, event);
        self.until("the event applied or refused", |g| {
            let out = g.read(k, i, &["status", &event]).ok()?;
            (!out.contains("pending") && !out.starts_with("entries 0")).then_some(out)
        })
    }

    fn request(&self, i: usize, words: [&str; 2]) {
        self.ok(NODE, &["--request", &self.config(i), words[0], words[1]]);
    }

    /// Until every member in `who` reports `ok`.
    fn members(&mut self, what: &str, who: &[usize], ok: impl Fn(&Status) -> bool) -> Vec<Status> {
        self.until(what, |g| {
            let all: Option<Vec<Status>> = who.iter().map(|&i| g.status(i)).collect();
            all.filter(|all| all.iter().all(&ok))
        })
    }
}

impl Drop for Gate {
    fn drop(&mut self) {
        for child in self.children.iter_mut().flatten() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// The single-line facts of a read's reply, as the client prints them.
fn fact<'a>(out: &'a str, word: &str) -> Option<&'a str> {
    out.lines()
        .find_map(|l| l.strip_prefix(word)?.strip_prefix(' '))
}

fn healthy(s: &Status) -> bool {
    s.refused_cut.is_none() && !s.duties_halted && s.equivocations == 0 && !s.conflict
}

// Gate 1: identities, genesis, issuance, a question through the review over the relay, digests,
// a restart, a duplicate, the evidence split into shared and private (AT-T79-50–59).
#[test]
fn at_t79_50_gate_1_on_one_machine() {
    let started = Instant::now();
    let mut g = Gate::new();

    for i in 0..=N {
        let dir = g.dir(i).display().to_string();
        let out = g.ok(NODE, &["--new-identity", &dir]);
        g.keys.push(Keys {
            consortium: printed(&out, "consortium"),
            circle: printed(&out, "circle"),
            hpke: printed(&out, "hpke"),
            transport: printed(&out, "transport"),
        });
    }
    fs::create_dir(g.root.join("issuer")).unwrap();
    g.issuer = printed(
        &g.ok(ISSUER, &["--new-key", &g.path("issuer/issuer.key")]),
        "issuer",
    );
    let mut assembly = format!(
        "threshold = {T}\nfamily = \"answer-review/v1\"\nissuer = \"{}\"\nrelays = [\"{}\"]\n",
        g.issuer, g.keys[RELAY].transport
    );
    for k in &g.keys[..N] {
        assembly += &format!(
            "[[member]]\nconsortium = \"{}\"\ncircle = \"{}\"\nhpke = \"{}\"\n",
            k.consortium, k.circle, k.hpke
        );
    }
    fs::write(g.root.join("genesis.toml"), assembly).unwrap();
    let (input, genesis) = (g.path("genesis.toml"), g.path("genesis.bin"));
    let assembled = g.ok(NODE, &["--genesis-assemble", &input, &genesis]);
    assert_eq!(
        g.ok(NODE, &["--genesis-digest", &genesis]),
        assembled,
        "AT-T79-50"
    );
    assert!(assembled.contains(&format!("threshold {T}\n")));
    assert_eq!(
        assembled
            .lines()
            .filter(|l| l.starts_with("member "))
            .count(),
        N
    );
    assert!(assembled.contains(&format!("relay {}\n", g.keys[RELAY].transport)));
    fs::copy(&genesis, g.shared().join("genesis.bin")).unwrap();
    fs::write(g.shared().join("genesis-digest.txt"), &assembled).unwrap();
    fs::write(g.shared().join("revision.txt"), revision()).unwrap();
    g.lines = format!(
        "network_id = \"{}\"\ngenesis = \"{genesis}\"\nfamily = \"answer-review/v1\"\n\
         answer_config = \"{}\"\nissuer = \"{}\"\n",
        printed(&assembled, "network_id"),
        printed(&assembled, "answer_config_hex"),
        printed(&assembled, "issuer")
    );
    for i in 0..N {
        g.write_member(i);
        let announced = g.ok(NODE, &["--announce", &g.config(i)]);
        g.announcements.push(announced.trim().to_string());
    }
    for i in 0..N {
        g.write_member(i);
    }
    g.write_relay();
    for i in 0..=N {
        g.start(i);
    }
    g.members("members caught up", &[0, 1, 2, 3], |s| {
        s.caught_up && s.circle_caught_up && s.cuts.len() >= 2 && healthy(s)
    });
    g.members("the relay bound to every member", &[RELAY], |s| {
        s.bound == N
    });
    g.snapshot("started");

    let (key, genesis) = (g.path("issuer/issuer.key"), g.path("genesis.bin"));
    for k in 0..PERSONS {
        fs::create_dir(g.root.join(format!("persons/p{k}"))).unwrap();
        let [pending, request, reply, credential] =
            ["pending", "request", "reply", "credential"].map(|f| g.person(k, f));
        let slot = k.to_string();
        let asked = g.ok(CLIENT, &["--request", &genesis, &slot, &pending, &request]);
        assert_eq!(asked, format!("slot {k}\n"), "AT-T79-51");
        assert_eq!(g.ok(ISSUER, &["--issue", &key, &request, &reply]), asked);
        g.ok(
            CLIENT,
            &["--finalize", &genesis, &pending, &reply, &credential],
        );
    }

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
    let mut secrets = vec![fs::read_to_string(g.root.join("issuer/issuer.key")).unwrap()];
    for i in 0..=N {
        for file in ["member.key", "circle.key", "hpke.key", "transport.key"] {
            secrets.push(fs::read_to_string(g.dir(i).join(file)).unwrap());
        }
    }
    let secrets: Vec<String> = secrets.iter().map(|s| s.trim().to_string()).collect();
    for entry in walk(&g.evidence) {
        let text = String::from_utf8_lossy(&fs::read(&entry).unwrap()).into_owned();
        let leaked = secrets.iter().find(|s| text.contains(s.as_str()));
        assert!(leaked.is_none(), "AT-T79-59: {}", entry.display());
    }
    let transcript = fs::read_to_string(g.private().join("transcript.txt")).unwrap();
    let events = transcript.lines().filter_map(|l| l.strip_prefix("event "));
    let mut circle: Vec<String> = [CONTEXT, STEM].map(String::from).to_vec();
    circle.extend(ALTERNATIVES.map(String::from));
    circle.extend(events.map(String::from));
    let last = g.shared().join(format!("{:02}-stopped/m0.status", g.stage));
    let last = fs::read_to_string(last).unwrap();
    assert_eq!(last.lines().filter(|l| l.starts_with("digest ")).count(), 2);
    for entry in walk(&g.shared()) {
        let text = String::from_utf8_lossy(&fs::read(&entry).unwrap()).into_owned();
        let found = circle.iter().find(|c| text.contains(c.as_str()));
        assert!(
            found.is_none(),
            "AT-T79-59: {found:?} in {}",
            entry.display()
        );
        let read = text
            .lines()
            .find(|l| PRINTED.iter().any(|p| l.starts_with(p)));
        assert!(read.is_none(), "AT-T79-59: {read:?} in {}", entry.display());
    }
    g.log(&format!("gate 1 done in {:?}\n", started.elapsed()));
    println!("gate 1 evidence: {}", g.evidence.display());
}

/// The revision under test: the commit and whether tracked files differ from it.
fn revision() -> String {
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .arg("-C")
            .arg(env!("CARGO_MANIFEST_DIR"))
            .args(args)
            .output()
            .ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
    };
    let commit = git(&["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let dirty = git(&["status", "--porcelain", "--untracked-files=no"]);
    let state = match dirty.as_deref() {
        Some("") => "clean",
        Some(_) => "modified",
        None => "unknown",
    };
    format!("revision {commit}\ntracked_files {state}\n")
}

fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        match path.is_dir() {
            true => files.extend(walk(&path)),
            false => files.push(path),
        }
    }
    files
}
