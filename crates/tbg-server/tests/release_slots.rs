//! P66 two-slot release on one host (U12–U15, BR-UP05/UP06/UP09): two
//! real server processes' worth of tasks share one database; a match that
//! started on generation A keeps running while B is promoted, new players
//! go to B, the drained A refuses new matches, reconnects return to A,
//! a signed-in account cannot open a second session on the other slot,
//! rollback moves only new admissions, and A retires only when idle.
//! Ranked results from both generations settle exactly once (U15).

use std::path::PathBuf;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use rusqlite::Connection;
use tbg_core::contract::MatchContract;
use tbg_core::protocol::{self, AccountError, ClientMsg, ErrorCode, ServerMsg, PROTOCOL_VERSION};
use tbg_server::store::release::{read_directory, SlotRow};
use tbg_server::store::release_admin::{self as admin, retire_blockers};
use tbg_server::{Config, Shared};

type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

struct Slot {
    addr: String,
}

async fn slot(db: &PathBuf, name: &str) -> Slot {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let config = Config {
        max_per_ip: 1_000,
        register_per_ip_hour: 1_000,
        login_per_ip_min: 1_000,
        api_per_ip_min: 10_000,
        reconnect_secs: 3,
        db: Some(db.clone()),
        slot: Some(name.into()),
        generation: format!("g{name}"),
        ws_endpoint: format!("/ws/slot-{name}"),
        ..Config::default()
    };
    tokio::spawn(tbg_server::run_with(config, listener, Shared::default()));
    Slot { addr }
}

impl Slot {
    fn url(&self, name: &str) -> String {
        format!(
            "ws://{}/ws/slot-{name}?contract={}",
            self.addr,
            MatchContract::current().id()
        )
    }
}

fn db() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "tbg-release-{}-{}",
        std::process::id(),
        rand_suffix()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("db.sqlite3")
}

fn rand_suffix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

fn admin_conn(db: &PathBuf) -> Connection {
    let c = Connection::open(db).unwrap();
    c.busy_timeout(Duration::from_secs(5)).unwrap();
    c
}

/// Wait until `pred` holds on the directory (≤ 15 s: heartbeats are 1 s,
/// the ranked handoff runs every 5 s).
async fn until(db: &PathBuf, what: &str, pred: impl Fn(&[SlotRow]) -> bool) {
    for _ in 0..300 {
        let rows = read_directory(&admin_conn(db)).unwrap_or_default();
        if pred(&rows) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("timed out waiting for {what}");
}

async fn send(ws: &mut Ws, msg: &ClientMsg) {
    let frame = tokio_tungstenite::tungstenite::Message::Binary(protocol::encode(msg).into());
    ws.send(frame).await.unwrap();
}

/// Next non-noise message (pings answered, views skipped).
async fn next(ws: &mut Ws, wait: Duration) -> Option<ServerMsg> {
    loop {
        let frame = tokio::time::timeout(wait, ws.next()).await.ok()??.ok()?;
        let tokio_tungstenite::tungstenite::Message::Binary(b) = frame else {
            continue;
        };
        match protocol::decode_server(&b).ok()? {
            ServerMsg::Ping { id } => send(ws, &ClientMsg::Pong { id }).await,
            ServerMsg::NetStats { .. } | ServerMsg::View(_) => {}
            other => return Some(other),
        }
    }
}

/// Wait for a message matching `want` (≤ 10 s); `None` if it never came.
async fn expect(ws: &mut Ws, want: impl Fn(&ServerMsg) -> bool) -> Option<ServerMsg> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    while tokio::time::Instant::now() < deadline {
        match next(ws, Duration::from_millis(500)).await {
            Some(m) if want(&m) => return Some(m),
            _ => {}
        }
    }
    None
}

/// Connect, say hello (optionally resuming) and return the session token.
async fn join(url: &str, resume: Option<protocol::Token>) -> (Ws, protocol::Token) {
    let mut ws = tokio_tungstenite::connect_async(url).await.unwrap().0;
    send(
        &mut ws,
        &ClientMsg::Hello {
            version: PROTOCOL_VERSION,
            resume,
        },
    )
    .await;
    let token = match expect(&mut ws, |m| matches!(m, ServerMsg::Welcome { .. })).await {
        Some(ServerMsg::Welcome { token, .. }) => token,
        other => panic!("no welcome: {other:?}"),
    };
    (ws, token)
}

