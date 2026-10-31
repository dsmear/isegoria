//! Collected cut export from a stopped node's replica (`docs/04` §Collected cut export from a
//! stopped node's replica).

#[allow(dead_code)]
#[path = "support/holder.rs"]
mod holder;
#[allow(dead_code)]
#[cfg(unix)]
#[path = "support/process.rs"]
mod process;
#[allow(dead_code)]
#[path = "support/replica.rs"]
mod replica;
#[allow(dead_code)]
#[path = "support/production.rs"]
mod support;
use curve25519_dalek::Scalar;
use ed25519_dalek::{Signature, SigningKey, Verifier};
use holder::*;
use network::cid::cid;
use network::cut::{Cut, MemberObject};
use network::log::TransparencyLog;
use network::replica::{DurableReplica, FeedWriter};
use node::extract::{export_cut, extract, CutConfig};
use node::production;
use replica::*;
use sha2::{Digest, Sha256, Sha512};
use std::fs;
use std::path::{Path, PathBuf};
use support::*;

const EXPORT: &str = "--archive-export-cut";

/// A `cut.toml` for `number`, `view` and `excluded`, written as `<name>.toml`.
fn cut_toml(f: &Fixture, name: &str, number: u64, view: u64, excluded: &[usize]) -> PathBuf {
    let path = f.root.join(format!("{name}.toml"));
    let text = format!("number = {number}\nview = {view}\nexcluded = {excluded:?}\n");
    fs::write(&path, text).unwrap();
    path
}

fn wanted(number: u64, view: u64, excluded: &[usize]) -> CutConfig {
    CutConfig {
        number,
        view,
        excluded: excluded.to_vec(),
    }
}

/// Member `index`'s signature of `cut` in `view` with the identity as R: lenient Ed25519
/// verification accepts it, `verify_strict` refuses it.
fn lenient_only(f: &Fixture, index: usize, cut: &Cut, view: u64) -> Signature {
    let key = SigningKey::from_bytes(&[index as u8 + 1; 32]);
    assert_eq!(key.verifying_key(), f.members[index].public());
    let cp = cut.checkpoint_in(view, NET, f.tiers.consortium().member_set_hash());
    let mut h = Sha256::new();
    let height = cp.height.to_le_bytes();
    let fields: [&[u8]; 5] = [
        b"isegoria/checkpoint/v2",
        &cp.network_id,
        &cp.member_set_hash,
        &height,
        &cp.head,
    ];
    for field in fields {
        h.update((field.len() as u64).to_le_bytes());
        h.update(field);
    }
    let message: [u8; 32] = h.finalize().into();
    let mut r = [0u8; 32];
    r[0] = 1;
    let wide = Sha512::new()
        .chain_update(r)
        .chain_update(key.verifying_key().as_bytes())
        .chain_update(message)
        .finalize();
    let k = Scalar::from_bytes_mod_order_wide(&wide.as_slice().try_into().unwrap());
    let bytes = [r, (k * key.to_scalar()).to_bytes()].concat();
    let signature = Signature::from_bytes(&bytes.try_into().unwrap());
    assert!(key.verifying_key().verify(&message, &signature).is_ok());
    assert!(key
        .verifying_key()
        .verify_strict(&message, &signature)
        .is_err());
    signature
}

/// Puts `signature` by member `index` on `cut` in `view` first on that member's feed in `dir`.
fn add_signature(f: &Fixture, dir: &Path, index: usize, cut: &Cut, view: u64, sig: Signature) {
    let writers = f.tiers.writers().clone();
    let mut replica = DurableReplica::open(&dir.join("replica"), writers).unwrap();
    let object = MemberObject::CutSignature {
        cut: cut.clone(),
        view,
        member: index,
        signature: sig,
        named: Vec::new(),
    }
    .encode();
    let writer = FeedWriter::from_seed(NET, [index as u8 + 1; 32]);
    let entry = writer.sign(TransparencyLog::new().append(cid(&object)));
    replica.insert(entry, object).unwrap();
}

/// Runs each `(name, node, wanted, expected error)` export of `dir`'s replica and checks that it
/// changed no file there and published nothing.
fn refused(f: &Fixture, dir: &Path, cases: Vec<(&str, String, CutConfig, &str)>) {
    for (name, node, wanted, expected) in cases {
        let output = f.root.join(format!("{name}.cut"));
        let before = tree(dir);
        let result = export_cut(&config(&node), &wanted, &output).map(|_| ());
        assert_eq!(format!("{result:?}"), format!("Err({expected})"), "{name}");
        assert_eq!(tree(dir), before, "{name}");
        assert!(!output.exists(), "{name}");
    }
}

