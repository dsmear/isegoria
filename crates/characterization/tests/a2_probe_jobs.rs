//! The A2 probe's supervisor (`docs/19` §13.11) on stand-in children (`true`, `false`, `sleep`)
//! and constructed records, never a fit: stops, kills, locks, ledgers, plans, the cost rule.
#![cfg(target_os = "linux")]

use characterization::probe::{stopped_record, Case, Cost, Phase, Task, CASES};
use characterization::probe_jobs::{
    confirm_replicates, lock_dir, read_ledger, session, supervise, validate, write_part, Caps,
    Journal, Ledger, Plan, Refusal, Session, LEDGER_HEADER, POLL,
};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Barrier;
use std::time::{Duration, Instant};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("a2-jobs-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

fn task(case: Case) -> Task {
    Task {
        phase: Phase::Smoke,
        case,
        replicate: 0,
    }
}

fn all() -> Vec<Task> {
    CASES.into_iter().map(task).collect()
}

fn caps(run_ms: u64, wall_ms: u64) -> Caps {
    Caps {
        cpu_ms: 3_600_000,
        wall_ms,
        run_ms,
        rss_kib: 1 << 30,
    }
}

fn record(t: &Task, outcome: &str) -> String {
    let cost = Cost {
        wall_ms: 0,
        cpu_ms: Some(0),
        hwm_kib: None,
    };
    stopped_record(t, outcome, "stand-in", cost)
}

fn fake(t: &Task, out: &Path) -> Command {
    write_part(out, t, &record(t, "read")).unwrap();
    Command::new("true")
}

fn sleep(_: &Task, _: &Path) -> Command {
    let mut c = Command::new("sleep");
    c.arg("30");
    c
}

fn run<F>(
    tasks: &[Task],
    jobs: usize,
    out: &Path,
    caps: Caps,
    f: F,
) -> io::Result<Result<Session, Refusal>>
where
    F: Fn(&Task, &Path) -> Command,
{
    let plan = Plan::new("schema\tstand-in\n", Phase::Smoke, 1, tasks.to_vec());
    supervise(&plan, jobs, out, caps, "test", f)
}

fn lines(out: &Path, name: &str) -> Vec<String> {
    let text = fs::read_to_string(out.join(name)).unwrap_or_default();
    text.lines().map(str::to_string).collect()
}

fn pids(out: &Path) -> Vec<u32> {
    let started = lines(out, "ledger.tsv");
    started
        .iter()
        .filter_map(|l| l.strip_prefix("start\t")?.split('\t').nth(1)?.parse().ok())
        .collect()
}

/// A process neither gone, zombie (`Z`) nor being reaped (`X`, `x`).
fn alive(pid: u32) -> bool {
    fs::read_to_string(format!("/proc/{pid}/stat"))
        .map(|s| !s[s.rfind(')').unwrap() + 2..].starts_with(['Z', 'X', 'x']))
        .unwrap_or(false)
}

/// One test at a time spawns: a fork in another test thread would hold a copy of a directory's
/// lock until its `exec`, a refusal no single supervisor process meets.
fn serial() -> std::sync::MutexGuard<'static, ()> {
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

/// Each ended run is recorded once; a later session skips them and runs nothing again.
#[test]
fn ended_runs_are_recorded_once_and_never_rerun() {
    let _serial = serial();
    let out = scratch("resume");
    let first = run(&all(), 3, &out, caps(60_000, 60_000), fake)
        .unwrap()
        .unwrap();
    assert_eq!(
        (first.outcomes.get("read"), first.stop.as_deref()),
        (Some(&6), None)
    );
    let again = run(&all(), 3, &out, caps(60_000, 60_000), fake)
        .unwrap()
        .unwrap();
    assert_eq!((again.skipped, again.outcomes.len()), (6, 0));
    assert_eq!(lines(&out, "records.jsonl").len(), 6);
    assert_eq!(lines(&out, "ledger.tsv")[0], LEDGER_HEADER);
    assert!(fs::read_dir(out.join("parts")).unwrap().next().is_none());
}

/// A gate refusal is an observation of the probe, recorded, and stops nothing.
#[test]
fn a_gate_refusal_is_an_observation_not_a_stop() {
    let _serial = serial();
    let out = scratch("refused");
    let refused = |t: &Task, o: &Path| {
        write_part(o, t, &record(t, "refused")).unwrap();
        Command::new("true")
    };
    let s = run(&all(), 2, &out, caps(60_000, 60_000), refused)
        .unwrap()
        .unwrap();
    assert_eq!((s.outcomes.get("refused"), s.stop), (Some(&6), None));
}

/// A failed child stops new launches, kills the running ones and refuses any later session.
#[test]
fn a_failed_child_stops_new_launches_and_kills_the_running_ones() {
    let _serial = serial();
    let out = scratch("failed");
    let launch = |t: &Task, o: &Path| match t.case {
        Case::C0 => Command::new("false"),
        Case::C1 => sleep(t, o),
        _ => fake(t, o),
    };
    let s = run(&all(), 2, &out, caps(60_000, 60_000), launch)
        .unwrap()
        .unwrap();
    assert_eq!(
        (s.outcomes.get("failed"), s.outcomes.get("killed")),
        (Some(&1), Some(&1))
    );
    assert!(
        s.stop
            .as_deref()
            .unwrap()
            .starts_with("technical: smoke-C0-0"),
        "{s:?}"
    );
    assert_eq!(lines(&out, "records.jsonl").len(), 2);
    assert!(pids(&out).iter().all(|&p| !alive(p)));
    let again = run(&all(), 2, &out, caps(60_000, 60_000), fake).unwrap();
    assert!(matches!(again, Err(Refusal::Stopped(_))));
}

/// An invalid record, another run's record and a record carrying the defect are technical stops,
/// each record kept apart in `rejected/` and the run recorded as failed.
#[test]
fn an_invalid_foreign_or_defective_record_is_a_technical_stop() {
    let _serial = serial();
    let defective = |t: &Task| {
        let r = record(t, "read");
        format!(
            "{},\"defect\":\"selection over DTF_MAX\"}}",
            &r[..r.len() - 1]
        )
    };
    type Writer<'a> = (&'a str, &'a dyn Fn(&Task) -> String);
    let writers: [Writer; 3] = [
        ("garbage", &|_: &Task| "not a record".to_string()),
        ("foreign", &|_: &Task| record(&task(Case::C5), "read")),
        ("defect", &defective),
    ];
    for (name, write) in writers {
        let out = scratch(name);
        let launch = |t: &Task, o: &Path| {
            write_part(o, t, &write(t)).unwrap();
            Command::new("true")
        };
        let s = run(&all()[..3], 1, &out, caps(60_000, 60_000), launch)
            .unwrap()
            .unwrap();
        assert_eq!(s.outcomes.get("failed"), Some(&1), "{name}: {s:?}");
        assert!(
            s.stop.unwrap().starts_with("technical: smoke-C0-0"),
            "{name}"
        );
        assert!(out.join("rejected/smoke-C0-0.json").exists(), "{name}");
        let records = lines(&out, "records.jsonl");
        assert_eq!(records.len(), 1, "{name}");
        assert!(records[0].contains("\"outcome\":\"failed\""), "{name}");
    }
}

/// A supervisor error kills the running children, records them and a stop, and is returned.
#[test]
fn a_supervisor_error_kills_the_running_children_and_records_a_stop() {
    let _serial = serial();
    let out = scratch("error");
    let launch = |t: &Task, o: &Path| match t.case {
        Case::C0 => sleep(t, o),
        _ => Command::new("/nonexistent/a2-probe-stand-in"),
    };
    let r = run(&all()[..2], 2, &out, caps(60_000, 60_000), launch);
    assert!(r.is_err());
    assert!(pids(&out).iter().all(|&p| !alive(p)));
    let ledger = read_ledger(&out).unwrap();
    assert!(ledger.stop.unwrap().starts_with("supervisor error"));
    assert_eq!(
        ledger.ended.get("smoke-C0-0").map(|e| e.0.as_str()),
        Some("killed")
    );
}

/// The run limit kills every running child, and the stop refuses a resumption.
#[test]
fn the_run_limit_kills_every_running_child_and_refuses_a_resumption() {
    let _serial = serial();
    let out = scratch("run-limit");
    let s = run(&all(), 2, &out, caps(300, 60_000), sleep)
        .unwrap()
        .unwrap();
    assert_eq!(s.outcomes.get("killed"), Some(&2));
    assert!(s.wall_ms < 5_000, "{s:?}");
    assert!(s.stop.as_deref().unwrap().starts_with("run limit: smoke-C"));
    assert!(pids(&out).iter().all(|&p| !alive(p)));
    let refused = run(&all(), 2, &out, caps(300, 60_000), sleep).unwrap();
    assert!(matches!(refused, Err(Refusal::Stopped(_))), "{refused:?}");
}

/// The wall cap kills every running child, each recorded as killed.
#[test]
fn the_wall_cap_kills_every_running_child() {
    let _serial = serial();
    let out = scratch("wall");
    let s = run(&all(), 2, &out, caps(60_000, 400), sleep)
        .unwrap()
        .unwrap();
    assert_eq!(
        (s.outcomes.get("killed"), s.stop.as_deref()),
        (Some(&2), Some("wall cap"))
    );
    let ledger = read_ledger(&out).unwrap();
    assert_eq!((ledger.ended.len(), ledger.unfinished.len()), (2, 0));
}

/// The check interval actually reached is recorded: the poll, plus the loop's own work.
#[test]
fn the_largest_gap_between_checks_is_recorded() {
    let _serial = serial();
    let out = scratch("gap");
    let s = run(&all(), 3, &out, caps(60_000, 60_000), fake)
        .unwrap()
        .unwrap();
    let poll = POLL.as_millis() as u64;
    assert!(s.max_gap_ms >= poll && s.max_gap_ms < 10 * poll, "{s:?}");
    println!("largest gap between checks: {} ms", s.max_gap_ms);
}

/// A second supervisor on the same directory is refused while the first holds it.
#[test]
fn a_second_supervisor_on_the_directory_is_refused() {
    let _serial = serial();
    let out = scratch("lock");
    fs::create_dir_all(&out).unwrap();
    let held = lock_dir(&out).unwrap().unwrap();
    assert!(matches!(
        run(&all(), 2, &out, caps(60_000, 60_000), fake).unwrap(),
        Err(Refusal::Locked)
    ));
    drop(held);
    let both = Barrier::new(2);
    let slow = |t: &Task, o: &Path| {
        write_part(o, t, &record(t, "read")).unwrap();
        let mut c = Command::new("sleep");
        c.arg("0.3");
        c
    };
    let outcomes: Vec<bool> = std::thread::scope(|s| {
        let go = || {
            both.wait();
            matches!(
                run(&all(), 2, &out, caps(60_000, 60_000), slow).unwrap(),
                Err(Refusal::Locked)
            )
        };
        let (a, b) = (s.spawn(go), s.spawn(go));
        vec![a.join().unwrap(), b.join().unwrap()]
    });
    assert_eq!(outcomes.iter().filter(|&&locked| locked).count(), 1);
    assert_eq!(lines(&out, "records.jsonl").len(), 6);
}

/// Stand-in supervisor for the next test: runs only with `A2_PROBE_STAND_IN` set.
#[test]
fn stand_in_supervisor() {
    let Ok(dir) = std::env::var("A2_PROBE_STAND_IN") else {
        return;
    };
    let _ = run(&all()[..2], 2, Path::new(&dir), caps(60_000, 60_000), sleep);
}

/// The children die with a supervisor killed by SIGKILL, and the next session refuses.
#[test]
fn the_children_die_with_a_killed_supervisor() {
    let _serial = serial();
    let out = scratch("orphans");
    fs::create_dir_all(&out).unwrap();
    let mut helper = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "stand_in_supervisor", "--test-threads=1"])
        .env("A2_PROBE_STAND_IN", &out)
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let t0 = Instant::now();
    while pids(&out).len() < 2 && t0.elapsed() < Duration::from_secs(20) {
        std::thread::sleep(Duration::from_millis(20));
    }
    let children = pids(&out);
    assert_eq!(children.len(), 2);
    helper.kill().unwrap();
    helper.wait().unwrap();
    let t1 = Instant::now();
    while children.iter().any(|&p| alive(p)) && t1.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(20));
    }
    let survivors: Vec<u32> = children.iter().copied().filter(|&p| alive(p)).collect();
    for p in &survivors {
        let _ = Command::new("kill").args(["-9", &p.to_string()]).status();
    }
    assert!(
        survivors.is_empty(),
        "children outlived the supervisor: {survivors:?}"
    );
    println!(
        "children gone {} ms after the supervisor's SIGKILL",
        t1.elapsed().as_millis()
    );
    let refused = run(&all()[..2], 2, &out, caps(60_000, 60_000), sleep).unwrap();
    assert!(matches!(refused, Err(Refusal::Unfinished(_))));
}

