//! Immutable business-rule contract for P39 Quick Challenge.
//! BR-R08: a direct, single-use invitation between two different sessions,
//! casual (no rating), no code entry, no login.
//! BR-R09: a rematch needs both requests, uses a fresh Challenge match, and
//! never enters a public queue.
//!
//! Append-only: change this file only in the same commit as the GAME-DESIGN
//! rule it guards, and name that rule in the commit message.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use tbg_core::protocol::{self, ClientMsg, InviteToken, MatchMode, ServerMsg, PROTOCOL_VERSION};
use tbg_core::Seat;
use tbg_server::Config;

type Ws = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

async fn server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let config = Config {
        tick_ms: 5,
        max_per_ip: 1_000,
        ..Config::default()
    };
    tokio::spawn(tbg_server::run(config, listener));
    format!("ws://{addr}/ws")
}

async fn connect(url: &str) -> Ws {
    connect_async(url).await.unwrap().0
}

async fn send(ws: &mut Ws, msg: &ClientMsg) {
    ws.send(Message::Binary(protocol::encode(msg).into()))
        .await
        .unwrap();
}

async fn recv(ws: &mut Ws) -> Option<ServerMsg> {
    loop {
        let msg = tokio::time::timeout(Duration::from_secs(10), ws.next())
            .await
            .ok()??
            .ok()?;
        let Message::Binary(bytes) = msg else {
            continue;
        };
        match protocol::decode_server(&bytes).unwrap() {
            ServerMsg::Ping { id } => {
                let _ = ws
                    .send(Message::Binary(
                        protocol::encode(&ClientMsg::Pong { id }).into(),
                    ))
                    .await;
            }
            ServerMsg::NetStats { .. } => {}
            other => return Some(other),
        }
    }
}

async fn hello(ws: &mut Ws) {
    send(
        ws,
        &ClientMsg::Hello {
            version: PROTOCOL_VERSION,
            resume: None,
        },
    )
    .await;
    assert!(matches!(recv(ws).await, Some(ServerMsg::Welcome { .. })));
}

async fn create_challenge(ws: &mut Ws) -> InviteToken {
    send(ws, &ClientMsg::CreateChallenge).await;
    match recv(ws).await {
        Some(ServerMsg::ChallengeWaiting { token }) => token,
        other => panic!("expected ChallengeWaiting, got {other:?}"),
    }
}

struct Start {
    seat: Seat,
    mode: MatchMode,
    you_name: String,
    opponent: String,
    you_rating: Option<u16>,
    opponent_rating: Option<u16>,
}

async fn start(ws: &mut Ws) -> Start {
    loop {
        match recv(ws).await {
            Some(ServerMsg::MatchStart {
                you,
                you_name,
                you_rating,
                opponent,
                mode,
                opponent_rating,
            }) => {
                return Start {
                    seat: you,
                    mode,
                    you_name,
                    opponent,
                    you_rating,
                    opponent_rating,
                }
            }
            Some(_) => {}
            None => panic!("socket closed before MatchStart"),
        }
    }
}

async fn rematch_open(ws: &mut Ws) {
    loop {
        match recv(ws).await {
            Some(ServerMsg::RematchOpen { .. }) => return,
            Some(_) => {}
            None => panic!("socket closed before RematchOpen"),
        }
    }
}

async fn rematch_state(ws: &mut Ws) -> (bool, bool) {
    loop {
        match recv(ws).await {
            Some(ServerMsg::RematchState {
                you_requested,
                opponent_requested,
            }) => return (you_requested, opponent_requested),
            Some(_) => {}
            None => panic!("socket closed before RematchState"),
        }
    }
}

async fn no_match_start(ws: &mut Ws, wait: Duration) {
    let deadline = tokio::time::Instant::now() + wait;
    loop {
        let left = deadline.saturating_duration_since(tokio::time::Instant::now());
        if left.is_zero() {
            return;
        }
        match tokio::time::timeout(left, recv(ws)).await {
            Ok(Some(ServerMsg::MatchStart { .. })) => panic!("unexpected MatchStart"),
            Ok(Some(_)) => continue,
            Ok(None) | Err(_) => return,
        }
    }
}

