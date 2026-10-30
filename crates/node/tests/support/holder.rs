//! Holder configurations, the node binary and file snapshots shared by the archive tests.

use super::support::{Fixture, LINEAGE};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

/// The node binary's output, or a failure if it has not exited within 30 s.
pub fn cli(args: &[&Path]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_isegoria-node"))
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let end = Instant::now() + Duration::from_secs(30);
    while child.try_wait().unwrap().is_none() {
        if Instant::now() > end {
            let _ = child.kill();
            panic!("{args:?} did not exit in time");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    child.wait_with_output().unwrap()
}

pub fn ok(args: &[&Path]) -> String {
    let out = cli(args);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

pub fn public(seed: u8) -> String {
    node::hex(
        &network::consortium::Member::from_seed([seed; 32])
            .public()
            .to_bytes(),
    )
}

pub fn list(values: &[String]) -> String {
    let quoted: Vec<_> = values.iter().map(|v| format!("\"{v}\"")).collect();
    format!("[{}]", quoted.join(","))
}

pub fn job(f: &Fixture, certificate: &Path, roster: &[u8]) -> String {
    let roster: Vec<_> = roster.iter().map(|s| public(*s)).collect();
    roster_job(f, certificate, &roster)
}

pub fn roster_job(f: &Fixture, certificate: &Path, roster: &[String]) -> String {
    format!("\n[[archive_maintenance.jobs]]\nlineage = \"{}\"\nprevious_cut_file = \"{}\"\ntarget_cut_file = \"{}\"\nview = 4\nexcluded = []\nmanifest_file = \"{}\"\nroster = {}\nbootstrap = []\n",
        node::hex(&LINEAGE), f.root.join("previous.cut").display(), f.root.join("target.cut").display(), certificate.display(), list(roster))
}

/// A tiered non-member holder named `name` with transport seed `seed` and `jobs`.
pub fn holder(f: &Fixture, name: &str, seed: u8, listen: &str, jobs: &str) -> String {
    let dir = f.root.join(name);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("transport.key"), node::hex(&[seed; 32])).unwrap();
    let base = f.node_text(0);
    let base: String = base
        .split("[member]")
        .next()
        .unwrap()
        .lines()
        .filter(|l| !l.starts_with("key_file"))
        .map(|l| format!("{l}\n"))
        .collect();
    base.replace(
        &f.root.join("node-state").display().to_string(),
        &dir.display().to_string(),
    )
    .replace(
        "listen = []",
        &format!("listen = [\"{listen}\"]\nstatus_every_ms = 50"),
    ) + &format!("\n[archive_maintenance]\ninterval_ms = 100\ndeadline_ms = 2000\n{jobs}")
}

pub fn config(text: &str) -> node::Config {
    toml::from_str(text).unwrap()
}

/// `name`'s holder configuration without archive jobs, in `<name>.toml`.
pub fn bare(f: &Fixture, name: &str, seed: u8) -> PathBuf {
    let text = holder(f, name, seed, "/ip4/127.0.0.1/tcp/0", "");
    let path = f.root.join(format!("{name}.toml"));
    fs::write(&path, text.split("\n[archive_maintenance]").next().unwrap()).unwrap();
    path
}

/// Every path under `dir`, with a regular file's bytes; nothing else is opened.
pub fn tree(dir: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let kind = fs::symlink_metadata(&path).unwrap().file_type();
        if kind.is_dir() {
            out.extend(tree(&path));
        }
        let bytes = kind.is_file().then(|| fs::read(&path).unwrap());
        out.push((path, bytes));
    }
    out.sort();
    out
}
