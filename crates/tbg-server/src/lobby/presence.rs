//! Builds the live snapshot once a second, only when it changed (BR-LV06,
//! BR-LV07): presence counts, the listed matches, the featured battle, and
//! the online accounts. Pure reads of lobby state; the API never calls it.

use std::cmp::Reverse;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tbg_core::protocol::LiveId;

use super::live::{Listing, LiveMatch};
use super::{Lobby, SessionState};
use crate::live::{
    mean_rating, Counts, LiveRow, LiveSeat, LiveSnapshot, OnlineRow, PresenceStatus,
    FEATURED_HOLD_S, LIST_ROWS,
};

impl Lobby {
    /// Publish a new snapshot if anything changed since the last one.
    pub(super) fn publish_live(&mut self, now: Instant) {
        let snapshot = self.live_snapshot(now);
        if **self.live_tx.borrow() != snapshot {
            self.live_tx.send_replace(Arc::new(snapshot));
        }
    }

    fn live_snapshot(&mut self, now: Instant) -> LiveSnapshot {
        let (counts, accounts) = self.presence();
        let featured = self.featured_id(now);
        let mut listed: Vec<(u32, Instant, LiveId, LiveRow)> = self
            .live
            .iter()
            .filter(|(_, m)| m.listing == Listing::Public)
            .map(|(id, m)| {
                let row = live_row(*id, m, now);
                (
                    mean_rating(row.players.each_ref().map(|p| p.rating)),
                    m.started,
                    *id,
                    row,
                )
            })
            .collect();
        // BR-LV07: highest mean, then longest running, then lowest id.
        listed.sort_by_key(|(mean, started, id, _)| (Reverse(*mean), *started, *id));
        let mut rows: Vec<LiveRow> = listed.into_iter().map(|(.., row)| row).collect();
        if let Some(pos) = featured.and_then(|id| rows.iter().position(|r| r.id == id)) {
            let row = rows.remove(pos);
            rows.insert(0, row);
        }
        rows.truncate(LIST_ROWS);
        LiveSnapshot {
            counts,
            featured,
            matches: rows,
            accounts,
        }
    }

    /// BR-LV06: online sessions (with a connection), distinct accounts among
    /// them, seats of running matches, and spectators; plus the account rows.
    fn presence(&self) -> (Counts, Vec<OnlineRow>) {
        let mut counts = Counts::default();
        let mut seen = HashSet::new();
        let mut accounts = Vec::new();
        for s in self.sessions.values() {
            if matches!(s.state, SessionState::InMatch { .. }) {
                counts.in_battle += 1;
            }
            if s.conn.is_none() {
                continue;
            }
            counts.players += 1;
            let Some((id, profile)) = &s.account else {
                continue;
            };
            if !seen.insert(*id) {
                continue;
            }
            accounts.push(OnlineRow {
                name: profile.name.clone(),
                rating: profile.rating,
                status: presence_status(&s.state),
            });
        }
        counts.accounts = seen.len() as u32;
        counts.watching = self.live.values().map(|m| m.watchers.len() as u32).sum();
        accounts.sort_by(|a, b| b.rating.cmp(&a.rating).then_with(|| a.name.cmp(&b.name)));
        (counts, accounts)
    }

    /// BR-LV07: keep the featured battle for 30 s unless it ends; otherwise
    /// the listed match with the highest mean rating takes the spot.
    fn featured_id(&mut self, now: Instant) -> Option<LiveId> {
        let hold = Duration::from_secs(FEATURED_HOLD_S);
        let held = self
            .featured
            .filter(|(id, since)| self.live.contains_key(id) && now.duration_since(*since) < hold);
        let best = self
            .live
            .iter()
            .filter(|(_, m)| m.listing == Listing::Public)
            .min_by_key(|(id, m)| {
                let mean = mean_rating(m.players.each_ref().map(|p| p.rating));
                (Reverse(mean), m.started, **id)
            })
            .map(|(id, _)| *id);
        match held {
            Some((id, _)) => Some(id),
            None => {
                if best != self.featured.map(|(id, _)| id) {
                    self.featured = best.map(|id| (id, now));
                }
                best
            }
        }
    }
}

fn presence_status(state: &SessionState) -> PresenceStatus {
    match state {
        SessionState::Queued => PresenceStatus::Queued,
        SessionState::InMatch { .. } => PresenceStatus::Playing,
        SessionState::Watching { .. } => PresenceStatus::Watching,
        SessionState::Idle | SessionState::InRoom | SessionState::InChallenge { .. } => {
            PresenceStatus::Idle
        }
    }
}

fn live_row(id: LiveId, m: &LiveMatch, now: Instant) -> LiveRow {
    LiveRow {
        id,
        mode: m.mode,
        arena: m.arena,
        players: [0, 1].map(|i| LiveSeat {
            name: m.players[i].name.clone(),
            rating: m.players[i].rating,
            kind: m.kinds[i],
            tier: m.progress.tiers[i],
        }),
        elapsed_s: now.duration_since(m.started).as_secs() as u32,
        hp_bp: m.progress.hp_bp,
        spectators: m.progress.spectators,
    }
}
