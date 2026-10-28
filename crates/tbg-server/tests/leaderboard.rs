//! Read-only ranking API (GAME-DESIGN BR-R07, BR-P05). Each test names
//! the rule it protects. HTTP goes over raw TCP (no client dependency).

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tbg_core::protocol::{self, ClientMsg, ServerMsg, PROTOCOL_VERSION};
use tbg_server::botclient::{play_one, BotOpts};
use tbg_server::Config;

type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

struct TempDb {
    _dir: std::path::PathBuf,
    db: std::path::PathBuf,
}

fn temp_db(name: &str) -> TempDb {
    let dir = std::env::temp_dir().join(format!("tbg-leaderboard-{}-{}", std::process::id(), name));
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("test.sqlite3");
    let _ = std::fs::remove_file(&db);
    TempDb { _dir: dir, db }
}

async fn live_server(name: &str) -> (String, String, TempDb) {
    let t = temp_db(name);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let url = format!("ws://127.0.0.1:{port}/ws");
    let base = format!("http://127.0.0.1:{port}");
    let config = Config {
        tick_ms: 1,
        reconnect_secs: 30,
        max_per_ip: 1_000,
        // Bot fills register once per connection; the limit counts attempts.
        register_per_ip_hour: 100_000,
        db: Some(t.db.clone()),
        ..Config::default()
    };
    tokio::spawn(tbg_server::run(config, listener));
    (base, url, t)
}

async fn connect(url: &str) -> Ws {
    tokio_tungstenite::connect_async(url).await.unwrap().0
}

async fn send(ws: &mut Ws, msg: &ClientMsg) {
    ws.send(tokio_tungstenite::tungstenite::Message::Binary(
        protocol::encode(msg).into(),
    ))
    .await
    .unwrap();
}

