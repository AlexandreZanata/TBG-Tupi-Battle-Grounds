//! Integration tests over real WebSockets against an in-process server.
//! Matches run at a 5 ms tick so a full match takes seconds.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use tbg_core::protocol::{self, ClientMsg, ErrorCode, ServerMsg, PROTOCOL_VERSION};
use tbg_core::{EndReason, PlayerIntention, Rejection, Seat};
use tbg_server::botclient::{play_one, BotOpts};
use tbg_server::Config;

type Ws = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

async fn server(reconnect_secs: u64) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let config = Config {
        tick_ms: 5,
        reconnect_secs,
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

/// Next server message, answering pings and skipping `NetStats`;
/// `None` once the socket closes.
async fn recv(ws: &mut Ws) -> Option<ServerMsg> {
    loop {
        let msg = tokio::time::timeout(Duration::from_secs(10), ws.next())
            .await
            .ok()??
            .ok()?;
        let Message::Binary(b) = msg else { continue };
        match protocol::decode_server(&b).unwrap() {
            // Best-effort: the server may have closed already (e.g. after
            // a rate-limit error); a failed pong must not fail the test.
            ServerMsg::Ping { id } => {
                let _ = ws
                    .send(Message::Binary(
                        protocol::encode(&ClientMsg::Pong { id }).into(),
                    ))
                    .await;
            }
            // Latency housekeeping can arrive between any two replies.
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

fn fast(url: &str) -> BotOpts {
    BotOpts {
        timeout: Duration::from_secs(60),
        ..BotOpts::new(url)
    }
}

#[tokio::test]
async fn two_bots_finish_a_match() {
    let url = server(30).await;
    let (a, b) = tokio::join!(play_one(fast(&url), 1), play_one(fast(&url), 2));
    assert!(a.completed && b.completed, "{:?} / {:?}", a.error, b.error);
    assert_eq!(a.outcome, b.outcome, "both see the same result");
    assert_ne!(a.seat, b.seat);
    assert!(!a.rtt_ms.is_empty() || !b.rtt_ms.is_empty() || a.view_gaps_ms.len() > 10);
}

#[tokio::test]
async fn cannot_evolve_opponent_unit() {
    // BR-A02: the seat comes from the connection, not from the message.
    let url = server(30).await;
    let bot = tokio::spawn(play_one(fast(&url), 3));
    let mut ws = connect(&url).await;
    hello(&mut ws).await;
    send(&mut ws, &ClientMsg::QuickMatch).await;
    let mut you = None;
    while let Some(msg) = recv(&mut ws).await {
        match msg {
            ServerMsg::MatchStart { you: s, .. } => you = Some(s),
            ServerMsg::View(v) => {
                if let Some(enemy) = v.units.iter().find(|u| Some(u.seat) != you) {
                    let intent = PlayerIntention::Evolve {
                        unit: enemy.id,
                        branch: 0,
                    };
                    send(&mut ws, &ClientMsg::Intent { seq: 9, intent }).await;
                }
            }
            ServerMsg::Rejected { seq: 9, reason } => {
                assert_eq!(reason, Rejection::NotYourUnit);
                bot.abort();
                return;
            }
            _ => {}
        }
    }
    panic!("never rejected");
}

#[tokio::test]
async fn malformed_frame_closes_connection() {
    let url = server(30).await;
    let mut ws = connect(&url).await;
    hello(&mut ws).await;
    ws.send(Message::Binary(vec![0xff, 0x13, 0x07].into()))
        .await
        .unwrap();
    assert_eq!(
        recv(&mut ws).await,
        Some(ServerMsg::Error {
            code: ErrorCode::BadMessage
        })
    );
    assert_eq!(recv(&mut ws).await, None);
}

#[tokio::test]
async fn version_mismatch_rejected() {
    let url = server(30).await;
    let mut ws = connect(&url).await;
    send(
        &mut ws,
        &ClientMsg::Hello {
            version: 999,
            resume: None,
        },
    )
    .await;
    assert_eq!(
        recv(&mut ws).await,
        Some(ServerMsg::VersionMismatch {
            server: PROTOCOL_VERSION
        })
    );
}

#[tokio::test]
async fn rate_limit_closes_flooder() {
    let url = server(30).await;
    let mut ws = connect(&url).await;
    hello(&mut ws).await;
    for _ in 0..200 {
        if ws
            .send(Message::Binary(
                protocol::encode(&ClientMsg::Pong { id: 0 }).into(),
            ))
            .await
            .is_err()
        {
            break;
        }
    }
    let mut saw = false;
    while let Some(msg) = recv(&mut ws).await {
        saw |= msg
            == ServerMsg::Error {
                code: ErrorCode::RateLimited,
            };
    }
    assert!(saw, "flooder was not rate limited");
}

#[tokio::test]
async fn reconnect_resumes_match() {
    let url = server(30).await;
    let mut dropper = fast(&url);
    dropper.drop_after = Some(Duration::from_millis(300));
    dropper.rejoin_after = Some(Duration::from_secs(2));
    let (a, b) = tokio::join!(play_one(dropper, 4), play_one(fast(&url), 5));
    assert!(
        a.dropped && a.reconnected,
        "dropper did not reconnect: {:?}",
        a.error
    );
    assert!(a.completed && b.completed, "{:?} / {:?}", a.error, b.error);
    assert_ne!(b.outcome.map(|o| o.reason), Some(EndReason::Forfeit));
}

#[tokio::test]
async fn forfeit_after_window() {
    let url = server(1).await;
    let mut leaver = fast(&url);
    leaver.drop_after = Some(Duration::from_millis(300));
    let (a, b) = tokio::join!(play_one(leaver, 6), play_one(fast(&url), 7));
    assert!(a.dropped && !a.completed);
    let o = b.outcome.expect("stayer got a result");
    assert_eq!(o.reason, EndReason::Forfeit);
    assert_eq!(o.winner, b.seat);
    assert!(matches!(b.seat, Some(Seat::A | Seat::B)));
}

#[tokio::test]
async fn intent_is_acknowledged_next_tick() {
    // P9: the acting seat gets a view with `last_seq >= seq` (or a
    // rejection) within 2 ticks + local slack after its intention.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/ws", listener.local_addr().unwrap());
    let config = Config {
        tick_ms: 50,
        max_per_ip: 1_000,
        ..Config::default()
    };
    tokio::spawn(tbg_server::run(config, listener));
    let (mut a, mut b) = (connect(&url).await, connect(&url).await);
    hello(&mut a).await;
    hello(&mut b).await;
    send(&mut a, &ClientMsg::QuickMatch).await;
    send(&mut b, &ClientMsg::QuickMatch).await;
    // Wait for the first view so the shop offers exist server-side.
    let mut first_view = false;
    while !first_view {
        match recv(&mut a).await {
            Some(ServerMsg::View(_)) => first_view = true,
            Some(_) => {}
            None => panic!("socket closed before first view"),
        }
    }
    let seq = 7u32;
    let t0 = std::time::Instant::now();
    send(
        &mut a,
        &ClientMsg::Intent {
            seq,
            intent: PlayerIntention::Purchase { offer: 0 },
        },
    )
    .await;
    loop {
        match tokio::time::timeout(Duration::from_secs(10), recv(&mut a)).await {
            Ok(Some(ServerMsg::View(v))) if v.last_seq >= seq => break,
            Ok(Some(ServerMsg::Rejected { seq: s, .. })) if s == seq => break,
            Ok(Some(_)) => continue,
            _ => panic!("no ack view for seq {seq}"),
        }
    }
    let ms = t0.elapsed().as_millis();
    assert!(ms <= 500, "ack took {ms} ms (> 2 ticks + slack)");
}

#[tokio::test]
async fn protocol_pong_keeps_socket_alive() {
    // P9: a client that answers protocol pings but sends no app message
    // for 20 s (a hidden tab) is still connected (idle timeout is 15 s).
    let url = server(30).await;
    let (mut ws, _) = connect_async(url.as_str()).await.unwrap();
    send(
        &mut ws,
        &ClientMsg::Hello {
            version: PROTOCOL_VERSION,
            resume: None,
        },
    )
    .await;
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    loop {
        let msg = tokio::time::timeout(Duration::from_secs(25), async {
            use futures_util::StreamExt;
            ws.next().await
        })
        .await
        .expect("timed out waiting for server frames")
        .expect("stream ended")
        .expect("socket error");
        match msg {
            Message::Binary(b) => match protocol::decode_server(&b).unwrap() {
                ServerMsg::Welcome { .. } => {}
                ServerMsg::Ping { id } => {
                    send(&mut ws, &ClientMsg::Pong { id }).await;
                }
                _ => {}
            },
            Message::Ping(payload) => {
                use futures_util::SinkExt;
                ws.send(Message::Pong(payload)).await.unwrap();
            }
            Message::Close(_) => panic!("server closed the hidden-tab socket"),
            _ => {}
        }
        if std::time::Instant::now() >= deadline {
            break;
        }
    }
    // Still alive: the server answers a fresh ping round.
    send(&mut ws, &ClientMsg::QuickMatch).await;
    loop {
        let msg = tokio::time::timeout(Duration::from_secs(10), async {
            use futures_util::StreamExt;
            ws.next().await
        })
        .await
        .expect("no reply after 20 s of protocol-only activity")
        .expect("stream ended")
        .expect("socket error");
        if let Message::Binary(b) = msg {
            match protocol::decode_server(&b).unwrap() {
                ServerMsg::Queued => break,
                ServerMsg::Ping { id } => {
                    send(&mut ws, &ClientMsg::Pong { id }).await;
                }
                _ => continue,
            }
        }
    }
}

#[tokio::test]
async fn private_room_pairs_owner_and_guest() {
    let url = server(30).await;
    let (mut owner, mut guest) = (connect(&url).await, connect(&url).await);
    hello(&mut owner).await;
    hello(&mut guest).await;
    send(&mut guest, &ClientMsg::JoinRoom { code: *b"ZZZZZ" }).await;
    assert_eq!(recv(&mut guest).await, Some(ServerMsg::RoomNotFound));
    send(&mut owner, &ClientMsg::CreateRoom).await;
    let Some(ServerMsg::RoomCreated { code }) = recv(&mut owner).await else {
        panic!("no room code");
    };
    send(&mut guest, &ClientMsg::JoinRoom { code }).await;
    let starts = [recv(&mut owner).await, recv(&mut guest).await];
    let seats: Vec<Seat> = starts
        .iter()
        .filter_map(|m| match m {
            Some(ServerMsg::MatchStart { you, .. }) => Some(*you),
            _ => None,
        })
        .collect();
    assert_eq!(seats.len(), 2, "both got MatchStart: {starts:?}");
    assert_ne!(seats[0], seats[1]);
}

#[tokio::test]
async fn records_insert_and_count() {
    // P5: one row per finished match, with per-seat JSON.
    let dir = std::env::temp_dir().join(format!("tbg-records-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("test.sqlite3");
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/ws", listener.local_addr().unwrap());
    let config = Config {
        tick_ms: 5,
        max_per_ip: 1_000,
        db: Some(db.clone()),
        ..Config::default()
    };
    tokio::spawn(tbg_server::run(config, listener));
    let (a, b) = tokio::join!(play_one(fast(&url), 11), play_one(fast(&url), 12));
    assert!(a.completed && b.completed);
    tokio::time::sleep(Duration::from_millis(300)).await;
    let records = tbg_server::records::Records::open(&db).unwrap();
    assert_eq!(records.count().unwrap(), 1);
    let _ = std::fs::remove_dir_all(dir);
}

#[tokio::test]
async fn draining_server_refuses_new_matches() {
    // P5 graceful shutdown: no new matches once draining.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/ws", listener.local_addr().unwrap());
    let shared = tbg_server::Shared::default();
    shared
        .draining
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let config = Config {
        tick_ms: 5,
        ..Config::default()
    };
    tokio::spawn(tbg_server::run_with(config, listener, shared));
    let mut ws = connect(&url).await;
    hello(&mut ws).await;
    send(&mut ws, &ClientMsg::QuickMatch).await;
    assert_eq!(
        recv(&mut ws).await,
        Some(ServerMsg::Error {
            code: ErrorCode::ServerFull
        })
    );
}
