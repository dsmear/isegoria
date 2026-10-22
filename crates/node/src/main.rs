//! `isegoria-node <config.toml>`: runs a node until SIGTERM or Ctrl-C (`docs/04` §A node
//! process); `--announce <config.toml>` prints a member's signed key configuration. Exit 2 for
//! a configuration that does not hold, 1 for a node that does not start.

use std::path::PathBuf;
use std::process::ExitCode;

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
            eprintln!("usage: isegoria-node <config.toml> | --announce <config.toml>");
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
