//! Every `ServerMsg` the game sees, plus link status: match lifecycle,
//! queue/room state, latency numbers for the overlay, connection loss.

use std::collections::VecDeque;

use bevy::prelude::*;
use tbg_core::protocol::{RoomCode, ServerMsg};

use super::{LatestView, MatchInfo, ViewBuffer};
use crate::audio::{Sound, Sounds};
use crate::link::MatchLink;
use crate::net::NetStatus;
use crate::overlay::OverlayStats;
use crate::ui::{Banner, Toasts};
use crate::GameState;

/// What the queue screen shows.
#[derive(Resource, Default)]
pub struct QueueInfo {
    pub searching: bool,
    pub room: Option<RoomCode>,
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
) {
    let now = time.elapsed_secs_f64();
    for msg in link.poll(now) {
        match msg {
            ServerMsg::MatchStart { you, opponent } => {
                *info = MatchInfo {
                    you: Some(you),
                    opponent,
                    ..default()
                };
                *buffer = ViewBuffer::default();
                *extra.0 = super::MatchStats::default();
                latest.0 = None;
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
            ServerMsg::Queued => ui.2.searching = true,
            ServerMsg::RoomCreated { code } => ui.2.room = Some(code),
            ServerMsg::RoomNotFound => {
                ui.0.push("Room not found");
                *link = MatchLink::Idle;
                next.set(GameState::Menu);
            }
            ServerMsg::NetStats { rtt_ms } => stats.0.rtt_ms = Some(f64::from(rtt_ms)),
            ServerMsg::Error { code } => ui.0.push(format!("Server: {code:?}")),
            _ => {}
        }
    }
    stats.0.lag_ms = link.lag_ms();
    ui.1 .0 = match link.status() {
        Some(NetStatus::Reconnecting) => Some("Reconnecting...".into()),
        Some(NetStatus::Connecting) => Some("Connecting...".into()),
        _ => None,
    };
    if let Some(NetStatus::Failed(why)) = link.status() {
        ui.0.push(why.clone());
        *link = MatchLink::Idle;
        next.set(GameState::Menu);
    }
}
