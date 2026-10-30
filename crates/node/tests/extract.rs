//! Segment extraction from a stopped node's replica (`docs/04` §Segment extraction from a
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
use holder::*;
use network::archive::Segment;
use network::cid::cid;
use network::cut::Cut;
use network::replica::SignedEntry;
use network::state::{Origin, Shape, ShapeEntry};
use network::tier::Tier;
use node::extract::extract;
use node::production::{self, Error};
use replica::*;
use std::fs;
use std::path::{Path, PathBuf};
use support::*;

const SEEDS: [u8; 3] = [41, 42, 43];

/// The records of the object file at `path`.
fn records(path: &Path) -> Vec<Vec<u8>> {
    let bytes = fs::read(path).unwrap();
    let (mut at, mut out) = (16, Vec::new());
    while at < bytes.len() {
        let len = u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap()) as usize;
        out.push(bytes[at + 8..at + 8 + len].to_vec());
        at += 8 + len;
    }
    out
}

/// An object file holding `records`.
fn object_file(records: impl IntoIterator<Item = Vec<u8>>) -> Vec<u8> {
    let mut bytes = b"isegoria-obj-v1\n".to_vec();
    for record in records {
        bytes.extend((record.len() as u64).to_le_bytes());
        bytes.extend(record);
    }
    bytes
}

/// Runs each `(name, node, input, expected error)` case and checks it changed nothing.
fn refused(dir: &Path, cases: Vec<(&str, String, production::Config, &str)>) {
    for (name, node, input, expected) in cases {
        let before = tree(dir);
        let result = extract(&config(&node), &input).map(|_| ());
        assert_eq!(format!("{result:?}"), format!("Err({expected})"), "{name}");
        assert_eq!(tree(dir), before, "{name}");
        assert!(!input.segment_file.exists(), "{name}");
    }
}

// Initial and successor cuts yield exactly their public deltas; only the lock is added (AT-NET-34).
#[test]
fn initial_and_successor_cuts_yield_exactly_their_public_deltas() {
    let f = Fixture::new("extract-delta", false);
    let first = first(&f);
    let dir = f.root.join("member");
    replica(
        &f,
        &dir,
        &[(&f.target, 4, &[0, 1, 2]), (&first, 0, &[0, 1])],
    );
    let node = config(&member(&f, &dir));
    let before = tree(&dir);
    let successor = input(&f, Some(&f.previous), &f.target, 4, "successor.bin");
    let initial = input(&f, None, &first, 0, "initial.bin");
    let prefix: &[u8] = b"trusted prefix";
    for (input, delta, signers) in [
        (&successor, vec![PUBLIC], vec![0, 1, 2]),
        (&initial, vec![prefix, PUBLIC], vec![0, 1]),
    ] {
        let id = extract(&node, input).unwrap();
        let bytes = fs::read(&input.segment_file).unwrap();
        assert_eq!(id, cid(&bytes));
        let segment = Segment::decode(&bytes).unwrap();
        let objects: Vec<_> = segment.entries.iter().map(|(_, o)| o.as_slice()).collect();
        assert_eq!(objects, delta);
        let public = |w: &[u8; 32]| f.tiers.tier(w) == Some(Tier::Public);
        assert!(segment.entries.iter().all(|(e, _)| public(&e.writer)));
        assert!(!bytes.windows(PRIVATE.len()).any(|w| w == PRIVATE));
        let indices: Vec<_> = segment.signatures.iter().map(|(i, _)| *i).collect();
        assert_eq!(indices, signers);
        assert_eq!(
            segment.previous.is_none(),
            input.previous_cut_file.is_none()
        );
    }
    let expected = f.segment.encode().unwrap();
    assert_eq!(fs::read(&successor.segment_file).unwrap(), expected);
    let mut after = tree(&dir);
    let lock = after
        .iter()
        .position(|(p, _)| *p == dir.join("lock"))
        .unwrap();
    assert_eq!(after.remove(lock).1, Some(Vec::new()));
    assert_eq!(after, before);
}

