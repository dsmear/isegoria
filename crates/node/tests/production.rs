use network::archive::{
    Context, Layout, Segment, SignedManifest, Snapshot, MAX_MANIFEST, MAX_SEGMENT,
};
use network::cid::cid;
use network::consortium::{Consortium, Member};
use network::cut::{Cut, Mark};
use network::log::TransparencyLog;
use network::replica::{FeedWriter, Replica};
use network::state::{Origin, RootStatement, Shape, ShapeEntry};
use network::tier::Tiers;
use node::production::{self, Config, Error};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const NET: [u8; 32] = [61; 32];
const LINEAGE: [u8; 32] = [62; 32];
const PUBLIC: &[u8] = b"independently authenticated public delta";
const PRIVATE: &[u8] = b"private sentinel never in output";

struct Fixture {
    root: PathBuf,
    members: Vec<Member>,
    tiers: Tiers,
    previous: Cut,
    target: Cut,
    segment: Segment,
}

impl Fixture {
    fn new(name: &str, closes: bool) -> Self {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join("archive-production")
            .join(format!("{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let members: Vec<_> = (1..=3).map(|i| Member::from_seed([i; 32])).collect();
        let circle: Vec<_> = (11..=13)
            .map(|i| Member::from_seed([i; 32]).public())
            .collect();
        let tiers = Tiers::new(
            NET,
            Consortium::new(members.iter().map(Member::public).collect(), 2),
            &circle,
        )
        .unwrap();
        let public = FeedWriter::from_seed(NET, [1; 32]);
        let mut log = TransparencyLog::new();
        let first = public.sign(log.append(cid(b"trusted prefix")));
        let previous = Cut {
            number: 7,
            epoch: 2,
            closes: false,
            marks: vec![Mark {
                writer: first.writer,
                len: 1,
                head: first.entry.hash,
            }],
        };
        let entry = public.sign(log.append(cid(PUBLIC)));
        let mut replica = Replica::new(tiers.writers().clone());
        replica.insert(entry.clone(), PUBLIC.to_vec()).unwrap();
        let circle_writer = FeedWriter::from_seed(NET, [11; 32]);
        let mut private_log = TransparencyLog::new();
        let private = circle_writer.sign(private_log.append(cid(PRIVATE)));
        replica.insert(private.clone(), PRIVATE.to_vec()).unwrap();
        let mut marks = vec![
            Mark {
                writer: entry.writer,
                len: 2,
                head: entry.entry.hash,
            },
            Mark {
                writer: private.writer,
                len: 1,
                head: private.entry.hash,
            },
        ];
        marks.sort_by_key(|m| m.writer);
        let target = Cut {
            number: 8,
            epoch: 2,
            closes,
            marks,
        };
        let context = Context {
            tiers: &tiers,
            lineage: LINEAGE,
            previous: Some(&previous),
            target: &target,
            view: 4,
            excluded: &[],
        };
        let cp = target.checkpoint_in(4, NET, tiers.consortium().member_set_hash());
        let signatures = members
            .iter()
            .enumerate()
            .map(|(i, m)| (i, m.sign(&cp)))
            .collect();
        let snapshot = closes.then(|| {
            let shape = Shape::new(vec![ShapeEntry {
                position: [1; 32],
                leaf_hash: [2; 32],
                origin: Origin::Live,
            }])
            .unwrap();
            Snapshot {
                statement: RootStatement {
                    network_id: NET,
                    lineage: LINEAGE,
                    member_set_hash: tiers.consortium().member_set_hash(),
                    epoch: 2,
                    cut_number: 8,
                    cut_digest: target.digest(),
                    inputs_root: [3; 32],
                    state_root: shape.root(),
                    shape_digest: shape.digest(),
                },
                shape,
            }
        });
        let segment = Segment::build(&context, &replica, signatures, snapshot).unwrap();
        fs::write(root.join("previous.cut"), previous.encode()).unwrap();
        fs::write(root.join("target.cut"), target.encode()).unwrap();
        fs::write(root.join("segment.bin"), segment.encode().unwrap()).unwrap();
        for i in 0..3 {
            fs::write(
                root.join(format!("member-{i}.key")),
                node::hex(&[i as u8 + 1; 32]),
            )
            .unwrap();
        }
        Self {
            root,
            members,
            tiers,
            previous,
            target,
            segment,
        }
    }

    fn context(&self) -> Context<'_> {
        Context {
            tiers: &self.tiers,
            lineage: LINEAGE,
            previous: Some(&self.previous),
            target: &self.target,
            view: 4,
            excluded: &[],
        }
    }

    fn node_text(&self, index: usize) -> String {
        let list = |keys: Vec<[u8; 32]>| {
            format!(
                "[{}]",
                keys.iter()
                    .map(|k| format!("\"{}\"", node::hex(k)))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        };
        let members = self
            .members
            .iter()
            .map(|m| m.public().to_bytes())
            .collect::<Vec<_>>();
        let circle = (11..=13)
            .map(|i| Member::from_seed([i; 32]).public().to_bytes())
            .collect::<Vec<_>>();
        let writers = members.iter().chain(&circle).copied().collect();
        format!("network_id = \"{}\"\ndata_dir = \"{}\"\nlisten = []\nwriters = {}\nkey_file = \"{}\"\n[consortium]\nmembers = {}\nthreshold = 2\ncircle = {}\n[member]\nindex = {index}\nevery_ms = 100\ncuts_per_epoch = 1\npatience = 2\nview_patience = 2\n", node::hex(&NET), self.root.join("node-state").display(), list(writers), self.root.join(format!("member-{index}.key")).display(), list(members), list(circle))
    }

    fn node(&self, index: usize) -> node::Config {
        toml::from_str(&self.node_text(index)).unwrap()
    }

    fn input(&self) -> Config {
        Config {
            lineage: node::hex(&LINEAGE),
            previous_cut_file: Some(self.root.join("previous.cut")),
            target_cut_file: self.root.join("target.cut"),
            view: 4,
            excluded: vec![],
            segment_file: self.root.join("segment.bin"),
            data_shards: 2,
            parity_shards: 1,
        }
    }

    fn shares(&self, manifest: &Path) -> Vec<PathBuf> {
        (0..3)
            .map(|i| {
                let path = self.root.join(format!("share-{i}.bin"));
                production::sign(&self.node(i), &self.input(), manifest, &path).unwrap();
                path
            })
            .collect()
    }

    fn no_state(&self) {
        assert!(!self.root.join("node-state").exists());
        assert!(!fs::read_dir(&self.root).unwrap().any(|e| e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".archive-stage-")));
    }
}

// Manual independently reproduced signatures feed existing maintenance and recovery (AT-NET-32).
#[test]
fn cli_workflow_produces_a_maintenance_certificate_and_public_shards() {
    let f = Fixture::new("workflow", false);
    let input = format!("lineage = \"{}\"\nprevious_cut_file = \"{}\"\ntarget_cut_file = \"{}\"\nview = 4\nexcluded = []\nsegment_file = \"{}\"\ndata_shards = 2\nparity_shards = 1\n", node::hex(&LINEAGE), f.root.join("previous.cut").display(), f.root.join("target.cut").display(), f.root.join("segment.bin").display());
    let input_path = f.root.join("production.toml");
    fs::write(&input_path, input).unwrap();
    let bundle = f.root.join("bundle");
    let manifest = bundle.join("manifest.bin");
    let run = |index, flag, rest: &[&Path]| {
        let config = f.root.join(format!("node-{index}.toml"));
        fs::write(&config, f.node_text(index)).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_isegoria-node"))
            .arg(flag)
            .arg(config)
            .arg(&input_path)
            .args(rest)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        String::from_utf8(result.stdout).unwrap()
    };
    let digest = run(0, "--archive-prepare", &[&bundle]);
    assert_eq!(
        fs::read_to_string(bundle.join("COMPLETE")).unwrap(),
        format!("isegoria/archive/bundle/v1\nmanifest {digest}")
    );
    assert_eq!(
        fs::read(bundle.join("segment.bin")).unwrap(),
        f.segment.encode().unwrap()
    );
    assert_eq!(
        fs::read(bundle.join("previous.cut")).unwrap(),
        f.previous.encode()
    );
    assert!(!fs::read(bundle.join("segment.bin"))
        .unwrap()
        .windows(PRIVATE.len())
        .any(|w| w == PRIVATE));
    let a = f.root.join("a.share");
    let b = f.root.join("b.share");
    assert_eq!(run(0, "--archive-sign", &[&manifest, &a]), digest);
    assert_eq!(run(1, "--archive-sign", &[&manifest, &b]), digest);
    let certificate = f.root.join("signed-manifest.bin");
    assert_eq!(
        run(2, "--archive-combine", &[&manifest, &certificate, &b, &a]),
        digest
    );
    let signed = SignedManifest::decode(&fs::read(&certificate).unwrap()).unwrap();
    assert_eq!(
        signed.signatures.iter().map(|s| s.0).collect::<Vec<_>>(),
        vec![0, 1]
    );
    let verified = signed.verify(&f.context()).unwrap();
    let mut shards = (0..3)
        .map(|i| Some(fs::read(bundle.join(format!("shard-{i:03}.bin"))).unwrap()))
        .collect::<Vec<_>>();
    shards[0] = None;
    assert_eq!(
        verified.recover(&f.context(), &shards).unwrap().segment,
        f.segment
    );
    f.no_state();
    let mut config = f.node(0);
    config.member = None;
    config.key_file = None;
    config.archive_maintenance = Some(node::archive::Config {
        interval_ms: 100,
        deadline_ms: 1000,
        jobs: vec![node::archive::JobConfig {
            lineage: node::hex(&LINEAGE),
            previous_cut_file: Some(bundle.join("previous.cut")),
            target_cut_file: bundle.join("target.cut"),
            view: 4,
            excluded: vec![],
            manifest_file: certificate,
            roster: (41..=43)
                .map(|i| node::hex(&Member::from_seed([i; 32]).public().to_bytes()))
                .collect(),
            bootstrap: vec![],
        }],
    });
    assert!(node::plan(config).is_ok());
}

// No signature is emitted without a verified segment and independent pinned inputs (AT-NET-32).
#[test]
fn candidate_and_context_mismatches_never_publish() {
    let f = Fixture::new("mismatch", false);
    let bundle = f.root.join("bundle");
    production::prepare(&f.node(0), &f.input(), &bundle).unwrap();
    let manifest = bundle.join("manifest.bin");
    let output = f.root.join("rejected.share");
    for case in 0..11 {
        let mut input = f.input();
        let mut node = f.node(0);
        match case {
            0 => input.lineage = node::hex(&[0; 32]),
            1 => input.view += 1,
            2 => input.previous_cut_file = None,
            3 => input.excluded = vec![0],
            4 => input.excluded = vec![1, 1],
            5 => input.excluded = vec![3],
            6 => input.data_shards = 1,
            7 => node.member.as_mut().unwrap().index = 1,
            8 => node.relay = true,
            9 => node.member = None,
            10 => node.consortium.members[1] = node.consortium.members[0].clone(),
            _ => unreachable!(),
        }
        assert!(
            production::sign(&node, &input, &manifest, &output).is_err(),
            "{case}"
        );
        assert!(!output.exists());
        f.no_state();
    }
    let mut segment = f.segment.clone();
    segment.entries[0].1[0] ^= 1;
    fs::write(f.root.join("segment.bin"), segment.encode().unwrap()).unwrap();
    assert!(production::sign(&f.node(0), &f.input(), &manifest, &output).is_err());
    assert!(!output.exists());
    f.no_state();
}

// Every share is checked, even when the other shares already meet quorum (AT-NET-32).
#[test]
fn combine_rejects_bad_extras_duplicates_foreign_and_different_manifests() {
    let f = Fixture::new("shares", false);
    let bundle = f.root.join("bundle");
    production::prepare(&f.node(0), &f.input(), &bundle).unwrap();
    let manifest = bundle.join("manifest.bin");
    let shares = f.shares(&manifest);
    let output = f.root.join("rejected.bin");
    assert!(production::combine(&f.node(0), &f.input(), &manifest, &shares[..1], &output).is_err());
    assert!(production::combine(
        &f.node(0),
        &f.input(),
        &manifest,
        &[shares[0].clone(), shares[0].clone()],
        &output
    )
    .is_err());
    let valid = fs::read(&shares[2]).unwrap();
    for case in 0..4 {
        let mut share = SignedManifest::decode(&valid).unwrap();
        match case {
            0 => {
                share.signatures[0].1 = f.members[0].sign(&f.target.checkpoint_in(
                    4,
                    NET,
                    f.tiers.consortium().member_set_hash(),
                ))
            }
            1 => share.signatures[0].0 = 9,
            2 => share.signatures.clear(),
            3 => {
                let built = network::archive::BuiltArchive::build(
                    &f.context(),
                    &f.segment,
                    Layout {
                        data_shards: 1,
                        parity_shards: 1,
                    },
                )
                .unwrap();
                share.manifest = built.manifest;
            }
            _ => unreachable!(),
        }
        fs::write(&shares[2], share.encode().unwrap()).unwrap();
        assert!(
            production::combine(&f.node(0), &f.input(), &manifest, &shares, &output).is_err(),
            "{case}"
        );
        assert!(!output.exists());
    }
    fs::write(&shares[2], valid).unwrap();
    let mut excluded = f.input();
    excluded.excluded = vec![2];
    assert!(production::combine(&f.node(0), &excluded, &manifest, &shares, &output).is_err());
    production::combine(&f.node(0), &excluded, &manifest, &shares[..2], &output).unwrap();
    f.no_state();
}

// Closing-cut shapes are verified and reproduced as input; no snapshot is invented (AT-NET-32).
#[test]
fn closing_snapshot_is_preserved_and_checked_before_signing() {
    let f = Fixture::new("closing", true);
    let bundle = f.root.join("bundle");
    production::prepare(&f.node(0), &f.input(), &bundle).unwrap();
    let manifest = bundle.join("manifest.bin");
    let output = f.root.join("rejected.share");
    let mut changed = f.segment.clone();
    changed.snapshot.as_mut().unwrap().statement.inputs_root[0] ^= 1;
    fs::write(f.root.join("segment.bin"), changed.encode().unwrap()).unwrap();
    assert!(production::sign(&f.node(0), &f.input(), &manifest, &output).is_err());
    changed.snapshot.as_mut().unwrap().statement.state_root[0] ^= 1;
    fs::write(f.root.join("segment.bin"), changed.encode().unwrap()).unwrap();
    assert!(production::prepare(&f.node(0), &f.input(), &f.root.join("bad-shape")).is_err());
    changed.snapshot = None;
    fs::write(f.root.join("segment.bin"), changed.encode().unwrap()).unwrap();
    assert!(production::prepare(&f.node(0), &f.input(), &f.root.join("missing-snapshot")).is_err());
    assert!(!output.exists());
    assert!(!f.root.join("bad-shape").exists());
    assert!(!f.root.join("missing-snapshot").exists());
    f.no_state();
}

// Canonical bounded reads and no-clobber publication preserve every existing output (AT-NET-32).
#[test]
fn bounds_codecs_and_existing_outputs_are_conservative() {
    let f = Fixture::new("bounds", false);
    let bundle = f.root.join("bundle");
    fs::create_dir(&bundle).unwrap();
    assert!(production::prepare(&f.node(0), &f.input(), &bundle).is_err());
    assert!(fs::read_dir(&bundle).unwrap().next().is_none());
    fs::remove_dir(&bundle).unwrap();
    production::prepare(&f.node(0), &f.input(), &bundle).unwrap();
    let manifest = bundle.join("manifest.bin");
    let shares = f.shares(&manifest);
    let output = f.root.join("existing.bin");
    fs::write(&output, b"preserve").unwrap();
    assert!(production::sign(&f.node(0), &f.input(), &manifest, &output).is_err());
    assert!(production::combine(&f.node(0), &f.input(), &manifest, &shares[..2], &output).is_err());
    assert_eq!(fs::read(&output).unwrap(), b"preserve");
    let rejected = f.root.join("rejected.share");
    let candidate = fs::read(&manifest).unwrap();
    for bytes in [
        vec![1, 255],
        [candidate, vec![0]].concat(),
        vec![0; MAX_MANIFEST + 1],
    ] {
        fs::write(&manifest, bytes).unwrap();
        assert!(production::sign(&f.node(0), &f.input(), &manifest, &rejected).is_err());
        assert!(!rejected.exists());
    }
    let file = fs::OpenOptions::new()
        .write(true)
        .open(f.root.join("segment.bin"))
        .unwrap();
    file.set_len(MAX_SEGMENT as u64 + 1).unwrap();
    assert!(matches!(
        production::prepare(&f.node(0), &f.input(), &f.root.join("oversized")),
        Err(Error::Config(node::ConfigError::Bounds("archive.segment")))
    ));
    assert!(!f.root.join("oversized").exists());
    f.no_state();
}

// Local configuration, segment, seed and share-count limits precede any publication (AT-NET-32).
#[test]
fn local_input_limits_and_required_choices_are_enforced() {
    let f = Fixture::new("local-bounds", false);
    let path = f.root.join("input.toml");
    let text = "lineage = \"00\"\ntarget_cut_file = \"target.cut\"\nview = 0\nexcluded = []\nsegment_file = \"segment.bin\"\ndata_shards = 2\nparity_shards = 1\n";
    fs::write(&path, text).unwrap();
    assert!(Config::load(&path).is_ok());
    for bytes in [
        text.replace("excluded = []\n", "").into_bytes(),
        (text.to_string() + "unknown = true\n").into_bytes(),
        vec![255],
        vec![0; node::archive::MAX_CONFIG + 1],
    ] {
        fs::write(&path, bytes).unwrap();
        assert!(Config::load(&path).is_err());
    }
    let bundle = f.root.join("bundle");
    production::prepare(&f.node(0), &f.input(), &bundle).unwrap();
    let manifest = bundle.join("manifest.bin");
    let output = f.root.join("rejected.bin");
    let shares = vec![f.root.join("not-read"); MAX_MANIFEST / 72 + 1];
    assert!(production::combine(&f.node(0), &f.input(), &manifest, &shares, &output).is_err());
    fs::write(f.root.join("member-0.key"), vec![b'0'; 129]).unwrap();
    assert!(production::sign(&f.node(0), &f.input(), &manifest, &output).is_err());
    assert!(!output.exists());
    let mut bytes = f.segment.encode().unwrap();
    bytes.push(0);
    fs::write(f.root.join("segment.bin"), bytes).unwrap();
    assert!(production::prepare(&f.node(0), &f.input(), &f.root.join("noncanonical")).is_err());
    assert!(!f.root.join("noncanonical").exists());
    f.no_state();
}
