//! A tiered node's genesis (`docs/04` §Operating contract, T88-P1, AT-NET-40): the document its
//! TOML names, the repeated values compared with it, and the operators' digest command.

#[path = "support/genesis.rs"]
mod genesis;
#[cfg(unix)]
#[allow(dead_code)]
#[path = "support/process.rs"]
mod process;

use genesis::*;
use network::consortium::Member;
use network::genesis::{Genesis, GenesisError};
use node::genesis::Error;
use node::{hex, plan, Config, ConfigError};
use p2p::blind::GatewayKey;
use protocol::answer_path::AnswerConfig;
use protocol::answer_rule::ConsensusRule;
use std::path::{Path, PathBuf};

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("node_genesis")
        .join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The trial's shape: four members, three signing, two relays.
fn trial() -> Genesis {
    let members = (0..4).map(|i| seeded(1 + i, 11 + i, 21 + i)).collect();
    genesis(3, members, vec![key(41), key(42)])
}

fn list(keys: impl IntoIterator<Item = [u8; 32]>) -> String {
    let quoted: Vec<String> = keys
        .into_iter()
        .map(|k| format!("\"{}\"", hex(&k)))
        .collect();
    format!("[{}]", quoted.join(", "))
}

/// Member `i`'s signed key configuration under `g`, from HPKE seed `seed`, in hex.
fn announced(g: &Genesis, i: usize, seed: u8) -> String {
    let member = Member::from_seed([1 + i as u8; 32]);
    let key = GatewayKey::new(&g.tiers().unwrap(), &member, i, &[seed; 32]).unwrap();
    hex(&key.announcement().encode())
}

/// Member 0's gateway configuration under `g`, its document at `path`, its files in `root`.
fn member(root: &Path, g: &Genesis, path: &Path) -> String {
    for (name, seed) in [
        ("key", 1u8),
        ("circle", 11),
        ("hpke", 21),
        ("other-hpke", 29),
    ] {
        std::fs::write(root.join(name), hex(&[seed; 32])).unwrap();
    }
    let consortium: Vec<_> = g.members.iter().map(|m| m.consortium).collect();
    let circle: Vec<_> = g.members.iter().map(|m| m.circle).collect();
    let writers = consortium.iter().chain(&circle).copied();
    let hpke: Vec<String> = (0..4)
        .map(|i| format!("\"{}\"", announced(g, i, 21 + i as u8)))
        .collect();
    format!(
        "{}network_id = \"{}\"\ndata_dir = \"{}\"\nlisten = [\"/ip4/127.0.0.1/tcp/0\"]\n\
         writers = {}\nkey_file = \"{}\"\ncircle_key_file = \"{}\"\nhpke_key_file = \"{}\"\n\
         relays = {}\n\n[consortium]\nmembers = {}\nthreshold = {}\ncircle = {}\nhpke = [{}]\n\n\
         [member]\nindex = 0\nevery_ms = 100\ncuts_per_epoch = 2\npatience = 5\nview_patience = 10\n",
        lines(path, g),
        hex(&g.id()),
        root.join("data").display(),
        list(writers),
        root.join("key").display(),
        root.join("circle").display(),
        root.join("hpke").display(),
        list(g.relays.clone()),
        list(consortium.clone()),
        g.threshold,
        list(circle.clone()),
        hpke.join(", ")
    )
}

/// A scratch directory holding `g` and member 0's configuration text under it.
fn fixture(name: &str, g: &Genesis) -> (PathBuf, PathBuf, String) {
    let root = scratch(name);
    let path = root.join("genesis.bin");
    write(&path, g);
    let text = member(&root, g, &path);
    (root, path, text)
}

/// Plans `text` from a file in `root`; a refusal must leave no data directory.
fn planned(root: &Path, text: &str) -> Result<node::Plan, ConfigError> {
    let path = root.join("node.toml");
    std::fs::write(&path, text).unwrap();
    let result = Config::load(&path).and_then(plan);
    if result.is_err() {
        assert!(!root.join("data").exists(), "a refused start made files");
    }
    result
}

