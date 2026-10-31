//! Reconnect-window contracts for seats already detached at match creation.

use super::*;
use crate::chan::ConnEvent;
use tbg_core::protocol::ServerMsg;
use tbg_core::{ArcanaPreset, EndReason, Outcome};

#[test]
fn online_sequence_zero_cannot_reset_duplicate_protection() {
    // ARCHITECTURE section 5: authoritative intentions are applied once;
    // offline seq=0 must never reset an online seat's acknowledgement.
    let players = Seat::BOTH.map(|seat| Player {
        token: [seat.index() as u8; 16],
        name: format!("Player {seat:?}"),
        conn: None,
        account: None,
        arcanas: ArcanaPreset::NEUTRAL,
        stronghold: tbg_core::StrongholdKind::Classic,
    });
    let mut state = MatchState::new(1);
    while !state.is_live() {
        state.step();
    }
    state.players[0].energy = 100_000;
    let mut game = Match {
        result_id: [63; 16],
        state,
        players,
        detached_at: [None; 2],
        stats: Default::default(),
        pending: Default::default(),
        applied: [false; 2],
        mode: MatchMode::Casual,
        metrics: Arc::new(Metrics::default()),
        replay: None,
        live: None,
        spectators: Vec::new(),
        pending_spectate: Vec::new(),
    };
    let command = |seq| MatchCmd::Intent {
        seat: Seat::A,
        seq,
        intent: tbg_core::PlayerIntention::Purchase { offer: 0 },
    };
    game.command(command(1));
    let after_first = game.state.clone();
    for sequence in [1, 0, 0, 1] {
        game.command(command(sequence));
        assert!(
            game.state == after_first,
            "replayed sequence {sequence} mutated state"
        );
    }
}

#[test]
fn a_view_drains_events_and_keeps_the_buffer() {
    // P30-D review: draining by `clear` keeps capacity for the next view;
    // a detached seat still loses its buffered events (they are
    // presentation-only and a reconnect gets the full state).
    let players = Seat::BOTH.map(|seat| Player {
        token: [seat.index() as u8; 16],
        name: format!("Player {seat:?}"),
        conn: None,
        account: None,
        arcanas: ArcanaPreset::NEUTRAL,
        stronghold: tbg_core::StrongholdKind::Classic,
    });
    let mut game = Match {
        result_id: [63; 16],
        state: MatchState::new(1),
        players,
        detached_at: [None; 2],
        stats: Default::default(),
        pending: Default::default(),
        applied: [false; 2],
        mode: MatchMode::Casual,
        metrics: Arc::new(Metrics::default()),
        replay: None,
        live: None,
        spectators: Vec::new(),
        pending_spectate: Vec::new(),
    };
    for _ in 0..40 {
        let events = game.state.step();
        game.pending[0].extend(events);
    }
    game.pending[0].push(tbg_core::Event::PhaseChanged {
        phase: tbg_core::Phase::Running,
    });
    let capacity = game.pending[0].capacity();
    game.send_view(Seat::A, false);
    assert!(game.pending[0].is_empty());
    assert_eq!(game.pending[0].capacity(), capacity);
}

#[test]
fn a_retransmitted_evolution_never_reuses_the_ruins_discount() {
    // D-55: the discount is spent once, on a valid paid Evolution; the
    // server refuses a repeated sequence before the core sees it.
    let players = Seat::BOTH.map(|seat| Player {
        token: [seat.index() as u8; 16],
        name: format!("Player {seat:?}"),
        conn: None,
        account: None,
        arcanas: ArcanaPreset::NEUTRAL,
        stronghold: tbg_core::StrongholdKind::Classic,
    });
    let mut state = MatchState::new(1).with_arena(tbg_core::Arena::Ruins);
    while !state.is_live() {
        state.step();
    }
    state.players[0].offers[0] = tbg_core::UnitKind::Shieldbearer;
    state.players[0].energy = 100_000;
    let mut game = Match {
        result_id: [63; 16],
        state,
        players,
        detached_at: [None; 2],
        stats: Default::default(),
        pending: Default::default(),
        applied: [false; 2],
        mode: MatchMode::Casual,
        metrics: Arc::new(Metrics::default()),
        replay: None,
        live: None,
        spectators: Vec::new(),
        pending_spectate: Vec::new(),
    };
    let intent = |seq, intent| MatchCmd::Intent {
        seat: Seat::A,
        seq,
        intent,
    };
    game.command(intent(1, tbg_core::PlayerIntention::Purchase { offer: 0 }));
    let unit = game.state.units.last().expect("bought").id;
    let evolve = tbg_core::PlayerIntention::Evolve { unit, branch: 0 };
    let before = game.state.players[0].energy;
    game.command(intent(2, evolve));
    let paid = before - game.state.players[0].energy;
    assert_eq!(
        paid,
        tbg_core::content::EVOLVE_COST[0] - tbg_core::content::ENERGY_SCALE
    );
    assert!(game.state.players[0].arena_discount_used);
    let after_first = game.state.clone();
    game.command(intent(2, evolve));
    assert!(
        game.state == after_first,
        "a repeated sequence changed the match"
    );
}

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
        (1, tbg_core::Arena::Forest, None),
        players,
        MatchMode::Challenge,
        lobby_tx,
        config,
        metrics.clone(),
        None,
    );
    let outcome = tokio::time::timeout(Duration::from_secs(3), async {
        while let Some(ConnEvent::Send(msg)) = conn_rx.recv().await {
            if let ServerMsg::MatchEnd { outcome, .. } = msg {
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
