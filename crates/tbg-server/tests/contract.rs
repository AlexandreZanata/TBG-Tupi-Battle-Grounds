//! Guard tests for the server contract: production defaults (ARCHITECTURE
//! §5 limits, GAME-DESIGN §5 reconnect window) and pairing (BR-R02: FIFO,
//! queues never mix, ratings ignored). A failure means a documented limit
//! or the matchmaking rule changed: update the docs in the same commit.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tbg_core::content::TICK_HZ;
use tbg_core::protocol::{self, ClientMsg, MatchMode, ServerMsg, PROTOCOL_VERSION};
use tbg_server::Config;

type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

#[test]
fn production_defaults() {
    let c = Config::default();
    // GAME-DESIGN §5: 30 s reconnect window; one tick = 1 / TICK_HZ.
    assert_eq!(c.reconnect_secs, 30);
    assert_eq!(c.tick_ms, 1_000 / u64::from(TICK_HZ));
    // ARCHITECTURE §5/§11: ≤ 8 sockets per IP, global cap, account and API
    // rate limits (P11/P13), all cores unless TBG_WORKERS says otherwise.
    assert_eq!((c.max_per_ip, c.max_connections), (8, 2_000));
    assert_eq!((c.register_per_ip_hour, c.login_per_ip_min), (3, 10));
    assert_eq!(c.api_per_ip_min, 120);
    assert_eq!(
        (
            c.challenge_ttl_secs,
            c.rematch_ttl_secs,
            c.invite_per_ip_min
        ),
        (600, 120, 30)
    );
    assert_eq!(c.workers, 0);
    assert_eq!(c.addr, "0.0.0.0:8080");
}

async fn server() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/ws", listener.local_addr().unwrap());
    let dir = std::env::temp_dir().join(format!("tbg-contract-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join(format!("{}.sqlite3", listener.local_addr().unwrap().port()));
    let config = Config {
        max_per_ip: 1_000,
        register_per_ip_hour: 1_000,
        db: Some(db),
        ..Config::default()
    };
    tokio::spawn(tbg_server::run(config, listener));
    url
}

async fn send(ws: &mut Ws, msg: &ClientMsg) {
    let frame = tokio_tungstenite::tungstenite::Message::Binary(protocol::encode(msg).into());
    ws.send(frame).await.unwrap();
}

/// Next meaningful server message (pings answered), `None` on timeout/close.
async fn recv_within(ws: &mut Ws, wait: Duration) -> Option<ServerMsg> {
    loop {
        let frame = tokio::time::timeout(wait, ws.next()).await.ok()??.ok()?;
        let tokio_tungstenite::tungstenite::Message::Binary(b) = frame else {
            continue;
        };
        match protocol::decode_server(&b).unwrap() {
            ServerMsg::Ping { id } => send(ws, &ClientMsg::Pong { id }).await,
            ServerMsg::NetStats { .. } | ServerMsg::View(_) => {}
            other => return Some(other),
        }
    }
}

async fn player(url: &str, account: Option<&str>) -> Ws {
    let mut ws = tokio_tungstenite::connect_async(url).await.unwrap().0;
    let hello = ClientMsg::Hello {
        version: PROTOCOL_VERSION,
        resume: None,
    };
    send(&mut ws, &hello).await;
    let welcome = recv_within(&mut ws, Duration::from_secs(5)).await;
    assert!(matches!(welcome, Some(ServerMsg::Welcome { .. })));
    if let Some(name) = account {
        send(&mut ws, &ClientMsg::Register { name: name.into() }).await;
        let reply = recv_within(&mut ws, Duration::from_secs(5)).await;
        assert!(
            matches!(reply, Some(ServerMsg::Registered { .. })),
            "{reply:?}"
        );
    }
    ws
}

/// Join a queue and wait for the server to confirm the place in it.
async fn queue(ws: &mut Ws, msg: ClientMsg) {
    send(ws, &msg).await;
    let reply = recv_within(ws, Duration::from_secs(5)).await;
    assert_eq!(reply, Some(ServerMsg::Queued), "{msg:?}");
}

/// The pair's `MatchStart`s: `(you_name, opponent, mode)` per side.
async fn started(ws: &mut Ws) -> (String, String, MatchMode) {
    match recv_within(ws, Duration::from_secs(5)).await {
        Some(ServerMsg::MatchStart {
            you_name,
            opponent,
            mode,
            ..
        }) => (you_name, opponent, mode),
        other => panic!("expected MatchStart, got {other:?}"),
    }
}

async fn still_waiting(ws: &mut Ws) {
    let next = recv_within(ws, Duration::from_millis(400)).await;
    assert!(next.is_none(), "a waiter was paired out of order: {next:?}");
}

#[tokio::test]
async fn br_r02_casual_pairs_first_come_first_served() {
    let url = server().await;
    let (mut a, mut b, mut c, mut d) = (
        player(&url, None).await,
        player(&url, None).await,
        player(&url, None).await,
        player(&url, None).await,
    );
    queue(&mut a, ClientMsg::QuickMatch).await;
    send(&mut b, &ClientMsg::QuickMatch).await;
    let (sa, sb) = tokio::join!(started(&mut a), started(&mut b));
    // Names cross over: each side sees the other's own name.
    assert_eq!(
        (sa.1.as_str(), sb.1.as_str()),
        (sb.0.as_str(), sa.0.as_str())
    );
    assert_eq!((sa.2, sb.2), (MatchMode::Casual, MatchMode::Casual));
    queue(&mut c, ClientMsg::QuickMatch).await;
    still_waiting(&mut c).await;
    send(&mut d, &ClientMsg::QuickMatch).await;
    let (sc, sd) = tokio::join!(started(&mut c), started(&mut d));
    assert_eq!((sc.1, sd.1), (sd.0, sc.0));
}

#[tokio::test]
async fn br_r02_ranked_is_fifo_and_never_mixes_with_casual() {
    let url = server().await;
    let mut casual = player(&url, None).await;
    let mut r1 = player(&url, Some("FifoOne")).await;
    let mut r2 = player(&url, Some("FifoTwo")).await;
    let mut r3 = player(&url, Some("FifoThree")).await;
    queue(&mut casual, ClientMsg::QuickMatch).await;
    queue(&mut r1, ClientMsg::RankedMatch).await;
    send(&mut r2, &ClientMsg::RankedMatch).await;
    let (s1, s2) = tokio::join!(started(&mut r1), started(&mut r2));
    assert_eq!((s1.0.as_str(), s1.1.as_str()), ("FifoOne", "FifoTwo"));
    assert_eq!((s2.2, s1.2), (MatchMode::Ranked, MatchMode::Ranked));
    // A ranked waiter and a casual waiter are never paired together.
    queue(&mut r3, ClientMsg::RankedMatch).await;
    still_waiting(&mut r3).await;
    still_waiting(&mut casual).await;
}
