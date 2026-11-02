//! Real local WebSocket helpers shared by P75 contract scenarios.
#![allow(dead_code)]
use futures_util::{SinkExt, StreamExt};
use tbg_core::protocol::{self, ClientMsg, ServerMsg, PROTOCOL_VERSION};
pub type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;
pub async fn send(ws: &mut Ws, msg: ClientMsg) {
    ws.send(tokio_tungstenite::tungstenite::Message::Binary(
        protocol::encode(&msg).into(),
    ))
    .await
    .unwrap();
}
pub async fn recv(ws: &mut Ws) -> ServerMsg {
    loop {
        let m = tokio::time::timeout(std::time::Duration::from_secs(5), ws.next())
            .await
            .expect("socket timeout")
            .expect("socket closed")
            .unwrap();
        if let tokio_tungstenite::tungstenite::Message::Binary(b) = m {
            let m = protocol::decode_server(&b).unwrap();
            match m {
                ServerMsg::Ping { id } => send(ws, ClientMsg::Pong { id }).await,
                ServerMsg::NetStats { .. } => {}
                other => return other,
            }
        }
    }
}
pub async fn until(ws: &mut Ws, p: impl Fn(&ServerMsg) -> bool) -> ServerMsg {
    loop {
        let m = recv(ws).await;
        if p(&m) {
            return m;
        }
    }
}
pub async fn connect(url: &str, grant: Option<&serde_json::Value>) -> Ws {
    let url = url.replacen("http://", "ws://", 1) + "/ws";
    let mut ws = tokio_tungstenite::connect_async(url).await.unwrap().0;
    let hello = if let Some(g) = grant {
        ClientMsg::AuthenticatedHello {
            version: PROTOCOL_VERSION,
            resume: None,
            access: tbg_server::botclient::native_auth::parse(g).unwrap().access,
        }
    } else {
        ClientMsg::Hello {
            version: PROTOCOL_VERSION,
            resume: None,
        }
    };
    send(&mut ws, hello).await;
    until(&mut ws, |m| matches!(m, ServerMsg::Welcome { .. })).await;
    ws
}
