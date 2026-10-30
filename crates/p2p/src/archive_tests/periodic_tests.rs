use super::maintenance_tests::{assigned, keys, path, source, transport_roster};
use super::*;
use tokio::sync::{mpsc, oneshot, watch};

fn schedule() -> MaintenanceSchedule {
    MaintenanceSchedule {
        interval: Duration::from_millis(80),
        deadline: Duration::from_secs(3),
    }
}
fn job<'a>(
    f: &'a Fixture,
    signed: &'a SignedManifest,
    roster: &'a [[u8; 32]],
) -> MaintenanceJob<'a> {
    MaintenanceJob {
        context: f.context(),
        signed,
        roster,
        bootstrap: &[],
    }
}
fn controlled_handle() -> (crate::Handle, mpsc::UnboundedReceiver<crate::Command>) {
    let (commands, receiver) = mpsc::unbounded_channel();
    (
        crate::Handle {
            peer_id: keys()[0].public().to_peer_id(),
            commands,
            set_aside: vec![],
            task: tokio::spawn(async {}),
        },
        receiver,
    )
}
async fn report(reports: &mut watch::Receiver<Option<MaintenanceReport>>) {
    tokio::time::timeout(Duration::from_secs(12), reports.changed())
        .await
        .unwrap()
        .unwrap();
}

// Invalid schedules are refused before any local command or contact (AT-NET-30).
#[tokio::test]
async fn invalid_schedules_do_no_work() {
    let f = Fixture::new();
    let (_, signed, _) = f.archive();
    let roster = transport_roster(&keys());
    let (mut node, mut commands) = controlled_handle();
    let jobs = [job(&f, &signed, &roster)];
    let (reports, _) = watch::channel(None);
    for (jobs, config, expected) in [
        (&[][..], schedule(), ScheduleError::Jobs),
        (
            &jobs[..],
            MaintenanceSchedule {
                interval: Duration::ZERO,
                ..schedule()
            },
            ScheduleError::Timing,
        ),
        (
            &jobs[..],
            MaintenanceSchedule {
                deadline: Duration::ZERO,
                ..schedule()
            },
            ScheduleError::Timing,
        ),
        (
            &jobs[..],
            MaintenanceSchedule {
                interval: Duration::MAX,
                ..schedule()
            },
            ScheduleError::Timing,
        ),
        (
            &jobs[..],
            MaintenanceSchedule {
                deadline: Duration::MAX,
                ..schedule()
            },
            ScheduleError::Timing,
        ),
    ] {
        let (_stop, stopped) = oneshot::channel();
        assert_eq!(
            node.run_archive_maintenance(jobs, config, stopped, reports.clone())
                .await,
            Err(expected)
        );
    }
    for count in [2, MAX_ARCHIVES + 1] {
        let jobs: Vec<_> = (0..count).map(|_| job(&f, &signed, &roster)).collect();
        let (_stop, stopped) = oneshot::channel();
        assert_eq!(
            node.run_archive_maintenance(&jobs, schedule(), stopped, reports.clone())
                .await,
            Err(ScheduleError::Jobs)
        );
    }
    assert!(commands.try_recv().is_err());
}

// Bootstrap limits accept their boundaries and reject oversized jobs before work (AT-NET-30).
#[tokio::test]
async fn bootstrap_bounds_before_work() {
    let f = Fixture::new();
    let (_, signed, _) = f.archive();
    let roster = transport_roster(&keys());
    let (mut node, mut commands) = controlled_handle();
    let unix: Multiaddr = format!("/unix/{}", "a".repeat(1020)).parse().unwrap();
    assert_eq!(unix.len(), 1024);
    let components = (0..16).fold(Multiaddr::empty(), |address, _| {
        address.with(libp2p::multiaddr::Protocol::Memory(0))
    });
    let plain: Multiaddr = "/ip4/127.0.0.1/tcp/9".parse().unwrap();
    for (addresses, valid) in [
        (vec![plain.clone(); 32], true),
        (vec![unix], true),
        (vec![components.clone()], true),
        (vec![plain; 33], false),
        (
            vec![format!("/unix/{}", "a".repeat(1021)).parse().unwrap()],
            false,
        ),
        (
            vec![components.with(libp2p::multiaddr::Protocol::Memory(0))],
            false,
        ),
    ] {
        let jobs = [MaintenanceJob {
            bootstrap: &addresses,
            ..job(&f, &signed, &roster)
        }];
        let (stop, stopped) = oneshot::channel();
        stop.send(()).unwrap();
        let (reports, _) = watch::channel(None);
        assert_eq!(
            node.run_archive_maintenance(&jobs, schedule(), stopped, reports)
                .await,
            if valid {
                Ok(())
            } else {
                Err(ScheduleError::Jobs)
            }
        );
    }
    assert!(commands.try_recv().is_err());
}

