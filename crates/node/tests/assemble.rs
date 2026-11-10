//! `--genesis-assemble` (`docs/04` §Operating contract, T88-P1, AT-NET-41, 42): the canonical
//! genesis from the operators' printed keys and the network's parameters, written to one new file.

#[path = "support/genesis.rs"]
mod genesis;

use genesis::*;
use network::genesis::{Genesis, GenesisError};
use node::genesis::{assemble, load, render, Error};
use node::{hex, plan, Config, ConfigError};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("node_assemble")
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

/// The assembly input for `g`, its relays listed in reverse order.
fn input(g: &Genesis) -> String {
    let mut text = format!(
        "threshold = {}\nfamily = \"answer-review/v1\"\nissuer = \"{}\"\nrelays = {}\n",
        g.threshold,
        hex(&g.issuer),
        list(g.relays.iter().rev().copied())
    );
    for m in &g.members {
        text += &format!(
            "\n[[member]]\nconsortium = \"{}\"\ncircle = \"{}\"\nhpke = \"{}\"\n",
            hex(&m.consortium),
            hex(&m.circle),
            hex(&m.hpke)
        );
    }
    text
}

/// Assembles `text` in `root`; a refusal must leave no output.
fn assembled(root: &Path, text: &str) -> Result<node::genesis::Document, ConfigError> {
    let (path, output) = (root.join("genesis.toml"), root.join("genesis.bin"));
    std::fs::write(&path, text).unwrap();
    let _ = std::fs::remove_file(&output);
    let result = assemble(&path, &output);
    if result.is_err() {
        assert!(!output.exists(), "a refused assembly wrote its output");
    }
    result
}

fn refused(field: &str, error: GenesisError) -> Result<(), ConfigError> {
    Err(ConfigError::Genesis(Error::Input(field.to_string(), error)))
}

/// A reader's TOML naming the document at `path` and repeating its values.
fn reader(root: &Path, g: &Genesis, path: &Path) -> String {
    format!(
        "{}network_id = \"{}\"\ndata_dir = \"{}\"\nlisten = [\"/ip4/127.0.0.1/tcp/0\"]\n\
         writers = {}\n\n[consortium]\nmembers = {}\nthreshold = {}\ncircle = {}\n",
        lines(path, g),
        hex(&g.id()),
        root.join("reader").display(),
        list(g.members.iter().flat_map(|m| [m.consortium, m.circle])),
        list(g.members.iter().map(|m| m.consortium)),
        g.threshold,
        list(g.members.iter().map(|m| m.circle)),
    )
}

// The input assembles the canonical document, which loads and a tiered TOML accepts (AT-NET-41).
#[test]
fn the_input_assembles_the_canonical_document() {
    let g = trial();
    let root = scratch("canonical");
    let document = assembled(&root, &input(&g)).unwrap();
    let path = root.join("genesis.bin");
    assert_eq!(std::fs::read(&path).unwrap(), g.encode());
    assert_eq!(document.genesis, g);
    assert_eq!(
        hex(&document.digest),
        "c807d5e01b746488cf6cf6cfee4f3b0f64b4e88ab36bd435bf89d37696ce1381"
    );
    assert_eq!(render(&document), render(&load(&path).unwrap()));
    let config = root.join("reader.toml");
    std::fs::write(&config, reader(&root, &g, &path)).unwrap();
    let planned = Config::load(&config).and_then(plan).unwrap();
    assert_eq!(planned.p2p.writers.network_id(), g.id());
}

// The configuration's printed hex is the value a tiered TOML repeats (AT-NET-42).
#[test]
fn the_printed_answer_config_is_what_a_toml_repeats() {
    let g = trial();
    let root = scratch("printed-config");
    let printed = render(&assembled(&root, &input(&g)).unwrap());
    let mut lines = printed.lines();
    let config = lines.find_map(|l| l.strip_prefix("answer_config_hex "));
    let config = config.expect("the hex line");
    let text = reader(&root, &g, &root.join("genesis.bin"));
    let text = text.replace(&hex(&g.answer_config), config);
    assert!(text.contains(&format!("answer_config = \"{config}\"\n")));
    let path = root.join("reader.toml");
    std::fs::write(&path, text).unwrap();
    assert!(Config::load(&path).and_then(plan).is_ok());
}

