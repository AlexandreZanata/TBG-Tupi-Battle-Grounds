//! Every `ServerMsg` the game sees, plus link status: match lifecycle,
//! queue/room state, latency numbers for the overlay, connection loss.

use std::collections::VecDeque;

use bevy::prelude::*;
use tbg_core::protocol::{
    ArcanaRejection, ErrorCode, InviteToken, MatchMode, RoomCode, ServerMsg, SpectateRefusal,
    StrongholdRejection,
};
use tbg_core::{MatchView, Seat};

use super::{spectate, LatestView, MatchInfo, RematchInfo, ViewBuffer};
use crate::audio::{Sound, Sounds};
use crate::config;
use crate::i18n::{Lang, Msg};
use crate::link::MatchLink;
use crate::net::{NetFailure, NetStatus, GIVE_UP};
use crate::overlay::OverlayStats;
use crate::ui::{
    account::{Account, AccountInbox},
    Banner, Toasts,
};
use crate::GameState;

/// What the queue screen shows.
#[derive(Resource, Default)]
pub struct QueueInfo {
    /// A late waiting reply must not reopen a screen after Cancel.
    pub cancelled: bool,
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
    lang: Res<Lang>,
    mut shop_pending: ResMut<crate::ui::PendingBuy>,
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
                arena,
            } => {
                info!("match arena: {}", arena.slug());
                // Practice names are built-in identities, not player data:
                // the link kind (never the name text) selects the label.
                let (you_name, opponent) = if matches!(*link, MatchLink::Local(_)) {
                    (
                        lang.t(Msg::BattleYou).to_string(),
                        lang.t(Msg::LocalTrainingBot).to_string(),
                    )
                } else {
                    (you_name, opponent)
                };
                *info = MatchInfo {
                    you: Some(you),
                    you_name,
                    you_rating,
                    opponent,
                    ranked: mode == tbg_core::protocol::MatchMode::Ranked,
                    opponent_rating,
                    arcanas,
                    arena,
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
                shop_pending.0 = None;
                crate::ui::surrender::reset(
                    &mut surrender,
                    mode == tbg_core::protocol::MatchMode::Ranked,
                );
                next.set(GameState::InMatch);
            }
            ServerMsg::View(view) => accept_view(
                *view,
                now,
                &mut info,
                &mut latest,
                &mut buffer,
                &mut stats.1 .0,
            ),
            // P47: a watcher's view, drawn like a seat's (see `sync::spectate`).
            ServerMsg::SpectatorView(view) => accept_view(
                spectate::match_view(&view),
                now,
                &mut info,
                &mut latest,
                &mut buffer,
                &mut stats.1 .0,
            ),
            ServerMsg::SpectateStart {
                id,
                mode,
                arena,
                players,
                arcanas,
            } => {
                // Seat A is "you" so the scene keeps its neutral camera.
                let [me, them] = players;
                info!("watching arena: {}", arena.slug());
                *info = MatchInfo {
                    you: Some(Seat::A),
                    you_name: me.name,
                    you_rating: me.rating,
                    opponent: them.name,
                    opponent_rating: them.rating,
                    ranked: mode == MatchMode::Ranked,
                    arcanas,
                    arena,
                    spectating: true,
                    ..default()
                };
                *buffer = ViewBuffer::default();
                *extra.0 = super::MatchStats::default();
                latest.0 = None;
                shop_pending.0 = None;
                config::save_watch(id);
                next.set(GameState::InMatch);
            }
            ServerMsg::SpectateEnd { outcome } => {
                config::clear_watch();
                config::clear_url_param("watch");
                match outcome {
                    Some(outcome) => {
                        info.outcome = Some(outcome);
                        next.set(GameState::Result);
                    }
                    None => {
                        ui.0.push(lang.t(Msg::SpectateUnavailable));
                        next.set(GameState::Menu);
                    }
                }
            }
            ServerMsg::SpectateUnavailable { reason } => {
                config::clear_watch();
                config::clear_url_param("watch");
                ui.0.push(lang.t(match reason {
                    SpectateRefusal::Full => Msg::SpectateFull,
                    SpectateRefusal::NotFound => Msg::SpectateNotFound,
                    _ => Msg::SpectateUnavailable,
                }));
                next.set(GameState::Menu);
            }
            ServerMsg::Rejected { seq, reason } => {
                if shop_pending.0.is_some_and(|(_, pending)| pending == seq) {
                    shop_pending.0 = None;
                }
                ui.0.push(crate::ui::rejection_text(reason, *lang))
            }
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
            ServerMsg::ChallengeWaiting { .. } if ui.2.cancelled => {}
            ServerMsg::ChallengeWaiting { token } => {
                ui.2.challenge = Some(token);
                ui.2.room = None;
                ui.2.searching = false;
                ui.2.joining_challenge = false;
                next.set(GameState::Queue);
            }
            ServerMsg::ChallengeUnavailable => {
                ui.0.say(Msg::ChallengeUnavailable);
                *link = MatchLink::Idle;
                *ui.2 = QueueInfo::default();
                next.set(GameState::Menu);
            }
            ServerMsg::RematchOpen { .. } if rematch.dismissed => {}
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
                ui.0.say(Msg::RoomNotFound);
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
                ui.0.say(Msg::AccountTakenOver);
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
                ui.0.say(Msg::AccountLoginRanked);
                *link = MatchLink::Idle;
                next.set(GameState::Menu);
            }
            ServerMsg::Error { code } => {
                if code == ErrorCode::RateLimited && ui.2.joining_challenge {
                    ui.0.say(Msg::ChallengeRateLimited);
                    *link = MatchLink::Idle;
                    *ui.2 = QueueInfo::default();
                    next.set(GameState::Menu);
                } else {
                    ui.0.say(error_msg(code));
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
            ServerMsg::ArcanasRejected { reason } => ui.0.say(match reason {
                ArcanaRejection::BadRanks => Msg::ArcanaRejectedBadRanks,
                ArcanaRejection::NotIdle => Msg::ArcanaRejectedNotIdle,
            }),
            // D-46: same for the Stronghold kind.
            ServerMsg::StrongholdSelected { kind } => crate::config::save_stronghold(kind),
            ServerMsg::StrongholdRejected { reason } => ui.0.say(match reason {
                StrongholdRejection::NotSelectable => Msg::StrongholdRejectedNotSelectable,
                StrongholdRejection::NotIdle => Msg::StrongholdRejectedNotIdle,
            }),
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
            Some(lang.tf(Msg::NetReconnecting, &[("secs", &format!("{left:.0}"))]))
        }
        Some(NetStatus::Connecting) => Some(lang.t(Msg::QueueConnecting).into()),
        _ => None,
    };
    if let Some(NetStatus::Failed(why)) = link.status() {
        #[cfg(target_os = "linux")]
        {
            account.auto_login = false;
            account.pending = None;
        }
        // A failed load-time probe is silent; the menu just stays usable.
        if !quiet {
            ui.0.say(match why {
                NetFailure::ConnectionLost => Msg::NetConnectionLost,
                NetFailure::VersionMismatch => Msg::NetVersion,
            });
            next.set(GameState::Menu);
        }
        *link = MatchLink::Idle;
    }
}

