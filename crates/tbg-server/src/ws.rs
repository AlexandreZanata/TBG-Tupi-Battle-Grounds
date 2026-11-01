//! One task per WebSocket: admission caps, handshake, rate limit, ping/RTT,
//! idle timeout, and routing. Stamps the seat on every intention (BR-A02).

use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::Ordering::Relaxed;
use std::time::{Duration, Instant};

use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{ConnectInfo, Path, RawQuery, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{sleep, timeout};

use tbg_core::protocol::{self, ClientMsg, ErrorCode, ServerMsg, Token, MAX_CLIENT_MSG_BYTES};
use tbg_core::Seat;

use crate::chan::{ConnEvent, ConnHandle, LobbyCmd, MatchCmd, CONN_QUEUE};

mod admission;
pub(crate) use admission::client_ip;
mod client_msg;
use crate::metrics::{percentile, Metrics};
use crate::AppState;
use admission::Slot;
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
    RawQuery(query): RawQuery,
) -> Response {
    upgrade(ws, peer, headers, app, query)
}

/// P66: the fixed path of one release slot (`/ws/slot-a`). A request for
/// another slot's path never reaches a match here (proxy misrouting).
pub(crate) async fn slot_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(app): State<AppState>,
    Path(slot): Path<String>,
    RawQuery(query): RawQuery,
) -> Response {
    let mine = app
        .release
        .slot()
        .is_some_and(|s| slot.strip_prefix("slot-") == Some(s));
    if !mine {
        return StatusCode::NOT_FOUND.into_response();
    }
    upgrade(ws, peer, headers, app, query)
}

/// P66 §6.5: a client that names its match contract must match this
/// process exactly; a client that names none is a legacy build and keeps
/// the protocol-version check of the handshake.
fn contract_mismatch(app: &AppState, query: Option<&str>) -> bool {
    let named = query
        .unwrap_or_default()
        .split('&')
        .find_map(|pair| pair.strip_prefix("contract="));
    named.is_some_and(|c| c != app.release.contract.id())
}

fn upgrade(
    ws: WebSocketUpgrade,
    peer: SocketAddr,
    headers: HeaderMap,
    app: AppState,
    query: Option<String>,
) -> Response {
    // D-97: every upgrade path, including slot paths, checks the browser Origin.
    if !crate::auth::websocket_origin(&headers, &app.config.auth_origins) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let mismatch = contract_mismatch(&app, query.as_deref());
    let ip = client_ip(peer, &headers);
    ws.max_message_size(MAX_CLIENT_MSG_BYTES)
        .max_frame_size(MAX_CLIENT_MSG_BYTES)
        // Inbound messages are capped at 1 KiB; the library's default
        // 128 KiB read allocation per socket buys no larger legal frame.
        .read_buffer_size(4096)
        // P10: small write buffer, bounded pending bytes (backpressure
        // stays visible as slow-client drops instead of memory growth).
        .write_buffer_size(4096)
        .max_write_buffer_size(64 * 1024)
        .on_upgrade(move |socket| async move {
            if mismatch {
                refuse_contract(socket, &app).await;
            } else {
                connection(socket, ip, app).await;
            }
        })
}

/// Reply like an incompatible protocol so every client build shows its
/// update path, then close before any session or seat exists.
async fn refuse_contract(mut socket: WebSocket, app: &AppState) {
    let reply = ServerMsg::VersionMismatch {
        server: protocol::PROTOCOL_VERSION,
    };
    let _ = send(&mut socket, &app.metrics, &reply).await;
    app.metrics.admit_update_required.fetch_add(1, Relaxed);
}