// An already sent or dropped stop channel starts no pass (AT-NET-30).
#[tokio::test]
async fn stopped_before_start_does_no_work() {
    let f = Fixture::new();
    let (_, signed, _) = f.archive();
    let roster = transport_roster(&keys());
    let jobs = [job(&f, &signed, &roster)];
    let (mut node, mut commands) = controlled_handle();
    for send in [true, false] {
        let (stop, stopped) = oneshot::channel();
        let (reports, mut observed) = watch::channel(None);
        if send {
            stop.send(()).unwrap();
        } else {
            drop(stop);
        }
        assert_eq!(
            node.run_archive_maintenance(&jobs, schedule(), stopped, reports)
                .await,
            Ok(())
        );
        assert!(commands.try_recv().is_err());
        assert!(observed.borrow_and_update().is_none());
    }
}

// Slow passes stay serial and subsequent sweeps wait a full delay without catch-up (AT-NET-30).
#[tokio::test]
async fn serial_fixed_delay_and_unobserved_reports() {
    let f = Fixture::new();
    let (_, signed, _) = f.archive();
    let roster = transport_roster(&keys());
    let jobs = [job(&f, &signed, &roster)];
    let index = assigned(&f, &signed, &roster, roster[0]);
    let (mut node, mut commands) = controlled_handle();
    let (stop, stopped) = oneshot::channel();
    let (reports, mut observed) = watch::channel(None);
    let run = node.run_archive_maintenance(&jobs, schedule(), stopped, reports);
    let drive = async {
        let Some(crate::Command::AuditArchive(_, _, _, ack)) = commands.recv().await else {
            panic!("expected audit");
        };
        tokio::time::sleep(schedule().interval * 3).await;
        assert!(commands.try_recv().is_err());
        let finished = tokio::time::Instant::now();
        assert!(ack
            .send(Ok(Audit {
                indices: vec![index],
                healthy: true
            }))
            .is_ok());
        let Some(crate::Command::AuditArchive(_, _, _, ack)) = commands.recv().await else {
            panic!("expected retry audit");
        };
        assert!(finished.elapsed() >= schedule().interval);
        observed.borrow_and_update();
        assert!(ack
            .send(Ok(Audit {
                indices: vec![index],
                healthy: true
            }))
            .is_ok());
        report(&mut observed).await;
        stop.send(()).unwrap();
    };
    let (result, ()) = tokio::join!(run, drive);
    assert_eq!(result, Ok(()));
    let latest = observed.borrow();
    let latest = latest.as_ref().unwrap();
    assert_eq!(latest.job, 0);
    assert_eq!(latest.digest, signed.manifest.digest());
    assert_eq!(latest.result, Ok(Maintenance::Healthy { index }));
    assert!(commands.try_recv().is_err());
}

// Closing the report observer does not stop work, while stop skips the next job (AT-NET-30).
#[tokio::test]
async fn stop_between_jobs_and_closed_observer() {
    let f = Fixture::new();
    let (_, signed, _) = f.archive();
    let mut other = Fixture::new();
    other.target.epoch = 1;
    let (_, second, _) = other.archive();
    let mut last = Fixture::new();
    last.target.epoch = 2;
    let (_, third, _) = last.archive();
    let roster = transport_roster(&keys());
    let jobs = [
        job(&f, &signed, &roster),
        job(&other, &second, &roster),
        job(&last, &third, &roster),
    ];
    let (mut node, mut commands) = controlled_handle();
    let (stop, stopped) = oneshot::channel();
    let (reports, observed) = watch::channel(None);
    drop(observed);
    let run = node.run_archive_maintenance(&jobs, schedule(), stopped, reports);
    let drive = async {
        let Some(crate::Command::AuditArchive(_, _, _, ack)) = commands.recv().await else {
            panic!("expected audit");
        };
        assert!(ack.send(Err(InstallError::Configuration)).is_ok());
        let Some(crate::Command::AuditArchive(_, digest, _, ack)) = commands.recv().await else {
            panic!("expected second job");
        };
        assert_eq!(digest, second.manifest.digest());
        stop.send(()).unwrap();
        assert!(ack
            .send(Ok(Audit {
                indices: vec![],
                healthy: true
            }))
            .is_ok());
    };
    let (result, ()) = tokio::join!(run, drive);
    assert_eq!(result, Ok(()));
    assert!(commands.try_recv().is_err());
}

