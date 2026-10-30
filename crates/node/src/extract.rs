//! Offline reads of a stopped node's replica: a pinned cut's public segment and a collected cut
//! (`docs/04` §Segment extraction and §Collected cut export from a stopped node's replica).

use crate::archive::{read_bounded, MAX_CONFIG, MAX_CUT};
use crate::production::{pinned, publish_file, tiers, Config, Error};
use network::archive::{verify_certificate, Segment, Snapshot, MAX_SEGMENT};
use network::cid::{cid, Cid};
use network::cut::{collect_with, Collected, Cut};
use network::replica::{DurableReplica, Replica};
use network::state::{RootStatement, Shape};
use network::tier::Tiers;
use serde::Deserialize;
use std::path::Path;

/// The stopped node's replica, read without writing while the data directory's lock is held.
fn stopped_replica(node: &crate::Config, tiers: &Tiers) -> Result<Replica, Error> {
    let _lock =
        crate::lock(&node.data_dir.join("lock"))?.ok_or(Error::Invalid("data directory in use"))?;
    let writers = tiers.writers().clone();
    DurableReplica::read(&node.data_dir.join("replica"), writers).map_err(Error::Replica)
}

fn collect(replica: &Replica, tiers: &Tiers, number: u64, excluded: &[usize]) -> Collected {
    let excluded = |_: &Cut| excluded.to_vec();
    collect_with(
        replica,
        tiers.consortium(),
        tiers.network_id(),
        number,
        &excluded,
    )
}

/// A closing target's pinned snapshot; refused for any other target.
fn snapshot(input: &Config, closes: bool) -> Result<Option<Snapshot>, Error> {
    let (root, shape) = match (&input.root_statement_file, &input.shape_file, closes) {
        (None, None, false) => return Ok(None),
        (Some(root), Some(shape), true) => (root, shape),
        _ => {
            return Err(Error::Invalid(
                "snapshot files must match the target's closing",
            ))
        }
    };
    let bytes = read_bounded(root, MAX_CUT, "archive.root_statement")?;
    let statement = RootStatement::decode(&bytes).map_err(|_| Error::Invalid("root codec"))?;
    if statement.encode() != bytes {
        return Err(Error::Invalid("root codec"));
    }
    let bytes = read_bounded(shape, MAX_SEGMENT, "archive.shape")?;
    let shape = Shape::decode(&bytes).map_err(|_| Error::Invalid("shape codec"))?;
    if shape.encode() != bytes {
        return Err(Error::Invalid("shape codec"));
    }
    Ok(Some(Snapshot { statement, shape }))
}

/// Publishes the pinned target's public segment, read from the stopped node's replica, at
/// `segment_file`; returns its CID.
pub fn extract(node: &crate::Config, input: &Config) -> Result<Cid, Error> {
    let pinned = pinned(node, input)?;
    let snapshot = snapshot(input, pinned.target.closes)?;
    let replica = stopped_replica(node, &pinned.tiers)?;
    let number = pinned.target.number;
    let signatures = match collect(&replica, &pinned.tiers, number, &pinned.excluded) {
        Collected::Ready(cut, view, signatures) if cut == pinned.target && view == pinned.view => {
            signatures
        }
        Collected::Ready(..) => return Err(Error::Invalid("replica collects another cut or view")),
        Collected::Pending => return Err(Error::Invalid("target not collected in replica")),
        Collected::Conflict(..) => return Err(Error::Invalid("conflicting cuts for the target")),
    };
    let bytes = Segment::build(&pinned.context(), &replica, signatures, snapshot)?.encode()?;
    publish_file(&input.segment_file, &bytes)?;
    Ok(cid(&bytes))
}

/// The cut `--archive-export-cut` exports and the view and exclusions it must be collected
/// under (`docs/04` §Collected cut export from a stopped node's replica).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CutConfig {
    pub number: u64,
    pub view: u64,
    pub excluded: Vec<usize>,
}

impl CutConfig {
    pub fn load(path: &Path) -> Result<Self, Error> {
        let bytes = read_bounded(path, MAX_CONFIG, "archive.cut_config")?;
        let text = String::from_utf8(bytes).map_err(|_| Error::Invalid("config UTF-8"))?;
        toml::from_str(&text).map_err(|_| Error::Invalid("cut config"))
    }
}

/// Publishes at `output` the cut the stopped node's replica collects for `input`, once its
/// signatures pass the segment's strict certificate check; returns the cut.
pub fn export_cut(node: &crate::Config, input: &CutConfig, output: &Path) -> Result<Cut, Error> {
    let tiers = tiers(node, &input.excluded)?;
    let replica = stopped_replica(node, &tiers)?;
    let (cut, signatures) = match collect(&replica, &tiers, input.number, &input.excluded) {
        Collected::Ready(cut, view, signatures) if view == input.view => (cut, signatures),
        Collected::Ready(..) => return Err(Error::Invalid("cut collected in another view")),
        Collected::Pending => return Err(Error::Invalid("cut not collected in replica")),
        Collected::Conflict(..) => return Err(Error::Invalid("conflicting cuts for the number")),
    };
    verify_certificate(&tiers, &input.excluded, &cut, input.view, &signatures)?;
    publish_file(output, &cut.encode())?;
    Ok(cut)
}
