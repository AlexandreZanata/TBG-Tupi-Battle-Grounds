//! Reconnect-window contracts for seats already detached at match creation.

use super::*;
use crate::chan::ConnEvent;
use tbg_core::{ArcanaPreset, EndReason, Outcome};

#[tokio::test]
async fn challenge_initial_detached_forfeits() {
    // BR-M03/BR-R08: a disconnect racing pairing gets the same window as
    // any later disconnect; an absent seat cannot run forever.
    let (conn_tx, mut conn_rx) = mpsc::channel(256);
    let players = Seat::BOTH.map(|seat| Player {
        token: [seat.index() as u8; 16],
        name: format!("Player {seat:?}"),
        conn: (seat == Seat::B).then(|| ConnHandle {
            id: 1,
            tx: conn_tx.clone(),
        }),
        account: None,
        arcanas: ArcanaPreset::DEFAULT,
        stronghold: tbg_core::StrongholdKind::DEFAULT,
    });
    let (lobby_tx, mut lobby_rx) = mpsc::channel(8);
    let config = Arc::new(Config {
        tick_ms: 5,
        reconnect_secs: 1,
        ..Config::default()
    });
    let metrics = Arc::new(Metrics::default());
    let started = Instant::now();
    let _match_tx = spawn(
        1,
        players,
        MatchMode::Challenge,
        lobby_tx,
        config,
        metrics.clone(),
        None,
    );
    let outcome = tokio::time::timeout(Duration::from_secs(3), async {
        while let Some(ConnEvent::Send(msg)) = conn_rx.recv().await {
            if let ServerMsg::MatchEnd { outcome } = msg {
                return outcome;
            }
        }
        panic!("connected seat never received MatchEnd");
    })
    .await
    .unwrap();
    assert!(
        started.elapsed() >= Duration::from_secs(1),
        "the reconnect window must be honored"
    );
    assert_eq!(
        outcome,
        Outcome {
            winner: Some(Seat::B),
            reason: EndReason::Forfeit
        }
    );
    assert!(matches!(
        lobby_rx.recv().await,
        Some(LobbyCmd::MatchEnded { .. })
    ));
    assert_eq!(metrics.matches_running.load(Relaxed), 0);
}
