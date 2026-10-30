//! P47 integration tests over real WebSockets (BR-LV01–BR-LV05, BR-LV09):
//! watching a listed match, its refusals, the watcher count the players see,
//! and a spectator that never reads.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use tbg_core::protocol::{
    self, ClientMsg, ErrorCode, MatchMode, ServerMsg, SpectateRefusal, PROTOCOL_VERSION,
};
use tbg_core::{Outcome, PlayerIntention};
use tbg_server::botclient::{play_one, BotOpts, BotResult};
use tbg_server::Config;

type Ws = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

async fn server_with(config: Config) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(tbg_server::run(config, listener));
    format!("ws://{addr}/ws")
}

async fn server(spectators_per_match: usize) -> String {
    server_with(Config {
        tick_ms: 5,
        reconnect_secs: 30,
        max_per_ip: 1_000,
        max_connections: 2_000,
        spectators_per_match,
        db: None,
        ..Config::default()
    })
    .await
}

fn http_base(ws_url: &str) -> String {
    let host = ws_url
        .strip_prefix("ws://")
        .and_then(|s| s.strip_suffix("/ws"))
        .unwrap();
    format!("http://{host}")
}

async fn connect(url: &str) -> Ws {
    connect_async(url).await.unwrap().0
}

async fn send(ws: &mut Ws, msg: &ClientMsg) {
    ws.send(Message::Binary(protocol::encode(msg).into()))
        .await
        .unwrap();
}

/// Next server message, answering pings; `None` once the socket closes.
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

