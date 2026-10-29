//! Operator-pinned archive configuration checks (`docs/04` §Archive maintenance in a node process).

#[path = "support/archive.rs"]
mod support;
use node::{plan, Config, ConfigError};
use support::*;

fn checked(f: &Fixture, name: &str, text: &str) -> Result<(), ConfigError> {
    let path = f.save_config(name, text);
    plan(Config::load(&path)?).map(|_| ())
}

// Trusted jobs are explicit and checked before node files are created (AT-NET-31).
#[test]
fn trusted_inputs_are_independent_and_preflighted() {
    let f = Fixture::new("config-context");
    let job = f.job(&[]);
    let text = f.config("data", "/ip4/127.0.0.1/tcp/0", &[], Some(&job));
    let mut different = f.target.clone();
    different.epoch = 1;
    let other = f.root.join("other.cut");
    std::fs::write(&other, different.encode()).unwrap();
    let cases = [
        (
            "target",
            text.replace(
                &f.root.join("target.cut").display().to_string(),
                &other.display().to_string(),
            ),
        ),
        (
            "previous",
            text.replace(
                "view = 0",
                &format!(
                    "view = 0\nprevious_cut_file = \"{}\"",
                    f.root.join("target.cut").display()
                ),
            ),
        ),
        (
            "lineage",
            text.replace(&node::hex(&LINEAGE), &node::hex(&[0; 32])),
        ),
        ("view", text.replace("view = 0", "view = 1")),
        ("excluded", text.replace("excluded = []", "excluded = [0]")),
        (
            "excluded-duplicate",
            text.replace("excluded = []", "excluded = [0,0]"),
        ),
        (
            "excluded-range",
            text.replace("excluded = []", "excluded = [1]"),
        ),
        ("roster-duplicate", text.replace(&public(42), &public(41))),
        (
            "roster-short",
            text.replace(&format!(",\"{}\"", public(42)), ""),
        ),
        ("roster-invalid", text.replace(&public(42), "00")),
        (
            "zero-interval",
            text.replace("interval_ms = 100", "interval_ms = 0"),
        ),
        (
            "zero-deadline",
            text.replace("deadline_ms = 3000", "deadline_ms = 0"),
        ),
        ("no-schedule", text.replace("interval_ms = 100\n", "")),
        ("no-exclusions", text.replace("excluded = []\n", "")),
        ("no-bootstrap", text.replace("bootstrap = []\n", "")),
        ("unknown", text.clone() + "unknown = 1\n"),
        ("duplicate-jobs", text.clone() + &job),
        (
            "no-tiers",
            text.replace(&format!("circle = [\"{}\"]\n", public(11)), ""),
        ),
    ];
    for (name, text) in cases {
        assert!(checked(&f, name, &text).is_err(), "{name}");
        assert!(!f.root.join("data").exists(), "{name} created node files");
    }
    assert_eq!(checked(&f, "valid", &text), Ok(()));
    assert!(f.root.join("data/transport.key").exists());
}

