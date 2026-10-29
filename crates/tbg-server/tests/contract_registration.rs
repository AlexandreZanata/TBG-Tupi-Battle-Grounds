//! Registration contract (BR-P01, P11 rate limit). Append-only: change a
//! test here only in the same commit as the rule it guards.
//!
//! Regression (2026-10-29, production): a player whose old account was
//! "TINAS" could not create "tinas1". Attempts refused because the name was
//! taken were counted against the per-IP register limit (3 per hour), so a
//! free name was refused after a couple of taken ones. Only accounts that
//! were actually created may count.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tbg_core::protocol::{self, AccountError, ClientMsg, ServerMsg, PROTOCOL_VERSION};
use tbg_server::Config;

type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// A server with its own database and the default register limit (3 per
/// IP per hour); every test connection comes from 127.0.0.1.
async fn server(name: &str) -> String {
    let dir = std::env::temp_dir().join(format!("tbg-register-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("test.sqlite3");
    let _ = std::fs::remove_file(&db);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/ws", listener.local_addr().unwrap());
    let config = Config {
        tick_ms: 5,
        max_per_ip: 1_000,
        db: Some(db),
        ..Config::default()
    };
    assert_eq!(
        config.register_per_ip_hour, 3,
        "this contract assumes the default limit"
    );
    tokio::spawn(tbg_server::run(config, listener));
    url
}

async fn send(ws: &mut Ws, msg: &ClientMsg) {
    let bytes = protocol::encode(msg);
    ws.send(tokio_tungstenite::tungstenite::Message::Binary(
        bytes.into(),
    ))
    .await
    .unwrap();
}

/// Next account-relevant reply (housekeeping and Arcana echoes skipped).
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
            ServerMsg::NetStats { .. } | ServerMsg::ArcanasSelected { .. } => {}
            other => return Some(other),
        }
    }
}

async fn connect(url: &str) -> Ws {
    let mut ws = tokio_tungstenite::connect_async(url).await.unwrap().0;
    let hello = ClientMsg::Hello {
        version: PROTOCOL_VERSION,
        resume: None,
    };
    send(&mut ws, &hello).await;
    assert!(matches!(
        recv(&mut ws).await,
        Some(ServerMsg::Welcome { .. })
    ));
    ws
}

/// `Ok(account name)` or `Err(code)`.
async fn register(ws: &mut Ws, name: &str) -> Result<String, AccountError> {
    send(ws, &ClientMsg::Register { name: name.into() }).await;
    match recv(ws).await {
        Some(ServerMsg::Registered { profile, .. }) => Ok(profile.name),
        Some(ServerMsg::AccountError { code }) => Err(code),
        other => panic!("unexpected reply {other:?}"),
    }
}

#[tokio::test]
async fn br_p01_taken_names_do_not_consume_the_register_limit() {
    let url = server("taken").await;
    let mut old_device = connect(&url).await;
    assert_eq!(register(&mut old_device, "TINAS").await, Ok("TINAS".into()));
    let mut player = connect(&url).await;
    // Names are unique ignoring case (BR-P01): both are refused as taken...
    assert_eq!(
        register(&mut player, "tinas").await,
        Err(AccountError::NameTaken)
    );
    assert_eq!(
        register(&mut player, "TiNaS").await,
        Err(AccountError::NameTaken)
    );
    // ...but those refusals created nothing, so a free name still works.
    assert_eq!(register(&mut player, "tinas1").await, Ok("tinas1".into()));
}

#[tokio::test]
async fn register_limit_still_caps_created_accounts() {
    let url = server("cap").await;
    for name in ["first", "second", "third"] {
        let mut ws = connect(&url).await;
        assert_eq!(register(&mut ws, name).await, Ok(name.into()));
    }
    // Three accounts created from this IP within the hour: the fourth waits.
    let mut ws = connect(&url).await;
    assert_eq!(
        register(&mut ws, "fourth").await,
        Err(AccountError::RateLimited)
    );
    // An invalid name is refused as invalid, never as rate limited.
    assert_eq!(
        register(&mut ws, "1bad").await,
        Err(AccountError::NameInvalid)
    );
}