/// The first message that matches `want`; everything before it is skipped.
async fn until(ws: &mut Ws, want: impl Fn(&ServerMsg) -> bool) -> ServerMsg {
    loop {
        let msg = recv(ws).await.expect("socket closed before the reply");
        if want(&msg) {
            return msg;
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

/// Raw HTTP/1.0 GET; returns (status, head, body).
async fn get(base: &str, path: &str) -> (u16, String, Vec<u8>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let host = base.strip_prefix("http://").unwrap();
    let mut stream = tokio::net::TcpStream::connect(host).await.unwrap();
    let req = format!("GET {path} HTTP/1.0\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).await.unwrap();
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await.unwrap();
    let head_end = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map_or(raw.len(), |i| i + 4);
    let head = String::from_utf8_lossy(&raw[..head_end]).into_owned();
    let status = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    (status, head, raw[head_end..].to_vec())
}

async fn live(base: &str) -> serde_json::Value {
    let (_, _, body) = get(base, "/api/live").await;
    serde_json::from_slice(&body).unwrap()
}

/// Poll `/api/live` until at least `want` matches are listed.
async fn wait_listed(base: &str, want: usize) -> serde_json::Value {
    for _ in 0..200 {
        let v = live(base).await;
        if v["matches"].as_array().is_some_and(|m| m.len() >= want) {
            return v;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("no listed match appeared");
}

fn parse_id(hex: &str) -> u64 {
    u64::from_str_radix(hex, 16).unwrap()
}

fn fast(url: &str) -> BotOpts {
    BotOpts {
        timeout: Duration::from_secs(120),
        ..BotOpts::new(url)
    }
}

fn spawn_bots(
    url: &str,
) -> (
    tokio::task::JoinHandle<BotResult>,
    tokio::task::JoinHandle<BotResult>,
) {
    (
        tokio::spawn(play_one(fast(url), 21)),
        tokio::spawn(play_one(fast(url), 22)),
    )
}

#[tokio::test]
async fn a_guest_watches_a_casual_bot_match_to_the_end() {
    let url = server(200).await;
    let base = http_base(&url);
    let (bot_a, bot_b) = spawn_bots(&url);
    let listed = wait_listed(&base, 1).await;
    let row = &listed["matches"][0];
    assert_eq!(row["mode"], "casual");
    assert_eq!(row["players"][0]["rating"], serde_json::Value::Null);

    let mut ws = connect(&url).await;
    hello(&mut ws).await;
    let id = parse_id(listed["featured"].as_str().unwrap());
    send(&mut ws, &ClientMsg::Spectate { id }).await;
    let start = until(&mut ws, |m| matches!(m, ServerMsg::SpectateStart { .. })).await;
    let ServerMsg::SpectateStart { arena, mode, .. } = start else {
        unreachable!()
    };
    assert_eq!(mode, MatchMode::Casual);
    assert_eq!(
        arena.slug(),
        row["arena"].as_str().unwrap(),
        "the watched arena"
    );
    assert!(matches!(
        until(&mut ws, |m| matches!(m, ServerMsg::SpectatorView(_))).await,
        ServerMsg::SpectatorView(_)
    ));

    // BR-LV02: a spectator holds no seat, so an intention is refused.
    send(
        &mut ws,
        &ClientMsg::Intent {
            seq: 1,
            intent: PlayerIntention::Purchase { offer: 0 },
        },
    )
    .await;
    until(&mut ws, |m| {
        matches!(
            m,
            ServerMsg::Error {
                code: ErrorCode::NotInMatch
            }
        )
    })
    .await;

    let end = until(&mut ws, |m| matches!(m, ServerMsg::SpectateEnd { .. })).await;
    assert!(matches!(
        end,
        ServerMsg::SpectateEnd {
            outcome: Some(Outcome { .. })
        }
    ));
    assert!(bot_a.await.unwrap().completed);
    assert!(bot_b.await.unwrap().completed);
}

#[tokio::test]
async fn private_rooms_are_never_listed_or_watchable() {
    let url = server(200).await;
    let base = http_base(&url);
    let mut owner = connect(&url).await;
    hello(&mut owner).await;
    send(&mut owner, &ClientMsg::CreateRoom).await;
    let Some(ServerMsg::RoomCreated { code }) = recv(&mut owner).await else {
        panic!("expected RoomCreated");
    };
    let mut guest = connect(&url).await;
    hello(&mut guest).await;
    send(&mut guest, &ClientMsg::JoinRoom { code }).await;
    until(&mut owner, |m| matches!(m, ServerMsg::MatchStart { .. })).await;

    // The snapshot publishes at most once a second: wait for the room's seats.
    let mut listed = live(&base).await;
    for _ in 0..100 {
        if listed["online"]["in_battle"] == 2 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
        listed = live(&base).await;
    }
    assert_eq!(
        listed["matches"].as_array().unwrap().len(),
        0,
        "rooms are private"
    );
    assert_eq!(
        listed["online"]["in_battle"], 2,
        "the room's seats still count"
    );

    let mut spectator = connect(&url).await;
    hello(&mut spectator).await;
    send(&mut spectator, &ClientMsg::Spectate { id: 0x1234 }).await;
    assert_eq!(
        recv(&mut spectator).await,
        Some(ServerMsg::SpectateUnavailable {
            reason: SpectateRefusal::NotFound
        })
    );
}

#[tokio::test]
async fn a_queued_player_cannot_watch_and_an_unknown_id_is_not_found() {
    let url = server(200).await;
    let mut queued = connect(&url).await;
    hello(&mut queued).await;
    send(&mut queued, &ClientMsg::QuickMatch).await;
    assert_eq!(recv(&mut queued).await, Some(ServerMsg::Queued));
    send(&mut queued, &ClientMsg::Spectate { id: 1 }).await;
    assert_eq!(
        recv(&mut queued).await,
        Some(ServerMsg::SpectateUnavailable {
            reason: SpectateRefusal::NotIdle
        })
    );

    let mut idle = connect(&url).await;
    hello(&mut idle).await;
    send(&mut idle, &ClientMsg::Spectate { id: 1 }).await;
    assert_eq!(
        recv(&mut idle).await,
        Some(ServerMsg::SpectateUnavailable {
            reason: SpectateRefusal::NotFound
        })
    );
}

#[tokio::test]
async fn a_full_match_refuses_the_next_spectator() {
    let url = server(1).await;
    let base = http_base(&url);
    let (bot_a, bot_b) = spawn_bots(&url);
    let listed = wait_listed(&base, 1).await;
    let id = parse_id(listed["featured"].as_str().unwrap());

    let mut first = connect(&url).await;
    hello(&mut first).await;
    send(&mut first, &ClientMsg::Spectate { id }).await;
    until(&mut first, |m| matches!(m, ServerMsg::SpectateStart { .. })).await;

    let mut second = connect(&url).await;
    hello(&mut second).await;
    send(&mut second, &ClientMsg::Spectate { id }).await;
    assert_eq!(
        recv(&mut second).await,
        Some(ServerMsg::SpectateUnavailable {
            reason: SpectateRefusal::Full
        })
    );
    drop((bot_a, bot_b));
}

#[tokio::test]
async fn players_see_the_watcher_count_and_the_match_arena() {
    let url = server(200).await;
    let mut p1 = connect(&url).await;
    let mut p2 = connect(&url).await;
    hello(&mut p1).await;
    hello(&mut p2).await;
    send(&mut p1, &ClientMsg::QuickMatch).await;
    send(&mut p2, &ClientMsg::QuickMatch).await;
    let ServerMsg::MatchStart { arena, .. } =
        until(&mut p1, |m| matches!(m, ServerMsg::MatchStart { .. })).await
    else {
        unreachable!()
    };
    let base = http_base(&url);
    let listed = wait_listed(&base, 1).await;
    let id = parse_id(listed["featured"].as_str().unwrap());

    let mut spectator = connect(&url).await;
    hello(&mut spectator).await;
    send(&mut spectator, &ClientMsg::Spectate { id }).await;
    let ServerMsg::SpectateStart { arena: watched, .. } = until(&mut spectator, |m| {
        matches!(m, ServerMsg::SpectateStart { .. })
    })
    .await
    else {
        unreachable!()
    };
    assert_eq!(watched, arena, "the spectator sees the seats' arena");
    let watching = until(
        &mut p1,
        |m| matches!(m, ServerMsg::View(v) if v.spectators == 1),
    )
    .await;
    assert!(matches!(watching, ServerMsg::View(_)));

    send(&mut spectator, &ClientMsg::StopSpectating).await;
    until(
        &mut p1,
        |m| matches!(m, ServerMsg::View(v) if v.spectators == 0),
    )
    .await;
}

#[tokio::test]
async fn a_spectator_that_never_reads_is_dropped_and_the_seats_finish() {
    let url = server(200).await;
    let base = http_base(&url);
    let (bot_a, bot_b) = spawn_bots(&url);
    let listed = wait_listed(&base, 1).await;
    let id = parse_id(listed["featured"].as_str().unwrap());

    // Subscribes, then never reads a byte: its queue must fill and be dropped.
    let mut deaf = connect(&url).await;
    hello(&mut deaf).await;
    send(&mut deaf, &ClientMsg::Spectate { id }).await;

    let a = bot_a.await.unwrap();
    let b = bot_b.await.unwrap();
    assert!(a.completed && b.completed, "both seats finish");
    assert!(
        !a.dropped && !b.dropped,
        "no seat is detached by a spectator"
    );
    drop(deaf);
}
