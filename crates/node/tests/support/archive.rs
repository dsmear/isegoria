use network::archive::{BuiltArchive, Context, Layout, Segment, SignedManifest};
use network::consortium::{Consortium, Member};
use network::cut::Cut;
use network::replica::Replica;
use network::tier::Tiers;
use node::hex;
use std::path::{Path, PathBuf};

pub const NET: [u8; 32] = [113; 32];
pub const LINEAGE: [u8; 32] = [114; 32];

pub struct Fixture {
    pub root: PathBuf,
    pub tiers: Tiers,
    pub target: Cut,
    pub segment: Segment,
    pub signed: SignedManifest,
}

pub fn quoted(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|s| format!("\"{s}\""))
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub fn public(seed: u8) -> String {
    hex(&Member::from_seed([seed; 32]).public().to_bytes())
}

impl Fixture {
    pub fn new(name: &str) -> Self {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join("node-archive")
            .join(format!("{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let member = Member::from_seed([1; 32]);
        let tiers = Tiers::new(
            NET,
            Consortium::new(vec![member.public()], 1),
            &[Member::from_seed([11; 32]).public()],
        )
        .unwrap();
        let target = Cut {
            number: 0,
            epoch: 0,
            closes: false,
            marks: vec![],
        };
        let context = Context {
            tiers: &tiers,
            lineage: LINEAGE,
            previous: None,
            target: &target,
            view: 0,
            excluded: &[],
        };
        let cp = target.checkpoint_in(0, NET, tiers.consortium().member_set_hash());
        let segment = Segment::build(
            &context,
            &Replica::new(tiers.writers().clone()),
            vec![(0, member.sign(&cp))],
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
        std::fs::write(root.join("target.cut"), target.encode()).unwrap();
        std::fs::write(root.join("manifest.bin"), signed.encode().unwrap()).unwrap();
        Self {
            root,
            tiers,
            target,
            segment,
            signed,
        }
    }

    pub fn context(&self) -> Context<'_> {
        Context {
            tiers: &self.tiers,
            lineage: LINEAGE,
            previous: None,
            target: &self.target,
            view: 0,
            excluded: &[],
        }
    }

    pub fn config(&self, name: &str, listen: &str, peers: &[String], job: Option<&str>) -> String {
        let listen = listen.split("/p2p/").next().unwrap();
        let mut text = format!("network_id = \"{}\"\ndata_dir = \"{}\"\nlisten = [\"{listen}\"]\npeers = {}\nwriters = [\"{}\",\"{}\"]\nstatus_every_ms = 50\nsync_every_ms = 100\n\n[consortium]\nmembers = [\"{}\"]\nthreshold = 1\ncircle = [\"{}\"]\n",
            hex(&NET), self.root.join(name).display(), quoted(peers), public(1), public(11), public(1), public(11));
        if let Some(job) = job {
            text +=
                &format!("\n[archive_maintenance]\ninterval_ms = 100\ndeadline_ms = 3000\n{job}");
        }
        text
    }

    pub fn job(&self, bootstrap: &[String]) -> String {
        format!("\n[[archive_maintenance.jobs]]\nlineage = \"{}\"\ntarget_cut_file = \"{}\"\nview = 0\nexcluded = []\nmanifest_file = \"{}\"\nroster = [\"{}\",\"{}\"]\nbootstrap = {}\n",
            hex(&LINEAGE), self.root.join("target.cut").display(), self.root.join("manifest.bin").display(), public(41), public(42), quoted(bootstrap))
    }

    pub fn save_config(&self, name: &str, text: &str) -> PathBuf {
        let path = self.root.join(format!("{name}.toml"));
        std::fs::write(&path, text).unwrap();
        path
    }

    pub fn seed(&self, name: &str, seed: u8) {
        let dir = self.root.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("transport.key"), hex(&[seed; 32])).unwrap();
    }

    pub fn record(&self, name: &str) -> PathBuf {
        self.root
            .join(name)
            .join("replica/archive")
            .join(format!("{}.arc", hex(&self.signed.manifest.digest())))
    }
}

pub fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}
