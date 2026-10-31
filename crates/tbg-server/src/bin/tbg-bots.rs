//! Bot load/latency tool: 2N bot clients quick-match against a server over
//! real WebSockets and report completion, RTT, view gaps, reconnects.
//!
//! `tbg-bots --url ws://127.0.0.1:8080/ws --matches N [--delay-ms D]
//!   [--jitter-ms J] [--chaos-pct P] [--ramp-ms R] [--bots B] [--accounts]
//!   [--ranked] [--spectators S]`
//! `--spectators S` runs S anonymous spectators beside the bots (P47).
//! Exit code 1 if any bot failed.

use std::time::Duration;

use tbg_core::Pcg32;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use tbg_server::botclient::spectators;
use tbg_server::botclient::{play_one, BotOpts, BotResult};
use tbg_server::metrics::percentile;

struct Args {
    url: String,
    matches: u32,
    delay_ms: u64,
    jitter_ms: u64,
    chaos_pct: u32,
    ramp_ms: u64,
    /// Override the bot count (default 2 × matches), e.g. 1 to face a browser.
    bots: Option<u64>,
    /// P11: register a `Tb…` account per bot, then play logged in.
    accounts: bool,
    /// P12: ranked queue (implies `--accounts`).
    ranked: bool,
    /// Per-bot match deadline; 25-kind bot matches outlast the 360 s default (D-50).
    timeout_s: Option<u64>,
    /// P47: anonymous spectators watching listed matches while the bots play.
    spectators: u64,
}

fn args() -> Args {
    let mut a = Args {
        url: "ws://127.0.0.1:8080/ws".into(),
        matches: 1,
        delay_ms: 0,
        jitter_ms: 0,
        chaos_pct: 0,
        ramp_ms: 20,
        bots: None,
        accounts: false,
        ranked: false,
        timeout_s: None,
        spectators: 0,
    };
    let raw: Vec<String> = std::env::args().skip(1).collect();
    // Bare flags take no value; everything else comes in key/value pairs.
    let raw: Vec<String> = raw
        .into_iter()
        .filter(|arg| {
            if arg == "--accounts" {
                a.accounts = true;
                false
            } else if arg == "--ranked" {
                // Ranked play needs accounts; one flag turns both on.
                a.accounts = true;
                a.ranked = true;
                false
            } else {
                true
            }
        })
        .collect();
    for pair in raw.chunks(2) {
        let (Some(key), Some(value)) = (pair.first(), pair.get(1)) else {
            break;
        };
        let num = || {
            value
                .parse::<u64>()
                .unwrap_or_else(|_| panic!("{key} needs a number"))
        };
        match key.as_str() {
            "--url" => a.url = value.clone(),
            "--matches" => a.matches = num() as u32,
            "--delay-ms" => a.delay_ms = num(),
            "--jitter-ms" => a.jitter_ms = num(),
            "--chaos-pct" => a.chaos_pct = num() as u32,
            "--ramp-ms" => a.ramp_ms = num(),
            "--bots" => a.bots = Some(num()),
            "--timeout-s" => a.timeout_s = Some(num()),
            "--spectators" => a.spectators = num(),
            other => panic!("unknown argument {other}"),
        }
    }
    a
}

#[tokio::main]
async fn main() {
    // wss:// fails without a process-level crypto provider (rustls 0.23).
    let _ = rustls::crypto::ring::default_provider().install_default();
    let a = args();
    let mut rng = Pcg32::new(0xb075, 1);
    let mut tasks = Vec::new();
    let started = std::time::Instant::now();
    for i in 0..a.bots.unwrap_or(u64::from(a.matches) * 2) {
        let mut opts = BotOpts::new(a.url.clone());
        opts.delay_ms = a.delay_ms;
        opts.jitter_ms = a.jitter_ms;
        opts.accounts = a.accounts;
        opts.ranked = a.ranked;
        if let Some(s) = a.timeout_s {
            opts.timeout = Duration::from_secs(s);
        }
        if rng.below(100) < a.chaos_pct {
            opts.drop_after = Some(Duration::from_secs(u64::from(20 + rng.below(41))));
            opts.rejoin_after = Some(Duration::from_secs(5));
        }
        tasks.push(tokio::spawn(play_one(opts, i + 1)));
        tokio::time::sleep(Duration::from_millis(a.ramp_ms)).await;
    }
    // P47: spectators watch while the bots play, then stop with them.
    let stop = Arc::new(AtomicBool::new(false));
    let watchers = tokio::spawn(spectators::run(a.url.clone(), a.spectators, stop.clone()));
    let mut results = Vec::new();
    for t in tasks {
        results.push(t.await.unwrap_or_else(|e| BotResult {
            error: Some(format!("task: {e}")),
            ..Default::default()
        }));
    }
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    if a.spectators > 0 {
        let s = watchers.await.unwrap_or_default();
        println!(
            "spectators {} | watch starts {} | refused {} | views {} | ended {} | errors {} | bytes {}",
            a.spectators, s.watch_starts, s.refused, s.views, s.ended, s.errors, s.bytes
        );
    }
    let ok = report(&a, &results, started.elapsed());
    std::process::exit(if ok { 0 } else { 1 });
}