// The extracted segment passes the real prepare, sign, combine and install commands (AT-NET-34).
#[test]
fn extracted_segment_is_prepared_certified_and_installed() {
    let f = Fixture::new("extract-workflow", false);
    let dir = f.root.join("member");
    replica(&f, &dir, &[(&f.target, 4, &[0, 1, 2])]);
    let members: Vec<PathBuf> = (0..3)
        .map(|i| {
            let path = f.root.join(format!("member-{i}.toml"));
            let text = if i == 0 {
                member(&f, &dir)
            } else {
                f.node_text(i)
            };
            fs::write(&path, text).unwrap();
            path
        })
        .collect();
    let production = f.root.join("production.toml");
    let input = input(&f, Some(&f.previous), &f.target, 4, "extracted.bin");
    fs::write(&production, text(&input)).unwrap();
    let printed = ok(&[Path::new("--archive-extract"), &members[0], &production]);
    let bytes = fs::read(&input.segment_file).unwrap();
    assert_eq!(printed, format!("{}\n", node::hex(&cid(&bytes).0)));
    let bundle = f.root.join("bundle");
    let digest = ok(&[
        Path::new("--archive-prepare"),
        &members[0],
        &production,
        &bundle,
    ]);
    assert_eq!(fs::read(bundle.join("segment.bin")).unwrap(), bytes);
    let manifest = bundle.join("manifest.bin");
    let shares = [f.root.join("a.share"), f.root.join("b.share")];
    for (member, share) in members.iter().zip(&shares) {
        let sign = [
            Path::new("--archive-sign"),
            member,
            &production,
            &manifest,
            share,
        ];
        assert_eq!(ok(&sign), digest);
    }
    let certificate = f.root.join("certificate.bin");
    let combine = [
        Path::new("--archive-combine"),
        &members[2],
        &production,
        &manifest,
        &certificate,
        &shares[0],
        &shares[1],
    ];
    assert_eq!(ok(&combine), digest);
    let mut indices: Vec<usize> = SEEDS
        .iter()
        .map(|seed| {
            let name = format!("holder-{seed}");
            let jobs = job(&f, &certificate, &SEEDS);
            let text = holder(&f, &name, *seed, "/ip4/127.0.0.1/tcp/0", &jobs);
            let path = f.root.join(format!("{name}.toml"));
            fs::write(&path, text).unwrap();
            let out = ok(&[Path::new("--archive-install"), &path, &bundle]);
            let (printed, state) = out.split_once('\n').unwrap();
            assert_eq!(format!("{printed}\n"), digest);
            let state = state.strip_prefix("shard ").unwrap();
            state.strip_suffix(" installed\n").unwrap().parse().unwrap()
        })
        .collect();
    indices.sort();
    assert_eq!(indices, vec![0, 1, 2]);
}

// Missing, altered or torn replica files are refused, read-only and unchanged (AT-NET-34).
#[test]
fn damaged_replica_data_is_refused_without_change() {
    let f = Fixture::new("extract-damage", false);
    let dir = f.root.join("member");
    replica(&f, &dir, &[(&f.target, 4, &[0, 1, 2])]);
    fs::write(dir.join("lock"), b"").unwrap();
    let pinned = || input(&f, Some(&f.previous), &f.target, 4, "segment-out.bin");
    let (entries, objects) = (dir.join("replica/entries"), dir.join("replica/objects"));
    let public = |r: &[u8]| SignedEntry::decode(r).is_some_and(|e| e.entry.payload == cid(PUBLIC));
    let flipped = |r: &[u8]| {
        let mut r = r.to_vec();
        *r.last_mut().unwrap() ^= 1;
        r
    };
    let (stored, objects_stored) = (records(&entries), records(&objects));
    let entries_with = |edit: &dyn Fn(&[u8]) -> Option<Vec<u8>>| {
        object_file(stored.iter().filter_map(|r| edit(r)))
    };
    let objects_with = |edit: &dyn Fn(&[u8]) -> Option<Vec<u8>>| {
        object_file(objects_stored.iter().filter_map(|r| edit(r)))
    };
    let cases: Vec<(&str, &Path, Vec<u8>, &str)> = vec![
        (
            "missing entry",
            &entries,
            entries_with(&|r| (!public(r)).then(|| r.to_vec())),
            "Archive(Delta)",
        ),
        (
            "missing object",
            &objects,
            objects_with(&|r| (r != PUBLIC).then(|| r.to_vec())),
            "Replica(Damaged)",
        ),
        (
            "altered object",
            &objects,
            objects_with(&|r| Some(if r == PUBLIC { flipped(r) } else { r.to_vec() })),
            "Replica(Damaged)",
        ),
        (
            "altered entry",
            &entries,
            entries_with(&|r| Some(if public(r) { flipped(r) } else { r.to_vec() })),
            "Replica(Refused(BadSignature))",
        ),
        (
            "torn tail",
            &entries,
            [fs::read(&entries).unwrap(), vec![1, 2, 3]].concat(),
            "Replica(Store(Torn))",
        ),
        (
            "torn header",
            &objects,
            b"isegoria-obj".to_vec(),
            "Replica(Store(Torn))",
        ),
        (
            "bad header",
            &objects,
            [
                &b"isegoria-log-v1\n"[..],
                &fs::read(&objects).unwrap()[16..],
            ]
            .concat(),
            "Replica(Store(BadHeader))",
        ),
    ];
    for (name, path, bytes, expected) in cases {
        let saved = fs::read(path).unwrap();
        fs::write(path, bytes).unwrap();
        refused(&dir, vec![(name, member(&f, &dir), pinned(), expected)]);
        fs::write(path, saved).unwrap();
    }
    let moved = f.root.join("moved-replica");
    fs::rename(dir.join("replica"), &moved).unwrap();
    let missing = "Replica(Store(Io(NotFound)))";
    refused(
        &dir,
        vec![("no replica", member(&f, &dir), pinned(), missing)],
    );
    assert!(!dir.join("replica").exists());
    fs::rename(&moved, dir.join("replica")).unwrap();
    let input = pinned();
    extract(&config(&member(&f, &dir)), &input).unwrap();
    let expected = f.segment.encode().unwrap();
    assert_eq!(fs::read(&input.segment_file).unwrap(), expected);
}

