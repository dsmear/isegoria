//! `isegoria-ceremony`: the issuing committee's ceremony, one step per invocation (`docs/03`
//! §Committee setup for the trial, the files and commands).

use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str =
    "usage: isegoria-ceremony init <procedure.toml> <identity-dir> <ceremony-dir>\n\
    step <ceremony-dir> <identity-dir> [<message-file>...]\n\
    status <ceremony-dir>\n\
    answer <ceremony-dir> <request-file> <share-file>\n\
    abort <ceremony-dir>";

fn main() -> ExitCode {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let (command, rest) = match args.split_first() {
        Some((command, rest)) => (command.to_str().unwrap_or(""), rest),
        None => ("", &args[..]),
    };
    let result = match (command, rest) {
        ("init", [procedure, identity, dir]) => {
            ceremony::init(procedure, identity, dir).map(|r| r.render())
        }
        ("step", [dir, identity, files @ ..]) => {
            ceremony::step(dir, identity, files).map(|r| r.render())
        }
        ("status", [dir]) => ceremony::status(dir).map(|s| s.render()),
        ("answer", [dir, request, share]) => ceremony::answer(dir, request, share)
            .map(|slot| format!("answered {slot}\nshare {}\n", share.display())),
        ("abort", [dir]) => ceremony::abort(dir).map(|()| "aborted\n".to_string()),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(output) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("ceremony: {error:?}");
            ExitCode::from(2)
        }
    }
}