/// A malformed ledger is an error, never a zero cost; a run started twice or ended without
/// starting is malformed too.
#[test]
fn a_malformed_ledger_is_refused_not_read_as_zero() {
    let bad = [
        "end\tsmoke-C0-0\tread\t5\t5\tself\n",
        "ledger\ta2-probe/v2\nstart\tsmoke-C0-0\t7\nend\tsmoke-C0-0\tread\tNaN\t5\tself\n",
        "ledger\ta2-probe/v2\nbogus\n",
        "ledger\ta2-probe/v2\nend\tsmoke-C0-0\tread\t5\t5\tself\n",
        "ledger\ta2-probe/v2\nstart\tsmoke-C0-0\t7\nstart\tsmoke-C0-0\t8\n",
        "ledger\ta2-probe/v2\nsession\t10\t-1\t0\t0\t-\tx\n",
    ];
    for (i, text) in bad.iter().enumerate() {
        let out = scratch(&format!("ledger{i}"));
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join("ledger.tsv"), text).unwrap();
        let e = read_ledger(&out).unwrap_err();
        assert_eq!(e.kind(), io::ErrorKind::InvalidData, "{i}: {e}");
        assert!(
            run(&all(), 1, &out, caps(60_000, 60_000), fake).is_err(),
            "{i}"
        );
    }
}

