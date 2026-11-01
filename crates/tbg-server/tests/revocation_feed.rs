//! P71 (W3): a revocation written by one release slot reaches the other slot's
//! live sockets within seconds through the shared database, instead of after
//! that slot's Access expires (the D-99 limit).
#[path = "common/auth.rs"]
mod auth;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use futures_util::StreamExt;
use serde_json::json;
use tbg_core::protocol::{self, AuthRefusal, AuthStatus, ClientMsg, ServerMsg, PROTOCOL_VERSION};
use tbg_server::{auth::Clock, botclient::native_auth, Config};

static NEXT: AtomicU64 = AtomicU64::new(1);

async fn slot(db: &std::path::Path, clock: &Clock) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/ws", listener.local_addr().unwrap());
    let config = Config {
        db: Some(db.to_path_buf()),
        auth_clock: clock.clone(),
        tick_ms: 20,
        max_per_ip: 100,
        ..Config::default()
    };
    tokio::spawn(tbg_server::run(config, listener));
    url
}

async fn recv(ws: &mut auth::Ws) -> ServerMsg {
    loop {
        let event = tokio::time::timeout(Duration::from_secs(5), ws.next())
            .await
            .expect("socket response timeout")
            .expect("socket closed")
            .unwrap();
        let tokio_tungstenite::tungstenite::Message::Binary(bytes) = event else {
            continue;
        };
        match protocol::decode_server(&bytes).unwrap() {
            ServerMsg::Ping { id } => auth::send(ws, ClientMsg::Pong { id }).await,
            ServerMsg::NetStats { .. } => {}
            other => return other,
        }
    }
}

async fn until(ws: &mut auth::Ws, want: impl Fn(&ServerMsg) -> bool) -> ServerMsg {
    loop {
        let message = recv(ws).await;
        if want(&message) {
            return message;
        }
    }
}

async fn connect(url: &str, access: Option<protocol::AccessToken>) -> auth::Ws {
    let mut ws = tokio_tungstenite::connect_async(url).await.unwrap().0;
    let hello = match access {
        Some(access) => ClientMsg::AuthenticatedHello {
            version: PROTOCOL_VERSION,
            resume: None,
            access,
        },
        None => ClientMsg::Hello {
            version: PROTOCOL_VERSION,
            resume: None,
        },
    };
    auth::send(&mut ws, hello).await;
    until(&mut ws, |m| matches!(m, ServerMsg::Welcome { .. })).await;
    ws
}

#[tokio::test]
async fn p71_w3_a_revocation_on_one_slot_ends_the_family_on_the_other() {
    let db = std::env::temp_dir().join(format!(
        "tbg-p71-feed-{}-{}.sqlite",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let clock = Clock::controlled(1000);
    let a = slot(&db, &clock).await;
    let b = slot(&db, &clock).await;
    let on_a = connect(&a, None).await;
    let grant = auth::register(&on_a, "TwoSlots").await.unwrap();
    let key = grant.account_key.clone().unwrap();
    // Slot B issues its own family and keeps its own Access index.
    let on_b = connect(&b, None).await;
    let signed_in = auth::login(&on_b, &key).await.unwrap();
    let mut player = connect(&b, Some(signed_in.access)).await;
    until(&mut player, |m| {
        matches!(
            m,
            ServerMsg::AuthState {
                status: AuthStatus::Active,
                ..
            }
        )
    })
    .await;
    let (status, _) = native_auth::post(&a, "logout-all", json!({ "account_key": key }), None)
        .await
        .unwrap();
    assert_eq!(status, 200);
    // Slot B ends the family's socket: a Revoked notice, or the close itself.
    let ended = tokio::time::timeout(Duration::from_secs(6), async {
        loop {
            match player.next().await {
                None | Some(Err(_)) => return,
                Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_))) => return,
                Some(Ok(tokio_tungstenite::tungstenite::Message::Binary(bytes))) => {
                    if matches!(
                        protocol::decode_server(&bytes),
                        Ok(ServerMsg::AuthRequired {
                            reason: AuthRefusal::Revoked
                        })
                    ) {
                        return;
                    }
                }
                Some(Ok(_)) => {}
            }
        }
    })
    .await;
    assert!(ended.is_ok(), "slot B still honored the revoked family");
    let _ = std::fs::remove_file(&db);
}