// Noninitial jobs use a separately pinned predecessor and target (AT-NET-31).
#[test]
fn noninitial_job_loads_its_explicit_predecessor() {
    use network::archive::{BuiltArchive, Context, Layout, Segment, SignedManifest};
    use network::consortium::Member;
    use network::replica::Replica;
    let f = Fixture::new("config-previous");
    let previous = f.target.clone();
    let mut target = previous.clone();
    target.number = 1;
    let context = Context {
        tiers: &f.tiers,
        lineage: LINEAGE,
        previous: Some(&previous),
        target: &target,
        view: 0,
        excluded: &[],
    };
    let member = Member::from_seed([1; 32]);
    let checkpoint = target.checkpoint_in(0, NET, f.tiers.consortium().member_set_hash());
    let segment = Segment::build(
        &context,
        &Replica::new(f.tiers.writers().clone()),
        vec![(0, member.sign(&checkpoint))],
        None,
    )
    .unwrap();
    let built = BuiltArchive::build(
        &context,
        &segment,
        Layout {
            data_shards: 1,
            parity_shards: 1,
        },
    )
    .unwrap();
    let signed = SignedManifest {
        signatures: vec![(0, member.sign(&built.manifest.checkpoint()))],
        manifest: built.manifest,
    };
    let previous_file = f.root.join("previous.cut");
    std::fs::write(&previous_file, previous.encode()).unwrap();
    std::fs::write(f.root.join("target.cut"), target.encode()).unwrap();
    std::fs::write(f.root.join("manifest.bin"), signed.encode().unwrap()).unwrap();
    let job = f.job(&[]) + &format!("previous_cut_file = \"{}\"\n", previous_file.display());
    let text = f.config("data", "/ip4/127.0.0.1/tcp/0", &[], Some(&job));
    assert_eq!(checked(&f, "noninitial", &text), Ok(()));
    std::fs::remove_dir_all(f.root.join("data")).unwrap();
    assert!(checked(
        &f,
        "missing-previous",
        &f.config("data", "/ip4/127.0.0.1/tcp/0", &[], Some(&f.job(&[])))
    )
    .is_err());
    assert!(!f.root.join("data").exists());
}

// Canonical binary input, counts, addresses and file reads are bounded (AT-NET-31).
#[test]
fn malformed_and_oversized_inputs_fail_before_start() {
    let f = Fixture::new("config-bounds");
    let job = f.job(&[]);
    let text = f.config("data", "/ip4/127.0.0.1/tcp/0", &[], Some(&job));
    let manifest = f.root.join("manifest.bin");
    let valid = std::fs::read(&manifest).unwrap();
    let mut forged = valid.clone();
    *forged.last_mut().unwrap() ^= 1;
    for bytes in [
        forged,
        vec![0; (64 << 10) + 1],
        [valid.clone(), vec![0]].concat(),
        vec![1, 255],
    ] {
        std::fs::write(&manifest, bytes).unwrap();
        assert!(checked(&f, "manifest", &text).is_err());
        assert!(!f.root.join("data").exists());
    }
    std::fs::write(&manifest, valid).unwrap();
    let cut = f.root.join("target.cut");
    let valid = std::fs::read(&cut).unwrap();
    for bytes in [
        vec![0; (64 << 10) + 1],
        [valid.clone(), vec![0]].concat(),
        vec![1, 255],
    ] {
        std::fs::write(&cut, bytes).unwrap();
        assert!(checked(&f, "cut", &text).is_err());
    }
    std::fs::write(&cut, valid).unwrap();
    let empty = text
        .split("[[archive_maintenance.jobs]]")
        .next()
        .unwrap()
        .to_string()
        + "jobs = []\n";
    assert!(checked(&f, "empty-jobs", &empty).is_err());
    for (name, text) in [
        ("many-jobs", text.clone() + &job.repeat(256)),
        (
            "many-roster",
            text.replace(
                &format!("roster = [\"{}\",\"{}\"]", public(41), public(42)),
                &format!("roster = {}", quoted(&vec![public(41); 1025])),
            ),
        ),
        (
            "many-bootstrap",
            text.replace(
                "bootstrap = []",
                &format!(
                    "bootstrap = {}",
                    quoted(&vec!["/ip4/127.0.0.1/tcp/1".into(); 33])
                ),
            ),
        ),
        (
            "long-bootstrap",
            text.replace(
                "bootstrap = []",
                &format!("bootstrap = [\"{}\"]", "x".repeat(1025)),
            ),
        ),
        (
            "component-bootstrap",
            text.replace(
                "bootstrap = []",
                &format!("bootstrap = [\"{}\"]", "/p2p-circuit".repeat(17)),
            ),
        ),
        (
            "bad-bootstrap",
            text.replace("bootstrap = []", "bootstrap = [\"nowhere\"]"),
        ),
    ] {
        assert!(checked(&f, name, &text).is_err(), "{name}");
    }
    let huge = f.root.join("huge.toml");
    let file = std::fs::File::create(&huge).unwrap();
    file.set_len((1 << 20) + 1).unwrap();
    assert!(matches!(
        Config::load(&huge),
        Err(ConfigError::Bounds("config"))
    ));
    let directory = f.root.join("directory");
    std::fs::create_dir(&directory).unwrap();
    assert!(matches!(
        Config::load(&directory),
        Err(ConfigError::Bounds("config"))
    ));
    assert!(!f.root.join("data").exists());
}

