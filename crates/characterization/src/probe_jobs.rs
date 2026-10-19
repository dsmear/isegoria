//! The probe's supervisor (`docs/19` §13.11): each run in its own process; CPU, wall time and
//! resident memory polled and capped across sessions; any stop kills every child; no retry.

use crate::probe::{stopped_record, Cost, Phase, Task, SCHEMA};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

pub const POLL: Duration = Duration::from_millis(100);
pub const LEDGER_HEADER: &str = "ledger\ta2-probe/v2";
const TICK_MS: u64 = 10;
const MARGIN: f64 = 1.5;
const R_MAX: u32 = 8;
const R_MIN: u32 = 2;

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

/// Spawns `cmd` so that the kernel kills it if the spawning thread dies (Linux).
pub fn spawn_guarded(mut cmd: Command) -> io::Result<Child> {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::process::CommandExt;
        let parent = std::process::id() as libc::pid_t;
        // SAFETY: only async-signal-safe calls between fork and exec, and no allocation.
        unsafe {
            cmd.pre_exec(move || {
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL as libc::c_ulong) != 0 {
                    return Err(io::Error::last_os_error());
                }
                if libc::getppid() != parent {
                    return Err(io::Error::from_raw_os_error(libc::ESRCH));
                }
                Ok(())
            });
        }
    }
    cmd.spawn()
}

/// An exclusive advisory lock on `out/lock`, released when dropped or when the process dies.
pub struct DirLock(#[allow(dead_code)] fs::File);

/// The directory's lock, or `None` if another supervisor holds it.
pub fn lock_dir(out: &Path) -> io::Result<Option<DirLock>> {
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(out.join("lock"))?;
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        // SAFETY: `flock` on a descriptor this function owns.
        let r = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        if r != 0 {
            let e = io::Error::last_os_error();
            return match e.raw_os_error() {
                Some(libc::EWOULDBLOCK) => Ok(None),
                _ => Err(e),
            };
        }
    }
    Ok(Some(DirLock(file)))
}

/// Why a session does not start.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    Locked,
    Stopped(String),
    Unfinished(Vec<String>),
    Spent,
    Incompatible(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ledger {
    pub started: Vec<String>,
    pub ended: BTreeMap<String, (String, u64)>,
    pub unfinished: Vec<String>,
    pub stop: Option<String>,
    pub cpu_ms: u64,
    pub wall_ms: u64,
}

fn malformed(n: usize, why: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("ledger line {n}: {why}"),
    )
}

