//! Room contract over a real socket: Hello/Welcome, seats from connections,
//! live snapshots, authoritative purchases and hostile-frame closing.

use futures_util::{SinkExt, StreamExt};
use tbg_moba_core::protocol::{self, ClientMsg, ServerMsg, MOBA_PROTOCOL_VERSION};
use tbg_moba_core::{Destination, Lane, MobaIntention};
use tokio_tungstenite::tungstenite::Message;

type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn start() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(tbg_moba_server::serve(listener, 11));
    format!("ws://{addr}/moba/ws")
}

async fn join(url: &str) -> (Socket, tbg_moba_core::protocol::Welcome) {
    let (mut ws, _) = tokio_tungstenite::connect_async(url).await.unwrap();
    let hello = ClientMsg::Hello {
        protocol: MOBA_PROTOCOL_VERSION,
        name: "test".into(),
    };
    ws.send(Message::Binary(protocol::encode(&hello).into()))
        .await
        .unwrap();
    loop {
        if let Some(ServerMsg::Welcome(w)) = next(&mut ws).await {
            return (ws, w);
        }
    }
}

async fn next(ws: &mut Socket) -> Option<ServerMsg> {
    match tokio::time::timeout(std::time::Duration::from_secs(5), ws.next()).await {
        Ok(Some(Ok(Message::Binary(b)))) => protocol::decode_server(&b).ok(),
        _ => None,
    }
}

#[tokio::test]
async fn m41_seats_come_from_connections_and_purchases_are_authoritative() {
    let url = start().await;
    let (mut a, wa) = join(&url).await;
    let (_b, wb) = join(&url).await;
    assert_ne!(wa.seat, wb.seat);
    assert_ne!(
        wa.seat.team(),
        wb.seat.team(),
        "humans are balanced across teams"
    );

    let mut gold = None;
    let mut last_tick = 0;
    for _ in 0..40 {
        match next(&mut a).await {
            Some(ServerMsg::World(w)) => {
                assert!(w.tick >= last_tick);
                last_tick = w.tick;
            }
            Some(ServerMsg::Private(p)) => {
                assert_eq!(p.seat, wa.seat);
                gold = Some(p.gold);
            }
            _ => {}
        }
    }
    assert!(last_tick > 0 && gold.is_some());
    let buy = ClientMsg::Intention(MobaIntention::BuyTroop {
        slot: 0,
        to: Destination::Lane(Lane::Mid),
    });
    a.send(Message::Binary(protocol::encode(&buy).into()))
        .await
        .unwrap();
    let mut troops = 0;
    for _ in 0..40 {
        if let Some(ServerMsg::Private(p)) = next(&mut a).await {
            troops = troops.max(p.troops);
        }
    }
    assert_eq!(troops, 1, "the server spawned exactly the purchased Troop");
}

#[tokio::test]
async fn m42_hostile_frames_close_the_socket() {
    let url = start().await;
    let (mut ws, _) = join(&url).await;
    ws.send(Message::Binary(vec![0xFF; 64].into()))
        .await
        .unwrap();
    let closed = async {
        loop {
            match ws.next().await {
                None | Some(Err(_)) | Some(Ok(Message::Close(_))) => return true,
                _ => {}
            }
        }
    };
    assert!(
        tokio::time::timeout(std::time::Duration::from_secs(5), closed)
            .await
            .unwrap_or(false)
    );
}
