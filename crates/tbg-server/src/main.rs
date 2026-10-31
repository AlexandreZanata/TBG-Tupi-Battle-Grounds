//! `tbg-server` binary: read env config, bind, serve. On SIGTERM/Ctrl-C it
//! stops starting matches and exits once running ones finish (max 300 s).

use std::sync::atomic::Ordering::Relaxed;
use std::time::{Duration, Instant};

use tbg_server::{Config, Shared};

const DRAIN_LIMIT: Duration = Duration::from_secs(300);

fn main() {
    let mut config = Config::from_env();
    // P60: an invalid integrity setting is a start-up error, never a default.
    let settings = tbg_server::ranked::policy::RankedPolicy::from_env()
        .and_then(|p| tbg_server::ranked::policy::StatsMode::from_env().map(|s| (p, s)));
    match settings {
        Ok((policy, stats)) => (config.ranked, config.stats) = (policy, stats),
        Err(e) => {
            eprintln!("tbg-server: {e}");
            std::process::exit(2);
        }
    }
    // P10: worker count from `TBG_WORKERS` (0 = all cores); the runtime is
    // built by hand instead of `#[tokio::main]` so the count applies.
    let mut rt = tokio::runtime::Builder::new_multi_thread();
    rt.enable_all();
    if config.workers > 0 {
        rt.worker_threads(config.workers);
    }
    rt.build().expect("tokio runtime").block_on(serve(config));
}

async fn serve(config: Config) {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    let listener = tokio::net::TcpListener::bind(&config.addr)
        .await
        .unwrap_or_else(|e| panic!("tbg-server: cannot bind {}: {e}", config.addr));
    tracing::info!(addr = %config.addr, ?config, "tbg-server listening");
    let shared = Shared::default();
    let server = tokio::spawn(tbg_server::run_with(config, listener, shared.clone()));
    tokio::select! {
        result = server => {
            tracing::error!("server stopped: {result:?}");
            return;
        }
        _ = shutdown_signal() => {}
    }
    shared.draining.store(true, Relaxed);
    let started = Instant::now();
    loop {
        let running = shared.metrics.matches_running.load(Relaxed);
        if running == 0 || started.elapsed() > DRAIN_LIMIT {
            tracing::info!(running, "shutdown: done");
            return;
        }
        tracing::info!(running, "shutdown: waiting for matches");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut term = signal(SignalKind::terminate()).expect("install SIGTERM handler");
        tokio::select! {
            _ = term.recv() => {}
            _ = tokio::signal::ctrl_c() => {}
        }
    }
    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
}
