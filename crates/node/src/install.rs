//! Manual installation of a prepared, certified bundle into a stopped holder's store
//! (`docs/04` §Initial archive installation on a holder).

use crate::archive::read_bounded;
use crate::production::{marker, Error};
use network::archive::{Segment, MAX_SEGMENT};
use network::erasure::shard_hash;
use network::store::MAX_OBJECT;
use p2p::archive::{Admitted, Installed};
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub struct Outcome {
    pub digest: [u8; 32],
    pub index: usize,
    pub installed: Installed,
}

fn configured(bytes: &[u8]) -> Option<[u8; 32]> {
    let text = std::str::from_utf8(bytes).ok()?;
    let hex = text
        .strip_prefix("isegoria/archive/bundle/v1\nmanifest ")?
        .strip_suffix('\n')?;
    let digest = crate::unhex(hex, "archive.bundle").ok()?;
    (marker(&digest) == text).then_some(digest)
}

/// Installs this holder's assigned shard of the configured job that the bundle's marker names.
pub fn install(node: &crate::Config, bundle: &Path) -> Result<Outcome, Error> {
    let jobs = node
        .archive_maintenance
        .as_ref()
        .ok_or(Error::Invalid("archive_maintenance required"))?;
    let plan = crate::archive::plan(jobs, crate::tiers_of(node)?)?;
    let text = read_bounded(&bundle.join("COMPLETE"), 128, "archive.bundle")?;
    let digest = configured(&text).ok_or(Error::Invalid("bundle marker"))?;
    let job = plan
        .job(&digest)
        .ok_or(Error::Invalid("bundle not configured"))?;
    let key = crate::stored_transport_key(&node.data_dir.join("transport.key"))?;
    let own = key
        .public()
        .try_into_ed25519()
        .map_err(|_| Error::Invalid("transport key"))?
        .to_bytes();
    let index = job
        .signed
        .verify(&job.context)?
        .placement(&job.context, job.roster)?
        .iter()
        .position(|holder| *holder == own)
        .ok_or(Error::Invalid("unassigned holder"))?;
    let bytes = read_bounded(&bundle.join("segment.bin"), MAX_SEGMENT, "archive.segment")?;
    let segment = Segment::decode(&bytes)?;
    if segment.encode()? != bytes {
        return Err(Error::Invalid("segment codec"));
    }
    let admitted = Admitted::new(&job.context, job.signed, &segment).map_err(Error::Install)?;
    let name = format!("shard-{index:03}.bin");
    let shard = read_bounded(&bundle.join(name), MAX_OBJECT, "archive.shard")?;
    if shard_hash(&shard) != job.signed.manifest.shard_hashes()[index] {
        return Err(Error::Invalid("bundle shard differs from its signed hash"));
    }
    let _lock =
        crate::lock(&node.data_dir.join("lock"))?.ok_or(Error::Invalid("data directory in use"))?;
    let installed = admitted
        .install_offline(job.context.tiers, &node.data_dir.join("replica"), index)
        .map_err(Error::Install)?;
    Ok(Outcome {
        digest,
        index,
        installed,
    })
}