/// Another configuration, or another plan for the same phase, refuses the directory.
#[test]
fn an_incompatible_plan_is_refused() {
    let _serial = serial();
    let out = scratch("plan");
    run(&all()[..1], 1, &out, caps(60_000, 60_000), fake)
        .unwrap()
        .unwrap();
    let other = Plan::new("schema\tother\n", Phase::Smoke, 1, all()[..1].to_vec());
    let r = supervise(&other, 1, &out, caps(60_000, 60_000), "test", fake).unwrap();
    assert!(matches!(r, Err(Refusal::Incompatible(_))), "{r:?}");
    let more = Plan::new("schema\tstand-in\n", Phase::Smoke, 2, all());
    let r = supervise(&more, 1, &out, caps(60_000, 60_000), "test", fake).unwrap();
    assert!(matches!(r, Err(Refusal::Incompatible(_))), "{r:?}");
}

/// A record is valid only for its own run, with an observed outcome and a measured CPU.
#[test]
fn only_a_record_of_this_run_with_a_measured_cpu_is_valid() {
    let t = task(Case::C2);
    assert_eq!(
        validate(&record(&t, "read"), &t),
        Ok(("read".to_string(), 0))
    );
    assert_eq!(
        validate(&record(&t, "refused"), &t),
        Ok(("refused".to_string(), 0))
    );
    assert!(validate(&record(&t, "killed"), &t).is_err());
    assert!(validate(&record(&task(Case::C3), "read"), &t).is_err());
    let unmeasured = stopped_record(&t, "read", "stand-in", Cost::default());
    assert!(validate(&unmeasured, &t).is_err());
}

