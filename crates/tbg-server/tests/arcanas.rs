//! Arcanas online (D-40; BR-AR05, BR-AR06, BR-AR08): idle-only validated
//! selection, presets frozen at pairing and shown in `MatchStart`, saved
//! per account and restored at login. Append-only.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tbg_core::protocol::{self, ArcanaRejection, ClientMsg, ServerMsg, PROTOCOL_VERSION};
use tbg_core::{ArcanaPreset, Seat};
use tbg_server::Config;

type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn server(name: &str) -> String {
    let dir = std::env::temp_dir().join(format!("tbg-arcanas-{}-{name}", std::process::id()));
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

/// Next message that is not housekeeping or a match view.
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
            ServerMsg::NetStats { .. } | ServerMsg::View(_) => {}
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

async fn select(ws: &mut Ws, ranks: [u8; 6]) -> Option<ServerMsg> {
    send(ws, &ClientMsg::SetArcanas { ranks }).await;
    recv(ws).await
}

#[tokio::test]
async fn br_ar06_only_complete_presets_while_idle() {
    let url = server("idle").await;
    let mut a = connect(&url).await;
    let melee = ArcanaPreset::complete([2, 2, 2, 0, 0, 0]).unwrap();
    assert_eq!(
        select(&mut a, melee.ranks).await,
        Some(ServerMsg::ArcanasSelected { preset: melee })
    );
    for bad in [
        [2, 2, 2, 2, 0, 0],
        [3, 3, 0, 0, 0, 0],
        [0; 6],
        [1, 1, 1, 1, 1, 0],
    ] {
        assert_eq!(
            select(&mut a, bad).await,
            Some(ServerMsg::ArcanasRejected {
                reason: ArcanaRejection::BadRanks
            }),
            "{bad:?}"
        );
    }
    // Queued: frozen (BR-AR05).
    send(&mut a, &ClientMsg::QuickMatch).await;
    assert_eq!(recv(&mut a).await, Some(ServerMsg::Queued));
    assert_eq!(
        select(&mut a, ArcanaPreset::DEFAULT.ranks).await,
        Some(ServerMsg::ArcanasRejected {
            reason: ArcanaRejection::NotIdle
        })
    );
    // Cancel unlocks editing.
    send(&mut a, &ClientMsg::Cancel).await;
    assert_eq!(
        select(&mut a, ArcanaPreset::DEFAULT.ranks).await,
        Some(ServerMsg::ArcanasSelected {
            preset: ArcanaPreset::DEFAULT
        })
    );
}

#[tokio::test]
async fn br_ar05_presets_are_frozen_at_pairing_and_public_in_match_start() {
    let url = server("frozen").await;
    let (mut a, mut b) = (connect(&url).await, connect(&url).await);
    let control = ArcanaPreset::complete([0, 0, 0, 2, 2, 2]).unwrap();
    assert!(matches!(
        select(&mut a, control.ranks).await,
        Some(ServerMsg::ArcanasSelected { .. })
    ));
    // b keeps the default it was given at connection.
    send(&mut a, &ClientMsg::QuickMatch).await;
    assert_eq!(recv(&mut a).await, Some(ServerMsg::Queued));
    send(&mut b, &ClientMsg::QuickMatch).await;
    let mut starts = Vec::new();
    for ws in [&mut a, &mut b] {
        let Some(ServerMsg::MatchStart { you, arcanas, .. }) = recv(ws).await else {
            panic!("no MatchStart");
        };
        starts.push((you, arcanas));
    }
    let (a_seat, arcanas) = starts[0];
    let b_seat = if a_seat == Seat::A { Seat::B } else { Seat::A };
    assert_eq!(arcanas[a_seat.index()], control);
    assert_eq!(arcanas[b_seat.index()], ArcanaPreset::DEFAULT);
    assert_eq!(starts[1].1, arcanas, "both players see the same presets");
    // In a match: an edit is refused and changes nothing.
    assert_eq!(
        select(&mut a, ArcanaPreset::DEFAULT.ranks).await,
        Some(ServerMsg::ArcanasRejected {
            reason: ArcanaRejection::NotIdle
        })
    );
}

#[tokio::test]
async fn br_ar08_an_account_keeps_its_preset_across_devices() {
    let url = server("account").await;
    let mut first = connect(&url).await;
    send(
        &mut first,
        &ClientMsg::Register {
            name: "Arcanist".into(),
        },
    )
    .await;
    let Some(ServerMsg::Registered { key, .. }) = recv(&mut first).await else {
        panic!("not registered");
    };
    let ranged = ArcanaPreset::complete([2, 0, 2, 2, 0, 0]).unwrap();
    assert!(matches!(
        select(&mut first, ranged.ranks).await,
        Some(ServerMsg::ArcanasSelected { .. })
    ));
    // Another device logs in: the saved preset comes back after LoggedIn.
    let mut second = connect(&url).await;
    send(&mut second, &ClientMsg::Login { key }).await;
    assert!(matches!(
        recv(&mut second).await,
        Some(ServerMsg::LoggedIn { .. })
    ));
    assert_eq!(
        recv(&mut second).await,
        Some(ServerMsg::ArcanasSelected { preset: ranged })
    );
}
