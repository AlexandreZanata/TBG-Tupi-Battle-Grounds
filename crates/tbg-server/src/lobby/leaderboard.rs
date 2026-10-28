//! In-game leaderboard pages (P13, BR-R07): served from the store's
//! in-memory snapshot (no SQL per request); `you` comes from the session's
//! account name. The per-connection rate limit lives in `ws/client_msg.rs`.

use tbg_core::protocol::{LeaderRow, ServerMsg, Token, LEADERBOARD_PAGE};

use super::{send, Lobby};
use crate::chan::ConnEvent;

impl Lobby {
    pub(super) fn leaderboard(&mut self, token: Token, offset: u16) {
        let snap = self.store.as_ref().map(|s| s.snapshot());
        let Some(s) = self.sessions.get(&token) else {
            return;
        };
        let (total, rows, you) = match &snap {
            Some(snap) => {
                let rows = snap
                    .rows
                    .iter()
                    .skip(usize::from(offset))
                    .take(LEADERBOARD_PAGE)
                    .map(|r| LeaderRow {
                        rank: r.rank,
                        name: r.name.clone(),
                        rating: r.rating,
                        wins: r.wins,
                        losses: r.losses,
                        draws: r.draws,
                    })
                    .collect();
                let you = s
                    .account
                    .as_ref()
                    .and_then(|(_, p)| snap.rank_of(&p.name.to_ascii_lowercase()));
                (snap.total as u32, rows, you)
            }
            None => (0, Vec::new(), None),
        };
        let msg = ServerMsg::Leaderboard {
            total,
            offset,
            rows,
            you,
        };
        send(&s.conn, ConnEvent::Send(msg));
    }
}