// Exported cuts are their canonical bytes and feed extraction and preparation (AT-NET-35).
#[test]
fn exported_cuts_feed_extraction_and_preparation() {
    let f = Fixture::new("export-chain", false);
    let dir = f.root.join("member");
    replica(
        &f,
        &dir,
        &[(&f.previous, 2, &[0, 2]), (&f.target, 4, &[0, 1, 2])],
    );
    let node = f.root.join("member.toml");
    fs::write(&node, member(&f, &dir)).unwrap();
    let before = tree(&dir);
    let implicit = f.root.join("implicit.toml");
    fs::write(&implicit, "number = 8\nview = 4\n").unwrap();
    let unset = cli(&[
        Path::new(EXPORT),
        &node,
        &implicit,
        &f.root.join("implicit.cut"),
    ]);
    assert_eq!(unset.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unset.stderr).contains("cut config"));
    assert!(!f.root.join("implicit.cut").exists());
    let (previous, target) = (f.root.join("previous.out"), f.root.join("target.out"));
    for (cut, wanted, output) in [
        (&f.previous, cut_toml(&f, "previous", 7, 2, &[1]), &previous),
        (&f.target, cut_toml(&f, "target", 8, 4, &[]), &target),
    ] {
        let printed = ok(&[Path::new(EXPORT), &node, &wanted, output]);
        assert_eq!(printed, format!("{}\n", node::hex(&cut.digest())));
        assert_eq!(fs::read(output).unwrap(), cut.encode());
    }
    let input = production::Config {
        previous_cut_file: Some(previous.clone()),
        target_cut_file: target.clone(),
        segment_file: f.root.join("exported.bin"),
        ..f.input()
    };
    let production = f.root.join("production.toml");
    fs::write(&production, text(&input)).unwrap();
    ok(&[Path::new("--archive-extract"), &node, &production]);
    let segment = f.segment.encode().unwrap();
    assert_eq!(fs::read(&input.segment_file).unwrap(), segment);
    let bundle = f.root.join("bundle");
    ok(&[Path::new("--archive-prepare"), &node, &production, &bundle]);
    for (name, exported) in [("target.cut", &target), ("previous.cut", &previous)] {
        assert_eq!(
            fs::read(bundle.join(name)).unwrap(),
            fs::read(exported).unwrap()
        );
    }
    let mut after = tree(&dir);
    after.retain(|(path, _)| *path != dir.join("lock"));
    assert_eq!(after, before);
}

// Uncollected, other-view, short, conflicting or leniently signed cuts are refused (AT-NET-35).
#[test]
fn uncertified_cuts_are_refused_without_change() {
    let f = Fixture::new("export-refusals", false);
    let dir = f.root.join("member");
    replica(
        &f,
        &dir,
        &[(&f.previous, 2, &[0, 2]), (&f.target, 4, &[0, 1, 2])],
    );
    fs::write(dir.join("lock"), b"").unwrap();
    let node = member(&f, &dir);
    let untiered: String = node
        .lines()
        .filter(|l| !l.starts_with("circle ="))
        .map(|l| format!("{l}\n"))
        .collect();
    let threshold = node.replace("threshold = 2", "threshold = 3");
    let missing = "Invalid(\"cut not collected in replica\")";
    let exclusions = "Invalid(\"excluded members\")";
    let view = "Invalid(\"cut collected in another view\")";
    refused(
        &f,
        &dir,
        vec![
            ("other number", node.clone(), wanted(9, 4, &[]), missing),
            ("other view", node.clone(), wanted(8, 3, &[]), view),
            ("excluded signer", node.clone(), wanted(7, 2, &[0]), missing),
            ("threshold", threshold, wanted(7, 2, &[]), missing),
            ("out of range", node.clone(), wanted(8, 4, &[3]), exclusions),
            ("repeated", node.clone(), wanted(8, 4, &[1, 1]), exclusions),
            (
                "untiered",
                untiered,
                wanted(8, 4, &[]),
                "Invalid(\"tiered network required\")",
            ),
        ],
    );
    let mut rival = f.target.clone();
    rival.epoch = 3;
    let conflicted = f.root.join("conflicted");
    replica(
        &f,
        &conflicted,
        &[(&f.target, 4, &[0, 1, 2]), (&rival, 4, &[1, 2])],
    );
    fs::write(conflicted.join("lock"), b"").unwrap();
    let conflict = "Invalid(\"conflicting cuts for the number\")";
    let node = member(&f, &conflicted);
    refused(
        &f,
        &conflicted,
        vec![("conflict", node, wanted(8, 4, &[]), conflict)],
    );
    let lenient = f.root.join("lenient");
    replica(&f, &lenient, &[(&f.target, 4, &[0])]);
    let signature = lenient_only(&f, 1, &f.target, 4);
    add_signature(&f, &lenient, 1, &f.target, 4, signature);
    fs::write(lenient.join("lock"), b"").unwrap();
    let node = member(&f, &lenient);
    let signatures = "Archive(Signatures)";
    refused(
        &f,
        &lenient,
        vec![("lenient", node.clone(), wanted(8, 4, &[]), signatures)],
    );
    let input = production::Config {
        segment_file: f.root.join("lenient.bin"),
        ..f.input()
    };
    let result = extract(&config(&node), &input).map(|_| ());
    assert_eq!(format!("{result:?}"), format!("Err({signatures})"));
}