fn differs(field: &'static str) -> Result<(), ConfigError> {
    Err(ConfigError::Genesis(Error::Differs(field)))
}

fn without(text: &str, prefixes: &[&str]) -> String {
    let kept = text
        .lines()
        .filter(|l| !prefixes.iter().any(|p| l.starts_with(p)));
    kept.map(|l| format!("{l}\n")).collect()
}

const REPEATED: [&str; 4] = ["genesis =", "family =", "answer_config =", "issuer ="];

// A tiered node runs under the id its genesis derives, with the document's set (AT-NET-40).
#[test]
fn the_genesis_derives_the_network_id() {
    let g = trial();
    let (root, path, text) = fixture("derives", &g);
    let member = planned(&root, &text).unwrap();
    assert_eq!(member.p2p.writers.network_id(), g.id());
    let tiers = member.p2p.tiers.as_ref().unwrap();
    assert_eq!(
        (tiers.network_id(), tiers.context()),
        (g.id(), g.tiers().unwrap().context())
    );
    assert_eq!(member.p2p.gateway.as_ref().unwrap().relays, g.relays);
    let reversed = text.replace(
        &list(g.relays.clone()),
        &list(g.relays.iter().rev().copied()),
    );
    assert!(reversed != text && planned(&root, &reversed).is_ok());
    let reader = without(
        text.split("\n[member]").next().unwrap(),
        &["key_file", "circle_key_file", "hpke_key_file", "relays"],
    );
    let read = planned(&root, &reader).unwrap();
    assert!(read.p2p.tiers.is_some() && read.p2p.member.is_none());
    assert_eq!(
        read.p2p.writers.network_id(),
        Genesis::decode(&std::fs::read(path).unwrap()).unwrap().id()
    );
}

// A tiered TOML without its genesis, or repeating its values without one, refuses (AT-NET-40).
#[test]
fn a_tiered_node_without_its_genesis_refuses_to_start() {
    let g = trial();
    let (root, _, text) = fixture("missing", &g);
    let missing = Err(ConfigError::Genesis(Error::Missing));
    let free = without(&text, &REPEATED);
    assert_eq!(planned(&root, &free).map(|_| ()), missing);
    let other = free.replace(&hex(&g.id()), &hex(&[7; 32]));
    assert_eq!(planned(&root, &other).map(|_| ()), missing);
    let untiered = without(&text, &["genesis =", "circle ="])
        .split("\nhpke = ")
        .next()
        .unwrap()
        .to_string()
        + "\n";
    let untiered = without(&untiered, &["circle_key_file", "hpke_key_file", "relays"]);
    assert_eq!(planned(&root, &untiered).map(|_| ()), missing);
}

