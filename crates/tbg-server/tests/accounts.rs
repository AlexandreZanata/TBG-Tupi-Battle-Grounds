//! Account store and protocol tests (GAME-DESIGN BR-P01–BR-P05).
//! Each test names the rule it protects.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tbg_core::protocol::{self, AccountError, ClientMsg, Profile, ServerMsg, PROTOCOL_VERSION};
use tbg_server::store::Store;
use tbg_server::Config;

struct TempDb {
    _dir: std::path::PathBuf,
    db: std::path::PathBuf,
}

fn temp_db(name: &str) -> TempDb {
    let dir = std::env::temp_dir().join(format!("tbg-accounts-{}-{}", std::process::id(), name));
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("test.sqlite3");
    let _ = std::fs::remove_file(&db);
    TempDb { _dir: dir, db }
}

fn hash_of(secret: &[u8; 16]) -> [u8; 32] {
    use sha2::Digest;
    sha2::Sha256::digest(secret).into()
}

fn secret_of(byte: u8) -> [u8; 16] {
    let mut s = [0u8; 16];
    for (i, b) in s.iter_mut().enumerate() {
        *b = byte.wrapping_add(i as u8);
    }
    s
}

type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn live_server(name: &str) -> (String, TempDb) {
    live_server_ttl(name, 30).await
}

async fn live_server_ttl(name: &str, reconnect_secs: u64) -> (String, TempDb) {
    let t = temp_db(name);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/ws", listener.local_addr().unwrap());
    let config = Config {
        tick_ms: 5,
        reconnect_secs,
        max_per_ip: 1_000,
        db: Some(t.db.clone()),
        ..Config::default()
    };
    tokio::spawn(tbg_server::run(config, listener));
    (url, t)
}

async fn send(ws: &mut Ws, msg: &ClientMsg) {
    ws.send(tokio_tungstenite::tungstenite::Message::Binary(
        protocol::encode(msg).into(),
    ))
    .await
    .unwrap();
}

