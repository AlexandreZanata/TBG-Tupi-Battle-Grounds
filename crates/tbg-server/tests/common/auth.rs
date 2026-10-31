//! Synthetic HTTP issuance and real socket attachment for D-97 contract fixtures.
#![allow(dead_code)]
use futures_util::SinkExt;
use serde_json::json;
use tbg_core::protocol::{self, AccountError, ClientMsg};
use tbg_server::botclient::native_auth::{self, Grant};
pub type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;
fn origin(ws: &Ws) -> String {
    let tokio_tungstenite::MaybeTlsStream::Plain(stream) = ws.get_ref() else {
        panic!("loopback fixture requires plain TCP");
    };
    format!("ws://{}/ws", stream.peer_addr().unwrap())
}
pub async fn register(ws: &Ws, name: &str) -> Result<Grant, AccountError> {
    let body = json!({"name":name,"device_id":native_auth::random_hex(16).unwrap(),"persistent":false,"registration_retry_key":native_auth::random_hex(32).unwrap()});
    let (status, value) = native_auth::post(&origin(ws), "register", body, None)
        .await
        .unwrap();
    if status == 200 {
        Ok(native_auth::parse(&value).unwrap())
    } else {
        Err(match value["error"].as_str() {
            Some("name_taken") => AccountError::NameTaken,
            Some("name_invalid") => AccountError::NameInvalid,
            Some("rate_limited") => AccountError::RateLimited,
            _ => AccountError::Unavailable,
        })
    }
}
pub async fn login(ws: &Ws, key: &str) -> Result<Grant, AccountError> {
    let (status,value)=native_auth::post(&origin(ws),"login",json!({"account_key":key,"device_id":native_auth::random_hex(16).unwrap(),"persistent":false}),None).await.unwrap();
    if status == 200 {
        Ok(native_auth::parse(&value).unwrap())
    } else {
        Err(if value["error"] == "rate_limited" {
            AccountError::RateLimited
        } else {
            AccountError::BadKey
        })
    }
}
pub async fn attach(ws: &mut Ws, grant: &Grant, take_over: bool) {
    send(
        ws,
        ClientMsg::Reauthenticate {
            access: grant.access,
        },
    )
    .await;
    if take_over {
        send(ws, ClientMsg::ClaimAccount { take_over: true }).await;
    }
}
pub async fn send(ws: &mut Ws, msg: ClientMsg) {
    ws.send(tokio_tungstenite::tungstenite::Message::Binary(
        protocol::encode(&msg).into(),
    ))
    .await
    .unwrap();
}
