//! One WebSocket: handshake, bounded and rate-limited reads, buffered writes (M-40..M-42).

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use tbg_moba_core::protocol::{
    self, ClientMsg, ServerMsg, MAX_CLIENT_FRAME, MAX_CLIENT_FRAMES_PER_SECOND,
    MOBA_PROTOCOL_VERSION,
};
use tokio::sync::{mpsc, oneshot};

use crate::room::{Command, ConnId, RoomHandle, OUTBOX};

/// Time allowed for the Hello frame.
const HELLO_TIMEOUT: Duration = Duration::from_secs(5);
/// A socket silent this long is closed (clients ping every 2 s).
const IDLE_TIMEOUT: Duration = Duration::from_secs(30);

static NEXT_CONN: AtomicU64 = AtomicU64::new(1);

pub async fn upgrade(ws: WebSocketUpgrade, State(room): State<RoomHandle>) -> Response {
    ws.max_message_size(MAX_CLIENT_FRAME)
        .max_frame_size(MAX_CLIENT_FRAME)
        .on_upgrade(move |socket| handle(socket, room))
}

async fn handle(socket: WebSocket, room: RoomHandle) {
    let conn = NEXT_CONN.fetch_add(1, Ordering::Relaxed);
    let (mut sink, mut stream) = socket.split();
    if !hello(&mut stream).await {
        return;
    }
    let (outbox, mut frames) = mpsc::channel::<Bytes>(OUTBOX);
    let (reply, welcome) = oneshot::channel();
    let join = Command::Join {
        conn,
        outbox: outbox.clone(),
        reply,
    };
    if room.commands.send(join).await.is_err() {
        return;
    }
    let Ok(Some(welcome)) = welcome.await else {
        let _ = sink
            .send(Message::Binary(protocol::encode(&ServerMsg::Full).into()))
            .await;
        return;
    };
    let _ = outbox.try_send(protocol::encode(&ServerMsg::Welcome(welcome)).into());
    let writer = tokio::spawn(async move {
        while let Some(frame) = frames.recv().await {
            if sink.send(Message::Binary(frame)).await.is_err() {
                break;
            }
        }
    });
    read_loop(conn, &mut stream, &room, &outbox).await;
    let _ = room.commands.send(Command::Leave { conn }).await;
    writer.abort();
}

/// Waits for a well-formed Hello with the current protocol version.
async fn hello(stream: &mut futures_util::stream::SplitStream<WebSocket>) -> bool {
    let Ok(Some(Ok(Message::Binary(bytes)))) =
        tokio::time::timeout(HELLO_TIMEOUT, stream.next()).await
    else {
        return false;
    };
    matches!(protocol::decode_client(&bytes), Ok(ClientMsg::Hello { protocol, .. }) if protocol == MOBA_PROTOCOL_VERSION)
}

async fn read_loop(
    conn: ConnId,
    stream: &mut futures_util::stream::SplitStream<WebSocket>,
    room: &RoomHandle,
    outbox: &mpsc::Sender<Bytes>,
) {
    let mut window = Instant::now();
    let mut frames_in_window = 0u32;
    loop {
        let Ok(Some(Ok(message))) = tokio::time::timeout(IDLE_TIMEOUT, stream.next()).await else {
            return;
        };
        let bytes = match message {
            Message::Binary(bytes) => bytes,
            Message::Close(_) => return,
            // Text frames are not part of the protocol; ping/pong are handled by axum.
            Message::Text(_) => return,
            _ => continue,
        };
        if window.elapsed() >= Duration::from_secs(1) {
            window = Instant::now();
            frames_in_window = 0;
        }
        frames_in_window += 1;
        if frames_in_window > MAX_CLIENT_FRAMES_PER_SECOND {
            tracing::warn!(conn, "rate limit exceeded; closing");
            return;
        }
        match protocol::decode_client(&bytes) {
            Ok(ClientMsg::Intention(intention)) => {
                if room
                    .commands
                    .send(Command::Intention { conn, intention })
                    .await
                    .is_err()
                {
                    return;
                }
            }
            Ok(ClientMsg::Ping(t)) => {
                let _ = outbox.try_send(protocol::encode(&ServerMsg::Pong(t)).into());
            }
            Ok(ClientMsg::Hello { .. }) => {}
            Err(error) => {
                tracing::warn!(conn, ?error, "malformed frame; closing");
                return;
            }
        }
    }
}
