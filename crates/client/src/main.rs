//! `isegoria-client`: a participant's commands (`docs/05` §Distributed integration, §Built for
//! participants; `docs/03` T19-P5). Exit 2 for what does not hold.

use client::credential::{combine, finalize, read_credential, request, slot_request};
use client::event::{self, Commitment};
use client::read::{self, Ask};
use client::{hex, issuer_from_hex, Error, Network, Output};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: isegoria-client\n\
    --request <issuer-hex> <slot> <pending-file> <request-file>\n\
    --finalize <issuer-hex> <pending-file> <reply-file> <credential-file>\n\
    --slot-request <genesis> <slot> <pending-file> <request-file>\n\
    --combine <genesis> <pending-file> <credential-file> <share-file>...\n\
    --join <genesis> <credential-file> <event-file>\n\
    --deposit <genesis> <credential-file> <epoch> <question.toml> <event-file>\n\
    --commit <genesis> <credential-file> <question> <answer> <commitment-file> <event-file>\n\
    --reveal <genesis> <credential-file> <commitment-file> <event-file>\n\
    --send <genesis> <announcement-hex> <relay-address> <event-file>\n\
    --read <genesis> <credential-file> <announcement-hex> <relay-address> \
    status <event-file> | reviewer <from> | author";

fn text(arg: &OsString) -> Result<&str, Error> {
    arg.to_str().ok_or(Error::Malformed("argument"))
}

fn number<T: std::str::FromStr + ToString>(arg: &OsString, what: &'static str) -> Result<T, Error> {
    let text = text(arg)?;
    (text.parse::<T>().ok())
        .filter(|n| n.to_string() == text)
        .ok_or(Error::Malformed(what))
}

fn path(arg: &OsString) -> PathBuf {
    Path::new(arg).to_path_buf()
}

/// The network and the person's credential verified under its issuer key.
fn holder(
    genesis: &OsString,
    credential: &OsString,
) -> Result<(Network, identity::credential::AnonymousCredential), Error> {
    let net = Network::load(&path(genesis))?;
    let cred = read_credential(&path(credential), &net.issuer)?;
    Ok((net, cred))
}

fn runtime() -> Result<tokio::runtime::Runtime, Error> {
    let built = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();
    built.map_err(|e| Error::Io("runtime", e.kind()))
}

fn written(id: network::cid::Cid) -> String {
    format!("event {}\n", hex(&id.0))
}

fn credentials(flag: &str, rest: &[OsString]) -> Option<Result<String, Error>> {
    Some(match (flag, rest) {
        ("--request", [issuer, slot, pending, message]) => (|| {
            let (issuer, slot) = (issuer_from_hex(text(issuer)?)?, number(slot, "slot")?);
            request(&issuer, slot, &path(pending), &path(message))?;
            Ok(format!("slot {slot}\n"))
        })(),
        ("--finalize", [issuer, pending, reply, credential]) => (|| {
            let issuer = issuer_from_hex(text(issuer)?)?;
            finalize(&issuer, &path(pending), &path(reply), &path(credential))?;
            Ok(String::new())
        })(),
        ("--slot-request", [genesis, slot, pending, message]) => (|| {
            let (net, slot) = (Network::load(&path(genesis))?, number(slot, "slot")?);
            slot_request(&net.issuer, slot, &path(pending), &path(message))?;
            Ok(format!("slot {slot}\n"))
        })(),
        ("--combine", [genesis, pending, credential, shares @ ..]) if !shares.is_empty() => {
            (|| {
                let net = Network::load(&path(genesis))?;
                let shares: Vec<PathBuf> = shares.iter().map(path).collect();
                combine(&net.issuer, &path(pending), &shares, &path(credential))?;
                Ok(String::new())
            })()
        }
        _ => return None,
    })
}

fn events(flag: &str, rest: &[OsString]) -> Option<Result<String, Error>> {
    Some(match (flag, rest) {
        ("--join", [genesis, credential, out]) => (|| {
            let (net, cred) = holder(genesis, credential)?;
            let bytes = event::join(&net, &cred)?;
            event::write_event(&net, &bytes, &path(out), None).map(written)
        })(),
        ("--deposit", [genesis, credential, epoch, question, out]) => (|| {
            let (net, cred) = holder(genesis, credential)?;
            let epoch = number(epoch, "epoch")?;
            let question = event::question_file(&path(question))?;
            let bytes = event::deposit(&net, &cred, epoch, question)?;
            event::write_event(&net, &bytes, &path(out), None).map(written)
        })(),
        ("--commit", [genesis, credential, question, answer, opened, out]) => (|| {
            let (net, cred) = holder(genesis, credential)?;
            let question = number(question, "question")?;
            let answer = event::answer_of(text(answer)?).ok_or(Error::Malformed("answer"))?;
            let (bytes, kept) = event::commit(&net, &cred, question, answer)?;
            let (kept, opened) = (kept.encode(), path(opened));
            let also = Output(&opened, &kept, true, "commitment");
            event::write_event(&net, &bytes, &path(out), Some(also)).map(written)
        })(),
        ("--reveal", [genesis, credential, opened, out]) => (|| {
            let (net, cred) = holder(genesis, credential)?;
            let opened: Commitment = event::read_commitment(&path(opened))?;
            let bytes = event::reveal(&net, &cred, &opened)?;
            event::write_event(&net, &bytes, &path(out), None).map(written)
        })(),
        ("--send", [genesis, announcement, relay, file]) => (|| {
            let net = Network::load(&path(genesis))?;
            let key = event::announcement(&net, text(announcement)?)?;
            let relay = event::relay(&net, text(relay)?)?;
            let bytes = event::read_event(&path(file))?;
            let answer = runtime()?.block_on(event::send(&net, &key, relay, &bytes))?;
            Ok(event::render(&answer))
        })(),
        _ => return None,
    })
}

fn ask(rest: &[OsString]) -> Option<Result<Ask, Error>> {
    let (word, rest) = rest.split_first()?;
    Some(match (word.to_str()?, rest) {
        ("status", [file]) => event::read_event(&path(file)).map(Ask::Status),
        ("reviewer", [from]) => number(from, "from").map(|from| Ask::Reviewer { from }),
        ("author", []) => Ok(Ask::Author),
        _ => return None,
    })
}

fn reads(flag: &str, rest: &[OsString]) -> Option<Result<String, Error>> {
    let ("--read", [genesis, credential, announcement, relay, asked @ ..]) = (flag, rest) else {
        return None;
    };
    let asked = ask(asked)?;
    Some((|| {
        let asked = asked?;
        let (net, cred) = holder(genesis, credential)?;
        let key = event::announcement(&net, text(announcement)?)?;
        let relay = event::relay(&net, text(relay)?)?;
        let reply = runtime()?.block_on(read::read(&net, &cred, &key, relay, &asked))?;
        Ok(read::render(&reply))
    })())
}

/// The command's output, or `None` for arguments that name no command.
fn run(args: &[OsString]) -> Option<Result<String, Error>> {
    let (flag, rest) = args.split_first()?;
    let flag = flag.to_str()?;
    credentials(flag, rest)
        .or_else(|| events(flag, rest))
        .or_else(|| reads(flag, rest))
}

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    match run(&args) {
        Some(Ok(output)) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Some(Err(error)) => {
            eprintln!("client: {error:?}");
            ExitCode::from(2)
        }
        None => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}
