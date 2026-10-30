use network::archive::{Context, Segment, Snapshot};
use network::cid::cid;
use network::consortium::{Consortium, Member};
use network::cut::{Cut, Mark};
use network::log::TransparencyLog;
use network::replica::{FeedWriter, Replica};
use network::state::{Origin, RootStatement, Shape, ShapeEntry};
use network::tier::Tiers;
use node::production::{self, Config};
use std::fs;
use std::path::{Path, PathBuf};

pub const NET: [u8; 32] = [61; 32];
pub const LINEAGE: [u8; 32] = [62; 32];
pub const PUBLIC: &[u8] = b"independently authenticated public delta";
pub const PRIVATE: &[u8] = b"private sentinel never in output";

pub struct Fixture {
    pub root: PathBuf,
    pub members: Vec<Member>,
    pub tiers: Tiers,
    pub previous: Cut,
    pub target: Cut,
    pub segment: Segment,
}

impl Fixture {
    pub fn new(name: &str, closes: bool) -> Self {
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

    pub fn context(&self) -> Context<'_> {
        Context {
            tiers: &self.tiers,
            lineage: LINEAGE,
            previous: Some(&self.previous),
            target: &self.target,
            view: 4,
            excluded: &[],
        }
    }

    pub fn node_text(&self, index: usize) -> String {
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

    pub fn node(&self, index: usize) -> node::Config {
        toml::from_str(&self.node_text(index)).unwrap()
    }

    pub fn input(&self) -> Config {
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

    pub fn shares(&self, manifest: &Path) -> Vec<PathBuf> {
        (0..3)
            .map(|i| {
                let path = self.root.join(format!("share-{i}.bin"));
                production::sign(&self.node(i), &self.input(), manifest, &path).unwrap();
                path
            })
            .collect()
    }

    pub fn no_state(&self) {
        assert!(!self.root.join("node-state").exists());
        assert!(!fs::read_dir(&self.root).unwrap().any(|e| e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".archive-stage-")));
    }
}