// Real-node sweeps continue past an invalid job and report the later assignment (AT-NET-30).
#[tokio::test]
async fn real_multi_job_error_isolation() {
    let f = Fixture::new();
    let (segment, signed, _) = f.archive();
    let mut other = Fixture::new();
    other.target.epoch = 1;
    let (_, mut invalid, _) = other.archive();
    invalid.signatures.clear();
    let roster = transport_roster(&keys());
    let jobs = [job(&other, &invalid, &roster), job(&f, &signed, &roster)];
    let mut node = crate::Handle::spawn(keys()[0].clone(), f.config(None)).unwrap();
    let index = assigned(&f, &signed, &roster, roster[0]);
    node.install_archive(&f.context(), &signed, &segment, &[index])
        .await
        .unwrap();
    let (stop, stopped) = oneshot::channel();
    let (reports, mut observed) = watch::channel(None);
    let run = node.run_archive_maintenance(&jobs, schedule(), stopped, reports);
    let drive = async {
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                report(&mut observed).await;
                if observed.borrow().as_ref().unwrap().job == 1 {
                    break;
                }
            }
        })
        .await
        .unwrap();
        stop.send(()).unwrap();
    };
    let (result, ()) = tokio::join!(run, drive);
    assert_eq!(result, Ok(()));
    let latest = observed.borrow();
    let latest = latest.as_ref().unwrap();
    assert_eq!(latest.job, 1);
    assert_eq!(latest.digest, signed.manifest.digest());
    assert_eq!(latest.result, Ok(Maintenance::Healthy { index }));
}

// Failed recovery is retried and periodic audits repair subsequent live corruption (AT-NET-30).
#[tokio::test]
async fn periodic_failure_retry_and_live_repair() {
    let f = Fixture::new();
    let (segment, signed, _) = f.archive();
    let roster = transport_roster(&keys());
    let bootstrap = discovery_fixture(160, NET, None, vec![], None).await;
    let addresses = [bootstrap.address.clone()];
    let jobs = [MaintenanceJob {
        bootstrap: &addresses,
        ..job(&f, &signed, &roster)
    }];
    let dir = Temp::new();
    let mut node = crate::Handle::spawn(keys()[0].clone(), f.config(Some(dir.0.clone()))).unwrap();
    let index = assigned(&f, &signed, &roster, roster[0]);
    let (stop, stopped) = oneshot::channel();
    let (reports, mut observed) = watch::channel(None);
    let run = node.run_archive_maintenance(&jobs, schedule(), stopped, reports);
    let drive = async {
        report(&mut observed).await;
        assert!(observed.borrow().as_ref().unwrap().result.is_err());
        assert!(!path(&dir, &signed).exists());
        let _source = source(&f, &signed, &segment, &bootstrap).await;
        tokio::time::timeout(Duration::from_secs(12), async {
            loop {
                report(&mut observed).await;
                if observed.borrow().as_ref().unwrap().result == Ok(Maintenance::Repaired { index })
                {
                    break;
                }
            }
        })
        .await
        .unwrap();
        let mut bytes = fs::read(path(&dir, &signed)).unwrap();
        *bytes.last_mut().unwrap() ^= 1;
        fs::write(path(&dir, &signed), bytes).unwrap();
        tokio::time::timeout(Duration::from_secs(12), async {
            loop {
                report(&mut observed).await;
                if observed.borrow().as_ref().unwrap().result == Ok(Maintenance::Repaired { index })
                {
                    break;
                }
            }
        })
        .await
        .unwrap();
        stop.send(()).unwrap();
    };
    let (result, ()) = tokio::join!(run, drive);
    assert_eq!(result, Ok(()));
    assert_eq!(
        read_record(&path(&dir, &signed), &f.tiers, signed.manifest.digest())
            .unwrap()
            .shards
            .keys()
            .copied()
            .collect::<Vec<_>>(),
        vec![index]
    );
}