/// Next server message, answering app pings; `None` once closed.
async fn recv(ws: &mut Ws) -> Option<ServerMsg> {
    loop {
        let msg = tokio::time::timeout(Duration::from_secs(10), ws.next())
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

/// Next non-`View` message: views keep arriving during a match, so a
/// reply can come after one under load.
async fn recv_no_views(ws: &mut Ws) -> Option<ServerMsg> {
    loop {
        match recv(ws).await? {
            ServerMsg::View(_) => continue,
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

/// `Hello` (optionally resuming) and return the session token.
async fn hello_token(ws: &mut Ws, resume: Option<protocol::Token>) -> protocol::Token {
    send(
        ws,
        &ClientMsg::Hello {
            version: PROTOCOL_VERSION,
            resume,
        },
    )
    .await;
    match recv(ws).await {
        Some(ServerMsg::Welcome { token, .. }) => token,
        other => panic!("expected Welcome, got {other:?}"),
    }
}

async fn connect(url: &str) -> Ws {
    tokio_tungstenite::connect_async(url).await.unwrap().0
}

/// Register and return the (plain key, profile).
async fn register_as(ws: &mut Ws, name: &str) -> (String, Profile) {
    send(ws, &ClientMsg::Register { name: name.into() }).await;
    match recv(ws).await {
        Some(ServerMsg::Registered { key, profile }) => (key, profile),
        other => panic!("expected Registered, got {other:?}"),
    }
}

#[tokio::test]
async fn br_p02_migrate_twice_is_noop() {
    let t = temp_db("migrate");
    let db = &t.db;
    Store::open(db).unwrap();
    // Second open migrates again: must be a no-op, data intact.
    let store = Store::open(db).unwrap();
    let secret = secret_of(1);
    let (_, profile) = store
        .register("Aldric".into(), hash_of(&secret))
        .await
        .unwrap();
    assert_eq!(profile.name, "Aldric");
    assert_eq!(store.match_count().await.unwrap(), 0);
}

#[tokio::test]
async fn br_p01_register_then_login() {
    let t = temp_db("reglogin");
    let db = &t.db;
    let store = Store::open(db).unwrap();
    let secret = secret_of(2);
    let (id, profile) = store
        .register("Aldric".into(), hash_of(&secret))
        .await
        .unwrap();
    assert_eq!(profile.rating, 1000);
    assert_eq!(profile.ranked_games, 0);
    let (same_id, same) = store
        .login("aldric".into(), hash_of(&secret))
        .await
        .unwrap();
    assert_eq!(id, same_id);
    assert_eq!(same, profile);
}

#[tokio::test]
async fn br_p01_duplicate_name_other_case_taken() {
    let t = temp_db("dup");
    let db = &t.db;
    let store = Store::open(db).unwrap();
    store
        .register("Aldric".into(), hash_of(&secret_of(3)))
        .await
        .unwrap();
    assert_eq!(
        store
            .register("ALDRIC".into(), hash_of(&secret_of(4)))
            .await,
        Err(AccountError::NameTaken)
    );
    assert_eq!(
        store
            .register("aldric".into(), hash_of(&secret_of(5)))
            .await,
        Err(AccountError::NameTaken)
    );
}

#[tokio::test]
async fn br_p01_register_replies_key_that_parses() {
    let (url, _t) = live_server("proto-reg").await;
    let mut ws = connect(&url).await;
    hello(&mut ws).await;
    let (key, profile) = register_as(&mut ws, "Aldric").await;
    assert_eq!(profile.name, "Aldric");
    // The key parses and round-trips the typed name (BR-P02).
    let (name, _) = tbg_core::account::parse_key(&key).expect("key must parse");
    assert_eq!(name, "Aldric");
}

#[tokio::test]
async fn br_p01_register_name_taken_and_invalid() {
    let (url, _t) = live_server("proto-taken").await;
    let mut ws = connect(&url).await;
    hello(&mut ws).await;
    register_as(&mut ws, "Aldric").await;
    // Same name in another case → NameTaken.
    send(
        &mut ws,
        &ClientMsg::Register {
            name: "ALDRIC".into(),
        },
    )
    .await;
    assert!(matches!(
        recv(&mut ws).await,
        Some(ServerMsg::AccountError {
            code: AccountError::NameTaken
        })
    ));
    // Bad shape → NameInvalid.
    send(&mut ws, &ClientMsg::Register { name: "x".into() }).await;
    assert!(matches!(
        recv(&mut ws).await,
        Some(ServerMsg::AccountError {
            code: AccountError::NameInvalid
        })
    ));
}

#[tokio::test]
async fn br_p02_login_same_profile_and_bad_secret() {
    let (url, _t) = live_server("proto-login").await;
    let mut ws = connect(&url).await;
    hello(&mut ws).await;
    let (key, profile) = register_as(&mut ws, "Brienne").await;
    // A new socket with the key logs into the same profile.
    let mut ws2 = connect(&url).await;
    hello(&mut ws2).await;
    send(&mut ws2, &ClientMsg::Login { key: key.clone() }).await;
    match recv(&mut ws2).await {
        Some(ServerMsg::LoggedIn { profile: p }) => assert_eq!(p, profile),
        other => panic!("expected LoggedIn, got {other:?}"),
    }
    // Tampered secret → BadKey on a third socket.
    let mut bad: Vec<u8> = key.into_bytes();
    let last = bad.len() - 1;
    bad[last] = if bad[last] == b'0' { b'1' } else { b'0' };
    let mut ws3 = connect(&url).await;
    hello(&mut ws3).await;
    send(
        &mut ws3,
        &ClientMsg::Login {
            key: String::from_utf8(bad).unwrap(),
        },
    )
    .await;
    assert!(matches!(
        recv(&mut ws3).await,
        Some(ServerMsg::AccountError {
            code: AccountError::BadKey
        })
    ));
}

#[tokio::test]
async fn br_p01_register_rate_limited_per_ip() {
    // Default `TBG_REGISTER_PER_IP_HOUR` is 3: the 4th register is refused.
    let (url, _t) = live_server("proto-ratelimit").await;
    for (i, name) in ["Rlone", "Rltwo", "Rlthree"].iter().enumerate() {
        let mut ws = connect(&url).await;
        hello(&mut ws).await;
        register_as(&mut ws, name).await;
        let _ = i;
    }
    let mut ws = connect(&url).await;
    hello(&mut ws).await;
    send(
        &mut ws,
        &ClientMsg::Register {
            name: "Rlfour".into(),
        },
    )
    .await;
    assert!(matches!(
        recv(&mut ws).await,
        Some(ServerMsg::AccountError {
            code: AccountError::RateLimited
        })
    ));
}

#[tokio::test]
async fn br_p03_second_login_takes_over_live_match() {
    let (url, _t) = live_server("proto-takeover").await;
    let mut a = connect(&url).await;
    let mut b = connect(&url).await;
    let old_token = hello_token(&mut a, None).await;
    hello(&mut b).await;
    let (key, _) = register_as(&mut a, "Takeover").await;
    send(&mut a, &ClientMsg::QuickMatch).await;
    send(&mut b, &ClientMsg::QuickMatch).await;
    // Both sides reach a running match.
    for ws in [&mut a, &mut b] {
        loop {
            match recv(ws).await {
                Some(ServerMsg::MatchStart { .. }) => break,
                Some(_) => continue,
                None => panic!("match never started"),
            }
        }
    }
    // A second login with the same key takes the match over (BR-P03).
    let mut a2 = connect(&url).await;
    hello(&mut a2).await;
    send(&mut a2, &ClientMsg::Login { key }).await;
    let mut saw_start = false;
    let mut saw_view = false;
    for _ in 0..50 {
        match recv(&mut a2).await {
            Some(ServerMsg::LoggedIn { .. }) => continue,
            Some(ServerMsg::MatchStart { .. }) => saw_start = true,
            Some(ServerMsg::View(_)) => {
                saw_view = true;
                break;
            }
            Some(_) => continue,
            None => panic!("takeover socket closed"),
        }
    }
    assert!(saw_start && saw_view, "no MatchStart + View after takeover");
    // The old socket is told why, then closed by the takeover (BR-P03).
    let mut told = false;
    loop {
        match tokio::time::timeout(Duration::from_secs(10), async {
            use futures_util::StreamExt;
            a.next().await
        })
        .await
        {
            Ok(None) => break,
            Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Binary(b)))) => {
                told |= protocol::decode_server(&b).is_ok_and(|m| {
                    m == ServerMsg::Error {
                        code: protocol::ErrorCode::SessionTakenOver,
                    }
                });
            }
            Ok(Some(_)) => continue,
            Err(_) => panic!("old socket not closed after takeover"),
        }
    }
    assert!(told, "old device was not told its session was taken over");
    // The closed device auto-resumes with its old token: it gets a fresh
    // guest session, never the match it lost (BR-P03).
    let mut a3 = connect(&url).await;
    let token = hello_token(&mut a3, Some(old_token)).await;
    assert_ne!(token, old_token, "old session survived the takeover");
    let resumed = tokio::time::timeout(Duration::from_millis(500), async {
        loop {
            match recv(&mut a3).await {
                Some(ServerMsg::MatchStart { .. } | ServerMsg::View(_)) => return true,
                Some(_) => continue,
                None => return false,
            }
        }
    })
    .await;
    assert!(
        !matches!(resumed, Ok(true)),
        "old device resumed into the taken-over match"
    );
}

#[tokio::test]
async fn br_p04_account_messages_refused_in_match() {
    let (url, _t) = live_server("proto-busy").await;
    let mut a = connect(&url).await;
    let mut b = connect(&url).await;
    hello(&mut a).await;
    hello(&mut b).await;
    send(&mut a, &ClientMsg::QuickMatch).await;
    send(&mut b, &ClientMsg::QuickMatch).await;
    for ws in [&mut a, &mut b] {
        loop {
            match recv(ws).await {
                Some(ServerMsg::MatchStart { .. }) => break,
                Some(_) => continue,
                None => panic!("match never started"),
            }
        }
    }
    // Mid-match register/logout are refused, not applied.
    send(
        &mut a,
        &ClientMsg::Register {
            name: "Busy".into(),
        },
    )
    .await;
    assert!(matches!(
        recv_no_views(&mut a).await,
        Some(ServerMsg::AccountError {
            code: AccountError::Unavailable
        })
    ));
    send(&mut a, &ClientMsg::Logout).await;
    assert!(matches!(
        recv_no_views(&mut a).await,
        Some(ServerMsg::AccountError {
            code: AccountError::Unavailable
        })
    ));
}

#[tokio::test]
async fn br_p02_bad_secret_and_unknown_name_share_badkey() {
    // BR-P02/BR-P03: no name oracle — identical error and code path.
    let t = temp_db("badkey");
    let db = &t.db;
    let store = Store::open(db).unwrap();
    let secret = secret_of(6);
    store
        .register("Aldric".into(), hash_of(&secret))
        .await
        .unwrap();
    let mut tampered = secret;
    tampered[0] ^= 0xff;
    assert_eq!(
        store.login("aldric".into(), hash_of(&tampered)).await,
        Err(AccountError::BadKey)
    );
    assert_eq!(
        store.login("unknown".into(), hash_of(&secret)).await,
        Err(AccountError::BadKey)
    );
}

#[tokio::test]
async fn br_p04_resumed_session_reports_its_account() {
    // P14: a reload resumes the session by token; the new socket must learn
    // the session still holds the account (the client's profile is gone).
    let (url, _t) = live_server("proto-resume-account").await;
    let mut a = connect(&url).await;
    let token = hello_token(&mut a, None).await;
    register_as(&mut a, "Resumer").await;
    drop(a);
    let mut a2 = connect(&url).await;
    assert_eq!(hello_token(&mut a2, Some(token)).await, token);
    match recv(&mut a2).await {
        Some(ServerMsg::LoggedIn { profile }) => assert_eq!(profile.name, "Resumer"),
        other => panic!("expected LoggedIn after resume, got {other:?}"),
    }
}

#[tokio::test]
async fn br_p03_expired_session_returns_as_guest_and_ranked_is_refused() {
    // BR-P03: the idle session is dropped after `reconnect_secs`, so the old
    // token no longer carries the account. BR-R01: that fresh guest session
    // is refused by the ranked queue with `LoginRequired` — the client must
    // log in again (see net/login.rs).
    let (url, _t) = live_server_ttl("proto-expired", 1).await;
    let mut a = connect(&url).await;
    let old = hello_token(&mut a, None).await;
    register_as(&mut a, "Expired").await;
    drop(a);
    // The lobby sweeps expired sessions every 10 s.
    tokio::time::sleep(Duration::from_secs(11)).await;
    let mut b = connect(&url).await;
    let fresh = hello_token(&mut b, Some(old)).await;
    assert_ne!(
        fresh, old,
        "old account session survived the reconnect window"
    );
    // A resumed session would answer Queued; a fresh guest gets refused.
    send(&mut b, &ClientMsg::RankedMatch).await;
    assert!(matches!(
        recv(&mut b).await,
        Some(ServerMsg::Error {
            code: protocol::ErrorCode::LoginRequired
        })
    ));
}