// Missing or damaged replica files and existing outputs are refused without change (AT-NET-35).
#[test]
fn damaged_replica_or_existing_output_changes_nothing() {
    let f = Fixture::new("export-damage", false);
    let dir = f.root.join("member");
    replica(&f, &dir, &[(&f.target, 4, &[0, 1, 2])]);
    fs::write(dir.join("lock"), b"").unwrap();
    let node = member(&f, &dir);
    let (entries, objects) = (dir.join("replica/entries"), dir.join("replica/objects"));
    let saved = [fs::read(&entries).unwrap(), fs::read(&objects).unwrap()];
    let torn = [saved[0].clone(), vec![1, 2, 3]].concat();
    let mut altered = saved[1].clone();
    *altered.last_mut().unwrap() ^= 1;
    for (name, path, bytes, expected) in [
        ("torn", &entries, torn, "Replica(Store(Torn))"),
        ("altered", &objects, altered, "Replica(Damaged)"),
    ] {
        fs::write(path, bytes).unwrap();
        refused(
            &f,
            &dir,
            vec![(name, node.clone(), wanted(8, 4, &[]), expected)],
        );
        fs::write(&entries, &saved[0]).unwrap();
        fs::write(&objects, &saved[1]).unwrap();
    }
    let moved = f.root.join("moved");
    fs::rename(dir.join("replica"), &moved).unwrap();
    let absent = "Replica(Store(Io(NotFound)))";
    refused(
        &f,
        &dir,
        vec![("absent", node.clone(), wanted(8, 4, &[]), absent)],
    );
    assert!(!dir.join("replica").exists());
    fs::rename(&moved, dir.join("replica")).unwrap();
    let output = f.root.join("existing.cut");
    fs::write(&output, b"existing").unwrap();
    let result = export_cut(&config(&node), &wanted(8, 4, &[]), &output).map(|_| ());
    assert_eq!(format!("{result:?}"), "Err(Io(AlreadyExists))");
    assert_eq!(fs::read(&output).unwrap(), b"existing");
    let fresh = f.root.join("fresh.cut");
    export_cut(&config(&node), &wanted(8, 4, &[]), &fresh).unwrap();
    assert_eq!(fs::read(&fresh).unwrap(), f.target.encode());
}

#[cfg(unix)]
mod processes {
    use super::process::{address, until, Process};
    use super::*;
    use network::cut::{collect_with, Collected};
    use std::time::Duration;