// Wrong context or snapshot and pending, conflicting or other certificates are refused (AT-NET-34).
#[test]
fn wrong_context_or_certificate_is_refused() {
    let f = Fixture::new("extract-context", false);
    let first = first(&f);
    let dir = f.root.join("member");
    replica(
        &f,
        &dir,
        &[(&f.target, 4, &[0, 1, 2]), (&first, 0, &[0, 1])],
    );
    fs::write(dir.join("lock"), b"").unwrap();
    let node = member(&f, &dir);
    let mut other = f.target.clone();
    other.epoch = 3;
    let unsigned = Cut {
        number: 9,
        ..f.target.clone()
    };
    let mut forked = f.previous.clone();
    forked.marks[0].head = [9; 32];
    let pin = |previous, target, view, name| input(&f, previous, target, view, name);
    let untiered: String = node
        .lines()
        .filter(|l| !l.starts_with("circle ="))
        .map(|l| format!("{l}\n"))
        .collect();
    let foreign = node.replace(&node::hex(&NET), &node::hex(&[7; 32]));
    let (later, missing) = (
        "Invalid(\"replica collects another cut or view\")",
        "Invalid(\"target not collected in replica\")",
    );
    refused(
        &dir,
        vec![
            (
                "other view",
                node.clone(),
                pin(Some(&f.previous), &f.target, 5, "a"),
                later,
            ),
            (
                "other cut",
                node.clone(),
                pin(Some(&f.previous), &other, 4, "b"),
                later,
            ),
            (
                "unsigned",
                node.clone(),
                pin(Some(&f.target), &unsigned, 4, "c"),
                missing,
            ),
            (
                "excluded signers",
                node.clone(),
                production::Config {
                    excluded: vec![1, 2],
                    ..pin(Some(&f.previous), &f.target, 4, "d")
                },
                missing,
            ),
            (
                "threshold",
                node.replace("threshold = 2", "threshold = 3"),
                pin(None, &first, 0, "e"),
                missing,
            ),
            (
                "predecessor number",
                node.clone(),
                pin(Some(&first), &f.target, 4, "f"),
                "Archive(Cut)",
            ),
            (
                "predecessor head",
                node.clone(),
                pin(Some(&forked), &f.target, 4, "g"),
                "Archive(Delta)",
            ),
            (
                "no predecessor",
                node.clone(),
                pin(None, &f.target, 4, "h"),
                "Archive(Cut)",
            ),
            (
                "unexpected snapshot",
                node.clone(),
                production::Config {
                    root_statement_file: Some(f.root.join("root.bin")),
                    shape_file: Some(f.root.join("shape.bin")),
                    ..pin(Some(&f.previous), &f.target, 4, "i")
                },
                "Invalid(\"snapshot files must match the target's closing\")",
            ),
            (
                "untiered",
                untiered,
                pin(Some(&f.previous), &f.target, 4, "j"),
                "Invalid(\"tiered network required\")",
            ),
            (
                "other network",
                foreign,
                pin(Some(&f.previous), &f.target, 4, "k"),
                "Replica(Refused(BadSignature))",
            ),
        ],
    );
    let conflicted = f.root.join("conflicted");
    replica(
        &f,
        &conflicted,
        &[(&f.target, 4, &[0, 1, 2]), (&other, 4, &[1, 2])],
    );
    fs::write(conflicted.join("lock"), b"").unwrap();
    let conflict = "Invalid(\"conflicting cuts for the target\")";
    let node = member(&f, &conflicted);
    refused(
        &conflicted,
        vec![(
            "conflict",
            node,
            pin(Some(&f.previous), &f.target, 4, "l"),
            conflict,
        )],
    );
}

