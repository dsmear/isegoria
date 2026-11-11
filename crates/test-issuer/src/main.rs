//! `isegoria-test-issuer`: the test issuer's commands and a person's side of issuance
//! (`docs/05` §Distributed integration, credentials before T19). Exit 2 for what does not hold.

use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;
use test_issuer::{finalize, hex, issue, issuer_from_hex, new_key, request, Error};

const PROVISIONAL: &str = "isegoria-test-issuer: provisional, for tests only; \
    no uniqueness, no limit on issuance (docs/05, docs/01 D65)";

const USAGE: &str = "usage: isegoria-test-issuer --new-key <issuer.key>\n\
    --issue <issuer.key> <request-file> <reply-file>\n\
    --request <issuer-hex> <slot> <pending-file> <request-file>\n\
    --finalize <issuer-hex> <pending-file> <reply-file> <credential-file>";

fn text(arg: &OsString) -> Result<&str, Error> {
    arg.to_str().ok_or(Error::Malformed("argument"))
}

fn slot(arg: &OsString) -> Result<u64, Error> {
    text(arg)?.parse().map_err(|_| Error::Malformed("slot"))
}

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
        ("--request", [issuer, number, pending, message]) => (|| {
            let (issuer, number) = (issuer_from_hex(text(issuer)?)?, slot(number)?);
            request(&issuer, number, &path(pending), &path(message))?;
            Ok(format!("slot {number}\n"))
        })(),
        ("--finalize", [issuer, pending, reply, credential]) => (|| {
            let issuer = issuer_from_hex(text(issuer)?)?;
            finalize(&issuer, &path(pending), &path(reply), &path(credential))?;
            Ok(String::new())
        })(),
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
