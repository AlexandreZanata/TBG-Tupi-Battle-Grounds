//! React to one server message: queue, play with the bot, answer pings.

use tokio::time::Instant;

use tbg_core::protocol::{ClientMsg, ServerMsg};

use super::{BotOpts, BotResult, Session, Step};

pub(super) fn handle(
    msg: ServerMsg,
    opts: &BotOpts,
    s: &mut Session,
    r: &mut BotResult,
    send: &mut impl FnMut(&mut Session, ClientMsg),
) -> Result<Option<Step>, String> {
    match msg {
        ServerMsg::Welcome { token, .. } => {
            if s.token.replace(token).is_none() {
                send(s, ClientMsg::QuickMatch);
            }
        }
        ServerMsg::MatchStart { you, .. } => {
            if r.seat.is_none() {
                s.drop_at = opts.drop_after.map(|d| Instant::now() + d);
            }
            r.seat = Some(you);
        }
        ServerMsg::View(v) => {
            let now = Instant::now();
            if let Some(prev) = s.last_view.replace(now) {
                r.view_gaps_ms
                    .push(now.duration_since(prev).as_millis() as u32);
            }
            if let Some(intent) = s.bot.decide(&v) {
                s.seq += 1;
                let seq = s.seq;
                send(s, ClientMsg::Intent { seq, intent });
            }
        }
        ServerMsg::Rejected { .. } => r.rejections += 1,
        ServerMsg::MatchEnd { outcome } => {
            r.outcome = Some(outcome);
            r.completed = true;
            return Ok(Some(Step::Done));
        }
        ServerMsg::Ping { id } => send(s, ClientMsg::Pong { id }),
        ServerMsg::NetStats { rtt_ms } => r.rtt_ms.push(rtt_ms),
        ServerMsg::VersionMismatch { server } => {
            return Err(format!("version mismatch (server {server})"))
        }
        ServerMsg::Error { code } => return Err(format!("server error {code:?}")),
        ServerMsg::Queued | ServerMsg::RoomCreated { .. } | ServerMsg::RoomNotFound => {}
    }
    Ok(None)
}
