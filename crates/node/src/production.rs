//! Manual archive preparation and independently reproduced co-signing
//! (`docs/04` §Manual archive preparation and co-signing).

use crate::archive::{read_bounded, MAX_CONFIG, MAX_CUT};
use crate::{keys, tiers_of, unhex, ConfigError};
use network::archive::{BuiltArchive, Context, Layout, Manifest, Segment, SignedManifest};
use network::archive::{MAX_MANIFEST, MAX_SEGMENT};
use network::consortium::Member;
use network::cut::Cut;
use network::tier::Tiers;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub lineage: String,
    pub previous_cut_file: Option<PathBuf>,
    pub target_cut_file: PathBuf,
    pub view: u64,
    pub excluded: Vec<usize>,
    pub segment_file: PathBuf,
    pub data_shards: usize,
    pub parity_shards: usize,
    /// A closing target's snapshot, read only by extraction (`docs/04` §Segment extraction).
    pub root_statement_file: Option<PathBuf>,
    pub shape_file: Option<PathBuf>,
}

#[derive(Debug)]
pub enum Error {
    Config(ConfigError),
    Archive(network::archive::Error),
    Install(p2p::archive::InstallError),
    Replica(network::replica::DiskError),
    Io(io::ErrorKind),
    Invalid(&'static str),
}

impl From<ConfigError> for Error {
    fn from(error: ConfigError) -> Self {
        Self::Config(error)
    }
}

impl From<network::archive::Error> for Error {
    fn from(error: network::archive::Error) -> Self {
        Self::Archive(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error.kind())
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Error> {
        let bytes = read_bounded(path, MAX_CONFIG, "archive.production")?;
        let text = String::from_utf8(bytes).map_err(|_| Error::Invalid("config UTF-8"))?;
        toml::from_str(&text).map_err(|_| Error::Invalid("production config"))
    }
}

/// The operator's pinned archive context, checked against the node's tiered configuration.
pub(crate) struct Pinned {
    pub(crate) tiers: Tiers,
    lineage: [u8; 32],
    pub(crate) previous: Option<Cut>,
    pub(crate) target: Cut,
    pub(crate) view: u64,
    pub(crate) excluded: Vec<usize>,
}

impl Pinned {
    pub(crate) fn context(&self) -> Context<'_> {
        Context {
            tiers: &self.tiers,
            lineage: self.lineage,
            previous: self.previous.as_ref(),
            target: &self.target,
            view: self.view,
            excluded: &self.excluded,
        }
    }
}

struct Candidate {
    pinned: Pinned,
    segment: Vec<u8>,
    built: BuiltArchive,
}

impl Candidate {
    fn context(&self) -> Context<'_> {
        self.pinned.context()
    }

    fn reproduce(&self, path: &Path) -> Result<(), Error> {
        let bytes = read_bounded(path, MAX_MANIFEST, "archive.candidate")?;
        let manifest = Manifest::decode(&bytes)?;
        if manifest.encode()? != bytes || manifest != self.built.manifest {
            return Err(Error::Invalid(
                "candidate differs from independently rebuilt manifest",
            ));
        }
        Ok(())
    }
}

fn cut(path: &Path) -> Result<Cut, Error> {
    let bytes = read_bounded(path, MAX_CUT, "archive.cut")?;
    let cut = Cut::decode(&bytes).ok_or(Error::Invalid("cut codec"))?;
    if cut.encode() != bytes {
        return Err(Error::Invalid("cut codec"));
    }
    Ok(cut)
}

pub(crate) fn pinned(node: &crate::Config, input: &Config) -> Result<Pinned, Error> {
    let members = keys(&node.consortium.members, "consortium.members")?;
    if members
        .iter()
        .map(|k| k.to_bytes())
        .collect::<HashSet<_>>()
        .len()
        != members.len()
    {
        return Err(Error::Invalid("duplicate consortium member"));
    }
    let tiers = tiers_of(node)?.ok_or(Error::Invalid("tiered network required"))?;
    if input.excluded.len() > members.len()
        || input.excluded.iter().any(|i| *i >= members.len())
        || input.excluded.iter().collect::<HashSet<_>>().len() != input.excluded.len()
    {
        return Err(Error::Invalid("excluded members"));
    }
    Ok(Pinned {
        tiers,
        lineage: unhex(&input.lineage, "archive.lineage")?,
        previous: input.previous_cut_file.as_deref().map(cut).transpose()?,
        target: cut(&input.target_cut_file)?,
        view: input.view,
        excluded: input.excluded.clone(),
    })
}

fn candidate(node: &crate::Config, input: &Config) -> Result<Candidate, Error> {
    let pinned = pinned(node, input)?;
    let bytes = read_bounded(&input.segment_file, MAX_SEGMENT, "archive.segment")?;
    let segment = Segment::decode(&bytes)?;
    if segment.encode()? != bytes {
        return Err(Error::Invalid("segment codec"));
    }
    let built = BuiltArchive::build(
        &pinned.context(),
        &segment,
        Layout {
            data_shards: input.data_shards,
            parity_shards: input.parity_shards,
        },
    )?;
    Ok(Candidate {
        pinned,
        segment: bytes,
        built,
    })
}

struct Stage(PathBuf);

