//! In-game Ranking panel (P13, BR-R07): one page of the server's
//! leaderboard snapshot, your row highlighted, Prev/Next, and a link to the
//! full `/ranking/` page. A wanted page is re-requested at most once per
//! second (the server ignores faster requests) until its reply arrives.

use bevy::prelude::*;
use tbg_core::protocol::{ClientMsg, LeaderRow, LEADERBOARD_PAGE};

use super::UiAction;
use crate::link::MatchLink;
use crate::net::{Mode, NetLink};
use crate::{config, GameState};

pub(super) const PAGE: u16 = LEADERBOARD_PAGE as u16;
/// Server pacing is 1 s; the slack keeps a resend from being ignored.
const RESEND: f64 = 1.1;

pub(crate) struct Page {
    pub(super) total: u32,
    pub(super) offset: u16,
    pub(super) rows: Vec<LeaderRow>,
    pub(super) you: Option<u32>,
}

#[derive(Resource, Default)]
pub struct Ranking {
    pub(crate) open: bool,
    /// Page offset waiting for a reply.
    pub(super) want: Option<u16>,
    pub(super) page: Option<Page>,
}

impl Ranking {
    /// A `ServerMsg::Leaderboard` reply (routed by `sync::messages`).
    pub fn accept(&mut self, total: u32, offset: u16, rows: Vec<LeaderRow>, you: Option<u32>) {
        if self.want == Some(offset) {
            self.want = None;
        }
        self.page = Some(Page {
            total,
            offset,
            rows,
            you,
        });
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Ranking>()
        .add_systems(OnExit(GameState::Menu), close)
        .add_systems(
            Update,
            (
                request,
                super::ranking_ui::panel.run_if(resource_changed::<Ranking>),
            )
                .run_if(in_state(GameState::Menu)),
        );
}

/// Apply the P13 ranking buttons (called from `actions::apply`).
pub(super) fn apply_action(action: UiAction, ranking: &mut Ranking) {
    let at = ranking.page.as_ref().map_or(0, |p| p.offset);
    match action {
        UiAction::OpenRanking => {
            ranking.open = true;
            ranking.want = Some(0);
        }
        UiAction::RankingPrev => ranking.want = Some(at.saturating_sub(PAGE)),
        UiAction::RankingNext => ranking.want = Some(at.saturating_add(PAGE)),
        UiAction::CloseRanking => close_panel(ranking),
        UiAction::OpenFullRanking => config::open_url("/ranking/"),
        _ => {}
    }
}

fn close_panel(ranking: &mut Ranking) {
    ranking.open = false;
    ranking.want = None;
}

fn close(mut ranking: ResMut<Ranking>) {
    close_panel(&mut ranking);
}

fn request(
    ranking: Res<Ranking>,
    mut link: NonSendMut<MatchLink>,
    time: Res<Time>,
    mut sent_at: Local<Option<f64>>,
) {
    let Some(offset) = ranking.want else {
        return;
    };
    let now = time.elapsed_secs_f64();
    if sent_at.is_some_and(|t| now - t < RESEND) {
        return;
    }
    *sent_at = Some(now);
    // With nothing open, a listen-only probe carries the request; a later
    // Quick Match reuses the same socket.
    if !matches!(*link, MatchLink::Net(_)) {
        *link = MatchLink::Net(Box::new(NetLink::new(Mode::Probe, now)));
    }
    link.send(ClientMsg::Leaderboard { offset }, now);
}