// A closing cut needs its exact snapshot files, checked against the target (AT-NET-34).
#[test]
fn closing_cut_requires_its_matching_snapshot() {
    let f = Fixture::new("extract-closing", true);
    let dir = f.root.join("member");
    replica(&f, &dir, &[(&f.target, 4, &[0, 1, 2])]);
    fs::write(dir.join("lock"), b"").unwrap();
    let snapshot = f.segment.snapshot.clone().unwrap();
    let file = |name: &str, bytes: Vec<u8>| {
        let path = f.root.join(name);
        fs::write(&path, bytes).unwrap();
        path
    };
    let root = file("root.bin", snapshot.statement.encode());
    let shape = file("shape.bin", snapshot.shape.encode());
    let mut moved = snapshot.statement;
    moved.cut_number += 1;
    let moved = file("moved-root.bin", moved.encode());
    let other = Shape::new(vec![ShapeEntry {
        position: [1; 32],
        leaf_hash: [4; 32],
        origin: Origin::Live,
    }])
    .unwrap();
    let other = file("other-shape.bin", other.encode());
    let long = file(
        "long-shape.bin",
        [snapshot.shape.encode(), vec![0]].concat(),
    );
    let with = |name: &str, root: Option<&PathBuf>, shape: Option<&PathBuf>| production::Config {
        root_statement_file: root.cloned(),
        shape_file: shape.cloned(),
        ..input(&f, Some(&f.previous), &f.target, 4, name)
    };
    let files = "Invalid(\"snapshot files must match the target's closing\")";
    let node = member(&f, &dir);
    refused(
        &dir,
        vec![
            ("missing", node.clone(), with("a", None, None), files),
            (
                "no shape",
                node.clone(),
                with("b", Some(&root), None),
                files,
            ),
            (
                "no root",
                node.clone(),
                with("c", None, Some(&shape)),
                files,
            ),
            (
                "other cut",
                node.clone(),
                with("d", Some(&moved), Some(&shape)),
                "Archive(Snapshot)",
            ),
            (
                "other shape",
                node.clone(),
                with("e", Some(&root), Some(&other)),
                "Archive(Snapshot)",
            ),
            (
                "shape codec",
                node.clone(),
                with("f", Some(&root), Some(&long)),
                "Invalid(\"shape codec\")",
            ),
        ],
    );
    let valid = with("valid", Some(&root), Some(&shape));
    extract(&config(&node), &valid).unwrap();
    assert_eq!(
        fs::read(&valid.segment_file).unwrap(),
        f.segment.encode().unwrap()
    );
}

// Existing output files and directories are never overwritten or left staged (AT-NET-34).
#[test]
fn existing_outputs_are_never_overwritten() {
    let f = Fixture::new("extract-output", false);
    let dir = f.root.join("member");
    replica(&f, &dir, &[(&f.target, 4, &[0, 1, 2])]);
    let node = config(&member(&f, &dir));
    let input = input(&f, Some(&f.previous), &f.target, 4, "existing.bin");
    fs::write(&input.segment_file, b"existing").unwrap();
    let exists = Err::<(), _>(std::io::ErrorKind::AlreadyExists);
    let result = extract(&node, &input).map(|_| ());
    assert_eq!(
        result.map_err(|e| match e {
            Error::Io(kind) => kind,
            e => panic!("{e:?}"),
        }),
        exists
    );
    assert_eq!(fs::read(&input.segment_file).unwrap(), b"existing");
    fs::remove_file(&input.segment_file).unwrap();
    fs::create_dir(&input.segment_file).unwrap();
    assert!(extract(&node, &input).is_err());
    assert!(fs::read_dir(&input.segment_file).unwrap().next().is_none());
    assert!(!fs::read_dir(&f.root).unwrap().any(|e| e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with(".archive-stage-")));
}

#[cfg(unix)]
mod processes {
    use super::process::{until, Process};
    use super::*;
    use std::time::Duration;

    // A running node's lock excludes extraction, which succeeds once it stops (AT-NET-34).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_running_node_excludes_extraction() {
        let f = Fixture::new("extract-running", false);
        let node = bare(&f, "holder", 41);
        let dir = f.root.join("holder");
        replica(&f, &dir, &[(&f.target, 4, &[0, 1, 2])]);
        let production = f.root.join("production.toml");
        let input = input(&f, Some(&f.previous), &f.target, 4, "running.bin");
        fs::write(&production, text(&input)).unwrap();
        let command = [Path::new("--archive-extract"), &node, &production];
        let mut running = Process::start(&node);
        until(&dir.join("status"), |s| !s.is_empty()).await;
        let refused = cli(&command);
        assert_eq!(refused.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&refused.stderr).contains("data directory in use"));
        assert!(!input.segment_file.exists());
        running.terminate(Duration::from_secs(5)).await;
        ok(&command);
        assert_eq!(
            fs::read(&input.segment_file).unwrap(),
            f.segment.encode().unwrap()
        );
    }
}
