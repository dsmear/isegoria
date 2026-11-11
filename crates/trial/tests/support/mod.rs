//! The harness of Milestone 2's process tests (`docs/05` §Gate 1 on one machine, §Gate 2's
//! rehearsal): one run's files, processes, commands and evidence, on loopback or in namespaces.
#![allow(dead_code)]

use p2p::Status;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

pub const NODE: &str = env!("CARGO_BIN_EXE_trial-node");
pub const CLIENT: &str = env!("CARGO_BIN_EXE_trial-client");
pub const ISSUER: &str = env!("CARGO_BIN_EXE_trial-test-issuer");
pub const N: usize = 4;
pub const T: usize = 3;
pub const RELAY: usize = N;
pub const PERSONS: usize = 14;
pub const LIMIT: Duration = Duration::from_secs(90);
pub const PERSONS_NS: usize = N + 1;
const PORT: u16 = 4001;

/// Where a run's processes are: all on loopback, or each node in a network namespace of its own
/// (`n0`–`n4`, the persons' commands in `n5`) on the bridge `br0` (`docs/05` §Gate 2's rehearsal).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Net {
    Loopback,
    Namespaces,
}

/// A key `--new-identity` printed.
pub fn printed(text: &str, word: &str) -> String {
    let value = text
        .lines()
        .find_map(|l| l.strip_prefix(word)?.strip_prefix(' '));
    value
        .unwrap_or_else(|| panic!("no {word} in {text}"))
        .to_string()
}

pub fn quoted(list: &[String]) -> String {
    let items: Vec<String> = list.iter().map(|s| format!("\"{s}\"")).collect();
    format!("[{}]", items.join(", "))
}

pub fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

pub fn peer_id(transport: &str) -> String {
    let bytes: Vec<u8> = (0..transport.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&transport[i..i + 2], 16).unwrap())
        .collect();
    let key = libp2p::identity::ed25519::PublicKey::try_from_bytes(&bytes).unwrap();
    libp2p::identity::PublicKey::from(key)
        .to_peer_id()
        .to_string()
}

pub struct Keys {
    pub consortium: String,
    pub circle: String,
    pub hpke: String,
    pub transport: String,
}

pub const CONTEXT: &str = "gate 1, a synthetic question";
pub const STEM: &str = "Which of these numbers is even?";
pub const ALTERNATIVES: [&str; 3] = ["two, synthetic", "three, synthetic", "five, synthetic"];

