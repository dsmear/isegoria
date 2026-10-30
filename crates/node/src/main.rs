//! `isegoria-node <config.toml>`: runs a node until SIGTERM or Ctrl-C (`docs/04` §A node
//! process); `--announce <config.toml>` prints a member's signed key configuration. Exit 2 for
//! a configuration that does not hold, 1 for a node that does not start.

use std::path::PathBuf;
use std::process::ExitCode;

fn archive_command(args: &[PathBuf]) -> Result<[u8; 32], node::production::Error> {
    use node::production::{self, Error};
    let [flag, node, input, rest @ ..] = args else {
        return Err(Error::Invalid("archive command arguments"));
    };
    let node = node::Config::load(node)?;
    let input = production::Config::load(input)?;
    match (flag.to_str(), rest) {
        (Some("--archive-prepare"), [output]) => production::prepare(&node, &input, output),
        (Some("--archive-sign"), [manifest, output]) => {
            production::sign(&node, &input, manifest, output)
        }
        (Some("--archive-combine"), [manifest, output, shares @ ..]) => {
            production::combine(&node, &input, manifest, shares, output)
        }
        _ => Err(Error::Invalid("archive command arguments")),
    }
}

async fn stopped() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        if let Ok(mut term) = signal(SignalKind::terminate()) {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {}
                _ = term.recv() => {}
            }
            return;
        }
    }
    let _ = tokio::signal::ctrl_c().await;
}

fn main() -> ExitCode {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if args
        .first()
        .and_then(|a| a.to_str())
        .is_some_and(|a| a.starts_with("--archive-"))
    {
        return match archive_command(&args) {
            Ok(digest) => {
                println!("{}", node::hex(&digest));
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("archive: {error:?}");
                ExitCode::from(2)
            }
        };
    }
    let path = match args.as_slice() {
        [flag, path] if flag.as_os_str() == "--announce" => {
            return match node::Config::load(path).and_then(|c| node::announcement(&c)) {
                Ok(hex) => {
                    println!("{hex}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("configuration: {e:?}");
                    ExitCode::from(2)
                }
            };
        }
        [path] => path.clone(),
        _ => {
            eprintln!("usage: isegoria-node <config.toml> | --announce <config.toml>\n\
                --archive-prepare <node.toml> <production.toml> <bundle-dir>\n\
                --archive-sign <node.toml> <production.toml> <manifest.bin> <share.bin>\n\
                --archive-combine <node.toml> <production.toml> <manifest.bin> <signed-manifest.bin> <share.bin>...");
            return ExitCode::from(2);
        }
    };
    let plan = match node::Config::load(&path).and_then(node::plan) {
        Ok(plan) => plan,
        Err(e) => {
            eprintln!("configuration: {e:?}");
            return ExitCode::from(2);
        }
    };
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("runtime: {e}");
            return ExitCode::FAILURE;
        }
    };
    match runtime.block_on(node::run(plan, stopped())) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("start: {e:?}");
            ExitCode::FAILURE
        }
    }
}
