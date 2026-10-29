//! P39 lobby lifecycle tests: expired invites, bounded cleanup, and rematch
//! notification without starting a match task.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;

use tbg_core::protocol::ServerMsg;

use super::{Lobby, RematchTicket, Session, SessionState};
use crate::chan::{ConnEvent, ConnHandle, LobbyCmd};
use crate::limits::Limits;
use crate::metrics::Metrics;
use crate::Config;

fn lobby() -> (Lobby, mpsc::Receiver<LobbyCmd>) {
    let (self_tx, self_rx) = mpsc::channel(8);
    (
        Lobby {
            sessions: HashMap::new(),
            queue: VecDeque::new(),
            ranked_queue: VecDeque::new(),
            queued_at: HashMap::new(),
            rooms: Vec::new(),
            challenges: Vec::new(),
            rematches: Vec::new(),
            self_tx,
            config: Arc::new(Config::default()),
            metrics: Arc::new(Metrics::default()),
            store: None,
            limits: Limits::default(),
            draining: Arc::new(AtomicBool::new(false)),
        },
        self_rx,
    )
}

fn add_session(
    lobby: &mut Lobby,
    token: [u8; 16],
    state: SessionState,
) -> mpsc::Receiver<ConnEvent> {
    let (tx, rx) = mpsc::channel(8);
    lobby.sessions.insert(
        token,
        Session {
            token,
            name: "Tester".into(),
            conn: Some(ConnHandle { id: 1, tx }),
            state,
            account: None,
            ip: "127.0.0.1".parse().unwrap(),
            detached_at: None,
        },
    );
    rx
}

fn expect(rx: &mut mpsc::Receiver<ConnEvent>, msg: ServerMsg) {
    match rx.try_recv() {
        Ok(ConnEvent::Send(got)) => assert_eq!(got, msg),
        Ok(ConnEvent::EnterMatch { .. }) => panic!("expected {msg:?}, got EnterMatch"),
        Ok(ConnEvent::LeaveMatch) => panic!("expected {msg:?}, got LeaveMatch"),
        Ok(ConnEvent::Close) => panic!("expected {msg:?}, got Close"),
        Err(error) => panic!("expected {msg:?}, got channel error {error:?}"),
    }
}

#[test]
fn cleanup_removes_expired_challenge_and_resets_owner() {
    let (mut lobby, _self_rx) = lobby();
    let invite = [7; 16];
    let owner = [1; 16];
    let mut owner_rx = add_session(
        &mut lobby,
        owner,
        SessionState::InChallenge { token: invite },
    );
    lobby
        .challenges
        .push((invite, owner, Instant::now() - Duration::from_secs(601)));

    lobby.cleanup();

    assert!(lobby.challenges.is_empty());
    assert!(matches!(
        lobby.sessions.get(&owner).map(|s| &s.state),
        Some(SessionState::Idle)
    ));
    expect(&mut owner_rx, ServerMsg::ChallengeUnavailable);
}

#[test]
fn cleanup_removes_many_expired_challenges_in_one_sweep() {
    let (mut lobby, _self_rx) = lobby();
    let owner = [1; 16];
    let _owner_rx = add_session(
        &mut lobby,
        owner,
        SessionState::InChallenge { token: [0; 16] },
    );
    for i in 0..100u8 {
        lobby
            .challenges
            .push(([i; 16], owner, Instant::now() - Duration::from_secs(601)));
    }

    lobby.cleanup();

    assert!(lobby.challenges.is_empty(), "expired invites must not grow");
}

#[test]
fn cleanup_expired_rematch_notifies_both_players() {
    let (mut lobby, _self_rx) = lobby();
    let a = [1; 16];
    let b = [2; 16];
    let mut a_rx = add_session(&mut lobby, a, SessionState::Idle);
    let mut b_rx = add_session(&mut lobby, b, SessionState::Idle);
    lobby.rematches.push(RematchTicket {
        players: [a, b],
        requested: [true, false],
        expires: Some(Instant::now() - Duration::from_secs(1)),
    });

    lobby.cleanup();

    assert!(lobby.rematches.is_empty());
    expect(&mut a_rx, ServerMsg::RematchClosed { declined: false });
    expect(&mut b_rx, ServerMsg::RematchClosed { declined: false });
}

#[test]
fn cleanup_rematch_with_missing_session_notifies_remaining_player() {
    let (mut lobby, _self_rx) = lobby();
    let a = [1; 16];
    let missing = [2; 16];
    let mut a_rx = add_session(&mut lobby, a, SessionState::Idle);
    lobby.rematches.push(RematchTicket {
        players: [a, missing],
        requested: [false, true],
        expires: Some(Instant::now() + Duration::from_secs(60)),
    });

    lobby.cleanup();

    assert!(lobby.rematches.is_empty());
    expect(&mut a_rx, ServerMsg::RematchClosed { declined: false });
}

#[test]
fn login_takeover_moves_the_rematch_token() {
    let (mut lobby, _self_rx) = lobby();
    let old = [1; 16];
    let new = [3; 16];
    let other = [2; 16];
    lobby.rematches.push(RematchTicket {
        players: [old, other],
        requested: [false, false],
        expires: Some(Instant::now() + Duration::from_secs(60)),
    });

    lobby.replace_rematch_player(old, new);

    assert_eq!(lobby.rematches[0].players, [new, other]);
}