/// The first words of the lines the client prints for events and reads.
pub const PRINTED: [&str; 14] = [
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
pub const PUBLIC: [&str; 14] = [
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

pub fn public_lines(status: &str) -> String {
    let public = |l: &&str| PUBLIC.contains(&l.split(' ').next().unwrap_or_default());
    status
        .lines()
        .filter(public)
        .map(|l| format!("{l}\n"))
        .collect()
}

/// One run: its files under `root`, its evidence under `root/evidence/shared` and
/// `root/evidence/private`, the processes killed when it ends however it ends.
pub struct Gate {
    pub root: PathBuf,
    pub evidence: PathBuf,
    pub stage: usize,
    pub keys: Vec<Keys>,
    pub ports: Vec<u16>,
    pub issuer: String,
    pub lines: String,
    pub announcements: Vec<String>,
    pub children: Vec<Option<Child>>,
    pub net: Net,
    pub started: Instant,
}

impl Gate {
    pub fn new(name: &str, net: Net) -> Gate {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(name)
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
            ports: (0..=N)
                .map(|_| match net {
                    Net::Loopback => free_port(),
                    Net::Namespaces => PORT,
                })
                .collect(),
            issuer: String::new(),
            lines: String::new(),
            announcements: Vec::new(),
            children: (0..=N).map(|_| None).collect(),
            net,
            started: Instant::now(),
        }
    }

    pub fn path(&self, name: &str) -> String {
        self.root.join(name).display().to_string()
    }

    pub fn dir(&self, i: usize) -> PathBuf {
        match i {
            RELAY => self.root.join("relay"),
            _ => self.root.join(format!("m{i}")),
        }
    }

    pub fn config(&self, i: usize) -> String {
        match i {
            RELAY => self.path("relay.toml"),
            _ => self.path(&format!("m{i}.toml")),
        }
    }

    pub fn person(&self, k: usize, file: &str) -> String {
        self.path(&format!("persons/p{k}/{file}"))
    }

    pub fn shared(&self) -> PathBuf {
        self.evidence.join("shared")
    }

    pub fn private(&self) -> PathBuf {
        self.evidence.join("private")
    }

    pub fn log(&self, text: &str) {
        let text = text.replace(&self.root.display().to_string(), "$RUN");
        let mut file = (OpenOptions::new().create(true).append(true))
            .open(self.private().join("transcript.txt"))
            .unwrap();
        file.write_all(text.as_bytes()).unwrap();
    }

    /// Runs the shell `line`, which changes the run's network; it must succeed.
    pub fn network(&self, line: &str) {
        let status = Command::new("sh").args(["-c", line]).status().unwrap();
        assert!(status.success(), "{line}");
    }

    /// Notes `what` with the run's time in the shared evidence's `faults.txt`.
    pub fn note(&self, what: &str) {
        let at = self.started.elapsed().as_secs_f32();
        let mut file = (OpenOptions::new().create(true).append(true))
            .open(self.shared().join("faults.txt"))
            .unwrap();
        writeln!(file, "{at:7.1}s {what}").unwrap();
    }

    /// Injects the fault the shell `line` makes, noted as `what`.
    pub fn fault(&self, what: &str, line: &str) {
        self.network(line);
        self.note(&format!("{what}: {line}"));
    }

    /// Bridges `br0` and `br1`, and a namespace per node and for the persons, each linked to
    /// `br0` by a veth `v<i>` with `netem` on the namespace's side.
    pub fn topology(&self, netem: &str) {
        self.network("ip link add br0 type bridge && ip link set br0 up");
        self.network("ip link add br1 type bridge && ip link set br1 up");
        for i in 0..=PERSONS_NS {
            let address = 10 + i;
            self.network(&format!(
                "ip netns add n{i} && ip link add v{i} type veth peer name eth0 netns n{i} && \
                 ip link set v{i} master br0 up && ip -n n{i} link set lo up && \
                 ip -n n{i} addr add 10.77.0.{address}/24 dev eth0 && ip -n n{i} link set eth0 up && \
                 tc -n n{i} qdisc replace dev eth0 root netem {netem}"
            ));
        }
        self.note(&format!("every namespace's link: netem {netem}"));
    }

    /// Kills node `i` with SIGKILL.
    pub fn kill(&mut self, i: usize) {
        let mut child = self.children[i].take().expect("running");
        child.kill().unwrap();
        child.wait().unwrap();
        self.note(&format!("node {i} killed (SIGKILL)"));
    }

    /// `bin` as a command, inside namespace `ns` when the run has namespaces.
    pub fn command(&self, ns: usize, bin: &str) -> Command {
        match self.net {
            Net::Loopback => Command::new(bin),
            Net::Namespaces => {
                let mut command = Command::new("ip");
                command.args(["netns", "exec", &format!("n{ns}"), bin]);
                command
            }
        }
    }

    /// Runs `bin` with `args`, its command line and output added to the transcript: its standard
    /// output, or its standard error when it exits otherwise than 0.
    pub fn call(&self, bin: &str, args: &[&str]) -> Result<String, String> {
        let output = (self.command(PERSONS_NS, bin).args(args)).output().unwrap();
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

    pub fn ok(&self, bin: &str, args: &[&str]) -> String {
        let result = self.call(bin, args);
        result.unwrap_or_else(|e| panic!("{bin} {args:?}: {e}"))
    }

    pub fn status(&self, i: usize) -> Option<Status> {
        node::parse(&fs::read_to_string(self.dir(i).join("status")).ok()?)
    }

    pub fn statuses(&self) -> Vec<Status> {
        (0..N).map(|i| self.status(i).expect("a status")).collect()
    }

    /// Copies every node's status file into the private evidence and its public lines into the
    /// shared one, under the stage's name.
    pub fn snapshot(&mut self, name: &str) {
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
    pub fn until<V>(&mut self, what: &str, mut f: impl FnMut(&Gate) -> Option<V>) -> V {
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

    pub fn start(&mut self, i: usize) {
        let log = (OpenOptions::new().create(true).append(true))
            .open(self.root.join(format!("node-{i}.log")))
            .unwrap();
        let child = self
            .command(i, NODE)
            .arg(self.config(i))
            .stdout(Stdio::null())
            .stderr(log)
            .spawn()
            .unwrap();
        self.children[i] = Some(child);
    }

    pub fn stop(&mut self, i: usize) {
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

    pub fn listen(&self, i: usize) -> String {
        match self.net {
            Net::Loopback => format!("/ip4/127.0.0.1/tcp/{}", self.ports[i]),
            Net::Namespaces => format!("/ip4/10.77.0.{}/tcp/{}", 10 + i, self.ports[i]),
        }
    }

    pub fn relay_address(&self) -> String {
        let peer = peer_id(&self.keys[RELAY].transport);
        format!("{}/p2p/{peer}", self.listen(RELAY))
    }

    /// The lines every node's TOML shares: the genesis's values, the writers, the consortium.
    pub fn common(&self, data_dir: &Path) -> String {
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
    pub fn write_member(&self, i: usize) {
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

    pub fn write_relay(&self) {
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
    pub fn send(&mut self, k: usize, event: &str, i: usize) -> String {
        let (genesis, relay) = (self.path("genesis.bin"), self.relay_address());
        let event = self.person(k, event);
        let announcement = self.announcements[i].clone();
        let args = ["--send", &genesis, &announcement, &relay, &event];
        let receipt = self.until("a receipt", |g| g.call(CLIENT, &args).ok());
        assert!(receipt.starts_with(&format!("logged {} ", self.keys[i].circle)));
        receipt
    }

    /// Person `k`'s `--read` of `ask` from member `i` through the relay.
    pub fn read(&self, k: usize, i: usize, ask: &[&str]) -> Result<String, String> {
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
    pub fn settled(&mut self, k: usize, event: &str, i: usize) -> String {
        let event = self.person(k, event);
        self.until("the event applied or refused", |g| {
            let out = g.read(k, i, &["status", &event]).ok()?;
            (!out.contains("pending") && !out.starts_with("entries 0")).then_some(out)
        })
    }

    pub fn request(&self, i: usize, words: [&str; 2]) {
        self.ok(NODE, &["--request", &self.config(i), words[0], words[1]]);
    }

    /// Until every member in `who` reports `ok`.
    pub fn members(
        &mut self,
        what: &str,
        who: &[usize],
        ok: impl Fn(&Status) -> bool,
    ) -> Vec<Status> {
        self.until(what, |g| {
            let all: Option<Vec<Status>> = who.iter().map(|&i| g.status(i)).collect();
            all.filter(|all| all.iter().all(&ok))
        })
    }

    /// Identities, the issuer's key, the genesis assembled and checked, every TOML; every node
    /// started.
    pub fn setup(&mut self) {
        for i in 0..=N {
            let dir = self.dir(i).display().to_string();
            let out = self.ok(NODE, &["--new-identity", &dir]);
            self.keys.push(Keys {
                consortium: printed(&out, "consortium"),
                circle: printed(&out, "circle"),
                hpke: printed(&out, "hpke"),
                transport: printed(&out, "transport"),
            });
        }
        fs::create_dir(self.root.join("issuer")).unwrap();
        self.issuer = printed(
            &self.ok(ISSUER, &["--new-key", &self.path("issuer/issuer.key")]),
            "issuer",
        );
        let mut assembly = format!(
            "threshold = {T}\nfamily = \"answer-review/v1\"\nissuer = \"{}\"\nrelays = [\"{}\"]\n",
            self.issuer, self.keys[RELAY].transport
        );
        for k in &self.keys[..N] {
            assembly += &format!(
                "[[member]]\nconsortium = \"{}\"\ncircle = \"{}\"\nhpke = \"{}\"\n",
                k.consortium, k.circle, k.hpke
            );
        }
        fs::write(self.root.join("genesis.toml"), assembly).unwrap();
        let (input, genesis) = (self.path("genesis.toml"), self.path("genesis.bin"));
        let assembled = self.ok(NODE, &["--genesis-assemble", &input, &genesis]);
        assert_eq!(
            self.ok(NODE, &["--genesis-digest", &genesis]),
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
        assert!(assembled.contains(&format!("relay {}\n", self.keys[RELAY].transport)));
        fs::copy(&genesis, self.shared().join("genesis.bin")).unwrap();
        fs::write(self.shared().join("genesis-digest.txt"), &assembled).unwrap();
        fs::write(self.shared().join("revision.txt"), revision()).unwrap();
        self.lines = format!(
            "network_id = \"{}\"\ngenesis = \"{genesis}\"\nfamily = \"answer-review/v1\"\n\
             answer_config = \"{}\"\nissuer = \"{}\"\n",
            printed(&assembled, "network_id"),
            printed(&assembled, "answer_config_hex"),
            printed(&assembled, "issuer")
        );
        for i in 0..N {
            self.write_member(i);
            let announced = self.ok(NODE, &["--announce", &self.config(i)]);
            self.announcements.push(announced.trim().to_string());
        }
        for i in 0..N {
            self.write_member(i);
        }
        self.write_relay();
        for i in 0..=N {
            self.start(i);
        }
    }

    /// Each person's credential by files: request, the test issuer's reply, finalization.
    pub fn credentials(&mut self) {
        let (key, genesis) = (self.path("issuer/issuer.key"), self.path("genesis.bin"));
        for k in 0..PERSONS {
            fs::create_dir(self.root.join(format!("persons/p{k}"))).unwrap();
            let [pending, request, reply, credential] =
                ["pending", "request", "reply", "credential"].map(|f| self.person(k, f));
            let slot = k.to_string();
            let asked = self.ok(CLIENT, &["--request", &genesis, &slot, &pending, &request]);
            assert_eq!(asked, format!("slot {k}\n"), "AT-T79-51");
            assert_eq!(self.ok(ISSUER, &["--issue", &key, &request, &reply]), asked);
            self.ok(
                CLIENT,
                &["--finalize", &genesis, &pending, &reply, &credential],
            );
        }
    }

    /// No secret key of the run in its evidence.
    pub fn no_secret_in_evidence(&self) {
        let mut secrets = vec![fs::read_to_string(self.root.join("issuer/issuer.key")).unwrap()];
        for i in 0..=N {
            for file in ["member.key", "circle.key", "hpke.key", "transport.key"] {
                secrets.push(fs::read_to_string(self.dir(i).join(file)).unwrap());
            }
        }
        let secrets: Vec<String> = secrets.iter().map(|s| s.trim().to_string()).collect();
        for entry in walk(&self.evidence) {
            let text = String::from_utf8_lossy(&fs::read(&entry).unwrap()).into_owned();
            let leaked = secrets.iter().find(|s| text.contains(s.as_str()));
            assert!(leaked.is_none(), "AT-T79-59: {}", entry.display());
        }
    }

    /// Nothing of `circle`, and no line the client prints, in the shared evidence.
    pub fn no_circle_content_shared(&self, circle: &[String]) {
        for entry in walk(&self.shared()) {
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
pub fn fact<'a>(out: &'a str, word: &str) -> Option<&'a str> {
    out.lines()
        .find_map(|l| l.strip_prefix(word)?.strip_prefix(' '))
}

pub fn healthy(s: &Status) -> bool {
    s.refused_cut.is_none() && !s.duties_halted && s.equivocations == 0 && !s.conflict
}

/// The revision under test: the commit and whether tracked files differ from it.
pub fn revision() -> String {
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

pub fn walk(dir: &Path) -> Vec<PathBuf> {
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
