//! The probe's supervisor (`docs/19` §13.11): each run in its own process; CPU, wall time and
//! resident memory polled and capped across sessions; partial records kept; no automatic retry.

use crate::probe::{stopped_record, Cost, Task};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::{Duration, Instant};

pub const POLL: Duration = Duration::from_millis(100);
const TICK_MS: u64 = 10;

/// Cumulative limits over every session of one output directory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Caps {
    pub cpu_ms: u64,
    pub wall_ms: u64,
    pub run_ms: u64,
    pub rss_kib: u64,
}

/// `docs/19` §13.11's caps: 2 CPU-hours, 45 minutes, 10 minutes a run, 2 GiB resident.
pub const DECLARED: Caps = Caps {
    cpu_ms: 2 * 3600 * 1000,
    wall_ms: 45 * 60 * 1000,
    run_ms: 10 * 60 * 1000,
    rss_kib: 2 * 1024 * 1024,
};

/// User plus system CPU of `/proc/<pid>` (`"self"` for this process), all its threads.
pub fn proc_cpu_ms(pid: &str) -> Option<u64> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let rest = &stat[stat.rfind(')')? + 2..];
    let fields: Vec<&str> = rest.split(' ').collect();
    let ticks = fields.get(11)?.parse::<u64>().ok()? + fields.get(12)?.parse::<u64>().ok()?;
    Some(ticks * TICK_MS)
}

/// A `kB` field of `/proc/<pid>/status`: `VmRSS` now, `VmHWM` the peak.
pub fn proc_kib(pid: &str, field: &str) -> Option<u64> {
    let status = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    let line = status.lines().find(|l| l.starts_with(field))?;
    line.split_whitespace().nth(1)?.parse().ok()
}

/// This process's cost so far, its wall time measured from `started`.
pub fn self_cost(started: Instant) -> Cost {
    Cost {
        wall_ms: started.elapsed().as_millis() as u64,
        cpu_ms: proc_cpu_ms("self"),
        hwm_kib: proc_kib("self", "VmHWM"),
    }
}

pub fn part_path(out: &Path, task: &Task) -> PathBuf {
    out.join("parts").join(format!("{}.json", task.key()))
}

/// Writes a run's record where the supervisor collects it, renamed into place once complete.
pub fn write_part(out: &Path, task: &Task, record: &str) -> io::Result<()> {
    let path = part_path(out, task);
    fs::create_dir_all(path.parent().expect("parts/"))?;
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, format!("{record}\n"))?;
    fs::rename(tmp, path)
}

/// Why a session does not start: a recorded stop, runs started and never ended, caps spent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    Stopped(String),
    Unfinished(Vec<String>),
    Spent,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ledger {
    pub ended: BTreeMap<String, (String, u64)>,
    pub unfinished: Vec<String>,
    pub stop: Option<String>,
    pub cpu_ms: u64,
    pub wall_ms: u64,
}

/// `ledger.tsv`: `start`, `end` (outcome, CPU, wall), `session` (wall, own CPU, peak, stop).
pub fn read_ledger(out: &Path) -> io::Result<Ledger> {
    let mut ledger = Ledger::default();
    let text = match fs::read_to_string(out.join("ledger.tsv")) {
        Ok(t) => t,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(ledger),
        Err(e) => return Err(e),
    };
    let mut started: Vec<String> = Vec::new();
    for line in text.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        let num = |i: usize| f.get(i).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
        match f.first().copied() {
            Some("start") => started.push(f[1].to_string()),
            Some("end") => {
                ledger.cpu_ms += num(3);
                ledger
                    .ended
                    .insert(f[1].to_string(), (f[2].to_string(), num(3)));
            }
            Some("session") => {
                ledger.wall_ms += num(1);
                ledger.cpu_ms += num(2);
            }
            Some("stop") => ledger.stop = f.get(1).map(|s| s.to_string()),
            _ => {}
        }
    }
    ledger.unfinished = started
        .into_iter()
        .filter(|k| !ledger.ended.contains_key(k))
        .collect();
    Ok(ledger)
}