// Every TOML value that does not repeat the document's refuses the start, by field (AT-NET-40).
#[test]
fn a_toml_that_differs_from_the_genesis_refuses_to_start() {
    let g = trial();
    let (root, _, text) = fixture("differs", &g);
    let swap = |text: &str, a: [u8; 32], b: [u8; 32]| {
        let (a, b) = (hex(&a), hex(&b));
        text.replace(&a, "SWAP").replace(&b, &a).replace("SWAP", &b)
    };
    let (m, c) = (
        |i: usize| g.members[i].consortium,
        |i: usize| g.members[i].circle,
    );
    let other_config = AnswerConfig {
        rule: ConsensusRule::new(13, 9, 3, 4).unwrap(),
        ..AnswerConfig::default()
    };
    let other_issuer = identity::credential::Issuer::new([78; 32])
        .public()
        .encode();
    let hpke_of = |i: usize| announced(&g, i, 21 + i as u8);
    let ids = format!("\"{}\"", hex(&g.id()));
    let cases: Vec<(&str, String, &str)> = vec![
        (
            "free id",
            text.replace(&ids, &format!("\"{}\"", hex(&[7; 32]))),
            "network_id",
        ),
        (
            "reordered members",
            swap(&text, m(0), m(1)),
            "consortium.members",
        ),
        (
            "member key",
            text.replace(
                &format!("\"{}\"]\nthreshold", hex(&m(3))),
                &format!("\"{}\"]\nthreshold", hex(&key(9))),
            ),
            "consortium.members",
        ),
        (
            "reordered circle",
            swap(&text, c(2), c(3)),
            "consortium.circle",
        ),
        (
            "circle key",
            text.replace(
                &format!("\"{}\"]\nhpke", hex(&c(3))),
                &format!("\"{}\"]\nhpke", hex(&key(19))),
            ),
            "consortium.circle",
        ),
        (
            "no circle",
            without(&text, &["circle ="]),
            "consortium.circle",
        ),
        (
            "threshold 2",
            text.replace("threshold = 3", "threshold = 2"),
            "consortium.threshold",
        ),
        (
            "threshold 4",
            text.replace("threshold = 3", "threshold = 4"),
            "consortium.threshold",
        ),
        (
            "relay key",
            text.replace(&hex(&g.relays[1]), &hex(&key(43))),
            "relays",
        ),
        (
            "no relay",
            text.replace(&list(g.relays.clone()), &list([g.relays[0]])),
            "relays",
        ),
        (
            "extra relay",
            text.replace(
                &list(g.relays.clone()),
                &list([g.relays[0], g.relays[1], key(43)]),
            ),
            "relays",
        ),
        (
            "repeated relay",
            text.replace(
                &list(g.relays.clone()),
                &list([g.relays[0], g.relays[1], g.relays[1]]),
            ),
            "relays",
        ),
        (
            "family",
            text.replace("answer-review/v1", "answer-review/v2"),
            "family",
        ),
        ("no family", without(&text, &["family ="]), "family"),
        (
            "answer config",
            text.replace(&hex(&g.answer_config), &hex(&other_config.encode())),
            "answer_config",
        ),
        (
            "no answer config",
            without(&text, &["answer_config ="]),
            "answer_config",
        ),
        (
            "issuer",
            text.replace(&hex(&g.issuer), &hex(&other_issuer)),
            "issuer",
        ),
        ("no issuer", without(&text, &["issuer ="]), "issuer"),
        (
            "hpke key",
            text.replace(&hpke_of(1), &announced(&g, 1, 30)),
            "consortium.hpke",
        ),
        (
            "short hpke",
            text.replace(&format!(", \"{}\"]", hpke_of(3)), "]"),
            "consortium.hpke",
        ),
    ];
    for (name, edited, field) in cases {
        assert_ne!(edited, text, "{name}");
        assert_eq!(
            planned(&root, &edited).map(|_| ()),
            differs(field),
            "{name}"
        );
    }
}

// A document other than the one the TOML repeats refuses, also with the id updated (AT-NET-40).
#[test]
fn a_different_document_refuses_to_start() {
    let g = trial();
    let (root, path, text) = fixture("other-document", &g);
    let mut issuer = g.clone();
    issuer.issuer = identity::credential::Issuer::new([78; 32])
        .public()
        .encode();
    let mut relay = g.clone();
    relay.relays[0] = key(43);
    relay.relays.sort();
    let mut threshold = g.clone();
    threshold.threshold = 4;
    for (name, other, field) in [
        ("issuer", issuer, "issuer"),
        ("relay", relay, "relays"),
        ("threshold", threshold, "consortium.threshold"),
    ] {
        std::fs::write(&path, other.encode()).unwrap();
        assert_eq!(
            planned(&root, &text).map(|_| ()),
            differs("network_id"),
            "{name}"
        );
        let id = text.replace(&hex(&g.id()), &hex(&other.id()));
        assert_eq!(planned(&root, &id).map(|_| ()), differs(field), "{name}");
    }
}

