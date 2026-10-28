//! One task per WebSocket: admission caps, handshake, rate limit, ping/RTT,
//! idle timeout, and routing. Stamps the seat on every intention (BR-A02).

use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::Ordering::Relaxed;
use std::time::{Duration, Instant};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{ConnectInfo, State};
use axum::http::HeaderMap;
use axum::response::Response;
use tokio::sync::{mpsc, oneshot};
use tokio::time::{sleep, timeout};

use tbg_core::protocol::{self, ClientMsg, ErrorCode, ServerMsg, Token, MAX_CLIENT_MSG_BYTES};
use tbg_core::Seat;

use crate::chan::{ConnEvent, ConnHandle, LobbyCmd, MatchCmd, CONN_QUEUE};

mod admission;
mod client_msg;
use crate::metrics::{percentile, Metrics};
use crate::AppState;
use admission::{client_ip, Slot};
use client_msg::{on_client_bytes, send};

const HANDSHAKE: Duration = Duration::from_secs(5);
const IDLE: Duration = Duration::from_secs(15);
const BUCKET_CAP: f64 = 40.0;
const BUCKET_REFILL_PER_S: f64 = 20.0;

pub(crate) async fn handler(
    ws: WebSocketUpgrade,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(app): State<AppState>,
) -> Response {
    let ip = client_ip(peer, &headers);
    ws.max_message_size(MAX_CLIENT_MSG_BYTES)
        .max_frame_size(MAX_CLIENT_MSG_BYTES)
        .on_upgrade(move |socket| connection(socket, ip, app))
}

pub(super) struct Conn {
    pub(super) token: Token,
    pub(super) matched: Option<(Seat, mpsc::Sender<MatchCmd>)>,
    pub(super) tokens: f64,
    pub(super) refilled: Instant,
    pub(super) pings: Vec<(u32, Instant)>,
    pub(super) next_ping: u32,
    pub(super) rtts: Vec<u16>,
    pub(super) bytes_in: u64,
    pub(super) bytes_out: u64,
    pub(super) started: Instant,
}

async fn connection(mut socket: WebSocket, ip: IpAddr, app: AppState) {
    let Some(_slot) = Slot::acquire(&app, ip) else {
        let err = ServerMsg::Error {
            code: ErrorCode::TooManyConnections,
        };
        let _ = socket
            .send(Message::Binary(protocol::encode(&err).into()))
            .await;
        return;
    };
    let id = app.next_conn_id.fetch_add(1, Relaxed);
    let (tx, mut rx) = mpsc::channel(CONN_QUEUE);
    let Some(token) = handshake(&mut socket, &app, ConnHandle { id, tx }).await else {
        return;
    };
    let mut c = Conn {
        token,
        matched: None,
        tokens: BUCKET_CAP,
        refilled: Instant::now(),
        pings: Vec::new(),
        next_ping: 0,
        rtts: Vec::new(),
        bytes_in: 0,
        bytes_out: 0,
        started: Instant::now(),
    };
    let reason = run_loop(&mut socket, &mut rx, &mut c, &app).await;
    if let Some((seat, m)) = &c.matched {
        let _ = m
            .send(MatchCmd::Detach {
                seat: *seat,
                conn_id: id,
            })
            .await;
    }
    let _ = app
        .lobby
        .send(LobbyCmd::Disconnected { token, conn_id: id })
        .await;
    c.rtts.sort_unstable();
    tracing::info!(
        id,
        secs = c.started.elapsed().as_secs(),
        bytes_in = c.bytes_in,
        bytes_out = c.bytes_out,
        rtt_p50 = percentile(&c.rtts, 0.5),
        rtt_p95 = percentile(&c.rtts, 0.95),
        reason,
        "conn closed"
    );
}

/// First message must be `Hello` within 5 s with our protocol version.
async fn handshake(socket: &mut WebSocket, app: &AppState, conn: ConnHandle) -> Option<Token> {
    let first = timeout(HANDSHAKE, socket.recv()).await.ok()??.ok()?;
    let Message::Binary(bytes) = first else {
        return None;
    };
    let Ok(ClientMsg::Hello { version, resume }) = protocol::decode_client(&bytes) else {
        let _ = send(
            socket,
            &app.metrics,
            &ServerMsg::Error {
                code: ErrorCode::BadMessage,
            },
        )
        .await;
        return None;
    };
    if version != protocol::PROTOCOL_VERSION {
        let reply = ServerMsg::VersionMismatch {
            server: protocol::PROTOCOL_VERSION,
        };
        let _ = send(socket, &app.metrics, &reply).await;
        return None;
    }
    let (reply, rx) = oneshot::channel();
    app.lobby
        .send(LobbyCmd::Hello {
            resume,
            conn,
            reply,
        })
        .await
        .ok()?;
    let (token, name) = rx.await.ok()?;
    send(socket, &app.metrics, &ServerMsg::Welcome { token, name })
        .await
        .ok()?;
    Some(token)
}

async fn run_loop(
    socket: &mut WebSocket,
    rx: &mut mpsc::Receiver<ConnEvent>,
    c: &mut Conn,
    app: &AppState,
) -> &'static str {
    let mut ping = tokio::time::interval(Duration::from_secs(1));
    let idle = sleep(IDLE);
    tokio::pin!(idle);
    loop {
        let reply = tokio::select! {
            msg = socket.recv() => match msg {
                Some(Ok(Message::Binary(bytes))) => {
                    idle.as_mut().reset(tokio::time::Instant::now() + IDLE);
                    c.bytes_in += bytes.len() as u64;
                    Metrics::add(&app.metrics.bytes_in, bytes.len() as u64);
                    match on_client_bytes(c, app, &bytes) {
                        Ok(reply) => reply,
                        Err(code) => {
                            let _ = send(socket, &app.metrics, &ServerMsg::Error { code }).await;
                            return "bad_message";
                        }
                    }
                }
                Some(Ok(Message::Close(_))) | None => return "client_closed",
                Some(Err(_)) => return "socket_error",
                Some(Ok(_)) => None,
            },
            ev = rx.recv() => match ev {
                Some(ConnEvent::Send(msg)) => Some(msg),
                Some(ConnEvent::EnterMatch { seat, tx }) => { c.matched = Some((seat, tx)); None }
                Some(ConnEvent::LeaveMatch) => { c.matched = None; None }
                Some(ConnEvent::Close) | None => return "server_closed",
            },
            _ = ping.tick() => {
                c.next_ping += 1;
                c.pings.push((c.next_ping, Instant::now()));
                if c.pings.len() > 8 { c.pings.remove(0); }
                Some(ServerMsg::Ping { id: c.next_ping })
            }
            _ = &mut idle => return "idle_timeout",
        };
        if let Some(msg) = reply {
            match send(socket, &app.metrics, &msg).await {
                Ok(n) => c.bytes_out += n,
                Err(_) => return "send_failed",
            }
        }
    }
}
