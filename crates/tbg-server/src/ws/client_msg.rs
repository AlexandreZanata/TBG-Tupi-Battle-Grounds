//! One validated client frame: rate limit, decode, route; pong → RTT.

use std::time::{Duration, Instant};

use axum::extract::ws::{Message, WebSocket};

use tbg_core::protocol::{self, ClientMsg, ErrorCode, ServerMsg};

use super::{Conn, BUCKET_CAP, BUCKET_REFILL_PER_S};
use crate::chan::{LobbyCmd, MatchCmd};
use crate::metrics::Metrics;
use crate::AppState;

const LEADERBOARD_EVERY: Duration = Duration::from_secs(1);

/// Validate one client frame; `Err` closes the connection.
pub(super) fn on_client_bytes(
    c: &mut Conn,
    app: &AppState,
    bytes: &[u8],
) -> Result<Option<ServerMsg>, ErrorCode> {
    let now = Instant::now();
    c.tokens = (c.tokens + now.duration_since(c.refilled).as_secs_f64() * BUCKET_REFILL_PER_S)
        .min(BUCKET_CAP);
    c.refilled = now;
    if c.tokens < 1.0 {
        return Err(ErrorCode::RateLimited);
    }
    c.tokens -= 1.0;
    let msg = protocol::decode_client(bytes).map_err(|_| ErrorCode::BadMessage)?;
    if c.binding.as_ref().is_some_and(|b| b.revoked()) {
        return Err(ErrorCode::LoginRequired);
    }
    match msg {
        ClientMsg::Reauthenticate { access } => return Ok(reauthenticate(c, app, access)),
        ClientMsg::ClaimAccount { take_over } => return Ok(claim(c, app, take_over)),
        ClientMsg::Register { .. } | ClientMsg::Login { .. } | ClientMsg::Logout => {
            return Ok(Some(required(protocol::AuthRefusal::Invalid)))
        }
        _ => {}
    }
    if let Some(reply) = authorize(c, app, &msg) {
        return Ok(Some(reply));
    }
    Ok(match msg {
        ClientMsg::Hello { .. } | ClientMsg::AuthenticatedHello { .. } => None,
        ClientMsg::Intent { seq, intent } => match &c.matched {
            Some((seat, m)) => {
                let _ = m.try_send(MatchCmd::Intent {
                    conn_id: c.id,
                    seat: *seat,
                    seq,
                    intent,
                });
                None
            }
            None => Some(ServerMsg::Error {
                code: ErrorCode::NotInMatch,
            }),
        },
        // BR-M06: same rate limit and seat stamping as an intention.
        ClientMsg::Surrender => match &c.matched {
            Some((seat, m)) => {
                let _ = m.try_send(MatchCmd::Surrender {
                    seat: *seat,
                    conn_id: c.id,
                });
                None
            }
            None => Some(ServerMsg::Error {
                code: ErrorCode::NotInMatch,
            }),
        },
        ClientMsg::Pong { id } => on_pong(c, app, id, now),
        // P13: one leaderboard request per second per connection.
        ClientMsg::Leaderboard { .. }
            if c.last_leaderboard
                .is_some_and(|t| now.duration_since(t) < LEADERBOARD_EVERY) =>
        {
            None
        }
        other => {
            if matches!(other, ClientMsg::Leaderboard { .. }) {
                c.last_leaderboard = Some(now);
            }
            let cmd = LobbyCmd::Client {
                token: c.token,
                msg: other,
            };
            app.lobby.try_send(cmd).err().map(|_| ServerMsg::Error {
                code: ErrorCode::ServerFull,
            })
        }
    })
}

fn required(reason: protocol::AuthRefusal) -> ServerMsg {
    ServerMsg::AuthRequired { reason }
}

fn authorize(c: &Conn, app: &AppState, msg: &ClientMsg) -> Option<ServerMsg> {
    if matches!(
        msg,
        ClientMsg::Pong { .. }
            | ClientMsg::Leaderboard { .. }
            | ClientMsg::Cancel
            | ClientMsg::StopSpectating
    ) {
        return None;
    }
    if !c.active.load(std::sync::atomic::Ordering::Acquire) {
        return Some(required(protocol::AuthRefusal::Invalid));
    }
    let Some(binding) = &c.binding else {
        return None;
    };
    // The already admitted seat may finish only this match after natural expiry.
    if c.matched.is_some() && matches!(msg, ClientMsg::Intent { .. } | ClientMsg::Surrender) {
        return None;
    }
    (!app.auth.now().is_ok_and(|now| binding.valid(now)))
        .then(|| required(protocol::AuthRefusal::Expired))
}

fn reauthenticate(
    c: &mut Conn,
    app: &AppState,
    access: protocol::AccessToken,
) -> Option<ServerMsg> {
    let Ok(binding) = app.auth.authenticate(access.0) else {
        return Some(required(protocol::AuthRefusal::Invalid));
    };
    if c.binding
        .as_ref()
        .is_some_and(|old| old.account != binding.account || old.family != binding.family)
        || (c.binding.is_none() && c.matched.is_some())
    {
        return Some(required(protocol::AuthRefusal::Invalid));
    }
    if c.binding.is_none() {
        c.active.store(false, std::sync::atomic::Ordering::Release);
    }
    c.binding = Some(binding);
    c.expired_notified = false;
    claim(c, app, false)
}

fn claim(c: &Conn, app: &AppState, take_over: bool) -> Option<ServerMsg> {
    let Some(binding) = &c.binding else {
        return Some(required(protocol::AuthRefusal::Invalid));
    };
    if !app.auth.now().is_ok_and(|now| binding.valid(now)) {
        return Some(required(protocol::AuthRefusal::Expired));
    }
    app.lobby
        .try_send(LobbyCmd::AuthClaim {
            token: c.token,
            conn_id: c.id,
            binding: binding.clone(),
            take_over,
        })
        .err()
        .map(|_| ServerMsg::Error {
            code: ErrorCode::ServerFull,
        })
}

fn on_pong(c: &mut Conn, app: &AppState, id: u32, now: Instant) -> Option<ServerMsg> {
    let i = c.pings.iter().position(|(p, _)| *p == id)?;
    let (_, sent) = c.pings.remove(i);
    let ms = now
        .duration_since(sent)
        .as_millis()
        .min(u128::from(u16::MAX)) as u16;
    c.rtts.push(ms);
    app.metrics.record_rtt(ms);
    if let Some((seat, m)) = &c.matched {
        let _ = m.try_send(MatchCmd::Rtt { seat: *seat, ms });
    }
    Some(ServerMsg::NetStats { rtt_ms: ms })
}

pub(super) async fn send(
    socket: &mut WebSocket,
    metrics: &Metrics,
    msg: &ServerMsg,
) -> Result<u64, axum::Error> {
    let bytes = protocol::encode(msg);
    let n = bytes.len() as u64;
    Metrics::add(&metrics.bytes_out, n);
    if matches!(msg, ServerMsg::View(_)) {
        Metrics::add(&metrics.view_messages, 1);
        Metrics::add(&metrics.view_bytes, n);
    }
    socket.send(Message::Binary(bytes.into())).await.map(|_| n)
}