/// `ledger.tsv`, read strictly: its header, then `start` (key, pid), `end` (key, outcome, CPU,
/// wall, source), `session` (wall, own CPU, peak, largest gap, stop, provenance), `stop`. A
/// session's stop binds without the `stop` line that follows it.
pub fn read_ledger(out: &Path) -> io::Result<Ledger> {
    let mut ledger = Ledger::default();
    let text = match fs::read_to_string(out.join("ledger.tsv")) {
        Ok(t) => t,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(ledger),
        Err(e) => return Err(e),
    };
    for (i, line) in text.lines().enumerate() {
        let n = i + 1;
        if n == 1 {
            if line != LEDGER_HEADER {
                return Err(malformed(n, "not an a2-probe/v2 ledger"));
            }
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        let num = |j: usize| -> io::Result<u64> {
            f[j].parse::<u64>()
                .map_err(|_| malformed(n, &format!("field {j} is not a count")))
        };
        match (f[0], f.len()) {
            ("start", 3) => {
                num(2)?;
                if ledger.started.iter().any(|k| k == f[1]) {
                    return Err(malformed(n, "a run started twice"));
                }
                ledger.started.push(f[1].to_string());
            }
            ("end", 6) => {
                let cpu = num(3)?;
                num(4)?;
                if !ledger.started.iter().any(|k| k == f[1]) {
                    return Err(malformed(n, "an end without its start"));
                }
                if ledger
                    .ended
                    .insert(f[1].to_string(), (f[2].to_string(), cpu))
                    .is_some()
                {
                    return Err(malformed(n, "a run ended twice"));
                }
                ledger.cpu_ms += cpu;
            }
            ("session", 7) => {
                ledger.wall_ms += num(1)?;
                ledger.cpu_ms += num(2)?;
                num(3)?;
                num(4)?;
                if f[5] != "-" && ledger.stop.is_none() {
                    ledger.stop = Some(f[5].to_string());
                }
            }
            ("stop", 2) => ledger.stop = Some(f[1].to_string()),
            _ => return Err(malformed(n, "unknown or truncated line")),
        }
    }
    ledger.unfinished = ledger
        .started
        .iter()
        .filter(|k| !ledger.ended.contains_key(*k))
        .cloned()
        .collect();
    Ok(ledger)
}

/// The confirmation's replicates by cost alone (`docs/19` §13.11): `min(8, ⌊(C − S)/(1.5 Σ c)⌋)`,
/// `c` each case's explore CPU; refused below 2, after a stop, or with an explore run not done.
pub fn confirm_replicates(ledger: &Ledger, cap_cpu_ms: u64) -> Result<u32, String> {
    if let Some(stop) = &ledger.stop {
        return Err(format!("a stop is recorded: {stop}"));
    }
    let mut sum = 0u64;
    for case in crate::probe::CASES {
        let key = format!("explore-{}-0", case.name());
        match ledger.ended.get(&key) {
            Some((outcome, cpu)) if outcome == "read" || outcome == "refused" => sum += cpu,
            other => return Err(format!("{key}: {other:?}, not a completed run")),
        }
    }
    let left = cap_cpu_ms.saturating_sub(ledger.cpu_ms) as f64;
    let per = MARGIN * sum as f64;
    let r = if per == 0.0 {
        R_MAX
    } else {
        R_MAX.min((left / per).floor() as u32)
    };
    if r < R_MIN {
        return Err(format!("{r} replicates fit the cap, fewer than {R_MIN}"));
    }
    Ok(r)
}

/// A phase's plan: its tasks and the text recorded as `plan-<phase>.tsv`, whose lines before
/// `phase` are the configuration every plan of the directory must share.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    pub phase: Phase,
    pub tasks: Vec<Task>,
    pub text: String,
}

impl Plan {
    pub fn new(config: &str, phase: Phase, replicates: u32, tasks: Vec<Task>) -> Plan {
        let mut text = config.to_string();
        text.push_str(&format!("phase\t{}\t{replicates}\n", phase.name()));
        for t in &tasks {
            text.push_str(&format!("task\t{}\t{}\n", t.key(), t.seed()));
        }
        Plan { phase, tasks, text }
    }

    /// A plan text's configuration: every line before its `phase` line.
    pub fn config(text: &str) -> &str {
        text.find("\nphase\t").map_or(text, |i| &text[..=i])
    }
}

pub fn plan_path(out: &Path, phase: Phase) -> PathBuf {
    out.join(format!("plan-{}.tsv", phase.name()))
}

/// The replicates a phase's recorded plan fixed, if one is recorded.
pub fn planned_replicates(out: &Path, phase: Phase) -> io::Result<Option<u32>> {
    let text = match fs::read_to_string(plan_path(out, phase)) {
        Ok(t) => t,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let line = text.lines().find(|l| l.starts_with("phase\t"));
    let r = line.and_then(|l| l.split('\t').nth(2)?.parse().ok());
    r.map(Some)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "plan without replicates"))
}

fn compatible(out: &Path, plan: &Plan) -> io::Result<Result<(), String>> {
    for entry in fs::read_dir(out)? {
        let path = entry?.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !(name.starts_with("plan-") && name.ends_with(".tsv")) {
            continue;
        }
        let text = fs::read_to_string(&path)?;
        if Plan::config(&text) != Plan::config(&plan.text) {
            return Ok(Err(format!("{name}: another configuration")));
        }
        if path == plan_path(out, plan.phase) && text != plan.text {
            return Ok(Err(format!("{name}: another plan for this phase")));
        }
    }
    Ok(Ok(()))
}

/// What one session did: runs ended by outcome, its CPU (children and itself), its wall time,
/// the largest sampled sum of resident sets, the largest gap between two checks, its stop.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Session {
    pub outcomes: BTreeMap<String, usize>,
    pub skipped: usize,
    pub cpu_ms: u64,
    pub wall_ms: u64,
    pub peak_rss_kib: u64,
    pub max_gap_ms: u64,
    pub stop: Option<String>,
}

