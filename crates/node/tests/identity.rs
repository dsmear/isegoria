//! `--new-identity` and a member node's identity record (`docs/04` §Increments A and B,
//! `docs/01` D58, AT-NET-36): four private secrets, public keys printed, no record made later.

#[path = "support/genesis.rs"]
mod genesis;
#[allow(dead_code)]
#[path = "support/process.rs"]
mod process;

use ed25519_dalek::SigningKey;
use libp2p::identity::Keypair;
use network::consortium::Member;
use p2p::record::{read, Found, Record};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SECRETS: [&str; 4] = ["member.key", "circle.key", "hpke.key", "transport.key"];

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("identity")
        .join(format!("{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn new_identity(dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_isegoria-node"))
        .arg("--new-identity")
        .arg(dir)
        .output()
        .unwrap()
}

fn seed(path: &Path) -> [u8; 32] {
    let text = fs::read_to_string(path).unwrap();
    assert_eq!(text.len(), 64);
    let bytes: Vec<u8> = (0..32)
        .map(|i| u8::from_str_radix(&text[2 * i..2 * i + 2], 16).unwrap())
        .collect();
    bytes.try_into().unwrap()
}

/// The four public keys the secrets in `dir` make, in the printed order.
fn publics(dir: &Path) -> [[u8; 32]; 4] {
    let [member, circle, hpke, transport] = SECRETS.map(|name| seed(&dir.join(name)));
    let transport = Keypair::ed25519_from_bytes(transport).unwrap();
    [
        Member::from_seed(member).public().to_bytes(),
        SigningKey::from_bytes(&circle).verifying_key().to_bytes(),
        p2p::blind::public_key(&hpke).unwrap(),
        transport.public().try_into_ed25519().unwrap().to_bytes(),
    ]
}

/// Every name in `dir`, sorted.
fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

// A new identity: four separate private secrets, a fresh record, public keys printed (AT-NET-36).
#[test]
fn a_new_identity_prints_only_its_public_keys() {
    let root = scratch("new");
    let (a, b) = (root.join("a"), root.join("b"));
    let out = new_identity(&a);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.stderr.is_empty());
    let printed = String::from_utf8(out.stdout).unwrap();
    let keys = publics(&a);
    let expected: String = ["consortium", "circle", "hpke", "transport"]
        .iter()
        .zip(&keys)
        .map(|(label, key)| format!("{label} {}\n", node::hex(key)))
        .collect();
    assert_eq!(printed, expected);
    let secrets = SECRETS.map(|name| seed(&a.join(name)));
    for (i, secret) in secrets.iter().enumerate() {
        assert!(!printed.contains(&node::hex(secret)));
        assert!(secrets[i + 1..].iter().all(|other| other != secret));
    }
    let mut all: Vec<String> = SECRETS.iter().map(|s| s.to_string()).collect();
    all.push("identity".into());
    all.sort();
    assert_eq!(names(&a), all);
    let record = Record::new(keys[0], keys[1]);
    assert_eq!(read(&a.join("identity")), Found::Present(record));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = |p: &Path| fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&a), 0o700);
        for name in SECRETS {
            assert_eq!(mode(&a.join(name)), 0o600, "{name}");
        }
    }
    assert!(new_identity(&b).status.success());
    let other = publics(&b);
    assert!(keys.iter().zip(&other).all(|(x, y)| x != y));
}

// An existing path or a missing parent is refused, and nothing is written over (AT-NET-36).
#[test]
fn an_existing_path_is_refused_unchanged() {
    let root = scratch("existing");
    let empty = root.join("empty");
    fs::create_dir(&empty).unwrap();
    let file = root.join("file");
    fs::write(&file, b"kept").unwrap();
    let made = root.join("made");
    assert!(new_identity(&made).status.success());
    let before = fs::read(made.join("member.key")).unwrap();
    for path in [&empty, &file, &made] {
        let out = new_identity(path);
        assert_eq!(out.status.code(), Some(2), "{}", path.display());
        assert!(out.stdout.is_empty());
    }
    assert!(names(&empty).is_empty());
    assert_eq!(fs::read(&file).unwrap(), b"kept");
    assert_eq!(fs::read(made.join("member.key")).unwrap(), before);
    let orphan = root.join("missing/child");
    assert_eq!(new_identity(&orphan).status.code(), Some(2));
    assert!(!root.join("missing").exists());
}

#[cfg(unix)]
mod processes {
    use super::process::{until, Process};
    use super::*;
    use std::net::TcpListener;
    use std::time::Duration;

