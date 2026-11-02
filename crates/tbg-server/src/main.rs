//! `tbg-server` binary: read env config, bind, serve. On SIGTERM/Ctrl-C it
//! stops starting matches and exits once running matches, queued results
//! and reconnect result windows are done (P66 BR-UP06: no time limit; a
//! stuck drain is reported, never cut).

use std::sync::atomic::Ordering::Relaxed;
use std::time::{Duration, Instant};

use tbg_server::release::Runtime;
use tbg_server::{Config, Shared};

/// How often a waiting drain logs what it still waits for.
const DRAIN_REPORT: Duration = Duration::from_secs(30);

fn main() -> std::process::ExitCode {
    if std::env::args().nth(1).as_deref() == Some("--build-info") {
        println!("{}", tbg_server::build_info::get());
        return std::process::ExitCode::SUCCESS;
    }
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
    rt.build().expect("tokio runtime").block_on(serve(config))
}

/// A server task that stops by itself is a crash: exiting non-zero lets systemd's `Restart=on-failure`
/// (the release slot units) bring the same generation back; a drained SIGTERM exit is a success.
async fn serve(config: Config) -> std::process::ExitCode {
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
            return std::process::ExitCode::FAILURE;
        }
        _ = shutdown_signal() => {}
    }
    let Some(release) = shared.release.get().cloned() else {
        return std::process::ExitCode::SUCCESS;
    };
    release.gate.shutdown.store(true, Relaxed);
    let started = Instant::now();
    let mut reported: Option<Instant> = None;
    loop {
        let load = Runtime::load(&shared.metrics, shared.store.get().map(|s| s.as_ref()));
        if load.matches_running == 0 && load.results_pending == 0 {
            break;
        }
        if reported.is_none_or(|t| t.elapsed() >= DRAIN_REPORT) {
            tracing::info!(
                running = load.matches_running,
                results_pending = load.results_pending,
                waited_s = started.elapsed().as_secs(),
                "shutdown: draining; running matches and results finish first"
            );
            reported = Some(Instant::now());
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    if let Some(store) = shared.store.get() {
        tbg_server::release::retire(&release, store).await;
    }
    tracing::info!(waited_s = started.elapsed().as_secs(), "shutdown: done");
    std::process::ExitCode::SUCCESS
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
