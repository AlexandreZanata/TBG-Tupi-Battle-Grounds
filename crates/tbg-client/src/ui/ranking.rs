//! In-game Ranking panel (P13, BR-R07): one page of the server's
//! leaderboard snapshot, your row highlighted, Prev/Next, and a link to the
//! full `/ranking/` page. A wanted page is re-requested at most once per
//! second (the server ignores faster requests) until its reply arrives.

use bevy::prelude::*;
use tbg_core::protocol::{ClientMsg, LeaderRow, LEADERBOARD_PAGE};

use super::{button, label, screen, UiAction, BUTTON, PANEL};
use crate::link::MatchLink;
use crate::net::{Mode, NetLink};
use crate::{config, GameState};

const PAGE: u16 = LEADERBOARD_PAGE as u16;
/// Server pacing is 1 s; the slack keeps a resend from being ignored.
const RESEND: f64 = 1.1;

pub(crate) struct Page {
    total: u32,
    offset: u16,
    rows: Vec<LeaderRow>,
    you: Option<u32>,
}

#[derive(Resource, Default)]
pub struct Ranking {
    pub(crate) open: bool,
    /// Page offset waiting for a reply.
    want: Option<u16>,
    page: Option<Page>,
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

#[derive(Component)]
struct RankingRoot;

pub fn plugin(app: &mut App) {
    app.init_resource::<Ranking>()
        .add_systems(OnExit(GameState::Menu), close)
        .add_systems(
            Update,
            (request, panel.run_if(resource_changed::<Ranking>)).run_if(in_state(GameState::Menu)),
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

fn cell(text: String, width: f32) -> impl Bundle {
    (
        Node {
            width: px(width),
            ..default()
        },
        children![label(text, 14.0)],
    )
}

fn row(cells: [String; 4], mine: bool) -> impl Bundle {
    let [rank, name, rating, record] = cells;
    (
        Node {
            flex_direction: FlexDirection::Row,
            padding: UiRect::axes(px(6.0), px(2.0)),
            border_radius: BorderRadius::all(px(6.0)),
            ..default()
        },
        BackgroundColor(if mine { BUTTON } else { Color::NONE }),
        children![
            cell(rank, 48.0),
            cell(name, 150.0),
            cell(rating, 60.0),
            cell(record, 90.0),
        ],
    )
}

fn leader_row(r: &LeaderRow, you: Option<u32>) -> impl Bundle + use<> {
    let cells = [
        format!("#{}", r.rank),
        r.name.clone(),
        r.rating.to_string(),
        format!("{}-{}-{}", r.wins, r.losses, r.draws),
    ];
    row(cells, you == Some(r.rank))
}

fn panel(mut commands: Commands, ranking: Res<Ranking>, roots: Query<Entity, With<RankingRoot>>) {
    for e in &roots {
        commands.entity(e).despawn();
    }
    if !ranking.open {
        return;
    }
    let page = ranking.page.as_ref();
    let at = page.map_or(0, |p| p.offset);
    let total = page.map_or(0, |p| p.total);
    let prev = if at > 0 && ranking.want.is_none() {
        UiAction::RankingPrev
    } else {
        UiAction::Disabled
    };
    let next = if u32::from(at) + u32::from(PAGE) < total && ranking.want.is_none() {
        UiAction::RankingNext
    } else {
        UiAction::Disabled
    };
    let status = match (page, ranking.want) {
        (_, Some(_)) => "Loading...".to_string(),
        (Some(p), None) if p.rows.is_empty() => "No ranked players yet".into(),
        (Some(p), None) => match p.you {
            Some(you) => format!("{} ranked players · you are #{you}", p.total),
            None => format!("{} ranked players", p.total),
        },
        (None, None) => String::new(),
    };
    commands
        .spawn((
            screen(JustifyContent::Center),
            DespawnOnExit(GameState::Menu),
            RankingRoot,
            GlobalZIndex(10),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(px(16.0)),
                    row_gap: px(4.0),
                    border_radius: BorderRadius::all(px(16.0)),
                    ..default()
                },
                BackgroundColor(PANEL),
            ))
            .with_children(|p| {
                p.spawn(label("Global Ranking", 26.0));
                p.spawn(label(status, 14.0));
                let head = ["Rank", "Name", "Rating", "W-L-D"].map(String::from);
                p.spawn(row(head, false));
                for r in page.map_or(&[][..], |p| &p.rows) {
                    p.spawn(leader_row(r, page.and_then(|p| p.you)));
                }
                p.spawn((
                    Node {
                        column_gap: px(12.0),
                        margin: UiRect::top(px(8.0)),
                        ..default()
                    },
                    children![button("Prev", prev, 124.0), button("Next", next, 124.0)],
                ));
                p.spawn(button(
                    "Open full ranking",
                    UiAction::OpenFullRanking,
                    260.0,
                ));
                p.spawn(button("Close", UiAction::CloseRanking, 260.0));
            });
        });
}
