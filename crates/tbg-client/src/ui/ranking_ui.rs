//! The Ranking panel view (P13): rebuilt whenever `Ranking` changes; one
//! column of 20 rows on portrait phones, two columns of 10 where the
//! screen is wide but short.

use bevy::prelude::*;
use tbg_core::protocol::LeaderRow;

use super::ranking::{Ranking, PAGE};
use super::{button, label, screen, UiAction, BUTTON, PANEL};
use crate::GameState;

#[derive(Component)]
pub(super) struct RankingRoot;

fn cell(text: String, width: f32) -> impl Bundle {
    (
        Node {
            width: px(width),
            ..default()
        },
        children![label(text, 13.0)],
    )
}

fn row(cells: [String; 4], mine: bool) -> impl Bundle {
    let [rank, name, rating, record] = cells;
    (
        Node {
            flex_direction: FlexDirection::Row,
            width: px(ROW_WIDTH),
            padding: UiRect::axes(px(6.0), px(1.0)),
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

fn leader_row(r: &LeaderRow, mine: bool) -> impl Bundle + use<> {
    let cells = [
        format!("#{}", r.rank),
        r.name.clone(),
        r.rating.to_string(),
        format!("{}-{}-{}", r.wins, r.losses, r.draws),
    ];
    row(cells, mine)
}

/// Logical width needed for two 10-row columns side by side.
const TWO_COLUMNS_WIDTH: f32 = 2.0 * ROW_WIDTH + 64.0;
const ROW_WIDTH: f32 = 348.0;

/// Rows as one column (portrait phones have the height) or two columns of
/// half a page each (desktop and landscape have the width, not the height).
fn columns(rows: &[LeaderRow], two: bool) -> Vec<&[LeaderRow]> {
    if rows.is_empty() {
        return vec![rows];
    }
    let per = if two {
        rows.len().div_ceil(2)
    } else {
        rows.len()
    };
    rows.chunks(per).collect()
}

/// Prev/Next actions: disabled at the edges and while a page loads.
fn nav(ranking: &Ranking) -> (UiAction, UiAction) {
    let page = ranking.page.as_ref();
    let at = page.map_or(0, |p| p.offset);
    let total = page.map_or(0, |p| p.total);
    let idle = ranking.want.is_none();
    let prev = if at > 0 && idle {
        UiAction::RankingPrev
    } else {
        UiAction::Disabled
    };
    let next = if u32::from(at) + u32::from(PAGE) < total && idle {
        UiAction::RankingNext
    } else {
        UiAction::Disabled
    };
    (prev, next)
}

fn status_text(ranking: &Ranking) -> String {
    match (&ranking.page, ranking.want) {
        (_, Some(_)) => "Loading...".into(),
        (Some(p), None) if p.rows.is_empty() => "No ranked players yet".into(),
        (Some(p), None) => match p.you {
            Some(you) => format!("{} ranked players - you are #{you}", p.total),
            None => format!("{} ranked players", p.total),
        },
        (None, None) => String::new(),
    }
}

/// The page's rows under a header, in one or two columns.
fn table(p: &mut ChildSpawnerCommands, ranking: &Ranking, two: bool) {
    let rows = ranking.page.as_ref().map_or(&[][..], |p| &p.rows);
    let you = ranking.page.as_ref().and_then(|p| p.you);
    p.spawn(Node {
        column_gap: px(16.0),
        align_items: AlignItems::FlexStart,
        ..default()
    })
    .with_children(|grid| {
        for col in columns(rows, two) {
            grid.spawn(Node {
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|c| {
                c.spawn(row(
                    ["Rank", "Name", "Rating", "W-L-D"].map(String::from),
                    false,
                ));
                for r in col {
                    c.spawn(leader_row(r, you == Some(r.rank)));
                }
            });
        }
    });
}

fn button_row(a: impl Bundle, b: impl Bundle) -> impl Bundle {
    (
        Node {
            column_gap: px(12.0),
            margin: UiRect::top(px(6.0)),
            ..default()
        },
        children![a, b],
    )
}

pub(super) fn panel(
    mut commands: Commands,
    ranking: Res<Ranking>,
    roots: Query<Entity, With<RankingRoot>>,
    window: Single<&Window>,
    scale: Res<UiScale>,
) {
    for e in &roots {
        commands.entity(e).despawn();
    }
    if !ranking.open {
        return;
    }
    let (prev, next) = nav(&ranking);
    let two = window.width() / scale.0 >= TWO_COLUMNS_WIDTH;
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
                p.spawn(label(status_text(&ranking), 14.0));
                table(p, &ranking, two);
                p.spawn(button_row(
                    button("Prev", prev, 124.0),
                    button("Next", next, 124.0),
                ));
                p.spawn(button_row(
                    button("Open full ranking", UiAction::OpenFullRanking, 220.0),
                    button("Close", UiAction::CloseRanking, 124.0),
                ));
            });
        });
}
