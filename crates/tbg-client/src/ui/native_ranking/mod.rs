//! Full-width Linux Hall of Champions using public server ranking data and existing art.

mod podium;
mod profile;
mod table;

use super::{account::Account, kit, ranking::Ranking, shell::LobbyData, UiAction};
use crate::{i18n::Lang, GameState};
use bevy::prelude::*;
use tbg_core::protocol::LeaderRow;

const INK: Color = Color::srgb_u8(9, 26, 32);
const EDGE: Color = Color::srgb_u8(151, 118, 65);

#[derive(Component)]
struct Root;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, render.run_if(in_state(GameState::Menu)))
        .add_systems(PostUpdate, podium::cover.after(bevy::ui::UiSystems::Layout));
}

fn words(lang: Lang, en: &str, pt: &str) -> String {
    if lang == Lang::PtBr {
        pt.into()
    } else {
        en.into()
    }
}

fn frame() -> impl Bundle {
    (BackgroundColor(INK), BorderColor::all(EDGE))
}

fn text(value: impl Into<String>, size: f32) -> impl Bundle {
    (super::label(value, size), TextLayout::no_wrap())
}

#[allow(clippy::too_many_arguments)]
fn render(
    mut commands: Commands,
    ranking: Res<Ranking>,
    account: Res<Account>,
    data: Res<LobbyData>,
    assets: Res<AssetServer>,
    lang: Res<Lang>,
    roots: Query<Entity, With<Root>>,
    positions: Query<&ScrollPosition, With<table::List>>,
    mut page_at: Local<u16>,
) {
    if !crate::config::bevy_ui() {
        return;
    }
    if !ranking.is_changed()
        && !account.is_changed()
        && !data.is_changed()
        && !lang.is_changed()
        && ((!ranking.open && roots.is_empty()) || (ranking.open && !roots.is_empty()))
    {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }
    if !ranking.open {
        return;
    }
    let offset = ranking.page.as_ref().map_or(0, |p| p.offset);
    let scroll = if offset == *page_at && !roots.is_empty() {
        positions.iter().next().cloned().unwrap_or_default()
    } else {
        ScrollPosition::default()
    };
    *page_at = offset;
    commands
        .spawn((
            Node {
                padding: UiRect::all(px(16)),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(9)),
                overflow: Overflow::clip(),
                row_gap: px(12),
                ..super::shell::page_node()
            },
            frame(),
            Root,
            GlobalZIndex(10),
            DespawnOnExit(GameState::Menu),
        ))
        .with_children(|root| {
            heading(root, &assets, *lang);
            root.spawn(Node {
                width: percent(100),
                flex_grow: 1.,
                min_height: px(0),
                column_gap: px(16),
                ..default()
            })
            .with_children(|body| {
                body.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.,
                    flex_basis: px(0),
                    min_width: px(0),
                    min_height: px(0),
                    row_gap: px(8),
                    ..default()
                })
                .with_children(|board| {
                    podium::spawn(board, &ranking, &assets, *lang);
                    table::spawn(board, &ranking, &account, *lang, scroll);
                });
                profile::spawn(body, &ranking, &account, &data, &assets, *lang);
            });
        });
}

fn heading(root: &mut ChildSpawnerCommands, assets: &AssetServer, lang: Lang) {
    root.spawn((
        Node {
            width: percent(100),
            min_height: px(78),
            flex_shrink: 0.,
            padding: UiRect::bottom(px(10)),
            border: UiRect::bottom(px(1)),
            align_items: AlignItems::Center,
            column_gap: px(14),
            ..default()
        },
        BorderColor::all(EDGE.with_alpha(0.5)),
    ))
    .with_children(|head| {
        head.spawn(kit::image(
            assets,
            "ranking/champions-trophy.webp",
            60.,
            64.,
        ));
        head.spawn(Node {
            flex_grow: 1.,
            row_gap: px(2),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|title| {
            title.spawn(kit::muted(
                words(lang, "THE HALL OF CHAMPIONS", "O SALÃO DOS CAMPEÕES"),
                9.,
            ));
            title.spawn(kit::heading(
                words(lang, "Global Rankings", "Ranking Global"),
                32.,
            ));
            title.spawn(kit::muted(
                words(
                    lang,
                    "Earn your place. Build your legacy.",
                    "Conquiste seu lugar. Construa seu legado.",
                ),
                12.,
            ));
        });
        for (label, value) in [
            (
                words(lang, "Scope", "Período"),
                words(lang, "All time", "Todo o período"),
            ),
            (words(lang, "Region", "Região"), "Global".into()),
            (words(lang, "Queue", "Fila"), "Ranked 1v1".into()),
        ] {
            head.spawn(Node {
                width: px(125),
                row_gap: px(4),
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|scope| {
                scope.spawn(kit::muted(label, 10.));
                scope.spawn((
                    Node {
                        width: percent(100),
                        padding: UiRect::all(px(9)),
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(px(4)),
                        ..default()
                    },
                    frame(),
                    children![text(value, 12.)],
                ));
            });
        }
        head.spawn(kit::button("×", UiAction::CloseRanking, 32.));
    });
}

fn rate(row: &LeaderRow) -> String {
    let games = u64::from(row.wins) + u64::from(row.losses) + u64::from(row.draws);
    if games == 0 {
        "—".into()
    } else {
        format!("{:.1}%", f64::from(row.wins) * 100. / games as f64)
    }
}