// A missing, non-canonical or invalid document, or another configuration, refuses (AT-NET-40).
#[test]
fn a_document_that_does_not_hold_refuses_to_start() {
    let g = trial();
    let (root, path, text) = fixture("invalid", &g);
    let decode = |e| Err(ConfigError::Genesis(Error::Decode(e)));
    std::fs::remove_file(&path).unwrap();
    let not_found = Err(ConfigError::Read(std::io::ErrorKind::NotFound));
    assert_eq!(planned(&root, &text).map(|_| ()), not_found);
    std::fs::create_dir(&path).unwrap();
    assert_eq!(
        planned(&root, &text).map(|_| ()),
        Err(ConfigError::Bounds("genesis"))
    );
    std::fs::remove_dir(&path).unwrap();
    std::fs::write(&path, [g.encode(), vec![0]].concat()).unwrap();
    assert_eq!(
        planned(&root, &text).map(|_| ()),
        decode(GenesisError::Malformed)
    );
    let mut repeated = g.clone();
    repeated.members[1].hpke = repeated.members[0].hpke;
    std::fs::write(&path, repeated.encode()).unwrap();
    assert_eq!(
        planned(&root, &text).map(|_| ()),
        decode(GenesisError::Repeated)
    );
    let mut config = g.clone();
    config.answer_config = [g.answer_config.clone(), vec![0]].concat();
    std::fs::write(&path, config.encode()).unwrap();
    let answer = Err(ConfigError::Genesis(Error::AnswerConfig));
    assert_eq!(planned(&root, &text).map(|_| ()), answer);
    let other = AnswerConfig {
        divided_weight: 0.5,
        ..AnswerConfig::default()
    };
    for valid in [
        other,
        AnswerConfig {
            rule: ConsensusRule::new(12, 8, 3, 4).unwrap(),
            ..other
        },
    ] {
        config.answer_config = valid.encode();
        assert_eq!(AnswerConfig::decode(&config.answer_config), Some(valid));
        std::fs::write(&path, config.encode()).unwrap();
        let pinned = text.replace(&hex(&g.id()), &hex(&config.id()));
        assert_eq!(planned(&root, &pinned).map(|_| ()), answer, "{valid:?}");
    }
    let mut issuer = g.clone();
    issuer.issuer[1] ^= 0x40;
    std::fs::write(&path, issuer.encode()).unwrap();
    assert_eq!(
        planned(&root, &text).map(|_| ()),
        Err(ConfigError::Genesis(Error::Issuer))
    );
}

// A member announces only the HPKE key its genesis holds (AT-NET-40).
#[test]
fn a_member_announces_only_its_genesis_hpke_key() {
    let g = trial();
    let (root, _, text) = fixture("announce", &g);
    let path = root.join("node.toml");
    std::fs::write(&path, &text).unwrap();
    let signed = node::announcement(&Config::load(&path).unwrap()).unwrap();
    assert_eq!(signed, announced(&g, 0, 21));
    let other = text.replace(
        &format!("hpke_key_file = \"{}\"", root.join("hpke").display()),
        &format!("hpke_key_file = \"{}\"", root.join("other-hpke").display()),
    );
    std::fs::write(&path, &other).unwrap();
    let refused = node::announcement(&Config::load(&path).unwrap()).map(|_| ());
    assert_eq!(refused, differs("hpke_key_file"));
}

#[cfg(unix)]
mod processes {
    use super::*;
    use std::process::{Command, Output};

    use crate::process::{until, Process};

