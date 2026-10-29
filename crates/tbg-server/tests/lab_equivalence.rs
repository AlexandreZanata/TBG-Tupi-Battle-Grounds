//! P16-C step 5 (D-33, contract C4): the real server, driven by its own
//! clock, plays exactly the match a headless replay of its applied
//! intentions produces. C4a runs the server at a 5 ms tick (the test
//! convention), C4b at the production 50 ms tick (1×, `--ignored`).
//! Equality at both speeds is what lets the balance lab run without a clock.

use std::time::Duration;

use tokio::net::TcpListener;
use tokio::sync::mpsc;

use tbg_core::replay::Replay;
use tbg_server::botclient::{play_one, BotOpts};
use tbg_server::Config;

async fn server(tick_ms: u64) -> (String, mpsc::UnboundedReceiver<Replay>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, rx) = mpsc::unbounded_channel();
    let config = Config {
        tick_ms,
        max_per_ip: 1_000,
        replays: Some(tx),
        ..Config::default()
    };
    tokio::spawn(tbg_server::run(config, listener));
    (format!("ws://{addr}/ws"), rx)
}

async fn server_equals_headless(tick_ms: u64) {
    let (url, mut replays) = server(tick_ms).await;
    let opts = BotOpts {
        timeout: Duration::from_secs(400),
        ..BotOpts::new(&url)
    };
    let (a, b) = tokio::join!(play_one(opts.clone(), 1), play_one(opts, 2));
    assert!(a.completed && b.completed, "{:?} / {:?}", a.error, b.error);
    let replay = tokio::time::timeout(Duration::from_secs(10), replays.recv())
        .await
        .expect("replay sent")
        .expect("channel open");
    assert!(
        replay.ops.iter().any(|op| op.intent.is_some()),
        "no intention was recorded"
    );
    let rebuilt = replay.run();
    assert_eq!(
        rebuilt.hash(),
        replay.final_hash,
        "headless replay diverged from the server"
    );
    assert_eq!(
        rebuilt.outcome, a.outcome,
        "outcome differs from what clients saw"
    );
    println!(
        "tick {tick_ms} ms: {} commands, end tick {}, outcome {:?}, hash {:016x}",
        replay.ops.len(),
        rebuilt.tick,
        rebuilt.outcome,
        replay.final_hash
    );
    // A wrong seed must not reproduce the match (the check has teeth).
    let wrong = Replay {
        seed: replay.seed.wrapping_add(1),
        ..replay
    };
    assert_ne!(wrong.run().hash(), wrong.final_hash);
}

#[tokio::test]
async fn c4a_fast_server_match_equals_headless_replay() {
    server_equals_headless(5).await;
}

#[tokio::test]
#[ignore = "a full 1x match takes 2-4 minutes; run once per balance phase and release"]
async fn c4b_realtime_server_match_equals_headless_replay() {
    server_equals_headless(50).await;
}
