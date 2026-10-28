//! React to one server message: queue, play with the bot, answer pings.

use tokio::time::Instant;

use tbg_core::protocol::{ClientMsg, ServerMsg};

use super::{BotOpts, BotResult, Session, Step};

/// Resolve every pending seq acknowledged by `last_seq` into `ack_ms`.
fn resolve_ack(s: &mut Session, r: &mut BotResult, last_seq: u32, now: Instant) {
    let mut i = 0;
    while i < s.pending.len() {
        if s.pending[i].0 <= last_seq {
            let (_, sent) = s.pending.remove(i);
            let ms = now
                .duration_since(sent)
                .as_millis()
                .min(u128::from(u16::MAX)) as u16;
            r.ack_ms.push(ms);
        } else {
            i += 1;
        }
    }
}

pub(super) fn handle(
    msg: ServerMsg,
    opts: &BotOpts,
    s: &mut Session,
    r: &mut BotResult,
    send: &mut impl FnMut(&mut Session, ClientMsg),
) -> Result<Option<Step>, String> {
    /// P11: `Tb` + 10 random alphanumerics (12 chars, starts with a letter).
    fn bot_name(s: &mut Session) -> String {
        const ALPHA: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut name = String::from("Tb");
        for _ in 0..10 {
            name.push(ALPHA[s.rng.below(62) as usize] as char);
        }
        name
    }

    match msg {
        ServerMsg::Welcome { token, .. } => {
            if s.token.replace(token).is_none() {
                if (opts.accounts || opts.ranked) && !s.registering {
                    s.registering = true;
                    let name = bot_name(s);
                    send(s, ClientMsg::Register { name });
                } else {
                    s.queue_start = Some(Instant::now());
                    send(s, ClientMsg::QuickMatch);
                }
            }
        }
        // P11: registered (and auto-logged-in); queue for real this time.
        // P12: ranked bots join the ranked queue right away.
        ServerMsg::Registered { .. } => {
            s.queue_start = Some(Instant::now());
            if opts.ranked {
                send(s, ClientMsg::RankedMatch);
            } else {
                send(s, ClientMsg::QuickMatch);
            }
        }
        ServerMsg::MatchStart { you, .. } => {
            if r.seat.is_none() {
                s.drop_at = opts.drop_after.map(|d| Instant::now() + d);
            }
            r.seat = Some(you);
            if let Some(t0) = s.queue_start.take() {
                let ms = t0.elapsed().as_millis().min(u128::from(u16::MAX)) as u16;
                r.queue_wait_ms.push(ms);
            }
        }
        ServerMsg::View(v) => {
            let now = Instant::now();
            if let Some(prev) = s.last_view.replace(now) {
                r.view_gaps_ms
                    .push(now.duration_since(prev).as_millis() as u32);
            }
            resolve_ack(s, r, v.last_seq, now);
            if let Some(intent) = s.bot.decide(&v) {
                s.seq += 1;
                let seq = s.seq;
                s.pending.push((seq, Instant::now()));
                send(s, ClientMsg::Intent { seq, intent });
            }
        }
        ServerMsg::Rejected { seq, .. } => {
            r.rejections += 1;
            resolve_ack(s, r, seq, Instant::now());
        }
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
        // Bots ignore account/rating replies; completion ends the run.
        ServerMsg::LoggedIn { .. }
        | ServerMsg::LoggedOut
        | ServerMsg::AccountError { .. }
        | ServerMsg::RatingChanged { .. }
        | ServerMsg::Leaderboard { .. } => {}
    }
    Ok(None)
}
