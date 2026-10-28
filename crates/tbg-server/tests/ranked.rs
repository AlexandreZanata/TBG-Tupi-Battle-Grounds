//! Ranked queue and Elo settlement (GAME-DESIGN BR-R01–BR-R07).
//! Each test names the rule it protects.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tbg_core::protocol::{
    self, ClientMsg, ErrorCode, MatchMode, Profile, ServerMsg, PROTOCOL_VERSION,
};
use tbg_server::store::Store;
use tbg_server::Config;

type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

struct TempDb {
    _dir: std::path::PathBuf,
    db: std::path::PathBuf,
}

fn temp_db(name: &str) -> TempDb {
    let dir = std::env::temp_dir().join(format!("tbg-ranked-{}-{}", std::process::id(), name));
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("test.sqlite3");
    let _ = std::fs::remove_file(&db);
    TempDb { _dir: dir, db }
}

async fn live_server(name: &str, reconnect_secs: u64) -> (String, TempDb) {
    let t = temp_db(name);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/ws", listener.local_addr().unwrap());
    let config = Config {
        tick_ms: 1,
        reconnect_secs,
        max_per_ip: 1_000,
        db: Some(t.db.clone()),
        ..Config::default()
    };
    tokio::spawn(tbg_server::run(config, listener));
    (url, t)
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

async fn register_as(ws: &mut Ws, name: &str) -> (String, Profile) {
    send(ws, &ClientMsg::Register { name: name.into() }).await;
    match recv(ws).await {
        Some(ServerMsg::Registered { key, profile }) => (key, profile),
        other => panic!("expected Registered, got {other:?}"),
    }
}

async fn login_as(ws: &mut Ws, key: &str) -> Profile {
    send(ws, &ClientMsg::Login { key: key.into() }).await;
    loop {
        match recv(ws).await {
            Some(ServerMsg::LoggedIn { profile }) => return profile,
            Some(ServerMsg::MatchStart { .. }) | Some(ServerMsg::View(_)) => continue,
            other => panic!("expected LoggedIn, got {other:?}"),
        }
    }
}

/// Drain until `MatchEnd`, answering nothing (bots are idle).
async fn play_idle(ws: &mut Ws) -> tbg_core::Outcome {
    loop {
        match recv(ws).await {
            Some(ServerMsg::MatchEnd { outcome }) => return outcome,
            Some(_) => continue,
            None => panic!("socket closed mid-match"),
        }
    }
}

#[tokio::test]
async fn br_r01_ranked_pairing_and_draw_rating() {
    // BR-R01/R02: two accounts queue ranked and pair; BR-R03: a draw moves
    // neither rating (equal 1000 vs 1000, placement K).
    let (url, t) = live_server("draw", 30).await;
    let mut a = connect(&url).await;
    let mut b = connect(&url).await;
    hello(&mut a).await;
    hello(&mut b).await;
    register_as(&mut a, "RankA").await;
    register_as(&mut b, "RankB").await;
    send(&mut a, &ClientMsg::RankedMatch).await;
    send(&mut b, &ClientMsg::RankedMatch).await;
    for ws in [&mut a, &mut b] {
        loop {
            match recv(ws).await {
                Some(ServerMsg::MatchStart {
                    mode: MatchMode::Ranked,
                    opponent_rating: Some(1000),
                    ..
                }) => break,
                Some(_) => continue,
                None => panic!("ranked match never started"),
            }
        }
    }
    let (oa, ob) = tokio::join!(play_idle(&mut a), play_idle(&mut b));
    assert_eq!(oa, ob);
    for ws in [&mut a, &mut b] {
        match recv(ws).await {
            Some(ServerMsg::RatingChanged {
                before: 1000,
                after: 1000,
                profile,
            }) => assert_eq!(profile.ranked_games, 1),
            other => panic!("expected unchanged rating, got {other:?}"),
        }
    }
    let store = Store::open(&t.db).unwrap();
    assert_eq!(store.ranked_match_count().await.unwrap(), 1);
}

#[tokio::test]
async fn br_r04_ranked_forfeit_is_a_loss() {
    // BR-R04: staying disconnected past the window forfeits — a ranked win
    // for the stayer (+20 placement) and a loss for the leaver.
    let (url, t) = live_server("forfeit", 1).await;
    let mut a = connect(&url).await;
    let mut b = connect(&url).await;
    hello(&mut a).await;
    hello(&mut b).await;
    let (key_a, _) = register_as(&mut a, "QuitA").await;
    register_as(&mut b, "StayB").await;
    send(&mut a, &ClientMsg::RankedMatch).await;
    send(&mut b, &ClientMsg::RankedMatch).await;
    for ws in [&mut a, &mut b] {
        loop {
            match recv(ws).await {
                Some(ServerMsg::MatchStart { .. }) => break,
                Some(_) => continue,
                None => panic!("match never started"),
            }
        }
    }
    drop(a);
    let outcome = play_idle(&mut b).await;
    assert_eq!(outcome.reason, tbg_core::EndReason::Forfeit);
    assert_eq!(outcome.winner, Some(tbg_core::Seat::B));
    match recv(&mut b).await {
        Some(ServerMsg::RatingChanged {
            before: 1000,
            after: 1020,
            profile,
        }) => {
            assert_eq!(profile.wins, 1);
            assert_eq!(profile.ranked_games, 1);
        }
        other => panic!("expected +20 for the stayer, got {other:?}"),
    }
    // The leaver's side settled too: re-login shows the loss.
    let mut a2 = connect(&url).await;
    hello(&mut a2).await;
    let profile = login_as(&mut a2, &key_a).await;
    assert_eq!(profile.rating, 980);
    assert_eq!(profile.losses, 1);
    let store = Store::open(&t.db).unwrap();
    assert_eq!(store.ranked_match_count().await.unwrap(), 1);
}

#[tokio::test]
async fn br_r01_guest_ranked_needs_login() {
    let (url, _t) = live_server("guest", 30).await;
    let mut ws = connect(&url).await;
    hello(&mut ws).await;
    send(&mut ws, &ClientMsg::RankedMatch).await;
    assert!(matches!(
        recv(&mut ws).await,
        Some(ServerMsg::Error {
            code: ErrorCode::LoginRequired
        })
    ));
}

#[tokio::test]
async fn br_r02_queues_never_mix() {
    // A casual waiter and a ranked waiter are not paired with each other.
    let (url, _t) = live_server("nomix", 30).await;
    let mut casual = connect(&url).await;
    let mut ranked = connect(&url).await;
    hello(&mut casual).await;
    hello(&mut ranked).await;
    register_as(&mut ranked, "SoloQ").await;
    send(&mut casual, &ClientMsg::QuickMatch).await;
    send(&mut ranked, &ClientMsg::RankedMatch).await;
    assert!(matches!(recv(&mut casual).await, Some(ServerMsg::Queued)));
    assert!(matches!(recv(&mut ranked).await, Some(ServerMsg::Queued)));
    let pair = tokio::time::timeout(Duration::from_secs(2), async {
        tokio::join!(recv(&mut casual), recv(&mut ranked))
    })
    .await;
    assert!(
        !matches!(
            pair,
            Ok((Some(ServerMsg::MatchStart { .. }), _) | (_, Some(ServerMsg::MatchStart { .. })))
        ),
        "queues mixed: {pair:?}"
    );
}

#[tokio::test]
async fn br_r06_casual_matches_never_touch_rating() {
    // BR-R06: casual (and private-room) matches leave rating columns alone.
    let (url, _t) = live_server("casual", 30).await;
    let mut a = connect(&url).await;
    let mut b = connect(&url).await;
    hello(&mut a).await;
    hello(&mut b).await;
    let (key_a, _) = register_as(&mut a, "CasA").await;
    register_as(&mut b, "CasB").await;
    send(&mut a, &ClientMsg::QuickMatch).await;
    send(&mut b, &ClientMsg::QuickMatch).await;
    for ws in [&mut a, &mut b] {
        loop {
            match recv(ws).await {
                Some(ServerMsg::MatchStart {
                    mode: MatchMode::Casual,
                    opponent_rating: None,
                    ..
                }) => break,
                Some(_) => continue,
                None => panic!("match never started"),
            }
        }
        // RatingChanged must never arrive for a casual match: drain the
        // steady view flow for a moment and reject any rating message.
        let deadline = std::time::Instant::now() + Duration::from_millis(500);
        while std::time::Instant::now() < deadline {
            match tokio::time::timeout(Duration::from_secs(5), recv(ws)).await {
                Ok(Some(ServerMsg::RatingChanged { .. })) => {
                    panic!("casual match changed rating")
                }
                Ok(_) => continue,
                Err(_) => break,
            }
        }
    }
    // Finish the match, then confirm the columns are untouched.
    let (oa, ob) = tokio::join!(play_idle(&mut a), play_idle(&mut b));
    assert_eq!(oa, ob);
    let mut a2 = connect(&url).await;
    hello(&mut a2).await;
    let profile = login_as(&mut a2, &key_a).await;
    assert_eq!(
        (
            profile.rating,
            profile.ranked_games,
            profile.wins,
            profile.losses
        ),
        (1000, 0, 0, 0)
    );
}
