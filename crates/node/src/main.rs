//! `isegoria-node <config.toml>`: runs a node until SIGTERM or Ctrl-C (`docs/04` §A node
//! process). Exit 2 for a configuration that does not start, 1 for a node that does not.

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
    let Some(path) = std::env::args_os().nth(1).map(PathBuf::from) else {
        eprintln!("usage: isegoria-node <config.toml>");
        return ExitCode::from(2);
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
