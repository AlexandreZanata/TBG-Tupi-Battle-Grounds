//! Every `ServerMsg` the game sees, plus link status: match lifecycle,
//! queue/room state, latency numbers for the overlay, connection loss.

use std::collections::VecDeque;

use bevy::prelude::*;
use tbg_core::protocol::{ErrorCode, InviteToken, RoomCode, ServerMsg};

use super::{LatestView, MatchInfo, RematchInfo, ViewBuffer};
use crate::audio::{Sound, Sounds};
use crate::config;
use crate::link::MatchLink;
use crate::net::{NetStatus, GIVE_UP};
use crate::overlay::OverlayStats;
use crate::ui::{
    account::{Account, AccountInbox},
    Banner, Toasts,
};
use crate::GameState;

/// What the queue screen shows.
#[derive(Resource, Default)]
pub struct QueueInfo {
    pub searching: bool,
    pub room: Option<RoomCode>,
    pub challenge: Option<InviteToken>,
    pub joining_challenge: bool,
}

/// Inter-arrival times of the last 50 views (ms), for the overlay.
#[derive(Resource, Default)]
pub struct ViewGaps(pub VecDeque<f32>);

#[allow(clippy::too_many_arguments)] // the one system that routes every message
pub(super) fn poll_link(
    mut link: NonSendMut<MatchLink>,
    time: Res<Time>,
    mut info: ResMut<MatchInfo>,
    mut latest: ResMut<LatestView>,
    mut buffer: ResMut<ViewBuffer>,
    mut ui: (ResMut<Toasts>, ResMut<Banner>, ResMut<QueueInfo>),
    mut stats: (ResMut<OverlayStats>, ResMut<ViewGaps>),
    mut next: ResMut<NextState<GameState>>,
    mut extra: (ResMut<super::MatchStats>, ResMut<Sounds>),
    mut rematch: ResMut<RematchInfo>,
    mut inbox: ResMut<AccountInbox>,
    mut account: ResMut<Account>,
    mut ranking: ResMut<crate::ui::ranking::Ranking>,
    mut surrender: ResMut<crate::ui::surrender::Surrender>,
) {
    let now = time.elapsed_secs_f64();
    for msg in link.poll(now) {
        match msg {
            ServerMsg::MatchStart {
                you,
                you_name,
                you_rating,
                opponent,
                mode,
                opponent_rating,
                arcanas,
            } => {
                *info = MatchInfo {
                    you: Some(you),
                    you_name,
                    you_rating,
                    opponent,
                    ranked: mode == tbg_core::protocol::MatchMode::Ranked,
                    opponent_rating,
                    arcanas,
                    challenge: mode == tbg_core::protocol::MatchMode::Challenge,
                    ..default()
                };
                // P12: a new match clears the previous rating count-up.
                account.rating_change = None;
                *buffer = ViewBuffer::default();
                *extra.0 = super::MatchStats::default();
                *rematch = RematchInfo::default();
                *ui.2 = QueueInfo::default();
                config::clear_url_param("challenge");
                latest.0 = None;
                crate::ui::surrender::reset(
                    &mut surrender,
                    mode == tbg_core::protocol::MatchMode::Ranked,
                );
                next.set(GameState::InMatch);
            }
            ServerMsg::View(view) => {
                if let Some((t, _)) = &buffer.last {
                    let gaps = &mut stats.1 .0;
                    gaps.push_back(((now - t) * 1000.0) as f32);
                    if gaps.len() > 50 {
                        gaps.pop_front();
                    }
                }
                info.last_tick = view.tick;
                buffer.events.extend(view.events.iter().copied());
                latest.0 = Some(view.clone());
                buffer.prev = buffer.last.take();
                buffer.last = Some((now, view));
            }
            ServerMsg::Rejected { reason, .. } => ui.0.push(crate::ui::rejection_text(reason)),
            ServerMsg::MatchEnd { outcome } => {
                let won = outcome.winner.is_some() && outcome.winner == info.you;
                extra
                    .1
                    .play(if won { Sound::Victory } else { Sound::Defeat });
                info.outcome = Some(outcome);
                next.set(GameState::Result);
            }
            ServerMsg::Queued => {
                ui.2.searching = true;
                ui.2.room = None;
                ui.2.challenge = None;
                ui.2.joining_challenge = false;
            }
            ServerMsg::RoomCreated { code } => {
                ui.2.room = Some(code);
                ui.2.challenge = None;
                ui.2.joining_challenge = false;
            }
            ServerMsg::ChallengeWaiting { token } => {
                ui.2.challenge = Some(token);
                ui.2.room = None;
                ui.2.searching = false;
                ui.2.joining_challenge = false;
                next.set(GameState::Queue);
            }
            ServerMsg::ChallengeUnavailable => {
                ui.0.push("Challenge expired or unavailable");
                *link = MatchLink::Idle;
                *ui.2 = QueueInfo::default();
                next.set(GameState::Menu);
            }
            ServerMsg::RematchOpen { opponent } => {
                rematch.open = true;
                rematch.closed = None;
                info.challenge = true;
                if info.opponent.is_empty() {
                    info.opponent = opponent;
                }
                next.set(GameState::Result);
            }
            ServerMsg::RematchState {
                you_requested,
                opponent_requested,
            } => {
                rematch.you_requested = you_requested;
                rematch.opponent_requested = opponent_requested;
            }
            ServerMsg::RematchClosed { declined } => {
                rematch.open = false;
                rematch.closed = Some(declined);
            }
            ServerMsg::RoomNotFound => {
                ui.0.push("Room not found");
                *link = MatchLink::Idle;
                *ui.2 = QueueInfo::default();
                next.set(GameState::Menu);
            }
            ServerMsg::NetStats { rtt_ms } => stats.0.rtt_ms = Some(f64::from(rtt_ms)),
            // P14 (BR-P03): the account signed in on another device. Leave the
            // match screen and stop reconnecting; the stored key still logs
            // in again on the next reload.
            ServerMsg::Error {
                code: ErrorCode::SessionTakenOver,
            } => {
                ui.0.push("Your account signed in on another device");
                account.profile = None;
                *link = MatchLink::Idle;
                next.set(GameState::Menu);
            }
            // Post-P14 (BR-P04): a guest asked for ranked. Drop the stale
            // profile and go back to the menu instead of a raw error toast.
            ServerMsg::Error {
                code: ErrorCode::LoginRequired,
            } => {
                account.profile = None;
                ui.0.push("Log in to play ranked");
                *link = MatchLink::Idle;
                next.set(GameState::Menu);
            }
            ServerMsg::Error { code } => {
                if code == ErrorCode::RateLimited && ui.2.joining_challenge {
                    ui.0.push("Too many challenge attempts, try again later");
                    *link = MatchLink::Idle;
                    *ui.2 = QueueInfo::default();
                    next.set(GameState::Menu);
                } else {
                    ui.0.push(format!("Server: {code:?}"));
                }
            }
            ServerMsg::Leaderboard {
                total,
                offset,
                rows,
                you,
            } => ranking.accept(total, offset, rows, you),
            // P11/P12: account replies belong to the account panel.
            msg @ (ServerMsg::Registered { .. }
            | ServerMsg::LoggedIn { .. }
            | ServerMsg::LoggedOut
            | ServerMsg::AccountError { .. }
            | ServerMsg::RatingChanged { .. }) => inbox.0.push(msg),
            // D-40: the server's word on the preset (after a change or a
            // login) becomes what the editor shows and the next match uses.
            ServerMsg::ArcanasSelected { preset } => crate::config::save_arcanas(preset),
            ServerMsg::ArcanasRejected { reason } => {
                ui.0.push(format!("Arcanas not changed: {reason:?}"))
            }
            _ => {}
        }
    }
    stats.0.lag_ms = link.lag_ms();
    stats.0.ack_ms = link.ack_ms();
    let quiet = link.is_quiet_probe();
    ui.1 .0 = match link.status() {
        _ if quiet => None,
        Some(NetStatus::Reconnecting { since }) => {
            let left = (GIVE_UP - (now - since)).max(0.0);
            Some(format!("Reconnecting... {left:.0} s"))
        }
        Some(NetStatus::Connecting) => Some("Connecting...".into()),
        _ => None,
    };
    if let Some(NetStatus::Failed(why)) = link.status() {
        // A failed load-time probe is silent; the menu just stays usable.
        if !quiet {
            ui.0.push(why.clone());
            next.set(GameState::Menu);
        }
        *link = MatchLink::Idle;
    }
}