/// What one session did: runs ended by outcome, its CPU (children and itself), its wall
/// time, the largest sampled sum of resident sets, and the stop it recorded.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Session {
    pub outcomes: BTreeMap<String, usize>,
    pub skipped: usize,
    pub cpu_ms: u64,
    pub wall_ms: u64,
    pub peak_rss_kib: u64,
    pub stop: Option<String>,
}

struct Running {
    task: Task,
    child: Child,
    started: Instant,
    cpu_ms: u64,
    rss_kib: u64,
}

struct Files {
    records: fs::File,
    ledger: fs::File,
}

impl Files {
    fn note(&mut self, line: &str) -> io::Result<()> {
        writeln!(self.ledger, "{line}")?;
        self.ledger.sync_data()
    }

    fn record(&mut self, line: &str) -> io::Result<()> {
        writeln!(self.records, "{}", line.trim_end())?;
        self.records.sync_data()
    }
}

fn json_field<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let at = line.find(&format!("\"{name}\":"))? + name.len() + 3;
    let rest = &line[at..];
    let end = rest.find([',', '}']).unwrap_or(rest.len());
    Some(rest[..end].trim_matches('"'))
}

fn finish(files: &mut Files, out: &Path, run: &mut Running, ok: bool) -> io::Result<(String, u64)> {
    let path = part_path(out, &run.task);
    let wall_ms = run.started.elapsed().as_millis() as u64;
    let (line, outcome, cpu) = match fs::read_to_string(&path) {
        Ok(line) if ok => {
            let outcome = json_field(&line, "outcome")
                .unwrap_or("unknown")
                .to_string();
            let cpu = json_field(&line, "cpu_ms").and_then(|v| v.parse().ok());
            (line, outcome, cpu.unwrap_or(run.cpu_ms))
        }
        _ => {
            let cost = Cost {
                wall_ms,
                cpu_ms: Some(run.cpu_ms),
                hwm_kib: None,
            };
            let why = format!("exit {:?}, no record", run.child.try_wait().ok().flatten());
            let line = stopped_record(&run.task, "failed", &why, cost);
            (line, "failed".to_string(), run.cpu_ms)
        }
    };
    files.record(&line)?;
    files.note(&format!(
        "end\t{}\t{outcome}\t{cpu}\t{wall_ms}",
        run.task.key()
    ))?;
    let _ = fs::remove_file(path);
    Ok((outcome, cpu))
}

fn kill(files: &mut Files, run: &mut Running, reason: &str) -> io::Result<()> {
    let _ = run.child.kill();
    let _ = run.child.wait();
    let cost = Cost {
        wall_ms: run.started.elapsed().as_millis() as u64,
        cpu_ms: Some(run.cpu_ms),
        hwm_kib: None,
    };
    files.record(&stopped_record(&run.task, "killed", reason, cost))?;
    files.note(&format!(
        "end\t{}\tkilled\t{}\t{}",
        run.task.key(),
        run.cpu_ms,
        cost.wall_ms
    ))
}