    fn port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    }

    fn list(keys: &[[u8; 32]]) -> String {
        let quoted: Vec<String> = keys
            .iter()
            .map(|k| format!("\"{}\"", node::hex(k)))
            .collect();
        format!("[{}]", quoted.join(","))
    }

    /// Member `index` of `keys`, `threshold` of them signing, keeping its files in `dir`,
    /// dialing `peers`.
    fn toml(
        dir: &Path,
        index: usize,
        keys: &[[[u8; 32]; 4]],
        threshold: usize,
        listen: u16,
        peers: &[String],
    ) -> String {
        let members: Vec<[u8; 32]> = keys.iter().map(|k| k[0]).collect();
        let circle: Vec<[u8; 32]> = keys.iter().map(|k| k[1]).collect();
        let writers: Vec<[u8; 32]> = members.iter().chain(&circle).copied().collect();
        let peers: Vec<String> = peers.iter().map(|p| format!("\"{p}\"")).collect();
        let entries = keys.iter().map(|k| network::genesis::GenesisMember {
            consortium: k[0],
            circle: k[1],
            hpke: k[2],
        });
        let document = super::genesis::genesis(threshold, entries.collect(), vec![]);
        let lines = super::genesis::write(&dir.with_extension("genesis"), &document);
        lines + &format!("network_id = \"{}\"\ndata_dir = \"{}\"\nlisten = [\"/ip4/127.0.0.1/tcp/{listen}\"]\npeers = [{}]\nwriters = {}\nkey_file = \"{}\"\ncircle_key_file = \"{}\"\nstatus_every_ms = 100\nsync_every_ms = 100\n[consortium]\nmembers = {}\nthreshold = {threshold}\ncircle = {}\n[member]\nindex = {index}\nevery_ms = 100\npatience = 10\nview_patience = 20\n",
            node::hex(&document.id()), dir.display(), peers.join(","), list(&writers), dir.join("member.key").display(), dir.join("circle.key").display(), list(&members), list(&circle))
    }

    // Four new identities, n = 4, t = 3: two others' rounds before writing, then cuts (AT-NET-37).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn new_identities_write_and_keys_without_a_record_stop() {
        let root = scratch("processes");
        let dirs: Vec<PathBuf> = (0..4).map(|i| root.join(format!("m{i}"))).collect();
        for dir in &dirs {
            assert!(new_identity(dir).status.success());
        }
        let keys: Vec<[[u8; 32]; 4]> = dirs.iter().map(|d| publics(d)).collect();
        let ports: Vec<u16> = (0..4).map(|_| port()).collect();
        let address = |i: usize| {
            let key = libp2p::identity::ed25519::PublicKey::try_from_bytes(&keys[i][3]).unwrap();
            let peer = libp2p::identity::PublicKey::from(key).to_peer_id();
            format!("/ip4/127.0.0.1/tcp/{}/p2p/{peer}", ports[i])
        };
        let configs: Vec<PathBuf> = (0..4)
            .map(|i| {
                let path = root.join(format!("m{i}.toml"));
                let peers: Vec<String> = (0..4).filter(|j| *j != i).map(address).collect();
                fs::write(&path, toml(&dirs[i], i, &keys, 3, ports[i], &peers)).unwrap();
                path
            })
            .collect();
        let mut nodes: Vec<Process> = configs[..2].iter().map(|c| Process::start(c)).collect();
        for (i, other) in [(0, "1"), (1, "0")] {
            let line = format!("synced [{other}]\ncircle_synced [{other}]\n");
            let text = until(&dirs[i].join("status"), |s| s.contains(&line)).await;
            let status = node::parse(&text).unwrap();
            assert!(!status.caught_up && !status.circle_caught_up, "{text}");
            assert!(status.cuts.is_empty() && status.stopped.is_none(), "{text}");
        }
        nodes.extend(configs[2..].iter().map(|c| Process::start(c)));
        for dir in &dirs {
            until(&dir.join("status"), |s| {
                node::parse(s).is_some_and(|s| {
                    s.caught_up && s.circle_caught_up && s.cuts.len() >= 2 && s.stopped.is_none()
                })
            })
            .await;
        }
        for node in &mut nodes {
            node.terminate(Duration::from_secs(5)).await;
        }

        let restored = root.join("restored");
        fs::create_dir(&restored).unwrap();
        for name in SECRETS {
            fs::copy(dirs[0].join(name), restored.join(name)).unwrap();
        }
        let path = root.join("restored.toml");
        fs::write(&path, toml(&restored, 0, &keys, 3, port(), &[])).unwrap();
        let mut node = Process::start(&path);
        until(&restored.join("status"), |s| {
            s.contains("stopped missing\n")
        })
        .await;
        node.terminate(Duration::from_secs(5)).await;
        assert!(!restored.join("identity").exists());
    }

    // A FIFO in the record's place starts the node stopped, reported unreadable, kept (AT-NET-36).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_fifo_record_is_reported_unreadable() {
        let root = scratch("fifo-record");
        let dirs = [root.join("m0"), root.join("m1")];
        for dir in &dirs {
            assert!(new_identity(dir).status.success());
        }
        let keys = [publics(&dirs[0]), publics(&dirs[1])];
        let record = dirs[0].join("identity");
        fs::remove_file(&record).unwrap();
        assert!(Command::new("mkfifo")
            .arg(&record)
            .status()
            .unwrap()
            .success());
        let path = root.join("m0.toml");
        fs::write(&path, toml(&dirs[0], 0, &keys, 2, port(), &[])).unwrap();
        let mut node = Process::start(&path);
        until(&dirs[0].join("status"), |s| {
            s.contains("stopped unreadable\n")
        })
        .await;
        node.terminate(Duration::from_secs(5)).await;
        let kind = fs::symlink_metadata(&record).unwrap().file_type();
        assert!(std::os::unix::fs::FileTypeExt::is_fifo(&kind));
        assert!(!dirs[0].join("identity.next").exists());
    }
}
