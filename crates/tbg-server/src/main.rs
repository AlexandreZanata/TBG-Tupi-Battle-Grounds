//! `tbg-server` binary: read env config, bind, serve. On SIGTERM/Ctrl-C it
//! stops starting matches and exits once running ones finish (max 300 s).

use std::sync::atomic::Ordering::Relaxed;
use std::time::{Duration, Instant};

use tbg_server::{Config, Shared};

const DRAIN_LIMIT: Duration = Duration::from_secs(300);

fn main() {
    let config = Config::from_env();
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
    maybe_profile();
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

// TEMPORARY (P10 profiling only, never committed): sample the server with
// pprof when TBG_PPROF="delay_s,duration_s" and write a text summary to
// TBG_PPROF_OUT. Bad input only logs; the server still starts.
fn maybe_profile() {
    let Ok(spec) = std::env::var("TBG_PPROF") else {
        return;
    };
    let Some((delay, dur)) = spec
        .split_once(',')
        .and_then(|(a, b)| Some((a.trim().parse().ok()?, b.trim().parse().ok()?)))
    else {
        tracing::error!("bad TBG_PPROF (want delay_s,duration_s), profiling off");
        return;
    };
    let (delay, dur): (u64, u64) = (delay, dur);
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(delay));
        let Ok(guard) = pprof::ProfilerGuardBuilder::default()
            .frequency(499)
            .build()
        else {
            tracing::error!("pprof guard failed, profiling off");
            return;
        };
        std::thread::sleep(Duration::from_secs(dur));
        let Ok(report) = guard.report().build() else {
            tracing::error!("pprof report failed");
            return;
        };
        let Ok(out_path) = std::env::var("TBG_PPROF_OUT") else {
            tracing::error!("TBG_PPROF_OUT not set, dropping profile");
            return;
        };
        let mut total = 0isize;
        let mut selfs: std::collections::HashMap<String, isize> = Default::default();
        let mut incl: std::collections::HashMap<String, isize> = Default::default();
        for (frames, n) in report.data.iter() {
            total += n;
            let names: Vec<String> = frames.frames.iter().flatten().map(|s| s.name()).collect();
            if let Some(first) = names.first() {
                *selfs.entry(first.clone()).or_default() += n;
            }
            let mut seen = std::collections::HashSet::new();
            for name in &names {
                if seen.insert(name.clone()) {
                    *incl.entry(name.clone()).or_default() += n;
                }
            }
        }
        let mut out = format!("total samples {total}\n== self ==\n");
        let mut v: Vec<_> = selfs.into_iter().collect();
        v.sort_by_key(|a| std::cmp::Reverse(a.1));
        for (k, n) in v.iter().take(15) {
            out += &format!("{:5.1}% {}\n", *n as f64 * 100.0 / total.max(1) as f64, k);
        }
        out += "== inclusive ==\n";
        let mut w: Vec<_> = incl.into_iter().collect();
        w.sort_by_key(|a| std::cmp::Reverse(a.1));
        for (k, n) in w.iter().take(15) {
            out += &format!("{:5.1}% {}\n", *n as f64 * 100.0 / total.max(1) as f64, k);
        }
        if let Err(e) = std::fs::write(&out_path, out) {
            tracing::error!("cannot write profile to {out_path}: {e}");
        }
    });
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