/// Player-facing text for a server error code; the two codes with their
/// own flows above (`SessionTakenOver`, `LoginRequired`) reuse those texts.
fn error_msg(code: ErrorCode) -> Msg {
    match code {
        ErrorCode::BadMessage => Msg::NetErrorBadMessage,
        ErrorCode::RateLimited => Msg::NetErrorRateLimited,
        ErrorCode::TooManyConnections => Msg::NetErrorTooManyConnections,
        ErrorCode::NotInMatch => Msg::NetErrorNotInMatch,
        ErrorCode::ServerFull => Msg::NetErrorServerFull,
        ErrorCode::LoginRequired => Msg::AccountLoginRanked,
        ErrorCode::SessionTakenOver => Msg::AccountTakenOver,
    }
}

/// A view from a seat or a watcher: the gap stats, the live view, the timeline.
fn accept_view(
    view: MatchView,
    now: f64,
    info: &mut MatchInfo,
    latest: &mut LatestView,
    buffer: &mut ViewBuffer,
    gaps: &mut VecDeque<f32>,
) {
    if let Some(t) = buffer.last_arrival.replace(now) {
        gaps.push_back(((now - t) * 1000.0) as f32);
        if gaps.len() > 200 {
            gaps.pop_front();
        }
    }
    info.last_tick = view.tick;
    latest.0 = Some(view.clone());
    buffer.timeline.push(now, view);
}
