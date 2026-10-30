//! Operator-pinned archive jobs and latest-pass process status
//! (`docs/04` §Archive maintenance in a node process).

use crate::{addresses, unhex, ConfigError};
use network::archive::{Context, SignedManifest, MAX_MANIFEST};
use network::cut::Cut;
use network::tier::Tiers;
use p2p::archive::{MaintenanceJob, MaintenanceReport, MaintenanceSchedule, MAX_ROSTER};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const MAX_CONFIG: usize = 1 << 20;
pub const MAX_CUT: usize = 64 << 10;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub interval_ms: u64,
    pub deadline_ms: u64,
    pub jobs: Vec<JobConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobConfig {
    pub lineage: String,
    pub previous_cut_file: Option<PathBuf>,
    pub target_cut_file: PathBuf,
    pub view: u64,
    pub excluded: Vec<usize>,
    pub manifest_file: PathBuf,
    pub roster: Vec<String>,
    pub bootstrap: Vec<String>,
}

struct Job {
    lineage: [u8; 32],
    previous: Option<Cut>,
    target: Cut,
    view: u64,
    excluded: Vec<usize>,
    signed: SignedManifest,
    roster: Vec<[u8; 32]>,
    bootstrap: Vec<libp2p::Multiaddr>,
}

impl Job {
    fn context<'a>(&'a self, tiers: &'a Tiers) -> Context<'a> {
        Context {
            tiers,
            lineage: self.lineage,
            previous: self.previous.as_ref(),
            target: &self.target,
            view: self.view,
            excluded: &self.excluded,
        }
    }
}

pub struct Plan {
    tiers: Tiers,
    pub schedule: MaintenanceSchedule,
    jobs: Vec<Job>,
}

impl Plan {
    pub(crate) fn jobs(&self) -> Vec<MaintenanceJob<'_>> {
        self.jobs
            .iter()
            .map(|job| MaintenanceJob {
                context: job.context(&self.tiers),
                signed: &job.signed,
                roster: &job.roster,
                bootstrap: &job.bootstrap,
            })
            .collect()
    }

    pub(crate) fn job(&self, digest: &[u8; 32]) -> Option<MaintenanceJob<'_>> {
        self.jobs()
            .into_iter()
            .find(|job| job.signed.manifest.digest() == *digest)
    }
}

pub(crate) fn read_bounded(
    path: &Path,
    limit: usize,
    field: &'static str,
) -> Result<Vec<u8>, ConfigError> {
    let read = |e: io::Error| ConfigError::Read(e.kind());
    let check = |metadata: std::fs::Metadata| {
        if !metadata.is_file() || metadata.len() > limit as u64 {
            Err(ConfigError::Bounds(field))
        } else {
            Ok(())
        }
    };
    check(std::fs::metadata(path).map_err(read)?)?;
    let file = File::open(path).map_err(read)?;
    check(file.metadata().map_err(read)?)?;
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(read)?;
    if bytes.len() > limit {
        return Err(ConfigError::Bounds(field));
    }
    Ok(bytes)
}

fn cut(path: &Path) -> Result<Cut, ConfigError> {
    let bytes = read_bounded(path, MAX_CUT, "archive.cut")?;
    let cut = Cut::decode(&bytes).ok_or(ConfigError::Archive("cut codec"))?;
    if cut.encode() != bytes {
        return Err(ConfigError::Archive("cut codec"));
    }
    Ok(cut)
}

pub(crate) fn plan(config: &Config, tiers: Option<Tiers>) -> Result<Plan, ConfigError> {
    let tiers = tiers.ok_or(ConfigError::Archive("consortium.circle"))?;
    let schedule = MaintenanceSchedule {
        interval: Duration::from_millis(config.interval_ms),
        deadline: Duration::from_millis(config.deadline_ms),
    };
    let now = tokio::time::Instant::now();
    if schedule.interval.is_zero()
        || schedule.deadline.is_zero()
        || now.checked_add(schedule.interval).is_none()
        || now.checked_add(schedule.deadline).is_none()
    {
        return Err(ConfigError::Archive("schedule"));
    }
    if config.jobs.is_empty() || config.jobs.len() > 256 {
        return Err(ConfigError::Archive("jobs"));
    }
    let mut jobs = Vec::new();
    let mut digests = HashSet::new();
    for config in &config.jobs {
        if config.excluded.len() > tiers.consortium().keys().len()
            || config
                .excluded
                .iter()
                .any(|i| *i >= tiers.consortium().keys().len())
            || config.excluded.iter().collect::<HashSet<_>>().len() != config.excluded.len()
        {
            return Err(ConfigError::Archive("excluded"));
        }
        if config.roster.len() > MAX_ROSTER {
            return Err(ConfigError::Archive("roster"));
        }
        let roster = config
            .roster
            .iter()
            .map(|key| {
                let key = unhex(key, "archive.roster")?;
                ed25519_dalek::VerifyingKey::from_bytes(&key)
                    .map_err(|_| ConfigError::Archive("roster"))?;
                Ok(key)
            })
            .collect::<Result<Vec<_>, ConfigError>>()?;
        if roster.iter().collect::<HashSet<_>>().len() != roster.len() {
            return Err(ConfigError::Archive("roster"));
        }
        if config.bootstrap.len() > 32 || config.bootstrap.iter().any(|a| a.len() > 1024) {
            return Err(ConfigError::Archive("bootstrap"));
        }
        let bootstrap = addresses(&config.bootstrap)?;
        if bootstrap
            .iter()
            .any(|a| a.len() > 1024 || a.iter().count() > 16)
        {
            return Err(ConfigError::Archive("bootstrap"));
        }
        let bytes = read_bounded(&config.manifest_file, MAX_MANIFEST, "archive.manifest")?;
        let signed = SignedManifest::decode(&bytes).map_err(ConfigError::ArchiveContext)?;
        if signed.encode().map_err(ConfigError::ArchiveContext)? != bytes {
            return Err(ConfigError::Archive("manifest codec"));
        }
        let job = Job {
            lineage: unhex(&config.lineage, "archive.lineage")?,
            previous: config.previous_cut_file.as_deref().map(cut).transpose()?,
            target: cut(&config.target_cut_file)?,
            view: config.view,
            excluded: config.excluded.clone(),
            signed,
            roster,
            bootstrap,
        };
        let verified = job
            .signed
            .verify(&job.context(&tiers))
            .map_err(ConfigError::ArchiveContext)?;
        verified
            .placement(&job.context(&tiers), &job.roster)
            .map_err(ConfigError::ArchiveContext)?;
        if !digests.insert(job.signed.manifest.digest()) {
            return Err(ConfigError::Archive("duplicate job"));
        }
        jobs.push(job);
    }
    Ok(Plan {
        schedule,
        jobs,
        tiers,
    })
}

pub fn render(enabled: bool, report: Option<&MaintenanceReport>) -> String {
    match (enabled, report) {
        (false, _) => "disabled\n".to_string(),
        (true, None) => "pending\n".to_string(),
        (true, Some(report)) => format!(
            "job {}\nmanifest {}\nresult {:?}\n",
            report.job,
            crate::hex(&report.digest),
            report.result
        ),
    }
}

pub fn write_status(
    path: &Path,
    enabled: bool,
    report: Option<&MaintenanceReport>,
) -> io::Result<()> {
    let next = path.with_extension("next");
    std::fs::write(&next, render(enabled, report))?;
    std::fs::rename(next, path)
}