/// A spawned child under guard: dropped on any path, it is killed and reaped.
struct Running {
    task: Task,
    child: Child,
    started: Instant,
    cpu_ms: u64,
    rss_kib: u64,
    noted: bool,
}

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Where a session writes: ledger lines (`note`) and records (`record`), each synced.
pub trait Journal {
    fn note(&mut self, line: &str) -> io::Result<()>;
    fn record(&mut self, line: &str) -> io::Result<()>;
}

struct Files {
    records: fs::File,
    ledger: fs::File,
}

impl Journal for Files {
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

/// A child's record, checked on the fields the supervisor relies on: one line, this schema,
/// key and seed, an observed outcome, a measured CPU and no defect. Returns outcome and CPU.
pub fn validate(line: &str, task: &Task) -> Result<(String, u64), String> {
    let line = line.trim_end();
    let one = line.starts_with('{') && line.ends_with('}') && !line.contains('\n');
    let field = |name: &str| json_field(line, name).unwrap_or("");
    if !one || field("schema") != SCHEMA {
        return Err("not a record of this schema".into());
    }
    if field("key") != task.key() || field("seed") != task.seed().to_string() {
        return Err("a record of another run".into());
    }
    if line.contains("\"defect\":") {
        return Err(format!("defect: {}", field("defect")));
    }
    let outcome = field("outcome");
    if outcome != "read" && outcome != "refused" {
        return Err(format!("outcome {outcome:?}"));
    }
    let cpu = field("cpu_ms")
        .parse()
        .map_err(|_| "no measured CPU".to_string())?;
    Ok((outcome.to_string(), cpu))
}

fn clean(reason: &str) -> String {
    reason.replace(['\t', '\n', '\r'], " ")
}

/// A run's end, written later: its own record (`line`), or a diagnostic one built from the
/// design, and a part to keep in `rejected/`.
struct Ended {
    task: Task,
    outcome: String,
    reason: String,
    cpu_ms: u64,
    wall_ms: u64,
    source: &'static str,
    line: Option<String>,
    rejected: Option<PathBuf>,
    noted: bool,
    pid: u32,
}

fn stopped(run: &Running, outcome: &str, reason: &str) -> Ended {
    Ended {
        task: run.task,
        outcome: outcome.into(),
        reason: clean(reason),
        cpu_ms: run.cpu_ms,
        wall_ms: run.started.elapsed().as_millis() as u64,
        source: "polled",
        line: None,
        rejected: None,
        noted: run.noted,
        pid: run.child.id(),
    }
}

fn write_ended(out: &Path, journal: &mut dyn Journal, e: Ended) -> io::Result<()> {
    if let Some(part) = &e.rejected {
        fs::create_dir_all(out.join("rejected"))?;
        let kept = out.join("rejected").join(format!("{}.json", e.task.key()));
        fs::rename(part, kept)?;
    }
    let line = match e.line {
        Some(line) => line,
        None => {
            let cost = Cost {
                wall_ms: e.wall_ms,
                cpu_ms: Some(e.cpu_ms),
                hwm_kib: None,
            };
            stopped_record(&e.task, &e.outcome, &e.reason, cost)
        }
    };
    if !e.noted {
        journal.note(&format!("start\t{}\t{}", e.task.key(), e.pid))?;
    }
    journal.record(&line)?;
    let _ = fs::remove_file(part_path(out, &e.task));
    journal.note(&format!(
        "end\t{}\t{}\t{}\t{}\t{}",
        e.task.key(),
        e.outcome,
        e.cpu_ms,
        e.wall_ms,
        e.source
    ))
}

/// A finished child's end, touching no file: its record if valid, otherwise a technical failure
/// to stop on, its part kept for `rejected/`.
fn collect(out: &Path, run: &Running, ok: bool) -> (Ended, Option<String>) {
    let path = part_path(out, &run.task);
    let read = fs::read_to_string(&path);
    let checked = match (ok, &read) {
        (false, _) => Err("the child exited with an error".to_string()),
        (true, Err(e)) => Err(format!("no record: {e}")),
        (true, Ok(line)) => validate(line, &run.task).map(|v| (v, line.clone())),
    };
    match checked {
        Ok(((outcome, cpu), line)) => {
            let ended = Ended {
                task: run.task,
                outcome,
                reason: String::new(),
                cpu_ms: cpu.max(run.cpu_ms),
                wall_ms: run.started.elapsed().as_millis() as u64,
                source: "self",
                line: Some(line),
                rejected: None,
                noted: run.noted,
                pid: run.child.id(),
            };
            (ended, None)
        }
        Err(why) => {
            let stop = format!("technical: {}: {why}", run.task.key());
            let mut ended = stopped(run, "failed", &why);
            ended.rejected = read.is_ok().then_some(path);
            (ended, Some(stop))
        }
    }
}

/// Runs a plan's tasks `out` has not ended, at most `jobs` at once, each `launch`ed guarded.
/// Refuses another supervisor, an incompatible plan, a recorded stop, an unfinished run or
/// spent caps; then runs [`session`] on the directory's files.
pub fn supervise<F>(
    plan: &Plan,
    jobs: usize,
    out: &Path,
    caps: Caps,
    provenance: &str,
    launch: F,
) -> io::Result<Result<Session, Refusal>>
where
    F: Fn(&Task, &Path) -> Command,
{
    fs::create_dir_all(out.join("parts"))?;
    let Some(_lock) = lock_dir(out)? else {
        return Ok(Err(Refusal::Locked));
    };
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
    if let Err(why) = compatible(out, plan)? {
        return Ok(Err(Refusal::Incompatible(why)));
    }
    fs::write(plan_path(out, plan.phase), &plan.text)?;
    let fresh = fs::metadata(out.join("ledger.tsv")).map_or(true, |m| m.len() == 0);
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
    if fresh {
        files.note(LEDGER_HEADER)?;
    }
    session(
        plan, jobs, out, caps, provenance, &ledger, launch, &mut files,
    )
    .map(Ok)
}

/// One session of `plan` after `ledger`, written to `journal`. On any stop or error every child
/// is killed and reaped before a diagnostic record is built; then the session and its stop.
#[allow(clippy::too_many_arguments)]
pub fn session<F>(
    plan: &Plan,
    jobs: usize,
    out: &Path,
    caps: Caps,
    provenance: &str,
    ledger: &Ledger,
    launch: F,
    journal: &mut dyn Journal,
) -> io::Result<Session>
where
    F: Fn(&Task, &Path) -> Command,
{
    let todo: Vec<Task> = plan
        .tasks
        .iter()
        .filter(|t| !ledger.ended.contains_key(&t.key()))
        .copied()
        .collect();
    let mut s = Session {
        skipped: plan.tasks.len() - todo.len(),
        ..Session::default()
    };
    let (t0, cpu0) = (Instant::now(), proc_cpu_ms("self").unwrap_or(0));
    let mut running: Vec<Running> = Vec::new();
    let mut pending: Vec<Ended> = Vec::new();
    let mut ended_cpu = 0u64;
    let mut state = Loop {
        todo: &todo,
        jobs,
        out,
        caps,
        ledger,
        t0,
        cpu0,
    };
    let result = state.run(
        &launch,
        journal,
        &mut running,
        &mut pending,
        &mut s,
        &mut ended_cpu,
    );
    let (stop, error) = match result {
        Ok(stop) => (stop, None),
        Err(e) => (Some(clean(&format!("supervisor error: {e}"))), Some(e)),
    };
    let reason = stop.clone().unwrap_or_default();
    for r in &mut running {
        let _ = r.child.kill();
    }
    let statuses: Vec<_> = running.iter_mut().map(|r| r.child.wait().ok()).collect();
    for (r, status) in running.iter().zip(statuses) {
        let done = status.is_some_and(|st| st.success());
        let (ended, failure) = if done {
            collect(out, r, true)
        } else {
            (stopped(r, "killed", &reason), None)
        };
        if failure.is_some() || !done {
            pending.push(ended);
        } else {
            pending.insert(0, ended);
        }
    }
    drop(running);
    let mut written = Ok(());
    for e in pending {
        ended_cpu += e.cpu_ms;
        *s.outcomes.entry(e.outcome.clone()).or_default() += 1;
        if written.is_ok() {
            written = write_ended(out, journal, e);
        }
    }
    let own_cpu = proc_cpu_ms("self").unwrap_or(cpu0).saturating_sub(cpu0);
    s.cpu_ms = ended_cpu + own_cpu;
    s.wall_ms = t0.elapsed().as_millis() as u64;
    s.stop = stop.clone();
    let line = format!(
        "session\t{}\t{own_cpu}\t{}\t{}\t{}\t{}",
        s.wall_ms,
        s.peak_rss_kib,
        s.max_gap_ms,
        stop.as_deref().unwrap_or("-"),
        clean(provenance)
    );
    written = written.and_then(|()| journal.note(&line));
    if let Some(reason) = stop {
        written = written.and_then(|()| journal.note(&format!("stop\t{reason}")));
    }
    match error {
        Some(e) => Err(e),
        None => written.map(|()| s),
    }
}

struct Loop<'a> {
    todo: &'a [Task],
    jobs: usize,
    out: &'a Path,
    caps: Caps,
    ledger: &'a Ledger,
    t0: Instant,
    cpu0: u64,
}

