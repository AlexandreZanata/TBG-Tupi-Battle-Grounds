//! Side columns identify each strategic Seat and its public Troop kill/death counts.

use super::{panel, text, Ctx};
use crate::{
    app::Screen,
    i18n::{Key, Lang},
    link::Feed,
    palette,
};
use bevy::prelude::*;
use tbg_moba_core::{Role, Seat, Team};

#[derive(Component)]
struct Row(Seat);
#[derive(Component)]
struct Status(Seat);
#[derive(Component)]
struct Name(Seat);

pub fn plugin(app: &mut App) {
    app.add_systems(Update, update.run_if(in_state(Screen::Match)));
}
pub fn spawn(root: &mut ChildSpawnerCommands, ctx: &Ctx) {
    for team in Team::ALL {
        let mut column = Node {
            position_type: PositionType::Absolute,
            top: px(78),
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            ..default()
        };
        if team == Team::Blue {
            column.left = px(10);
        } else {
            column.right = px(10);
        }
        root.spawn(column).with_children(|c| {
            for role in Role::ALL {
                row(c, ctx, Seat::of(team, role));
            }
        });
    }
}
fn row(parent: &mut ChildSpawnerCommands, ctx: &Ctx, seat: Seat) {
    let team = seat.team();
    let reverse = team == Team::Red;
    parent
        .spawn((
            panel(
                Node {
                    width: px(196),
                    height: px(54),
                    align_items: AlignItems::Center,
                    column_gap: px(8),
                    padding: UiRect::all(px(4)),
                    flex_direction: if reverse {
                        FlexDirection::RowReverse
                    } else {
                        FlexDirection::Row
                    },
                    ..default()
                },
                palette::team_dark(team),
            ),
            Row(seat),
        ))
        .with_children(|r| {
            r.spawn((
                Node {
                    width: px(38),
                    height: px(44),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::bottom(px(8)),
                    ..default()
                },
                ImageNode::new(ctx.assets.load(format!(
                    "battle/native/crest-{}.webp",
                    if reverse { "enemy" } else { "ally" }
                ))),
            ))
            .with_children(|crest| {
                crest.spawn(super::chrome::title(
                    (seat.0 % 5 + 1).to_string(),
                    20.,
                    &ctx.fonts,
                ));
            });
            r.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(5),
                align_items: if reverse {
                    AlignItems::FlexEnd
                } else {
                    AlignItems::FlexStart
                },
                ..default()
            })
            .with_children(|info| {
                info.spawn((
                    text(ctx.lang.role(seat.role()), 18., palette::TEXT),
                    Name(seat),
                ));
                info.spawn((text("", 12., palette::MUTED), Status(seat)));
            });
        });
}
#[allow(clippy::type_complexity)] // separate text queries prevent aliasing two mutable Text components
fn update(
    feed: Res<Feed>,
    lang: Res<Lang>,
    mut status: Query<(&Status, &mut Text), Without<Name>>,
    mut names: Query<(&Name, &mut Text, &mut TextColor), Without<Status>>,
    mut rows: Query<(&Row, &mut BorderColor)>,
) {
    let Some(world) = &feed.curr else { return };
    for (Status(seat), mut text) in &mut status {
        let Some(s) = world.seats.get(seat.0 as usize) else {
            continue;
        };
        let value = format!(
            "{} · {}/{}",
            lang.t(if s.human { Key::Player } else { Key::Bot }),
            s.kills,
            s.deaths
        );
        if text.0 != value {
            text.0 = value;
        }
    }
    for (Name(seat), mut text, mut color) in &mut names {
        let mine = feed.seat == Some(*seat);
        let value = if mine {
            format!("{} · {}", lang.role(seat.role()), lang.t(Key::You))
        } else {
            lang.role(seat.role()).to_string()
        };
        if text.0 != value {
            text.0 = value;
        }
        color.0 = if mine { palette::GOLD } else { palette::TEXT };
    }
    for (Row(seat), mut border) in &mut rows {
        *border = BorderColor::all(if feed.seat == Some(*seat) {
            palette::GOLD
        } else {
            palette::team_dark(seat.team())
        });
    }
}