// An existing output, a file or a directory, is refused and left as it was (AT-NET-41).
#[test]
fn an_existing_output_is_never_overwritten() {
    let g = trial();
    let root = scratch("existing");
    let (path, output) = (root.join("genesis.toml"), root.join("genesis.bin"));
    std::fs::write(&path, input(&g)).unwrap();
    let exists = Err(ConfigError::Read(std::io::ErrorKind::AlreadyExists));
    for before in [b"other".to_vec(), g.encode()] {
        std::fs::write(&output, &before).unwrap();
        assert_eq!(assemble(&path, &output).map(|_| ()), exists);
        assert_eq!(std::fs::read(&output).unwrap(), before);
    }
    std::fs::remove_file(&output).unwrap();
    std::fs::create_dir(&output).unwrap();
    assert_eq!(assemble(&path, &output).map(|_| ()), exists);
    assert_eq!(std::fs::read_dir(&output).unwrap().count(), 0);
    let missing = root.join("missing.toml");
    let not_found = Err(ConfigError::Read(std::io::ErrorKind::NotFound));
    assert_eq!(
        assemble(&missing, &root.join("new.bin")).map(|_| ()),
        not_found
    );
    assert!(!root.join("new.bin").exists());
}

// Each invalid, repeated or miscounted input is refused by its field, writing nothing (AT-NET-41).
#[test]
fn an_input_that_does_not_hold_is_refused_by_field() {
    use GenesisError::*;
    let g = trial();
    let root = scratch("refused");
    let text = input(&g);
    let m = |i: usize| g.members[i];
    let mut p = [0xff; 32];
    p[0] = 0xed;
    p[31] = 0x7f;
    let mut identity = [0; 32];
    identity[0] = 1;
    let mut flipped = g.issuer.clone();
    flipped[1] ^= 0x40;
    let swap = |from: [u8; 32], to: &str, nth: usize| {
        let from = hex(&from);
        let at = text.match_indices(&from).nth(nth).unwrap().0;
        format!("{}{to}{}", &text[..at], &text[at + from.len()..])
    };
    let members = text.split("\n[[member]]").next().unwrap().to_string();
    let block = format!("\n[[member]]{}", text.split("\n[[member]]").nth(1).unwrap());
    let cases: Vec<(&str, String, Result<(), ConfigError>)> = vec![
        (
            "family",
            text.replace("answer-review/v1", "answer-review/v2"),
            refused("family", Family),
        ),
        (
            "threshold 0",
            text.replace("threshold = 3", "threshold = 0"),
            refused("threshold", Threshold),
        ),
        (
            "threshold 5",
            text.replace("threshold = 3", "threshold = 5"),
            refused("threshold", Threshold),
        ),
        ("no member", members.clone(), refused("member", Members)),
        (
            "257 members",
            members.clone() + &block.repeat(257),
            refused("member", Members),
        ),
        (
            "not hex",
            swap(m(1).consortium, &"zz".repeat(32), 0),
            refused("member[1].consortium", Key),
        ),
        (
            "short",
            swap(m(1).consortium, &hex(&m(1).consortium)[2..], 0),
            refused("member[1].consortium", Key),
        ),
        (
            "small order",
            swap(m(2).circle, &hex(&identity), 0),
            refused("member[2].circle", Key),
        ),
        (
            "hpke over p",
            swap(m(0).hpke, &hex(&p), 0),
            refused("member[0].hpke", Key),
        ),
        (
            "repeated member",
            swap(m(3).consortium, &hex(&m(0).consortium), 0),
            refused("member[3].consortium", Repeated),
        ),
        (
            "across roles",
            swap(m(1).circle, &hex(&m(0).consortium), 0),
            refused("member[1].circle", Repeated),
        ),
        (
            "repeated hpke",
            swap(m(2).hpke, &hex(&m(0).hpke), 0),
            refused("member[2].hpke", Repeated),
        ),
        (
            "repeated relay",
            swap(g.relays[0], &hex(&g.relays[1]), 0),
            refused("relays[1]", Repeated),
        ),
        (
            "relay a member's",
            swap(g.relays[1], &hex(&m(2).circle), 0),
            refused("relays[0]", Repeated),
        ),
        (
            "relay not hex",
            swap(g.relays[1], "relay", 0),
            refused("relays[0]", Key),
        ),
        (
            "issuer",
            text.replace(&hex(&g.issuer), &hex(&flipped)),
            Err(ConfigError::Genesis(Error::Issuer)),
        ),
        (
            "issuer odd",
            text.replace(&hex(&g.issuer), &hex(&g.issuer)[1..]),
            Err(ConfigError::Genesis(Error::Issuer)),
        ),
    ];
    for (name, edited, expected) in cases {
        assert_ne!(edited, text, "{name}");
        assert_eq!(assembled(&root, &edited).map(|_| ()), expected, "{name}");
    }
    let transport = text.replacen("\nhpke = ", "\ntransport = \"00\"\nhpke = ", 1);
    let unknown = text.replace("threshold = 3", "threshold = 3\nanswer_config = \"00\"");
    for (name, edited, word) in [
        ("transport", transport, "transport"),
        ("answer_config", unknown, "answer_config"),
        ("no issuer", text.replace("issuer =", "#"), "issuer"),
    ] {
        let parse = assembled(&root, &edited).map(|_| ());
        assert!(
            matches!(&parse, Err(ConfigError::Parse(e)) if e.contains(word)),
            "{name}: {parse:?}"
        );
    }
}

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

