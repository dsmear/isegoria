//! Offline extraction of a pinned cut's public segment from a stopped node's replica
//! (`docs/04` §Segment extraction from a stopped node's replica).

use crate::archive::{read_bounded, MAX_CUT};
use crate::production::{pinned, publish_file, Config, Error};
use network::archive::{Segment, Snapshot, MAX_SEGMENT};
use network::cid::{cid, Cid};
use network::cut::{collect_with, Collected, Cut};
use network::replica::DurableReplica;
use network::state::{RootStatement, Shape};

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
    let replica = {
        let _lock = crate::lock(&node.data_dir.join("lock"))?
            .ok_or(Error::Invalid("data directory in use"))?;
        let writers = pinned.tiers.writers().clone();
        DurableReplica::read(&node.data_dir.join("replica"), writers).map_err(Error::Replica)?
    };
    let excluded = |_: &Cut| pinned.excluded.clone();
    let tiers = &pinned.tiers;
    let number = pinned.target.number;
    let signatures = match collect_with(
        &replica,
        tiers.consortium(),
        tiers.network_id(),
        number,
        &excluded,
    ) {
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