fn report(a: &Args, results: &[BotResult], elapsed: Duration) -> bool {
    let completed = results.iter().filter(|r| r.completed).count();
    let failed: Vec<&BotResult> = results.iter().filter(|r| !r.completed).collect();
    let mut rtt: Vec<u16> = results
        .iter()
        .flat_map(|r| r.rtt_ms.iter().copied())
        .collect();
    let mut gaps: Vec<u32> = results
        .iter()
        .flat_map(|r| r.view_gaps_ms.iter().copied())
        .collect();
    let mut ack: Vec<u16> = results
        .iter()
        .flat_map(|r| r.ack_ms.iter().copied())
        .collect();
    let mut queue: Vec<u16> = results
        .iter()
        .flat_map(|r| r.queue_wait_ms.iter().copied())
        .collect();
    rtt.sort_unstable();
    gaps.sort_unstable();
    ack.sort_unstable();
    queue.sort_unstable();
    let dropped = results.iter().filter(|r| r.dropped).count();
    let rejoined = results
        .iter()
        .filter(|r| r.dropped && r.reconnected && r.completed)
        .count();
    let mut reasons = std::collections::BTreeMap::new();
    for r in results.iter().filter_map(|r| r.outcome) {
        *reasons.entry(format!("{:?}", r.reason)).or_insert(0) += 1;
    }
    println!(
        "bots {} | completed {completed} | failed {} | elapsed {:.0}s",
        results.len(),
        failed.len(),
        elapsed.as_secs_f64()
    );
    println!("outcomes (per bot): {reasons:?}");
    println!(
        "rtt ms p50 {} p95 {} max {}",
        percentile(&rtt, 0.5),
        percentile(&rtt, 0.95),
        rtt.last().copied().unwrap_or(0)
    );
    println!(
        "view gap ms p50 {} p95 {} max {}",
        percentile(&gaps, 0.5),
        percentile(&gaps, 0.95),
        gaps.last().copied().unwrap_or(0)
    );
    println!(
        "ack ms p50 {} p95 {} max {}",
        percentile(&ack, 0.5),
        percentile(&ack, 0.95),
        ack.last().copied().unwrap_or(0)
    );
    println!(
        "queue wait ms p50 {} p95 {} max {}",
        percentile(&queue, 0.5),
        percentile(&queue, 0.95),
        queue.last().copied().unwrap_or(0)
    );
    println!("reconnects attempted {dropped} | succeeded {rejoined}");
    for f in failed.iter().take(5) {
        println!("failure: {:?}", f.error);
    }
    println!(
        "{{\"matches\":{},\"bots\":{},\"completed\":{completed},\"failed\":{},\"rtt_p50\":{},\"rtt_p95\":{},\"gap_p95\":{},\"gap_max\":{},\"ack_p50\":{},\"ack_p95\":{},\"queue_p95\":{},\"reconnects\":{dropped},\"reconnects_ok\":{rejoined},\"delay_ms\":{},\"jitter_ms\":{}}}",
        a.matches,
        results.len(),
        failed.len(),
        percentile(&rtt, 0.5),
        percentile(&rtt, 0.95),
        percentile(&gaps, 0.95),
        gaps.last().copied().unwrap_or(0),
        percentile(&ack, 0.5),
        percentile(&ack, 0.95),
        percentile(&queue, 0.95),
        a.delay_ms,
        a.jitter_ms
    );
    failed.is_empty()
}
