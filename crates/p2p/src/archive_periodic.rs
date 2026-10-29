//! Serial caller-configured archive maintenance (`docs/04` §Periodic local archive maintenance).

use super::{MaintainError, Maintenance, MAX_ARCHIVES};
use libp2p::Multiaddr;
use network::archive::{Context, SignedManifest};
use std::collections::BTreeSet;
use std::time::Duration;
use tokio::sync::{oneshot, watch};
use tokio::time::Instant;

pub struct MaintenanceJob<'a> {
    pub context: Context<'a>,
    pub signed: &'a SignedManifest,
    pub roster: &'a [[u8; 32]],
    pub bootstrap: &'a [Multiaddr],
}

#[derive(Clone, Copy)]
pub struct MaintenanceSchedule {
    pub interval: Duration,
    pub deadline: Duration,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MaintenanceReport {
    pub job: usize,
    pub digest: [u8; 32],
    pub result: Result<Maintenance, MaintainError>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ScheduleError {
    Jobs,
    Timing,
}

impl crate::Handle {
    /// Runs serial sweeps with a full interval after each sweep; reports retain the latest pass.
    /// Stop (or sender drop) finishes the active pass, including its commit acknowledgement.
    /// Await for graceful stop; dropping this future cannot retract queued writes (`docs/04`).
    pub async fn run_archive_maintenance(
        &mut self,
        jobs: &[MaintenanceJob<'_>],
        schedule: MaintenanceSchedule,
        mut stop: oneshot::Receiver<()>,
        reports: watch::Sender<Option<MaintenanceReport>>,
    ) -> Result<(), ScheduleError> {
        if jobs.is_empty()
            || jobs.len() > MAX_ARCHIVES
            || jobs.iter().any(|job| {
                job.bootstrap.len() > 32
                    || job
                        .bootstrap
                        .iter()
                        .any(|address| address.len() > 1024 || address.iter().count() > 16)
            })
            || jobs
                .iter()
                .map(|job| job.signed.manifest.digest())
                .collect::<BTreeSet<_>>()
                .len()
                != jobs.len()
        {
            return Err(ScheduleError::Jobs);
        }
        let now = Instant::now();
        if schedule.interval.is_zero()
            || schedule.deadline.is_zero()
            || now.checked_add(schedule.interval).is_none()
            || now.checked_add(schedule.deadline).is_none()
        {
            return Err(ScheduleError::Timing);
        }
        loop {
            for (index, job) in jobs.iter().enumerate() {
                if !matches!(stop.try_recv(), Err(oneshot::error::TryRecvError::Empty)) {
                    return Ok(());
                }
                let result = self
                    .maintain_archive(
                        &job.context,
                        job.signed,
                        job.roster,
                        job.bootstrap.to_vec(),
                        schedule.deadline,
                    )
                    .await;
                reports.send_replace(Some(MaintenanceReport {
                    job: index,
                    digest: job.signed.manifest.digest(),
                    result,
                }));
            }
            let next = Instant::now()
                .checked_add(schedule.interval)
                .ok_or(ScheduleError::Timing)?;
            tokio::select! {
                biased;
                _ = &mut stop => return Ok(()),
                _ = tokio::time::sleep_until(next) => {}
            }
        }
    }
}