fn run(args: &[&Path], cwd: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_isegoria-node"))
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap()
}

// Assembling the printed keys writes only the file `--genesis-digest` shows alike (AT-NET-41).
#[test]
fn the_printed_keys_assemble_what_the_digest_command_shows() {
    let root = scratch("process");
    let (cwd, out) = (root.join("cwd"), root.join("out"));
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(&out).unwrap();
    let flag = |f: &str| PathBuf::from(f);
    let mut text = format!(
        "threshold = 3\nfamily = \"answer-review/v1\"\nissuer = \"{}\"\nrelays = {}\n",
        hex(&issuer()),
        list([key(42), key(41)])
    );
    let mut printed = Vec::new();
    for i in 0..4 {
        let identity = run(
            &[&flag("--new-identity"), &root.join(format!("m{i}"))],
            &cwd,
        );
        assert!(identity.status.success());
        let keys = String::from_utf8(identity.stdout).unwrap();
        text += "\n[[member]]\n";
        for line in keys.lines().filter(|l| !l.starts_with("transport ")) {
            let (name, key) = line.split_once(' ').unwrap();
            text += &format!("{name} = \"{key}\"\n");
        }
        printed.push(keys);
    }
    let path = root.join("genesis.toml");
    std::fs::write(&path, &text).unwrap();
    let output = out.join("genesis.bin");
    let before = (tree(&root), tree(&cwd));
    let assemble = flag("--genesis-assemble");
    let made = run(&[&assemble, &path, &output], &cwd);
    assert!(
        made.status.success(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );
    let mut after = before.0.clone();
    after.push((output.clone(), Some(std::fs::read(&output).unwrap())));
    after.sort();
    assert_eq!((tree(&root), tree(&cwd)), (after.clone(), before.1.clone()));
    let shown = run(&[&flag("--genesis-digest"), &output], &cwd);
    assert_eq!(
        String::from_utf8(shown.stdout).unwrap(),
        String::from_utf8(made.stdout).unwrap()
    );
    let document = load(&output).unwrap();
    for (i, keys) in printed.iter().enumerate() {
        let m = document.genesis.members[i];
        let expected = format!(
            "consortium {}\ncircle {}\nhpke {}\n",
            hex(&m.consortium),
            hex(&m.circle),
            hex(&m.hpke)
        );
        assert!(keys.starts_with(&expected), "member {i}");
    }
    let again = run(&[&assemble, &path, &output], &cwd);
    assert_eq!(again.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&again.stderr).contains("AlreadyExists"));
    std::fs::write(&path, text.replace("threshold = 3", "threshold = 5")).unwrap();
    let other = out.join("other.bin");
    let invalid = run(&[&assemble, &path, &other], &cwd);
    assert_eq!(invalid.status.code(), Some(2));
    assert!(invalid.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&invalid.stderr);
    assert!(
        stderr.contains("Input(\"threshold\", Threshold)"),
        "{stderr}"
    );
    let mut written = after;
    written.retain(|(p, _)| p != &path);
    written.push((path.clone(), Some(std::fs::read(&path).unwrap())));
    written.sort();
    assert_eq!(tree(&root), written);
}
