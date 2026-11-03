//! Side columns: each team's five roles with portrait, level, health and respawn.

use bevy::prelude::*;
use tbg_moba_core::{Role, Seat, Team};

use super::{bar, panel, text, Ctx};
use crate::app::Screen;
use crate::i18n::{Key, Lang};
use crate::link::Feed;
use crate::palette;

#[derive(Component)]
struct Row(Seat);
#[derive(Component)]
struct Health(Seat);
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
                    width: px(44),
                    height: px(44),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(6)),
                    ..default()
                },
                BorderColor::all(palette::team(team)),
                ImageNode::new(super::chrome::portrait(
                    &ctx.assets,
                    tbg_moba_core::content::commander_champion(seat.role()),
                )),
            ));
            r.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                align_items: if reverse {
                    AlignItems::FlexEnd
                } else {
                    AlignItems::FlexStart
                },
                ..default()
            })
            .with_children(|info| {
                info.spawn((
                    text(ctx.lang.role(seat.role()), 18.0, palette::TEXT),
                    Name(seat),
                ));
                bar(info, 118.0, 7.0, palette::HP_ALLY, Health(seat));
                info.spawn((text("", 13.0, palette::MUTED), Status(seat)));
            });
        });
}

#[allow(clippy::type_complexity)] // four readouts of one row
fn update(
    feed: Res<Feed>,
    lang: Res<Lang>,
    mut health: Query<(&Health, &mut Node, &mut BackgroundColor)>,
    mut status: Query<(&Status, &mut Text), Without<Name>>,
    mut names: Query<(&Name, &mut Text, &mut TextColor), Without<Status>>,
    mut rows: Query<(&Row, &mut BorderColor)>,
) {
    let Some(world) = &feed.curr else { return };
    for (Health(seat), mut node, mut color) in &mut health {
        let s = world.seats[seat.0 as usize];
        node.width = percent(s.hp_permille as f32 / 10.0);
        color.0 = if s.respawn_ticks > 0 {
            palette::MUTED
        } else {
            palette::HP_ALLY
        };
    }
    for (Status(seat), mut t) in &mut status {
        let s = world.seats[seat.0 as usize];
        let value = if s.respawn_ticks > 0 {
            format!("{} {}s", lang.t(Key::Respawn), s.respawn_ticks / 20 + 1)
        } else {
            format!(
                "{} {} · {}/{}",
                lang.t(Key::Level),
                s.level,
                s.kills,
                s.deaths
            )
        };
        if t.0 != value {
            t.0 = value;
        }
    }
    for (Name(seat), mut t, mut color) in &mut names {
        let mine = feed.seat == Some(*seat);
        let value = if mine {
            format!("{} · {}", lang.role(seat.role()), lang.t(Key::You))
        } else {
            lang.role(seat.role()).to_string()
        };
        if t.0 != value {
            t.0 = value;
        }
        let wanted = if mine { palette::GOLD } else { palette::TEXT };
        if color.0 != wanted {
            color.0 = wanted;
        }
    }
    for (Row(seat), mut border) in &mut rows {
        let wanted = BorderColor::all(if feed.seat == Some(*seat) {
            palette::GOLD
        } else {
            palette::team_dark(seat.team())
        });
        if *border != wanted {
            *border = wanted;
        }
    }
}