pub(super) struct Conn {
    pub(super) id: u64,
    pub(super) binding: Option<crate::auth::Binding>,
    pub(super) active: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub(super) link: std::sync::Arc<crate::chan::Link>,
    pub(super) expired_notified: bool,
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
    /// P13: last `Leaderboard` request (one per second; extras ignored).
    pub(super) last_leaderboard: Option<Instant>,
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
    let handle = ConnHandle::new(id, tx);
    let active = handle.active.clone();
    let link = handle.link.clone();
    let Some((token, binding)) = handshake(&mut socket, &app, handle, ip).await else {
        return;
    };
    let mut c = Conn {
        id,
        binding,
        active,
        link,
        expired_notified: false,
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
        last_leaderboard: None,
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
async fn handshake(
    socket: &mut WebSocket,
    app: &AppState,
    conn: ConnHandle,
    ip: IpAddr,
) -> Option<(Token, Option<crate::auth::Binding>)> {
    let first = timeout(HANDSHAKE, socket.recv()).await.ok()??.ok()?;
    let Message::Binary(bytes) = first else {
        return None;
    };
    let (version, resume, access) = match protocol::decode_client(&bytes) {
        Ok(ClientMsg::Hello { version, resume }) => (version, resume, None),
        Ok(ClientMsg::AuthenticatedHello {
            version,
            resume,
            access,
        }) => (version, resume, Some(access)),
        _ => {
            let _ = send(
                socket,
                &app.metrics,
                &ServerMsg::Error {
                    code: ErrorCode::BadMessage,
                },
            )
            .await;
            return None;
        }
    };
    // Version is checked before Access lookup or any resume/profile/seat side effect.
    if version != protocol::PROTOCOL_VERSION {
        let _ = send(
            socket,
            &app.metrics,
            &ServerMsg::VersionMismatch {
                server: protocol::PROTOCOL_VERSION,
            },
        )
        .await;
        return None;
    }
    let (binding, lease) = match access {
        Some(access) => match app.auth.authenticate(access.0) {
            Ok(binding) => (Some(binding), false),
            Err(_) => match resume.and_then(|_| app.auth.authenticate_lease(access.0).ok()) {
                // P71 (BR-AS07): a drop that outlasted the Access takes its seat back.
                Some(binding) => (Some(binding), true),
                None => {
                    let _ = send(
                        socket,
                        &app.metrics,
                        &ServerMsg::AuthRequired {
                            reason: protocol::AuthRefusal::Invalid,
                        },
                    )
                    .await;
                    return None;
                }
            },
        },
        None => (None, false),
    };
    let (reply, rx) = oneshot::channel();
    let command = if let Some(binding) = binding.clone() {
        LobbyCmd::AuthenticatedHello {
            resume,
            conn,
            ip,
            binding,
            lease,
            reply,
        }
    } else {
        LobbyCmd::Hello {
            resume,
            conn,
            ip,
            reply,
        }
    };
    app.lobby.send(command).await.ok()?;
    let Some((token, name)) = rx.await.ok()? else {
        let _ = send(
            socket,
            &app.metrics,
            &ServerMsg::AuthRequired {
                reason: protocol::AuthRefusal::Invalid,
            },
        )
        .await;
        return None;
    };
    send(socket, &app.metrics, &ServerMsg::Welcome { token, name })
        .await
        .ok()?;
    Some((token, binding))
}

async fn run_loop(
    socket: &mut WebSocket,
    rx: &mut mpsc::Receiver<ConnEvent>,
    c: &mut Conn,
    app: &AppState,
) -> &'static str {
    let mut ping = tokio::time::interval(Duration::from_secs(1));
    // P9: protocol-level pings keep a hidden tab's socket alive; browsers
    // answer them even while the frame loop (and app-level pongs) is paused.
    let mut proto_ping = tokio::time::interval(Duration::from_secs(5));
    let idle = sleep(IDLE);
    tokio::pin!(idle);
    loop {
        if c.link.close_requested() {
            return "server_closed";
        }
        if c.binding.as_ref().is_some_and(|b| b.revoked()) {
            let _ = send(
                socket,
                &app.metrics,
                &ServerMsg::AuthRequired {
                    reason: protocol::AuthRefusal::Revoked,
                },
            )
            .await;
            return "auth_revoked";
        }
        let reply = tokio::select! {
            msg = socket.recv() => match msg {
                Some(Ok(Message::Binary(bytes))) => {
                    idle.as_mut().reset(tokio::time::Instant::now() + IDLE);
                    c.link.heard();
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
                Some(Ok(Message::Ping(payload))) => {
                    idle.as_mut().reset(tokio::time::Instant::now() + IDLE);
                    c.link.heard();
                    let _ = socket.send(Message::Pong(payload)).await;
                    None
                }
                Some(Ok(Message::Pong(_))) => {
                    idle.as_mut().reset(tokio::time::Instant::now() + IDLE);
                    c.link.heard();
                    None
                }
                Some(Ok(Message::Close(_))) | None => return "client_closed",
                Some(Err(_)) => return "socket_error",
                Some(Ok(_)) => None,
            },
            ev = rx.recv() => match ev {
                Some(ConnEvent::Send(msg)) => Some(msg),
                // P47: a shared, pre-encoded spectator view; never re-encoded.
                Some(ConnEvent::Frame(bytes)) => {
                    let n = bytes.len() as u64;
                    if socket.send(Message::Binary(bytes)).await.is_err() {
                        return "send_failed";
                    }
                    c.bytes_out += n;
                    Metrics::add(&app.metrics.bytes_out, n);
                    None
                }
                Some(ConnEvent::EnterMatch { seat, tx }) => { c.matched = Some((seat, tx)); None }
                Some(ConnEvent::LeaveMatch) => { c.matched = None; None }
                Some(ConnEvent::Close) | None => return "server_closed",
            },
            _ = ping.tick() => {
                if !c.expired_notified && c.binding.as_ref().is_some_and(|b|!app.auth.now().is_ok_and(|now|b.valid(now))) {
                    c.expired_notified=true;
                    let _=send(socket,&app.metrics,&ServerMsg::AuthRequired{reason:protocol::AuthRefusal::Expired}).await;
                }
                c.next_ping += 1;
                c.pings.push((c.next_ping, Instant::now()));
                if c.pings.len() > 8 { c.pings.remove(0); }
                Some(ServerMsg::Ping { id: c.next_ping })
            }
            _ = proto_ping.tick() => {
                if socket.send(Message::Ping(Bytes::new())).await.is_err() {
                    return "send_failed";
                }
                None
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