// Stop during discovered fetch waits for its bounded failure and preserves stored data (AT-NET-30).
#[tokio::test]
async fn stop_during_fetch_drains_pass() {
    let f = Fixture::new();
    let (segment, signed, _) = f.archive();
    let roster = transport_roster(&keys());
    let bootstrap = discovery_fixture(161, NET, None, vec![], None).await;
    let silent = discovery_fixture(
        162,
        NET,
        Some(bootstrap.address.clone()),
        (0..5)
            .map(|i| provider_key(&signed.manifest.digest(), i))
            .collect(),
        None,
    )
    .await;
    wait_advertisements(
        &bootstrap,
        &(0..5)
            .map(|i| (provider_key(&signed.manifest.digest(), i), silent.peer))
            .collect::<Vec<_>>(),
    )
    .await;
    let addresses = [bootstrap.address.clone()];
    let jobs = [MaintenanceJob {
        bootstrap: &addresses,
        ..job(&f, &signed, &roster)
    }];
    let dir = Temp::new();
    let mut node = crate::Handle::spawn(keys()[0].clone(), f.config(Some(dir.0.clone()))).unwrap();
    let index = assigned(&f, &signed, &roster, roster[0]);
    node.install_archive(&f.context(), &signed, &segment, &[(index + 1) % 5])
        .await
        .unwrap();
    let old = fs::read(path(&dir, &signed)).unwrap();
    let (stop, stopped) = oneshot::channel();
    let (reports, observed) = watch::channel(None);
    let run = node.run_archive_maintenance(&jobs, schedule(), stopped, reports);
    let drive = async {
        tokio::time::timeout(Duration::from_secs(2), async {
            while silent.requests.load(Ordering::Relaxed) == 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        stop.send(()).unwrap();
    };
    let (result, ()) = tokio::join!(run, drive);
    assert_eq!(result, Ok(()));
    assert_eq!(
        observed.borrow().as_ref().unwrap().result,
        Err(MaintainError::Timeout)
    );
    assert_eq!(fs::read(path(&dir, &signed)).unwrap(), old);
}

// Stop after commit acceptance waits for and reports the durable write acknowledgement (AT-NET-30).
#[tokio::test]
async fn stop_waits_for_commit_acknowledgement() {
    let f = Fixture::new();
    let (segment, signed, _) = f.archive();
    let roster = transport_roster(&keys());
    let bootstrap = discovery_fixture(163, NET, None, vec![], None).await;
    let _source = source(&f, &signed, &segment, &bootstrap).await;
    let addresses = [bootstrap.address.clone()];
    let jobs = [MaintenanceJob {
        bootstrap: &addresses,
        ..job(&f, &signed, &roster)
    }];
    let (mut node, mut commands) = controlled_handle();
    let dir = Temp::new();
    let mut store = Store::open(&f.tiers, Some(dir.0.join("archive"))).unwrap();
    let (stop, stopped) = oneshot::channel();
    let (reports, observed) = watch::channel(None);
    let completed = std::cell::Cell::new(false);
    let run = async {
        let result = node
            .run_archive_maintenance(&jobs, schedule(), stopped, reports)
            .await;
        completed.set(true);
        result
    };
    let drive = async {
        let Some(crate::Command::AuditArchive(context, digest, index, ack)) = commands.recv().await
        else {
            panic!("expected audit");
        };
        assert!(ack
            .send(store.audit(&f.tiers, context, digest, index))
            .is_ok());
        let Some(crate::Command::MaintainArchive(prepared, deadline, ack)) = commands.recv().await
        else {
            panic!("expected commit");
        };
        deadline.remaining().unwrap();
        store.install(&f.tiers, prepared).unwrap();
        stop.send(()).unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(!completed.get());
        assert!(observed.borrow().is_none());
        ack.send(Ok(())).unwrap();
    };
    let (result, ()) = tokio::join!(run, drive);
    assert_eq!(result, Ok(()));
    assert!(matches!(
        observed.borrow().as_ref().unwrap().result,
        Ok(Maintenance::Repaired { .. })
    ));
    assert!(read_record(&path(&dir, &signed), &f.tiers, signed.manifest.digest()).is_some());
    assert!(commands.try_recv().is_err());
}