    // A running node's lock excludes the export, which succeeds once it stops (AT-NET-35).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_running_node_excludes_the_export() {
        let f = Fixture::new("export-running", false);
        let node = bare(&f, "holder", 41);
        let dir = f.root.join("holder");
        replica(&f, &dir, &[(&f.target, 4, &[0, 1, 2])]);
        let wanted = cut_toml(&f, "target", 8, 4, &[]);
        let output = f.root.join("running.cut");
        let command = [Path::new(EXPORT), &node, &wanted, &output];
        let mut running = Process::start(&node);
        until(&dir.join("status"), |s| !s.is_empty()).await;
        let refused = cli(&command);
        assert_eq!(refused.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&refused.stderr).contains("data directory in use"));
        assert!(!output.exists());
        running.terminate(Duration::from_secs(5)).await;
        ok(&command);
        assert_eq!(fs::read(&output).unwrap(), f.target.encode());
    }

    // Cuts tiered member processes collect export as they reported and feed extraction (AT-NET-35).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cuts_collected_by_member_processes_are_exported() {
        let f = Fixture::new("export-runtime", false);
        let addresses: Vec<_> = (0..3).map(|i| address(90 + i)).collect();
        let configs: Vec<PathBuf> = (0..3)
            .map(|i| {
                let dir = f.root.join(format!("runtime-{i}"));
                fs::create_dir_all(&dir).unwrap();
                fs::write(dir.join("transport.key"), node::hex(&[90 + i as u8; 32])).unwrap();
                let key = |s: u8| {
                    network::consortium::Member::from_seed([s; 32])
                        .public()
                        .to_bytes()
                };
                let record = dir.join("identity");
                p2p::record::create(&record, key(1 + i as u8), key(11 + i as u8)).unwrap();
                let circle = f.root.join(format!("circle-{i}.key"));
                fs::write(&circle, node::hex(&[11 + i as u8; 32])).unwrap();
                let peers: Vec<_> = (0..3)
                    .filter(|j| *j != i)
                    .map(|j| addresses[j].clone())
                    .collect();
                let listen = addresses[i].split("/p2p/").next().unwrap();
                let settings = format!(
                    "listen = [\"{listen}\"]\npeers = {}\nstatus_every_ms = 100",
                    list(&peers)
                );
                let circle = format!("\ncircle_key_file = \"{}\"\n[consortium]", circle.display());
                let text = f
                    .node_text(i)
                    .replace(
                        &f.root.join("node-state").display().to_string(),
                        &dir.display().to_string(),
                    )
                    .replace("listen = []", &settings)
                    .replace("\n[consortium]", &circle)
                    .replace("cuts_per_epoch = 1\n", "cuts_per_epoch = 1000\n")
                    .replace(
                        "\npatience = 2\nview_patience = 2\n",
                        "\npatience = 10\nview_patience = 20\n",
                    );
                let path = f.root.join(format!("runtime-{i}.toml"));
                fs::write(&path, text).unwrap();
                path
            })
            .collect();
        let mut nodes: Vec<_> = configs.iter().map(|c| Process::start(c)).collect();
        for i in 0..3 {
            let status = f.root.join(format!("runtime-{i}/status"));
            until(&status, |s| {
                node::parse(s).is_some_and(|s| s.cuts.len() >= 2)
            })
            .await;
        }
        for node in &mut nodes {
            node.terminate(Duration::from_secs(5)).await;
        }
        let status = fs::read_to_string(f.root.join("runtime-0/status")).unwrap();
        let digests = node::parse(&status).unwrap().cuts;
        let writers = f.tiers.writers().clone();
        let stored = DurableReplica::read(&f.root.join("runtime-0/replica"), writers).unwrap();
        let mut exported = Vec::new();
        for number in 0..2u64 {
            let ready = collect_with(&stored, f.tiers.consortium(), NET, number, &|_| vec![]);
            let Collected::Ready(_, view, _) = ready else {
                panic!("cut {number}: {ready:?}");
            };
            let wanted = cut_toml(&f, &format!("export-{number}"), number, view, &[]);
            let output = f.root.join(format!("export-{number}.cut"));
            let printed = ok(&[Path::new(EXPORT), &configs[0], &wanted, &output]);
            assert_eq!(
                printed,
                format!("{}\n", node::hex(&digests[number as usize]))
            );
            let cut = Cut::decode(&fs::read(&output).unwrap()).unwrap();
            assert_eq!(cut.digest(), digests[number as usize]);
            let other = cut_toml(&f, &format!("other-{number}"), number, view + 1, &[]);
            let other = cli(&[
                Path::new(EXPORT),
                &configs[0],
                &other,
                &f.root.join("x.cut"),
            ]);
            assert_eq!(other.status.code(), Some(2));
            exported.push((output, view));
        }
        let input = production::Config {
            previous_cut_file: Some(exported[0].0.clone()),
            target_cut_file: exported[1].0.clone(),
            view: exported[1].1,
            segment_file: f.root.join("runtime-segment.bin"),
            ..f.input()
        };
        let production = f.root.join("runtime-production.toml");
        fs::write(&production, text(&input)).unwrap();
        ok(&[Path::new("--archive-extract"), &configs[0], &production]);
        let bundle = f.root.join("runtime-bundle");
        ok(&[
            Path::new("--archive-prepare"),
            &configs[0],
            &production,
            &bundle,
        ]);
    }
}
