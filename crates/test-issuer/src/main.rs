//! `isegoria-test-issuer`: the provisional test issuer's commands; a person runs its side in
//! `isegoria-client` (`docs/05` §Distributed integration, `docs/01` D65). Exit 2 on a refusal.

use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;
use test_issuer::{hex, issue, new_key, Error};

const PROVISIONAL: &str = "isegoria-test-issuer: provisional, for tests only; \
    no uniqueness, no limit on issuance (docs/05, docs/01 D65)";

const USAGE: &str = "usage: isegoria-test-issuer --new-key <issuer.key>\n\
    --issue <issuer.key> <request-file> <reply-file>\n\
    (a person's --request and --finalize are isegoria-client's)";

/// The command's output, or `None` for arguments that name no command.
fn run(args: &[OsString]) -> Option<Result<String, Error>> {
    let path = |arg: &OsString| Path::new(arg).to_path_buf();
    let (flag, rest) = args.split_first()?;
    Some(match (flag.to_str()?, rest) {
        ("--new-key", [key]) => {
            new_key(&path(key)).map(|k| format!("issuer {}\n", hex(&k.encode())))
        }
        ("--issue", [key, message, reply]) => {
            issue(&path(key), &path(message), &path(reply)).map(|s| format!("slot {s}\n"))
        }
        _ => return None,
    })
}

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    eprintln!("{PROVISIONAL}");
    match run(&args) {
        Some(Ok(output)) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Some(Err(error)) => {
            eprintln!("test issuer: {error:?}");
            ExitCode::from(2)
        }
        None => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}