async fn recv(ws: &mut Ws) -> Option<ServerMsg> {
    loop {
        let msg = tokio::time::timeout(Duration::from_secs(30), ws.next())
            .await
            .ok()??
            .ok()?;
        let tokio_tungstenite::tungstenite::Message::Binary(b) = msg else {
            continue;
        };
        match protocol::decode_server(&b).unwrap() {
            ServerMsg::Ping { id } => send(ws, &ClientMsg::Pong { id }).await,
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

async fn register_as(ws: &mut Ws, name: &str) -> String {
    send(ws, &ClientMsg::Register { name: name.into() }).await;
    match recv(ws).await {
        Some(ServerMsg::Registered { key, .. }) => key,
        other => panic!("expected Registered, got {other:?}"),
    }
}

/// Minimal HTTP/1.0 GET over TCP. Returns (status, headers, body).
async fn get(base: &str, path: &str) -> (u16, String, Vec<u8>) {
    let host = base.strip_prefix("http://").unwrap();
    let mut stream = tokio::net::TcpStream::connect(host).await.unwrap();
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let req = format!("GET {path} HTTP/1.0\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).await.unwrap();
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await.unwrap();
    let head_end = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|i| i + 4)
        .unwrap_or(raw.len());
    let head = String::from_utf8_lossy(&raw[..head_end]).into_owned();
    let status: u16 = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    (status, head, raw[head_end..].to_vec())
}

fn body_json(body: &[u8]) -> serde_json::Value {
    serde_json::from_slice(body).unwrap()
}

/// Read one idle player's match from start to end.
async fn play_out(ws: &mut Ws) {
    loop {
        match recv(ws).await {
            Some(ServerMsg::MatchStart { .. }) => break,
            Some(_) => continue,
            None => panic!("ranked match never started"),
        }
    }
    loop {
        match recv(ws).await {
            Some(ServerMsg::MatchEnd { .. }) => break,
            Some(_) => continue,
            None => panic!("socket closed mid-match"),
        }
    }
}

/// Play one idle ranked draw between two fresh accounts; returns A's key.
async fn ranked_draw(url: &str, a_name: &str, b_name: &str) -> String {
    let mut a = connect(url).await;
    let mut b = connect(url).await;
    hello(&mut a).await;
    hello(&mut b).await;
    let key = register_as(&mut a, a_name).await;
    register_as(&mut b, b_name).await;
    send(&mut a, &ClientMsg::RankedMatch).await;
    send(&mut b, &ClientMsg::RankedMatch).await;
    // Drain both sockets at once: at 1 ms ticks an unread socket fills its
    // queue and the server drops it as a slow client (a forfeit, not a draw).
    tokio::join!(play_out(&mut a), play_out(&mut b));
    key
}

async fn wait_total(base: &str, want: u64) -> serde_json::Value {
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    loop {
        let (status, _, body) = get(base, "/api/leaderboard").await;
        assert_eq!(status, 200);
        let v = body_json(&body);
        if v["total"] == want || std::time::Instant::now() > deadline {
            return v;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

#[tokio::test]
async fn br_r07_leaderboard_and_player_profile() {
    let (base, url, _t) = live_server("board").await;
    ranked_draw(&url, "BoardA", "BoardB").await;
    // BR-R07: both appear ordered by rating, then wins, then name.
    let board = wait_total(&base, 2).await;
    assert_eq!(board["rows"][0]["name"], "BoardA");
    assert_eq!(board["rows"][0]["rank"], 1);
    assert_eq!(board["rows"][1]["name"], "BoardB");
    assert_eq!(board["rows"][1]["rank"], 2);
    assert!(!board["updated_at"].as_str().unwrap().is_empty());
    let (status, head, _) = get(&base, "/api/leaderboard").await;
    assert_eq!(status, 200);
    assert!(head.contains("max-age=10"), "cache headers: {head}");
    // Paging: limit and offset slice the same order.
    let (_, _, body) = get(&base, "/api/leaderboard?limit=1").await;
    let page = body_json(&body);
    assert_eq!(page["rows"].as_array().unwrap().len(), 1);
    assert_eq!(page["total"], 2);
    let (_, _, body) = get(&base, "/api/leaderboard?limit=1&offset=1").await;
    assert_eq!(body_json(&body)["rows"][0]["name"], "BoardB");
    // BR-P05: the public profile has no private fields.
    let (status, _, body) = get(&base, "/api/players/BoardA").await;
    assert_eq!(status, 200);
    let p = body_json(&body);
    assert_eq!(
        (
            p["name"].as_str().unwrap(),
            p["rating"].as_u64().unwrap(),
            p["rank"].as_u64().unwrap(),
            p["games"].as_u64().unwrap(),
            p["win_rate"].as_f64().unwrap()
        ),
        ("BoardA", 1000, 1, 1, 0.0)
    );
    assert_eq!(p["joined"].as_str().unwrap().len(), 10);
    assert_eq!(p["recent"].as_array().unwrap().len(), 1);
    assert_eq!(p["recent"][0]["result"], "D");
    assert_eq!(p["recent"][0]["delta"], 0);
    assert_eq!(p["recent"][0]["opponent"], "BoardB");
    assert!(p.get("key_hash").is_none() && p.get("id").is_none());
    // Unknown name and bad shape both 404 with the same body.
    for path in ["/api/players/NoSuchPlayer", "/api/players/x"] {
        let (status, _, body) = get(&base, path).await;
        assert_eq!(status, 404, "{path}");
        assert_eq!(body_json(&body), serde_json::json!({"error": "not_found"}));
    }
}

#[tokio::test]
async fn br_r07_limit_clamped_to_100() {
    // Paging clamp: even `limit=1000` returns at most 100 rows. Needs more
    // than 100 ranked accounts, played as real ranked bot matches.
    let (base, url, _t) = live_server("clamp").await;
    // Register contention is off (each bot registers once into its own
    // server-side budget); the loop below just fills the queue steadily.
    let mut tasks = Vec::new();
    for i in 0..102u64 {
        let mut opts = BotOpts::new(url.clone());
        opts.accounts = true;
        opts.ranked = true;
        opts.timeout = Duration::from_secs(300);
        tasks.push(tokio::spawn(play_one(opts, 5000 + i)));
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let mut completed = 0;
    for t in tasks {
        let r = t.await.unwrap();
        assert!(r.error.is_none(), "{:?}", r.error);
        completed += r.completed as u32;
    }
    assert_eq!(completed, 102);
    let board = wait_total(&base, 102).await;
    let (status, _, body) = get(&base, "/api/leaderboard?limit=1000").await;
    assert_eq!(status, 200);
    assert_eq!(body_json(&body)["rows"].as_array().unwrap().len(), 100);
    assert_eq!(board["total"], 102);
    let (_, _, body) = get(&base, "/api/leaderboard?limit=1000&offset=100").await;
    assert_eq!(body_json(&body)["rows"].as_array().unwrap().len(), 2);
    // Empty page past the end still reports the total.
    let (_, _, body) = get(&base, "/api/leaderboard?offset=100000").await;
    let page = body_json(&body);
    assert!(page["rows"].as_array().unwrap().is_empty());
    assert_eq!(page["total"], 102);
}

#[tokio::test]
async fn api_rate_limit_returns_429() {
    let (base, _, _t) = live_server("ratelimit").await;
    // Default 120/min: the 121st request in a minute is refused as JSON.
    for i in 0..120 {
        let (status, _, _) = get(&base, "/api/leaderboard").await;
        assert_eq!(status, 200, "request {}", i + 1);
    }
    let last = get(&base, "/api/leaderboard").await;
    assert_eq!(last.0, 429);
    assert_eq!(
        body_json(&last.2),
        serde_json::json!({"error": "rate_limited"})
    );
}

#[tokio::test]
async fn br_r07_in_game_leaderboard_matches_api() {
    let (base, url, _t) = live_server("ws-board").await;
    let key = ranked_draw(&url, "WsBoardA", "WsBoardB").await;
    let board = wait_total(&base, 2).await;
    let mut ws = connect(&url).await;
    hello(&mut ws).await;
    send(&mut ws, &ClientMsg::Login { key }).await;
    assert!(matches!(
        recv(&mut ws).await,
        Some(ServerMsg::LoggedIn { .. })
    ));
    // Two requests back to back: the second is ignored (1 per second).
    send(&mut ws, &ClientMsg::Leaderboard { offset: 0 }).await;
    send(&mut ws, &ClientMsg::Leaderboard { offset: 1 }).await;
    let Some(ServerMsg::Leaderboard {
        total,
        offset,
        rows,
        you,
    }) = recv(&mut ws).await
    else {
        panic!("expected Leaderboard");
    };
    assert_eq!((total, offset, you), (2, 0, Some(1)));
    // Same order as the web API (BR-R07).
    let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
    let api: Vec<&str> = board["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, api);
    let extra = tokio::time::timeout(Duration::from_millis(500), recv(&mut ws)).await;
    assert!(
        !matches!(extra, Ok(Some(ServerMsg::Leaderboard { .. }))),
        "second request within 1 s was answered"
    );
}
