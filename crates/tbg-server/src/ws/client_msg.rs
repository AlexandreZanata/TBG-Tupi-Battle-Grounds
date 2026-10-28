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
    Ok(match msg {
        ClientMsg::Hello { .. } => None,
        ClientMsg::Intent { seq, intent } => match &c.matched {
            Some((seat, m)) => {
                let _ = m.try_send(MatchCmd::Intent {
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
                let _ = m.try_send(MatchCmd::Surrender { seat: *seat });
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
    socket.send(Message::Binary(bytes.into())).await.map(|_| n)
}
