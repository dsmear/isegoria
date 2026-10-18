//! The A2 probe's command line (`docs/19` §13.11): `plan`, `run` (the supervisor) and `one`
//! (a single run in its own process, launched by `run`).

use characterization::probe::{run, tasks, Case, Phase, Task};
use characterization::probe_jobs::{self_cost, supervise, write_part, Caps, Refusal, DECLARED};
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

const USAGE: &str = "\
usage: a2-probe <plan|run|one> [options]

  plan   the runs of a phase, in their order
  run    the runs of a phase, each in its own process, under the caps; records,
         ledger and parts go to <out>, and a stop or an unfinished run is never
         resumed automatically
  one    one run (used by run): its record to <out>/parts

options:
  --phase smoke|explore|confirm  (smoke)
  --replicates R                 (1)
  --case C0..C5 --replicate R    (one only)
  --jobs J                       (8)
  --out DIR                      (characterization-results/a2-probe)
  --max-cpu-s S --max-wall-s S --max-run-s S --max-rss-mib M
                                 cumulative caps, docs/19 §13.11's by default
";

struct Args {
    command: String,
    phase: Phase,
    replicates: u32,
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
        replicates: 1,
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
            "--replicates" => args.replicates = number(&value)? as u32,
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

fn supervised(args: &Args) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let all = tasks(args.phase, args.replicates);
    let launch = |task: &Task, out: &std::path::Path| {
        Command::new(&exe)
            .args([
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
            .stdout(Stdio::null())
            .spawn()
    };
    eprintln!(
        "{} runs planned; {} workers; caps {:?}; out {}",
        all.len(),
        args.jobs,
        args.caps,
        args.out.display()
    );
    match supervise(&all, args.jobs, &args.out, args.caps, launch) {
        Ok(Ok(session)) => {
            eprintln!(
                "outcomes {:?}, {} already ended; session CPU {} ms, wall {} ms, \
                 largest sampled resident sum {} KiB; stop: {}",
                session.outcomes,
                session.skipped,
                session.cpu_ms,
                session.wall_ms,
                session.peak_rss_kib,
                session.stop.as_deref().unwrap_or("none")
            );
            Ok(())
        }
        Ok(Err(Refusal::Stopped(reason))) => {
            Err(format!("refused: the ledger records a stop ({reason})"))
        }
        Ok(Err(Refusal::Unfinished(keys))) => {
            Err(format!("refused: runs started, never ended: {keys:?}"))
        }
        Ok(Err(Refusal::Spent)) => Err("refused: the cumulative caps are spent".into()),
        Err(e) => Err(e.to_string()),
    }
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
        "plan" => {
            for task in tasks(args.phase, args.replicates) {
                println!("{} seed {}", task.key(), task.seed());
            }
            Ok(())
        }
        "run" => supervised(&args),
        "one" => one(&args),
        _ => Err(format!("unknown command\n\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