// Nonregular input files are rejected before open can block (AT-NET-31).
#[cfg(unix)]
#[test]
fn fifo_is_refused_without_opening() {
    let f = Fixture::new("config-fifo");
    let fifo = f.root.join("fifo");
    assert!(std::process::Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .unwrap()
        .success());
    assert!(matches!(
        Config::load(&fifo),
        Err(ConfigError::Bounds("config"))
    ));
    let text = f
        .config("data", "/ip4/127.0.0.1/tcp/0", &[], Some(&f.job(&[])))
        .replace(
            &f.root.join("manifest.bin").display().to_string(),
            &fifo.display().to_string(),
        );
    assert!(checked(&f, "fifo", &text).is_err());
    assert!(!f.root.join("data").exists());
}

#[cfg(unix)]
mod processes {
    use super::*;
    use libp2p::identity::Keypair;
    use std::net::TcpListener;
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, Stdio};
    use std::time::{Duration, Instant};

    struct Process(Child);
    impl Process {
        fn start(config: &Path) -> Self {
            let log = std::fs::File::create(config.with_extension("log")).unwrap();
            Self(
                Command::new(env!("CARGO_BIN_EXE_isegoria-node"))
                    .arg(config)
                    .stdout(Stdio::null())
                    .stderr(log)
                    .spawn()
                    .unwrap(),
            )
        }
        fn kill(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
        async fn terminate(&mut self, limit: Duration) {
            assert!(Command::new("kill")
                .args(["-TERM", &self.0.id().to_string()])
                .status()
                .unwrap()
                .success());
            let end = Instant::now() + limit;
            loop {
                if let Some(status) = self.0.try_wait().unwrap() {
                    assert!(status.success(), "node exited {status}");
                    return;
                }
                assert!(
                    Instant::now() < end,
                    "node did not drain its maintenance pass"
                );
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
        }
    }
    impl Drop for Process {
        fn drop(&mut self) {
            self.kill();
        }
    }
    fn address(seed: u8) -> String {
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let peer = Keypair::ed25519_from_bytes([seed; 32])
            .unwrap()
            .public()
            .to_peer_id();
        format!("/ip4/127.0.0.1/tcp/{port}/p2p/{peer}")
    }
    async fn until(path: &Path, expected: impl Fn(&str) -> bool) -> String {
        let end = Instant::now() + Duration::from_secs(25);
        loop {
            let text = read(path);
            if expected(&text) {
                return text;
            }
            assert!(Instant::now() < end, "{}: {text}", path.display());
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }
    async fn new_status(path: &Path) {
        let end = Instant::now() + Duration::from_secs(3);
        let before = std::fs::metadata(path).and_then(|m| m.modified()).ok();
        loop {
            let after = std::fs::metadata(path).and_then(|m| m.modified()).ok();
            if after.is_some() && after != before {
                return;
            }
            assert!(
                Instant::now() < end,
                "normal status stopped during maintenance"
            );
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }
    async fn seeded_source(f: &Fixture, address: &str) -> (PathBuf, Process) {
        f.seed("source", 41);
        let config = f.save_config("source", &f.config("source", address, &[], None));
        let plan = node::plan(Config::load(&config).unwrap()).unwrap();
        let handle = p2p::Handle::spawn(plan.transport, plan.p2p).unwrap();
        handle
            .install_archive(&f.context(), &f.signed, &f.segment, &[0, 1])
            .await
            .unwrap();
        drop(handle);
        tokio::time::sleep(Duration::from_millis(100)).await;
        let process = Process::start(&config);
        until(&f.root.join("source/status"), |s| {
            node::parse(s).is_some_and(|s| !s.cuts.is_empty()) || !s.is_empty()
        })
        .await;
        (config, process)
    }

    // Executable jobs repair, audit after restart, and keep reporting through recovery (AT-NET-31).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn repair_bitrot_restart_and_status() {
        let f = Fixture::new("process-repair");
        let source_address = address(41);
        let (_, mut source) = seeded_source(&f, &source_address).await;
        f.seed("holder", 42);
        let holder_address = address(42);
        let config = f.save_config(
            "holder",
            &f.config(
                "holder",
                &holder_address,
                std::slice::from_ref(&source_address),
                Some(&f.job(std::slice::from_ref(&source_address))),
            ),
        );
        let archive_status = f.root.join("holder/archive-status");
        let status = f.root.join("holder/status");
        std::fs::write(&archive_status, "stale healthy").unwrap();
        let mut holder = Process::start(&config);
        until(&archive_status, |s| s.contains("Ok(Repaired")).await;
        let original = std::fs::read(f.record("holder")).unwrap();
        until(&archive_status, |s| s.contains("Ok(Healthy")).await;
        let mut damaged = original.clone();
        let last = damaged.len() - 1;
        damaged[last] ^= 1;
        std::fs::write(f.record("holder"), &damaged).unwrap();
        until(&f.record("holder"), |_| {
            std::fs::read(f.record("holder")).ok().as_ref() == Some(&original)
        })
        .await;
        until(&archive_status, |s| s.contains("Ok(Repaired")).await;
        new_status(&status).await;
        holder.kill();
        std::fs::remove_file(f.record("holder")).unwrap();
        std::fs::write(&archive_status, "old report").unwrap();
        holder = Process::start(&config);
        until(&archive_status, |s| s.contains("Ok(Repaired")).await;
        assert_eq!(std::fs::read(f.record("holder")).unwrap(), original);
        holder.terminate(Duration::from_secs(5)).await;
        assert!(read(&archive_status).contains("Ok("));
        source.terminate(Duration::from_secs(5)).await;
    }

    // SIGTERM drains recovery and clears stale reports (AT-NET-31).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn sigterm_drains_and_disabled_clears_status() {
        let f = Fixture::new("process-drain");
        f.seed("holder", 42);
        let address = address(42);
        let stalled = TcpListener::bind("127.0.0.1:0").unwrap();
        let unavailable = format!(
            "/ip4/127.0.0.1/tcp/{}",
            stalled.local_addr().unwrap().port()
        );
        let config = f.save_config(
            "holder",
            &f.config("holder", &address, &[], Some(&f.job(&[unavailable]))),
        );
        let archive_status = f.root.join("holder/archive-status");
        let status = f.root.join("holder/status");
        std::fs::write(&archive_status, "stale healthy").unwrap();
        let mut holder = Process::start(&config);
        until(&archive_status, |s| s == "pending\n").await;
        new_status(&status).await;
        new_status(&status).await;
        assert_eq!(read(&archive_status), "pending\n");
        let started = Instant::now();
        holder.terminate(Duration::from_secs(6)).await;
        assert!(
            started.elapsed() >= Duration::from_millis(500),
            "active pass was canceled"
        );
        let report = read(&archive_status);
        assert!(report.contains("result Err(Timeout)"), "{report}");
        f.save_config("holder", &f.config("holder", &address, &[], None));
        holder = Process::start(&config);
        until(&archive_status, |s| s == "disabled\n").await;
        holder.terminate(Duration::from_secs(3)).await;
    }

    // Invalid archive jobs exit before creating node files (AT-NET-31).
    #[test]
    fn executable_rejects_invalid_trust_before_starting() {
        let f = Fixture::new("process-invalid");
        let text = f
            .config("holder", "/ip4/127.0.0.1/tcp/0", &[], Some(&f.job(&[])))
            .replace("view = 0", "view = 1");
        let config = f.save_config("invalid", &text);
        let result = Command::new(env!("CARGO_BIN_EXE_isegoria-node"))
            .arg(config)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(2));
        assert!(!f.root.join("holder").exists());
    }
}
