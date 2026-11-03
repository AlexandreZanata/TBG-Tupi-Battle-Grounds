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
    let mut buy_slot = None;
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
                buy_slot = p
                    .offers
                    .iter()
                    .position(|k| p.gold >= tbg_moba_core::content::troop_cost(*k));
            }
            _ => {}
        }
    }
    assert!(last_tick > 0 && gold.is_some());
    let buy = ClientMsg::Intention(MobaIntention::BuyTroop {
        slot: buy_slot.expect("seed 11 provides an affordable opening offer") as u8,
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

#[tokio::test]
async fn m52_each_socket_receives_its_team_frame() {
    // M-52/M-65: each Team sees its purchased Troop, never the enemy base purchase.
    let url = start().await;
    let (mut blue, wb) = join(&url).await;
    let (mut red, wr) = join(&url).await;
    assert_ne!(wb.seat.team(), wr.seat.team());
    for (socket, seat) in [(&mut blue, wb.seat), (&mut red, wr.seat)] {
        let mut affordable = None;
        for _ in 0..20 {
            if let Some(ServerMsg::Private(p)) = next(socket).await {
                affordable = p
                    .offers
                    .iter()
                    .position(|k| p.gold >= tbg_moba_core::content::troop_cost(*k));
                break;
            }
        }
        let slot = affordable.expect("seed 11 provides an affordable opening offer") as u8;
        let buy = ClientMsg::Intention(MobaIntention::BuyTroop {
            slot,
            to: Destination::Lane(Lane::Top),
        });
        socket
            .send(Message::Binary(protocol::encode(&buy).into()))
            .await
            .unwrap();
        let mut checked = false;
        for _ in 0..30 {
            if let Some(ServerMsg::World(w)) = next(socket).await {
                assert!(w.tick < 100, "fixture must stay near initial spawns");
                assert!(!w
                    .units
                    .iter()
                    .any(|u| matches!(u.kind, tbg_moba_core::UnitKind::Commander(_))));
                let troops: Vec<_> = w
                    .units
                    .iter()
                    .filter(|u| matches!(u.kind, tbg_moba_core::UnitKind::Troop(_)))
                    .collect();
                assert!(troops.iter().all(|u| u.side.team() == Some(seat.team())));
                if troops.iter().any(|u| u.owner == Some(seat)) {
                    checked = true;
                    break;
                }
            }
        }
        assert!(checked, "socket did not receive its team's purchased Troop");
    }
}

#[tokio::test]
async fn m65_socket_cannot_send_manual_unit_orders() {
    // M-41/M-65: a legacy movement intention is valid wire data but never an allowed action.
    let url = start().await;
    let (mut socket, _) = join(&url).await;
    socket
        .send(Message::Binary(
            protocol::encode(&ClientMsg::Intention(MobaIntention::MoveCommander {
                to: tbg_moba_core::geom::Pos {
                    x: i32::MAX,
                    y: i32::MIN,
                },
            }))
            .into(),
        ))
        .await
        .unwrap();
    let mut rejected = false;
    for _ in 0..20 {
        if let Some(ServerMsg::Rejected(tbg_moba_core::Reject::ManualControlDisabled)) =
            next(&mut socket).await
        {
            rejected = true;
            break;
        }
    }
    assert!(
        rejected,
        "server must reject manual movement without closing or applying it"
    );
}