impl Stage {
    fn new(output: &Path) -> Result<Self, Error> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let parent = parent_of(output);
        for _ in 0..32 {
            let path = parent.join(format!(
                ".archive-stage-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        }
        Err(Error::Io(io::ErrorKind::AlreadyExists))
    }

    fn write(&self, name: &str, bytes: &[u8]) -> Result<(), Error> {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(self.0.join(name))?;
        file.write_all(bytes)?;
        file.sync_all()?;
        Ok(())
    }
}

impl Drop for Stage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub(crate) fn publish_file(output: &Path, bytes: &[u8]) -> Result<(), Error> {
    let stage = Stage::new(output)?;
    stage.write("file", bytes)?;
    fs::hard_link(stage.0.join("file"), output)?;
    sync_directory(parent_of(output))?;
    Ok(())
}

fn parent_of(path: &Path) -> &Path {
    path.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
}

fn sync_directory(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    File::open(path)?.sync_all()?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

/// The exact `COMPLETE` text of a bundle: logical completion, never authentication.
pub(crate) fn marker(digest: &[u8; 32]) -> String {
    format!(
        "isegoria/archive/bundle/v1\nmanifest {}\n",
        crate::hex(digest)
    )
}

/// Publishes a new bundle; only a bundle containing `COMPLETE` is complete.
pub fn prepare(node: &crate::Config, input: &Config, output: &Path) -> Result<[u8; 32], Error> {
    let candidate = candidate(node, input)?;
    let stage = Stage::new(output)?;
    let mut names = vec![
        "manifest.bin".to_string(),
        "segment.bin".to_string(),
        "target.cut".to_string(),
    ];
    stage.write("manifest.bin", &candidate.built.manifest.encode()?)?;
    stage.write("segment.bin", &candidate.segment)?;
    stage.write("target.cut", &candidate.pinned.target.encode())?;
    if let Some(previous) = &candidate.pinned.previous {
        stage.write("previous.cut", &previous.encode())?;
        names.push("previous.cut".to_string());
    }
    for (index, shard) in candidate.built.shards.iter().enumerate() {
        let name = format!("shard-{index:03}.bin");
        stage.write(&name, shard)?;
        names.push(name);
    }
    stage.write(
        "COMPLETE",
        marker(&candidate.built.manifest.digest()).as_bytes(),
    )?;
    fs::create_dir(output)?;
    let mut installed = Vec::new();
    let publish = (|| -> Result<(), Error> {
        for name in &names {
            fs::hard_link(stage.0.join(name), output.join(name))?;
            installed.push(name);
        }
        sync_directory(output)?;
        Ok(())
    })();
    if publish.is_err() {
        for name in installed {
            let _ = fs::remove_file(output.join(name));
        }
        let _ = fs::remove_dir(output);
    }
    publish?;
    fs::hard_link(stage.0.join("COMPLETE"), output.join("COMPLETE"))?;
    sync_directory(output)?;
    sync_directory(parent_of(output))?;
    Ok(candidate.built.manifest.digest())
}

pub fn sign(
    node: &crate::Config,
    input: &Config,
    manifest: &Path,
    output: &Path,
) -> Result<[u8; 32], Error> {
    let candidate = candidate(node, input)?;
    candidate.reproduce(manifest)?;
    if node.relay {
        return Err(Error::Invalid("relay cannot sign"));
    }
    let index = node
        .member
        .as_ref()
        .ok_or(Error::Invalid("member index required"))?
        .index;
    if candidate.pinned.excluded.contains(&index) {
        return Err(Error::Invalid("excluded signer"));
    }
    let key = node
        .key_file
        .as_ref()
        .ok_or(Error::Invalid("member key required"))?;
    let bytes = read_bounded(key, crate::MAX_SEED, "archive.member_key")?;
    let text = std::str::from_utf8(&bytes).map_err(|_| Error::Invalid("member seed UTF-8"))?;
    let member = Member::from_seed(unhex(text, "archive.member_key")?);
    if candidate.pinned.tiers.consortium().keys().get(index) != Some(&member.public()) {
        return Err(Error::Invalid("member key/index mismatch"));
    }
    let digest = candidate.built.manifest.digest();
    let signed = SignedManifest {
        signatures: vec![(index, member.sign(&candidate.built.manifest.checkpoint()))],
        manifest: candidate.built.manifest,
    };
    publish_file(output, &signed.encode()?)?;
    Ok(digest)
}

pub fn combine(
    node: &crate::Config,
    input: &Config,
    manifest: &Path,
    shares: &[PathBuf],
    output: &Path,
) -> Result<[u8; 32], Error> {
    if shares.is_empty() || shares.len() > MAX_MANIFEST / 72 {
        return Err(Error::Invalid("share count"));
    }
    let candidate = candidate(node, input)?;
    candidate.reproduce(manifest)?;
    if shares.len() > candidate.pinned.tiers.consortium().keys().len() {
        return Err(Error::Invalid("share count"));
    }
    let cp = candidate.built.manifest.checkpoint();
    let mut signatures = BTreeMap::new();
    for path in shares {
        let bytes = read_bounded(path, MAX_MANIFEST, "archive.share")?;
        let share = SignedManifest::decode(&bytes)?;
        if share.encode()? != bytes
            || share.manifest != candidate.built.manifest
            || share.signatures.len() != 1
        {
            return Err(Error::Invalid("share manifest/encoding/count"));
        }
        let (index, signature) = share.signatures[0];
        let key = candidate
            .pinned
            .tiers
            .consortium()
            .keys()
            .get(index)
            .ok_or(Error::Invalid("foreign signer"))?;
        if candidate.pinned.excluded.contains(&index)
            || !cp.verify_signature(key, &signature)
            || signatures.insert(index, signature).is_some()
        {
            return Err(Error::Invalid("invalid, duplicate or excluded signature"));
        }
    }
    let signed = SignedManifest {
        manifest: candidate.built.manifest.clone(),
        signatures: signatures.into_iter().collect(),
    };
    signed.verify(&candidate.context())?;
    let digest = signed.manifest.digest();
    publish_file(output, &signed.encode()?)?;
    Ok(digest)
}