async fn quick_pair(url: &str) -> (Ws, Ws, protocol::Token) {
    let (mut a, token) = join(url, None).await;
    let (mut b, _) = join(url, None).await;
    send(&mut a, &ClientMsg::QuickMatch).await;
    send(&mut b, &ClientMsg::QuickMatch).await;
    for ws in [&mut a, &mut b] {
        assert!(
            expect(ws, |m| matches!(m, ServerMsg::MatchStart { .. }))
                .await
                .is_some(),
            "match starts"
        );
    }
    (a, b, token)
}

/// HTTP GET on a slot's address, returning the JSON body.
async fn get_json(addr: &str, path: &str) -> serde_json::Value {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut s = tokio::net::TcpStream::connect(addr).await.unwrap();
    let req = format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    s.write_all(req.as_bytes()).await.unwrap();
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).await.unwrap();
    let text = String::from_utf8_lossy(&buf);
    let body = text.split("\r\n\r\n").nth(1).unwrap_or_default();
    serde_json::from_str(body).unwrap_or(serde_json::Value::Null)
}

async fn two_ready_slots() -> (PathBuf, Slot, Slot) {
    let db = db();
    let a = slot(&db, "a").await;
    let b = slot(&db, "b").await;
    until(&db, "both slots ready", |r| {
        r.len() == 2 && r.iter().all(|s| s.state == "ready")
    })
    .await;
    admin::promote(&mut admin_conn(&db), "a", "test", None).unwrap();
    admin::ranked_authority(&mut admin_conn(&db), "a", false, "test").unwrap();
    // The gate follows the directory within one heartbeat.
    tokio::time::sleep(Duration::from_millis(1_500)).await;
    (db, a, b)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u13_u15_a_running_match_survives_promotion_and_rollback() {
    let (db, a, b) = two_ready_slots().await;
    let contract = MatchContract::current().id();
    // A match starts on generation A.
    let (p1, mut p2, p1_token) = quick_pair(&a.url("a")).await;
    // Promote B: A drains, new admissions go to B.
    admin::promote(&mut admin_conn(&db), "b", "test", None).unwrap();
    tokio::time::sleep(Duration::from_millis(1_500)).await;
    let answer = get_json(
        &a.addr,
        &format!("/api/admission?contract={contract}&bucket=50"),
    )
    .await;
    assert_eq!(answer["endpoint"], "/ws/slot-b", "{answer}");
    let (mut q1, mut q2, _) = quick_pair(&b.url("b")).await;
    // A drained slot refuses new matches but keeps its running one.
    let (mut late, _) = join(&a.url("a"), None).await;
    send(&mut late, &ClientMsg::QuickMatch).await;
    let refused = expect(&mut late, |m| matches!(m, ServerMsg::Error { .. })).await;
    assert!(matches!(
        refused,
        Some(ServerMsg::Error {
            code: ErrorCode::ServerFull
        })
    ));
    // U13: a dropped player resumes on A, in the same match.
    drop(p1);
    tokio::time::sleep(Duration::from_millis(300)).await;
    let (mut p1, resumed) = join(&a.url("a"), Some(p1_token)).await;
    assert_eq!(
        resumed, p1_token,
        "the session (and its seat) survived on A"
    );
    // Rollback of new admissions to A: B drains, both matches keep going.
    admin::rollback(&mut admin_conn(&db), "a", "test", None).unwrap();
    tokio::time::sleep(Duration::from_millis(1_500)).await;
    let answer = get_json(
        &b.addr,
        &format!("/api/admission?contract={contract}&bucket=50"),
    )
    .await;
    assert_eq!(answer["endpoint"], "/ws/slot-a", "{answer}");
    // Both matches finish normally on the generation that admitted them.
    send(&mut p1, &ClientMsg::Surrender).await;
    send(&mut q1, &ClientMsg::Surrender).await;
    for ws in [&mut p1, &mut p2, &mut q1, &mut q2] {
        assert!(
            expect(ws, |m| matches!(m, ServerMsg::MatchEnd { .. }))
                .await
                .is_some(),
            "match ends"
        );
    }
    // U14: B (now draining) retires only once its match, its queued records
    // and any ranked authority it took are gone; the gate refuses before.
    until(&db, "both slots idle", |r| {
        r.iter()
            .all(|s| s.matches_running == 0 && s.results_pending == 0)
    })
    .await;
    until(&db, "b retirable", |r| {
        r.iter()
            .any(|s| s.slot == "b" && retire_blockers(s).is_empty())
    })
    .await;
    admin::mark_retired(&mut admin_conn(&db), "b", "test", None).unwrap();
    let rows = read_directory(&admin_conn(&db)).unwrap();
    let states: Vec<(&str, &str)> = rows
        .iter()
        .map(|r| (r.slot.as_str(), r.state.as_str()))
        .collect();
    assert_eq!(states, [("a", "active"), ("b", "retired")]);
    let matches: i64 = admin_conn(&db)
        .query_row("SELECT COUNT(*) FROM matches", [], |r| r.get(0))
        .unwrap();
    assert_eq!(matches, 2, "each finished match recorded exactly once");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u13_a_signed_in_account_holds_one_live_session_across_slots() {
    let (db, a, b) = two_ready_slots().await;
    let (mut on_a, _) = join(&a.url("a"), None).await;
    send(
        &mut on_a,
        &ClientMsg::Register {
            name: "SlotUser".into(),
        },
    )
    .await;
    let key = match expect(&mut on_a, |m| matches!(m, ServerMsg::Registered { .. })).await {
        Some(ServerMsg::Registered { key, .. }) => key,
        other => panic!("not registered: {other:?}"),
    };
    // Signing in through B while A holds a live session is refused.
    let (mut on_b, _) = join(&b.url("b"), None).await;
    send(&mut on_b, &ClientMsg::Login { key: key.clone() }).await;
    let refused = expect(&mut on_b, |m| {
        matches!(
            m,
            ServerMsg::AccountError { .. } | ServerMsg::LoggedIn { .. }
        )
    })
    .await;
    assert!(
        matches!(
            refused,
            Some(ServerMsg::AccountError {
                code: AccountError::Unavailable
            })
        ),
        "{refused:?}"
    );
    // Once the idle session on A disconnects, B may take the account.
    drop(on_a);
    tokio::time::sleep(Duration::from_millis(800)).await;
    send(&mut on_b, &ClientMsg::Login { key }).await;
    let ok = expect(&mut on_b, |m| {
        matches!(
            m,
            ServerMsg::AccountError { .. } | ServerMsg::LoggedIn { .. }
        )
    })
    .await;
    assert!(matches!(ok, Some(ServerMsg::LoggedIn { .. })), "{ok:?}");
    let owner: String = admin_conn(&db)
        .query_row("SELECT slot FROM account_owner", [], |r| r.get(0))
        .unwrap();
    assert_eq!(owner, "b");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u12_a_foreign_contract_is_refused_before_any_seat() {
    let (_db, a, _b) = two_ready_slots().await;
    let url = format!("ws://{}/ws/slot-a?contract=w1-r1-0000000000000000", a.addr);
    let mut ws = tokio_tungstenite::connect_async(url).await.unwrap().0;
    let reply = next(&mut ws, Duration::from_secs(5)).await;
    assert!(
        matches!(reply, Some(ServerMsg::VersionMismatch { .. })),
        "{reply:?}"
    );
    let answer = get_json(&a.addr, "/api/admission?contract=w1-r1-0000000000000000").await;
    assert_eq!(answer["status"], "update_required", "{answer}");
    // The other slot's path is never served here.
    let wrong = format!(
        "ws://{}/ws/slot-b?contract={}",
        a.addr,
        MatchContract::current().id()
    );
    assert!(tokio_tungstenite::connect_async(wrong).await.is_err());
}

/// Sign a fresh connection in under `name` and queue it for ranked play.
async fn ranked_player(url: &str, name: &str) -> Ws {
    let (mut ws, _) = join(url, None).await;
    send(&mut ws, &ClientMsg::Register { name: name.into() }).await;
    let reply = expect(&mut ws, |m| {
        matches!(
            m,
            ServerMsg::Registered { .. } | ServerMsg::AccountError { .. }
        )
    })
    .await;
    assert!(
        matches!(reply, Some(ServerMsg::Registered { .. })),
        "{reply:?}"
    );
    send(&mut ws, &ClientMsg::RankedMatch).await;
    ws
}

async fn ranked_pair(url: &str, names: [&str; 2]) -> (Ws, Ws) {
    let mut a = ranked_player(url, names[0]).await;
    let mut b = ranked_player(url, names[1]).await;
    for ws in [&mut a, &mut b] {
        let start = expect(ws, |m| {
            matches!(m, ServerMsg::MatchStart { .. } | ServerMsg::Error { .. })
        })
        .await;
        assert!(
            matches!(start, Some(ServerMsg::MatchStart { .. })),
            "{start:?}"
        );
    }
    (a, b)
}

/// The surrendering side loses; both receive their one rating change.
async fn settle(winner: &mut Ws, loser: &mut Ws) -> (u16, u16) {
    send(loser, &ClientMsg::Surrender).await;
    let mut after = [0u16; 2];
    for (i, ws) in [winner, loser].into_iter().enumerate() {
        match expect(ws, |m| matches!(m, ServerMsg::RatingChanged { .. })).await {
            Some(ServerMsg::RatingChanged { after: a, .. }) => after[i] = a,
            other => panic!("no rating change: {other:?}"),
        }
    }
    (after[0], after[1])
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u15_ranked_results_from_two_generations_settle_exactly_once() {
    let (db, a, b) = two_ready_slots().await;
    // A ranked match starts on A, which holds ranked authority.
    let (mut r1, mut r2) = ranked_pair(&a.url("a"), ["RankOldA", "RankOldB"]).await;
    // B is promoted and takes ranked authority from the idle holder (a
    // running match is not an in-flight result) on its next handoff tick.
    admin::promote(&mut admin_conn(&db), "b", "test", None).unwrap();
    until(&db, "b holds ranked authority", |r| {
        r.iter().any(|s| s.slot == "b" && s.ranked_authority)
    })
    .await;
    tokio::time::sleep(Duration::from_millis(1_200)).await;
    // A ranked match on B settles first and takes the next sequence…
    let (mut r3, mut r4) = ranked_pair(&b.url("b"), ["RankNewA", "RankNewB"]).await;
    assert_eq!(settle(&mut r3, &mut r4).await, (1020, 980));
    // …then the older match on A ends and still settles, once, after it.
    assert_eq!(settle(&mut r1, &mut r2).await, (1020, 980));
    until(&db, "both slots idle", |r| {
        r.iter()
            .all(|s| s.matches_running == 0 && s.results_pending == 0)
    })
    .await;
    let conn = admin_conn(&db);
    let (rows, keys): (i64, i64) = conn
        .query_row(
            "SELECT COUNT(*), COUNT(DISTINCT match_key) FROM ranked_matches",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!((rows, keys), (2, 2), "one row per ranked match");
    let games: Vec<(String, i64, i64)> = conn
        .prepare("SELECT name, rating, ranked_games FROM accounts ORDER BY name")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    let expected = [
        ("RankNewA", 1020, 1),
        ("RankNewB", 980, 1),
        ("RankOldA", 1020, 1),
        ("RankOldB", 980, 1),
    ];
    let expected: Vec<(String, i64, i64)> = expected
        .iter()
        .map(|(n, r, g)| (n.to_string(), *r, *g))
        .collect();
    assert_eq!(games, expected, "each rating changed exactly once");
    // Neither slot kept a journal entry behind.
    let dir = db.parent().unwrap();
    for name in ["ranked-pending", "ranked-pending-b"] {
        let left = std::fs::read_dir(dir.join(name))
            .map(|d| d.count())
            .unwrap_or(0);
        assert_eq!(left, 0, "{name} still holds results");
    }
}