async fn end_by_surrender(ws: &mut Ws, other: &mut Ws) {
    send(ws, &ClientMsg::Surrender).await;
    loop {
        match recv(ws).await {
            Some(ServerMsg::MatchEnd { .. }) => break,
            Some(_) => {}
            None => panic!("socket closed before MatchEnd"),
        }
    }
    loop {
        match recv(other).await {
            Some(ServerMsg::MatchEnd { .. }) => break,
            Some(_) => {}
            None => panic!("socket closed before MatchEnd"),
        }
    }
}

#[tokio::test]
async fn br_r08_quick_challenge_is_single_use_and_casual() {
    let url = server().await;
    let mut owner = connect(&url).await;
    let mut guest = connect(&url).await;
    hello(&mut owner).await;
    hello(&mut guest).await;

    let invite = create_challenge(&mut owner).await;

    // The owner cannot occupy the guest seat.
    send(&mut owner, &ClientMsg::JoinChallenge { token: invite }).await;
    assert_eq!(
        recv(&mut owner).await,
        Some(ServerMsg::ChallengeUnavailable)
    );

    // One different session joins with no code entry and no account.
    send(&mut guest, &ClientMsg::JoinChallenge { token: invite }).await;
    let owner_start = start(&mut owner).await;
    let guest_start = start(&mut guest).await;
    assert_eq!(owner_start.mode, MatchMode::Challenge);
    assert_eq!(guest_start.mode, MatchMode::Challenge);
    assert_ne!(owner_start.seat, guest_start.seat);
    assert!(!owner_start.you_name.is_empty());
    assert!(!owner_start.opponent.is_empty());
    assert_eq!(
        (owner_start.you_rating, owner_start.opponent_rating),
        (None, None),
        "a Challenge is casual and never changes Rating"
    );

    // The invitation is single-use: a third session cannot interfere.
    let mut third = connect(&url).await;
    hello(&mut third).await;
    send(&mut third, &ClientMsg::JoinChallenge { token: invite }).await;
    assert_eq!(
        recv(&mut third).await,
        Some(ServerMsg::ChallengeUnavailable)
    );
}

#[tokio::test]
async fn br_r09_rematch_requires_both_and_never_enters_a_queue() {
    let url = server().await;
    let mut owner = connect(&url).await;
    let mut guest = connect(&url).await;
    hello(&mut owner).await;
    hello(&mut guest).await;
    let invite = create_challenge(&mut owner).await;
    send(&mut guest, &ClientMsg::JoinChallenge { token: invite }).await;
    let _ = start(&mut owner).await;
    let _ = start(&mut guest).await;
    end_by_surrender(&mut owner, &mut guest).await;
    rematch_open(&mut owner).await;
    rematch_open(&mut guest).await;

    // One request waits; it does not start a match.
    send(&mut owner, &ClientMsg::RequestRematch).await;
    assert_eq!(rematch_state(&mut owner).await, (true, false));
    assert_eq!(rematch_state(&mut guest).await, (false, true));

    // A third player entering the public queue is not paired with them.
    let mut third = connect(&url).await;
    hello(&mut third).await;
    send(&mut third, &ClientMsg::QuickMatch).await;
    assert_eq!(recv(&mut third).await, Some(ServerMsg::Queued));
    no_match_start(&mut owner, Duration::from_millis(100)).await;
    no_match_start(&mut guest, Duration::from_millis(100)).await;

    // Both requests start a fresh Challenge match, still outside the queue.
    send(&mut guest, &ClientMsg::RequestRematch).await;
    let a = start(&mut owner).await;
    let b = start(&mut guest).await;
    assert_eq!(a.mode, MatchMode::Challenge);
    assert_eq!(b.mode, MatchMode::Challenge);
    assert_ne!(a.seat, b.seat);
    no_match_start(&mut third, Duration::from_millis(100)).await;
}