/// The confirmation's replicates follow the cost rule alone, and are refused without it.
#[test]
fn the_confirmation_s_replicates_follow_the_cost_rule_alone() {
    let ledger = |cpu: u64, outcome: &str, skip: Option<&str>| {
        let mut l = Ledger::default();
        for c in CASES.iter().filter(|c| Some(c.name()) != skip) {
            l.ended
                .insert(format!("explore-{}-0", c.name()), (outcome.into(), cpu));
            l.cpu_ms += cpu;
        }
        l.cpu_ms += 2_000;
        l
    };
    let cap = 7_200_000;
    assert_eq!(
        confirm_replicates(&ledger(100_000, "read", None), cap),
        Ok(7)
    );
    assert_eq!(
        confirm_replicates(&ledger(10_000, "refused", None), cap),
        Ok(8)
    );
    assert!(confirm_replicates(&ledger(100_000, "killed", None), cap).is_err());
    assert!(confirm_replicates(&ledger(100_000, "read", Some("C3")), cap).is_err());
    assert!(confirm_replicates(&ledger(700_000, "read", None), cap).is_err());
    let mut stopped = ledger(1_000, "read", None);
    stopped.stop = Some("cpu cap".into());
    assert!(confirm_replicates(&stopped, cap).is_err());
}

/// The binary exits 3 after a stop and 1 when the stop refuses the next session; smoke children,
/// refused at the gate, never fit.
#[test]
fn the_binary_exits_non_zero_after_a_stop() {
    let _serial = serial();
    let out = scratch("binary");
    let probe = |extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_a2-probe"))
            .args(["run", "--phase", "smoke", "--jobs", "2", "--out"])
            .arg(&out)
            .args(extra)
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap()
            .code()
    };
    assert_eq!(probe(&["--max-rss-mib", "0"]), Some(3));
    assert!(read_ledger(&out)
        .unwrap()
        .stop
        .is_some_and(|s| s == "memory cap"));
    assert_eq!(probe(&[]), Some(1));
}

/// A journal in memory: each started pid, each record, and at every failed record whether the
/// other started children were still alive; `fail_starts` refuses every `start` line.
#[derive(Default)]
struct Memory {
    fail_starts: bool,
    pids: Vec<(String, u32)>,
    records: Vec<String>,
    alive_at_failure: Vec<(String, bool)>,
}

