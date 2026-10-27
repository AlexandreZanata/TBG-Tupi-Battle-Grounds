//! `tbg-server` P1 spike: health check and binary echo WebSocket.
//!
//! `GET /health` answers `ok`; `GET /ws` echoes binary frames back.
//! Matchmaking and the authoritative loop arrive in later phases.

use axum::{
    extract::{
        WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::get,
    Router,
};

#[tokio::main]
async fn main() {
    let addr = std::env::var("TBG_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/ws", get(ws_handler));
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("tbg-server: cannot bind {addr}: {e}"));
    println!("tbg-server listening on {addr}");
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("tbg-server: serve failed: {e}"));
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.max_message_size(1024)
        .max_frame_size(1024)
        .on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Binary(bytes)) => {
                if socket.send(Message::Binary(bytes)).await.is_err() {
                    break;
                }
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }
}