    /// Every path under `dir`, with a regular file's bytes.
    fn tree(dir: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                out.extend(tree(&path));
            }
            let bytes = path.is_file().then(|| std::fs::read(&path).unwrap());
            out.push((path, bytes));
        }
        out.sort();
        out
    }

    fn digest(path: &Path, cwd: &Path) -> Output {
        Command::new(env!("CARGO_BIN_EXE_isegoria-node"))
            .arg("--genesis-digest")
            .arg(path)
            .current_dir(cwd)
            .output()
            .unwrap()
    }

    const DIGEST: &str = "c807d5e01b746488cf6cf6cfee4f3b0f64b4e88ab36bd435bf89d37696ce1381";
    const ID: &str = "4fad36d18e819ebd789fcd9d4cdb4262d28827b4f9bc920dd0b3642410240169";

    /// The command's text for the trial's document: known digest and id, then the document.
    fn expected(g: &Genesis) -> String {
        let mut text = format!("digest {DIGEST}\nnetwork_id {ID}\nversion 1\nthreshold 3\n");
        for (i, m) in g.members.iter().enumerate() {
            let keys = [m.consortium, m.circle, m.hpke].map(|k| hex(&k));
            text += &format!("member {i} {}\n", keys.join(" "));
        }
        text += "family answer-review/v1\nanswer_config group 12 quorum 9 threshold 3/4 \
                 divided_weight 0.25 low 0.3 high 0.7 quotas 1 2 3 new_author 2 \
                 half_life_closings 45\n";
        text += &format!("issuer {}\n", hex(&g.issuer));
        for relay in &g.relays {
            text += &format!("relay {}\n", hex(relay));
        }
        text
    }

    // The digest command prints the known answers, writes nothing, also beside a node (AT-NET-40).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn the_digest_command_prints_the_known_answers_and_writes_nothing() {
        let g = trial();
        let root = scratch("digest");
        let (confirm, cwd) = (root.join("confirm"), root.join("cwd"));
        std::fs::create_dir_all(&confirm).unwrap();
        std::fs::create_dir_all(&cwd).unwrap();
        let path = confirm.join("genesis.bin");
        let lines = write(&path, &g);
        let before = (tree(&confirm), tree(&cwd));
        let out = digest(&path, &cwd);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(String::from_utf8(out.stdout).unwrap(), expected(&g));
        assert_eq!((tree(&confirm), tree(&cwd)), before);
        let reader = format!(
            "{lines}network_id = \"{}\"\ndata_dir = \"{}\"\nlisten = [\"/ip4/127.0.0.1/tcp/0\"]\n\
             writers = {}\nstatus_every_ms = 50\n\n[consortium]\nmembers = {}\nthreshold = 3\n\
             circle = {}\n",
            hex(&g.id()),
            root.join("reader").display(),
            list(g.members.iter().flat_map(|m| [m.consortium, m.circle])),
            list(g.members.iter().map(|m| m.consortium)),
            list(g.members.iter().map(|m| m.circle)),
        );
        let config = root.join("reader.toml");
        std::fs::write(&config, reader).unwrap();
        let mut running = Process::start(&config);
        until(&root.join("reader/status"), |s| !s.is_empty()).await;
        let again = digest(&path, &cwd);
        assert_eq!(String::from_utf8(again.stdout).unwrap(), expected(&g));
        assert_eq!((tree(&confirm), tree(&cwd)), before);
        running.terminate(std::time::Duration::from_secs(5)).await;
        std::fs::write(&path, [g.encode(), vec![0]].concat()).unwrap();
        let refused = digest(&path, &cwd);
        assert_eq!(refused.status.code(), Some(2));
        assert!(refused.stdout.is_empty());
        assert!(String::from_utf8_lossy(&refused.stderr).contains("Malformed"));
    }

    // A node process whose TOML differs from its genesis exits 2, writing nothing (AT-NET-40).
    #[test]
    fn a_node_given_another_configuration_does_not_start() {
        let g = trial();
        let (root, _, text) = fixture("process", &g);
        let config = root.join("node.toml");
        std::fs::write(&config, text.replace("threshold = 3", "threshold = 2")).unwrap();
        let mut child = Command::new(env!("CARGO_BIN_EXE_isegoria-node"))
            .arg(&config)
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let end = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while child.try_wait().unwrap().is_none() {
            if std::time::Instant::now() > end {
                let _ = child.kill();
                let _ = child.wait();
                panic!("the node started");
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let out = child.wait_with_output().unwrap();
        assert_eq!(out.status.code(), Some(2));
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("Differs(\"consortium.threshold\")"),
            "{stderr}"
        );
        assert!(!root.join("data").exists());
    }
}