impl Loop<'_> {
    /// Launches and polls until every run ends or a stop: a technical failure goes to `pending`
    /// and ends the poll at once, unwritten.
    fn run<F>(
        &mut self,
        launch: &F,
        journal: &mut dyn Journal,
        running: &mut Vec<Running>,
        pending: &mut Vec<Ended>,
        s: &mut Session,
        ended_cpu: &mut u64,
    ) -> io::Result<Option<String>>
    where
        F: Fn(&Task, &Path) -> Command,
    {
        let mut next = 0;
        let mut last = Instant::now();
        loop {
            while running.len() < self.jobs.max(1) && next < self.todo.len() {
                let task = self.todo[next];
                next += 1;
                running.push(Running {
                    child: spawn_guarded(launch(&task, self.out))?,
                    task,
                    started: Instant::now(),
                    cpu_ms: 0,
                    rss_kib: 0,
                    noted: false,
                });
                let run = running.last_mut().expect("just pushed");
                journal.note(&format!("start\t{}\t{}", task.key(), run.child.id()))?;
                run.noted = true;
            }
            if running.is_empty() {
                return Ok(None);
            }
            std::thread::sleep(POLL);
            let now = Instant::now();
            s.max_gap_ms = s.max_gap_ms.max((now - last).as_millis() as u64);
            last = now;
            let mut stop = None;
            let mut i = 0;
            while i < running.len() {
                let run = &mut running[i];
                let pid = run.child.id().to_string();
                if let Some(cpu) = proc_cpu_ms(&pid) {
                    run.cpu_ms = run.cpu_ms.max(cpu);
                }
                run.rss_kib = proc_kib(&pid, "VmRSS").unwrap_or(run.rss_kib);
                let over = run.started.elapsed().as_millis() as u64 >= self.caps.run_ms;
                match run.child.try_wait()? {
                    Some(status) => {
                        let run = running.remove(i);
                        let (ended, failure) = collect(self.out, &run, status.success());
                        if let Some(why) = failure {
                            pending.push(ended);
                            return Ok(Some(why));
                        }
                        *ended_cpu += ended.cpu_ms;
                        *s.outcomes.entry(ended.outcome.clone()).or_default() += 1;
                        write_ended(self.out, journal, ended)?;
                    }
                    None if over => {
                        stop = stop.or(Some(format!("run limit: {}", run.task.key())));
                        i += 1;
                    }
                    None => i += 1,
                }
            }
            let own_cpu = proc_cpu_ms("self")
                .unwrap_or(self.cpu0)
                .saturating_sub(self.cpu0);
            let live: u64 = running.iter().map(|r| r.cpu_ms).sum();
            let rss = running.iter().map(|r| r.rss_kib).sum::<u64>()
                + proc_kib("self", "VmRSS").unwrap_or(0);
            s.peak_rss_kib = s.peak_rss_kib.max(rss);
            let cpu = self.ledger.cpu_ms + *ended_cpu + live + own_cpu;
            let wall = self.ledger.wall_ms + self.t0.elapsed().as_millis() as u64;
            if stop.is_none() {
                stop = if cpu >= self.caps.cpu_ms {
                    Some("cpu cap".to_string())
                } else if wall >= self.caps.wall_ms {
                    Some("wall cap".to_string())
                } else if rss >= self.caps.rss_kib {
                    Some("memory cap".to_string())
                } else {
                    None
                };
            }
            if stop.is_some() {
                return Ok(stop);
            }
        }
    }
}