impl Journal for Memory {
    fn note(&mut self, line: &str) -> io::Result<()> {
        if let Some(rest) = line.strip_prefix("start\t") {
            let (key, pid) = rest.split_once('\t').unwrap();
            self.pids.push((key.to_string(), pid.parse().unwrap()));
            if self.fail_starts {
                return Err(io::Error::other("stand-in write failure"));
            }
        }
        Ok(())
    }

    fn record(&mut self, line: &str) -> io::Result<()> {
        if line.contains("\"outcome\":\"failed\"") {
            for (key, pid) in &self.pids {
                if !line.contains(&format!("\"key\":\"{key}\"")) {
                    self.alive_at_failure.push((key.clone(), alive(*pid)));
                }
            }
        }
        self.records.push(line.to_string());
        Ok(())
    }
}

fn plan_of(tasks: &[Task]) -> Plan {
    Plan::new("schema\tstand-in\n", Phase::Smoke, 1, tasks.to_vec())
}

/// A child whose `start` cannot be written is killed and reaped before the error returns, the
/// caller still alive.
#[test]
fn a_child_is_killed_when_its_start_cannot_be_written() {
    let _serial = serial();
    let out = scratch("start-write");
    fs::create_dir_all(out.join("parts")).unwrap();
    let mut journal = Memory {
        fail_starts: true,
        ..Memory::default()
    };
    let tasks = [task(Case::C0)];
    let caps = caps(60_000, 60_000);
    let r = session(
        &plan_of(&tasks),
        1,
        &out,
        caps,
        "test",
        &Ledger::default(),
        sleep,
        &mut journal,
    );
    assert!(r.is_err());
    let (_, pid) = journal.pids[0];
    assert!(!alive(pid), "child {pid} outlived the failed write");
}

/// A technical failure kills and reaps every running child before its diagnostic record is
/// built; the failed run's record and its rejected part are kept.
#[test]
fn the_children_are_killed_before_a_failure_s_record_is_built() {
    let _serial = serial();
    for invalid in [false, true] {
        let out = scratch(if invalid {
            "order-invalid"
        } else {
            "order-failed"
        });
        fs::create_dir_all(out.join("parts")).unwrap();
        let launch = |t: &Task, o: &Path| match t.case {
            Case::C0 if invalid => {
                write_part(o, t, "not a record").unwrap();
                Command::new("true")
            }
            Case::C0 => Command::new("false"),
            _ => sleep(t, o),
        };
        let mut journal = Memory::default();
        let tasks = [task(Case::C0), task(Case::C1)];
        let caps = caps(60_000, 60_000);
        let s = session(
            &plan_of(&tasks),
            2,
            &out,
            caps,
            "test",
            &Ledger::default(),
            launch,
            &mut journal,
        )
        .unwrap();
        assert!(s.stop.unwrap().starts_with("technical: smoke-C0-0"));
        assert_eq!(
            journal.alive_at_failure,
            vec![("smoke-C1-0".to_string(), false)]
        );
        assert!(journal
            .records
            .iter()
            .any(|r| r.contains("\"key\":\"smoke-C0-0\"") && r.contains("\"outcome\":\"failed\"")));
        assert!(journal
            .records
            .iter()
            .any(|r| r.contains("\"key\":\"smoke-C1-0\"") && r.contains("\"outcome\":\"killed\"")));
        assert_eq!(out.join("rejected/smoke-C0-0.json").exists(), invalid);
    }
}

/// A ledger ending on a session line that names a stop refuses resumption and the cost rule,
/// although the `stop` line after it is missing.
#[test]
fn a_stop_in_the_session_line_binds_without_the_stop_line() {
    let _serial = serial();
    let out = scratch("session-stop");
    fs::create_dir_all(&out).unwrap();
    let mut text = format!("{LEDGER_HEADER}\n");
    for c in CASES {
        let k = format!("explore-{}-0", c.name());
        text += &format!("start\t{k}\t1\nend\t{k}\tread\t1000\t1\tself\n");
    }
    text += "session\t10\t0\t0\t100\tcpu cap\tx\n";
    fs::write(out.join("ledger.tsv"), &text).unwrap();
    let ledger = read_ledger(&out).unwrap();
    assert_eq!(ledger.stop.as_deref(), Some("cpu cap"));
    assert!(confirm_replicates(&ledger, 7_200_000).is_err());
    let r = run(&all(), 1, &out, caps(60_000, 60_000), fake).unwrap();
    assert_eq!(r, Err(Refusal::Stopped("cpu cap".into())));
}