/// Runs the `tasks` `out` has not ended, at most `jobs` at once, each through `launch` (a child
/// that writes [`part_path`]); refuses rather than resume after a stop or an unfinished run.
pub fn supervise<F>(
    tasks: &[Task],
    jobs: usize,
    out: &Path,
    caps: Caps,
    launch: F,
) -> io::Result<Result<Session, Refusal>>
where
    F: Fn(&Task, &Path) -> io::Result<Child>,
{
    fs::create_dir_all(out.join("parts"))?;
    let ledger = read_ledger(out)?;
    if let Some(reason) = ledger.stop.clone() {
        return Ok(Err(Refusal::Stopped(reason)));
    }
    if !ledger.unfinished.is_empty() {
        return Ok(Err(Refusal::Unfinished(ledger.unfinished)));
    }
    if ledger.cpu_ms >= caps.cpu_ms || ledger.wall_ms >= caps.wall_ms {
        return Ok(Err(Refusal::Spent));
    }
    let open = |name: &str| {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(out.join(name))
    };
    let mut files = Files {
        records: open("records.jsonl")?,
        ledger: open("ledger.tsv")?,
    };
    let todo: Vec<Task> = tasks
        .iter()
        .filter(|t| !ledger.ended.contains_key(&t.key()))
        .copied()
        .collect();
    let mut session = Session {
        skipped: tasks.len() - todo.len(),
        ..Session::default()
    };
    let (t0, cpu0) = (Instant::now(), proc_cpu_ms("self").unwrap_or(0));
    let (mut next, mut running, mut ended_cpu) = (0, Vec::<Running>::new(), 0u64);
    let mut stopping: Option<String> = None;
    loop {
        while stopping.is_none() && running.len() < jobs.max(1) && next < todo.len() {
            let task = todo[next];
            next += 1;
            files.note(&format!("start\t{}", task.key()))?;
            running.push(Running {
                child: launch(&task, out)?,
                task,
                started: Instant::now(),
                cpu_ms: 0,
                rss_kib: 0,
            });
        }
        if running.is_empty() {
            break;
        }
        std::thread::sleep(POLL);
        let mut i = 0;
        while i < running.len() {
            let pid = running[i].child.id().to_string();
            if let Some(cpu) = proc_cpu_ms(&pid) {
                running[i].cpu_ms = running[i].cpu_ms.max(cpu);
            }
            running[i].rss_kib = proc_kib(&pid, "VmRSS").unwrap_or(0);
            let over = running[i].started.elapsed().as_millis() as u64 >= caps.run_ms;
            match running[i].child.try_wait()? {
                Some(status) => {
                    let mut run = running.remove(i);
                    let (outcome, cpu) = finish(&mut files, out, &mut run, status.success())?;
                    ended_cpu += cpu;
                    *session.outcomes.entry(outcome).or_default() += 1;
                }
                None if over => {
                    let mut run = running.remove(i);
                    kill(&mut files, &mut run, "run limit")?;
                    ended_cpu += run.cpu_ms;
                    *session.outcomes.entry("killed".into()).or_default() += 1;
                    stopping.get_or_insert(format!("run limit: {}", run.task.key()));
                }
                None => i += 1,
            }
        }
        let own_cpu = proc_cpu_ms("self").unwrap_or(cpu0).saturating_sub(cpu0);
        let live: u64 = running.iter().map(|r| r.cpu_ms).sum();
        let rss =
            running.iter().map(|r| r.rss_kib).sum::<u64>() + proc_kib("self", "VmRSS").unwrap_or(0);
        session.peak_rss_kib = session.peak_rss_kib.max(rss);
        let cpu = ledger.cpu_ms + ended_cpu + live + own_cpu;
        let wall = ledger.wall_ms + t0.elapsed().as_millis() as u64;
        let cap = if cpu >= caps.cpu_ms {
            Some("cpu cap")
        } else if wall >= caps.wall_ms {
            Some("wall cap")
        } else if rss >= caps.rss_kib {
            Some("memory cap")
        } else {
            None
        };
        if let Some(cap) = cap {
            for mut run in running.drain(..) {
                kill(&mut files, &mut run, cap)?;
                ended_cpu += run.cpu_ms;
                *session.outcomes.entry("killed".into()).or_default() += 1;
            }
            stopping = Some(cap.to_string());
            break;
        }
    }
    let own_cpu = proc_cpu_ms("self").unwrap_or(cpu0).saturating_sub(cpu0);
    session.cpu_ms = ended_cpu + own_cpu;
    session.wall_ms = t0.elapsed().as_millis() as u64;
    session.stop = stopping.clone();
    files.note(&format!(
        "session\t{}\t{own_cpu}\t{}\t{}",
        session.wall_ms,
        session.peak_rss_kib,
        stopping.as_deref().unwrap_or("-")
    ))?;
    if let Some(reason) = stopping {
        files.note(&format!("stop\t{reason}"))?;
    }
    Ok(Ok(session))
}
