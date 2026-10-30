//! Arenas online (ARENAS-G, D-55): the server draws the arena at pairing,
//! both seats get the same one, a reconnect gets it again, and every new
//! match draws afresh. Clients have no message that names an arena.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use tbg_core::protocol::{self, ClientMsg, ServerMsg, PROTOCOL_VERSION};
use tbg_core::{Arena, Seat};
use tbg_server::Config;

type Ws = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

async fn server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let config = Config {
        tick_ms: 5,
        reconnect_secs: 30,
        max_per_ip: 1_000,
        ..Config::default()
    };
    tokio::spawn(tbg_server::run(config, listener));
    format!("ws://{addr}/ws")
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
        let Message::Binary(b) = msg else { continue };
        match protocol::decode_server(&b).unwrap() {
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

async fn hello(url: &str, resume: Option<[u8; 16]>) -> (Ws, [u8; 16]) {
    let mut ws = connect_async(url).await.unwrap().0;
    send(
        &mut ws,
        &ClientMsg::Hello {
            version: PROTOCOL_VERSION,
            resume,
        },
    )
    .await;
    match recv(&mut ws).await {
        Some(ServerMsg::Welcome { token, .. }) => (ws, token),
        other => panic!("expected Welcome, got {other:?}"),
    }
}

/// The arena and seat of the next `MatchStart`.
async fn start(ws: &mut Ws) -> (Arena, Seat) {
    loop {
        match recv(ws).await {
            Some(ServerMsg::MatchStart { arena, you, .. }) => return (arena, you),
            Some(_) => {}
            None => panic!("socket closed before MatchStart"),
        }
    }
}

#[tokio::test]
async fn both_seats_and_a_reconnect_see_the_drawn_arena() {
    let url = server().await;
    let (mut a, token) = hello(&url, None).await;
    let (mut b, _) = hello(&url, None).await;
    send(&mut a, &ClientMsg::QuickMatch).await;
    send(&mut b, &ClientMsg::QuickMatch).await;
    let (arena_a, seat_a) = start(&mut a).await;
    let (arena_b, seat_b) = start(&mut b).await;
    assert_eq!(arena_a, arena_b, "one arena for both seats");
    assert_ne!(seat_a, seat_b);
    // Drop A and resume: the same match, the same arena, no new draw.
    drop(a);
    tokio::time::sleep(Duration::from_millis(200)).await;
    let (mut again, _) = hello(&url, Some(token)).await;
    let (arena_again, seat_again) = start(&mut again).await;
    assert_eq!((arena_again, seat_again), (arena_a, seat_a));
}

#[tokio::test]
async fn every_new_match_draws_on_the_server() {
    // Twenty quick matches: the arena comes from the server alone and
    // varies between matches (five arenas, P(all equal) = 5^-19).
    let url = server().await;
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..20 {
        let (mut a, _) = hello(&url, None).await;
        let (mut b, _) = hello(&url, None).await;
        send(&mut a, &ClientMsg::QuickMatch).await;
        send(&mut b, &ClientMsg::QuickMatch).await;
        let (arena_a, _) = start(&mut a).await;
        let (arena_b, _) = start(&mut b).await;
        assert_eq!(arena_a, arena_b);
        seen.insert(arena_a.index());
    }
    assert!(seen.len() >= 2, "arenas drawn: {seen:?}");
}
