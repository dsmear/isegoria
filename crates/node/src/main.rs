//! `isegoria-node <config.toml>`: runs a node until SIGTERM or Ctrl-C (`docs/04` §A node
//! process); the flags below are the operators' commands. Exit 2 for a configuration or input
//! that does not hold, 1 for a node that does not start.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn transport_public_key(path: &Path) -> Result<String, String> {
    let config = node::Config::load(path).map_err(|e| format!("configuration: {e:?}"))?;
    let key = config.data_dir.join("transport.key");
    node::transport_public_key(&config)
        .map(|public| node::hex(&public))
        .map_err(|e| format!("transport key {}: {e:?}", key.display()))
}

/// `--request <node.toml> <command> <n>`: spools an operator's closure request (`docs/04`).
fn request(path: &Path, args: &[PathBuf]) -> ExitCode {
    let args: Option<Vec<String>> = args.iter().map(|a| Some(a.to_str()?.to_string())).collect();
    let config = node::Config::load(path);
    let spooled = config.and_then(|c| node::answer::request(&c, &args.unwrap_or_default()));
    match spooled {
        Ok(name) => {
            println!("{name}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("request: {e:?}");
            ExitCode::from(2)
        }
    }
}

fn archive_command(args: &[PathBuf]) -> Result<String, node::production::Error> {
    use node::production::{self, Error};
    if let [flag, node, bundle] = args {
        if flag.as_os_str() == "--archive-install" {
            let done = node::install::install(&node::Config::load(node)?, bundle)?;
            let state = match done.installed {
                p2p::archive::Installed::Written => "installed",
                p2p::archive::Installed::Present => "present",
            };
            return Ok(format!(
                "{}\nshard {} {state}",
                node::hex(&done.digest),
                done.index
            ));
        }
    }
    if let [flag, node, input, output] = args {
        if flag.as_os_str() == "--archive-export-cut" {
            let node = node::Config::load(node)?;
            let input = node::extract::CutConfig::load(input)?;
            let cut = node::extract::export_cut(&node, &input, output)?;
            return Ok(node::hex(&cut.digest()));
        }
    }
    let [flag, node, input, rest @ ..] = args else {
        return Err(Error::Invalid("archive command arguments"));
    };
    let node = node::Config::load(node)?;
    let input = production::Config::load(input)?;
    let digest = match (flag.to_str(), rest) {
        (Some("--archive-extract"), []) => node::extract::extract(&node, &input).map(|c| c.0),
        (Some("--archive-prepare"), [output]) => production::prepare(&node, &input, output),
        (Some("--archive-sign"), [manifest, output]) => {
            production::sign(&node, &input, manifest, output)
        }
        (Some("--archive-combine"), [manifest, output, shares @ ..]) => {
            production::combine(&node, &input, manifest, shares, output)
        }
        _ => Err(Error::Invalid("archive command arguments")),
    }?;
    Ok(node::hex(&digest))
}

/// Installs the SIGTERM and Ctrl-C handlers now, inside a runtime; the future completes at
/// the first of them, also one received before it is first polled.
#[cfg(unix)]
fn stopped() -> std::io::Result<impl std::future::Future<Output = ()>> {
    use tokio::signal::unix::{signal, SignalKind};
    let mut term = signal(SignalKind::terminate())?;
    let mut interrupt = signal(SignalKind::interrupt())?;
    Ok(async move {
        tokio::select! {
            _ = term.recv() => {}
            _ = interrupt.recv() => {}
        }
    })
}

#[cfg(not(unix))]
fn stopped() -> std::io::Result<impl std::future::Future<Output = ()>> {
    Ok(async {
        let _ = tokio::signal::ctrl_c().await;
    })
}

fn main() -> ExitCode {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if args
        .first()
        .and_then(|a| a.to_str())
        .is_some_and(|a| a.starts_with("--archive-"))
    {
        return match archive_command(&args) {
            Ok(output) => {
                println!("{output}");
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
        [flag, dir] if flag.as_os_str() == "--new-identity" => {
            return match node::new_identity(dir) {
                Ok(keys) => {
                    println!("consortium {}", node::hex(&keys.consortium));
                    println!("circle {}", node::hex(&keys.circle));
                    println!("hpke {}", node::hex(&keys.hpke));
                    println!("transport {}", node::hex(&keys.transport));
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("identity {}: {e}", dir.display());
                    ExitCode::from(2)
                }
            };
        }
        [flag, path] if flag.as_os_str() == "--genesis-digest" => {
            return match node::genesis::load(path) {
                Ok(document) => {
                    print!("{}", node::genesis::render(&document));
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("genesis {}: {e:?}", path.display());
                    ExitCode::from(2)
                }
            };
        }
        [flag, input, output] if flag.as_os_str() == "--genesis-assemble" => {
            return match node::genesis::assemble(input, output) {
                Ok(document) => {
                    print!("{}", node::genesis::render(&document));
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("genesis {}: {e:?}", input.display());
                    ExitCode::from(2)
                }
            };
        }
        [flag, path] if flag.as_os_str() == "--transport-public-key" => {
            return match transport_public_key(path) {
                Ok(hex) => {
                    println!("{hex}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{e}");
                    ExitCode::from(2)
                }
            };
        }
        [flag, path, args @ ..] if flag.as_os_str() == "--request" => return request(path, args),
        [path] => path.clone(),
        _ => {
            eprintln!("usage: isegoria-node <config.toml> | --announce <config.toml>\n\
                --new-identity <data-dir>\n\
                --genesis-digest <genesis-file>\n\
                --genesis-assemble <genesis.toml> <genesis-file>\n\
                --transport-public-key <node.toml>\n\
                --request <node.toml> close-commits <question> | close <question> | close-epoch <epoch>\n\
                --archive-extract <node.toml> <production.toml>\n\
                --archive-export-cut <node.toml> <cut.toml> <cut-file>\n\
                --archive-prepare <node.toml> <production.toml> <bundle-dir>\n\
                --archive-sign <node.toml> <production.toml> <manifest.bin> <share.bin>\n\
                --archive-combine <node.toml> <production.toml> <manifest.bin> <signed-manifest.bin> <share.bin>...\n\
                --archive-install <node.toml> <bundle-dir>");
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
    let stop = {
        let _inside = runtime.enter();
        match stopped() {
            Ok(stop) => stop,
            Err(e) => {
                eprintln!("signal handlers: {e}");
                return ExitCode::FAILURE;
            }
        }
    };
    match runtime.block_on(node::run(plan, stop)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("start: {e:?}");
            ExitCode::FAILURE
        }
    }
}
