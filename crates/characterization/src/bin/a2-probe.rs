//! The A2 probe's command line (`docs/19` §13.11): `plan`, `run` (the supervisor) and `one`
//! (a single run in its own process, launched by `run`).

use characterization::probe::{config_lines, run, tasks, Case, Phase, Task};
use characterization::probe_jobs::{
    confirm_replicates, planned_replicates, read_ledger, self_cost, supervise, write_part, Caps,
    Plan, Refusal, DECLARED,
};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

const USAGE: &str = "\
usage: a2-probe <plan|run|one> [options]

  plan   the runs of a phase, in their order
  run    the runs of a phase, each in its own process, under the caps; plan, records,
         ledger and parts go to <out>; any stop kills every running child and refuses
         a later session. Exit 0 when the phase ends, 3 after a stop, 1 if refused
  one    one run (used by run): its record to <out>/parts

options:
  --phase smoke|explore|confirm|candidates  (smoke)
  --replicates R                 (smoke only; explore runs 1, confirm takes R from the
                                 cost rule after explore, candidates refits confirm's 8)
  --case C0..C5 --replicate R    (one only)
  --jobs J                       (8)
  --out DIR                      (characterization-results/a2-probe)
  --max-cpu-s S --max-wall-s S --max-run-s S --max-rss-mib M
                                 cumulative caps, docs/19 §13.11's by default

run must start in the repository: it records the candidate from git and refuses
explore and confirm with uncommitted changes in crates/ or Cargo.lock.
";

const ENGINE_BASE: &str = "6274270";

struct Args {
    command: String,
    phase: Phase,
    replicates: Option<u32>,
    case: Option<Case>,
    replicate: u32,
    jobs: usize,
    out: PathBuf,
    caps: Caps,
}

fn parse(mut argv: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut args = Args {
        command: argv.next().ok_or("no command")?,
        phase: Phase::Smoke,
        replicates: None,
        case: None,
        replicate: 0,
        jobs: 8,
        out: PathBuf::from("characterization-results/a2-probe"),
        caps: DECLARED,
    };
    while let Some(flag) = argv.next() {
        let value = argv.next().ok_or(format!("{flag} needs a value"))?;
        let number = |v: &str| {
            v.parse::<u64>()
                .map_err(|_| format!("{flag} needs a number"))
        };
        match flag.as_str() {
            "--phase" => args.phase = Phase::parse(&value).ok_or("unknown phase")?,
            "--replicates" => args.replicates = Some(number(&value)? as u32),
            "--case" => args.case = Some(Case::parse(&value).ok_or("unknown case")?),
            "--replicate" => args.replicate = number(&value)? as u32,
            "--jobs" => args.jobs = number(&value)? as usize,
            "--out" => args.out = PathBuf::from(value),
            "--max-cpu-s" => args.caps.cpu_ms = number(&value)? * 1000,
            "--max-wall-s" => args.caps.wall_ms = number(&value)? * 1000,
            "--max-run-s" => args.caps.run_ms = number(&value)? * 1000,
            "--max-rss-mib" => args.caps.rss_kib = number(&value)? * 1024,
            other => return Err(format!("unknown option {other:?}")),
        }
    }
    Ok(args)
}

fn one(args: &Args) -> Result<(), String> {
    let started = Instant::now();
    let task = Task {
        phase: args.phase,
        case: args.case.ok_or("one needs --case")?,
        replicate: args.replicate,
    };
    let record = run(&task, || Some(self_cost(started)));
    write_part(&args.out, &task, &record).map_err(|e| e.to_string())
}

fn git_at(top: Option<&str>, args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("git");
    if let Some(top) = top {
        cmd.args(["-C", top]);
    }
    let out = cmd
        .args(args)
        .output()
        .map_err(|e| format!("git {args:?}: {e}"))?;
    if !out.status.success() {
        return Err(format!("git {args:?} failed"));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// `git args` run at the repository's top, whatever the current directory.
fn git(args: &[&str]) -> Result<String, String> {
    let top = git_at(None, &["rev-parse", "--show-toplevel"])?;
    git_at(Some(&top), args)
}

const DIAGNOSTIC: &str = "crates/scoring/src/latent.rs";

fn unchanged(top: &str, paths: &[&str]) -> Result<bool, String> {
    Command::new("git")
        .args(["-C", top, "diff", "--quiet", ENGINE_BASE, "--"])
        .args(paths)
        .status()
        .map(|s| s.success())
        .map_err(|e| e.to_string())
}

/// The candidate: the committed trees of `crates/`, `Cargo.lock` and the diagnostic file, clean
/// unless `smoke`; the engine as at `ENGINE_BASE` but for that file, which only `candidates` and
/// smoke may change.
fn candidate(phase: Phase) -> Result<String, String> {
    let top = git_at(None, &["rev-parse", "--show-toplevel"])?;
    let outside = format!(":(exclude){DIAGNOSTIC}");
    if !unchanged(&top, &["crates/protocol", "crates/scoring", &outside])? {
        return Err(format!(
            "crates/scoring or crates/protocol differ from {ENGINE_BASE} beyond {DIAGNOSTIC}"
        ));
    }
    let diagnostic = matches!(phase, Phase::Candidates | Phase::Smoke);
    if !diagnostic && !unchanged(&top, &[DIAGNOSTIC])? {
        return Err(format!(
            "{phase:?} runs {ENGINE_BASE}'s engine; {DIAGNOSTIC} differs"
        ));
    }
    let dirty = !git(&["status", "--porcelain", "--", "crates", "Cargo.lock"])?.is_empty();
    if dirty && phase != Phase::Smoke {
        return Err("uncommitted changes in crates/ or Cargo.lock".into());
    }
    let tree = git(&["rev-parse", "HEAD:crates"])?;
    let lock = git(&["rev-parse", "HEAD:Cargo.lock"])?;
    let latent = git(&["rev-parse", &format!("HEAD:{DIAGNOSTIC}")])?;
    let state = if dirty { "dirty" } else { "clean" };
    Ok(format!("candidate\t{tree}\t{lock}\t{latent}\t{state}\n"))
}

fn provenance() -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let digest = Sha256::digest(std::fs::read(&exe).map_err(|e| e.to_string())?);
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    Ok(format!("head={} exe={hex}", git(&["rev-parse", "HEAD"])?))
}

fn replicates(args: &Args) -> Result<u32, String> {
    match (args.phase, args.replicates) {
        (Phase::Smoke, r) => Ok(r.unwrap_or(1)),
        (Phase::Explore, None | Some(1)) => Ok(1),
        (Phase::Explore, Some(_)) => Err("explore runs one replicate".into()),
        (Phase::Candidates, None | Some(8)) => Ok(8),
        (Phase::Candidates, Some(_)) => Err("candidates refits confirm's 8 replicates".into()),
        (Phase::Confirm, Some(_)) => Err("confirm takes its replicates from the cost rule".into()),
        (Phase::Confirm, None) => {
            if let Some(r) =
                planned_replicates(&args.out, Phase::Confirm).map_err(|e| e.to_string())?
            {
                return Ok(r);
            }
            let ledger = read_ledger(&args.out).map_err(|e| e.to_string())?;
            confirm_replicates(&ledger, args.caps.cpu_ms)
        }
    }
}

fn supervised(args: &Args) -> Result<bool, String> {
    let r = replicates(args)?;
    let caps = args.caps;
    let config = format!(
        "{}caps\t{}\t{}\t{}\t{}\n{}",
        candidate(args.phase)?,
        caps.cpu_ms,
        caps.wall_ms,
        caps.run_ms,
        caps.rss_kib,
        config_lines(args.phase)
    );
    let plan = Plan::new(&config, args.phase, r, tasks(args.phase, r));
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let launch = |task: &Task, out: &std::path::Path| {
        let mut cmd = Command::new(&exe);
        cmd.args([
            "one",
            "--phase",
            task.phase.name(),
            "--case",
            task.case.name(),
        ])
        .args(["--replicate", &task.replicate.to_string()])
        .arg("--out")
        .arg(out)
        .stdin(Stdio::null())
        .stdout(Stdio::null());
        cmd
    };
    eprintln!(
        "{} runs planned ({} replicates); {} workers; caps {caps:?}; out {}",
        plan.tasks.len(),
        r,
        args.jobs,
        args.out.display()
    );
    let session = match supervise(&plan, args.jobs, &args.out, caps, &provenance()?, launch) {
        Ok(Ok(session)) => session,
        Ok(Err(Refusal::Locked)) => {
            return Err("refused: another supervisor holds the directory".into())
        }
        Ok(Err(Refusal::Stopped(why))) => {
            return Err(format!("refused: a stop is recorded ({why})"))
        }
        Ok(Err(Refusal::Unfinished(keys))) => {
            return Err(format!("refused: runs never ended: {keys:?}"))
        }
        Ok(Err(Refusal::Spent)) => return Err("refused: the cumulative caps are spent".into()),
        Ok(Err(Refusal::Incompatible(why))) => return Err(format!("refused: {why}")),
        Err(e) => return Err(format!("stopped on a supervisor error: {e}")),
    };
    eprintln!(
        "outcomes {:?}, {} already ended; session CPU {} ms, wall {} ms, largest sampled \
         resident sum {} KiB, largest gap between checks {} ms; stop: {}",
        session.outcomes,
        session.skipped,
        session.cpu_ms,
        session.wall_ms,
        session.peak_rss_kib,
        session.max_gap_ms,
        session.stop.as_deref().unwrap_or("none")
    );
    Ok(session.stop.is_none())
}

fn main() -> ExitCode {
    let args = match parse(std::env::args().skip(1)) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let result = match args.command.as_str() {
        "plan" => replicates(&args).map(|r| {
            for task in tasks(args.phase, r) {
                println!("{} seed {}", task.key(), task.seed());
            }
            true
        }),
        "run" => supervised(&args),
        "one" => one(&args).map(|()| true),
        _ => Err(format!("unknown command\n\n{USAGE}")),
    };
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(3),
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
