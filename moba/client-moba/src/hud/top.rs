//! Top scoreboard in the 1v1 battle layout: two mirrored heraldic plates
//! around the round match clock. Where the 1v1 plate shows Stronghold HP,
//! the 5v5 plate shows the team's Gold, Towers standing and kills.

use bevy::prelude::*;
use tbg_moba_core::Team;

use super::chrome::{self, frame, gradient, title};
use super::{clock, short, Ctx};
use crate::app::Screen;
use crate::i18n::{Key, Lang};
use crate::link::Feed;

const TOP: f32 = 12.;
const CLOCK_HEIGHT: f32 = 76.;
const PLATE_HEIGHT: f32 = 55.;

#[derive(Component)]
enum Readout {
    Clock,
    Name(Team),
    Towers(Team),
    Stats(Team),
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, update.run_if(in_state(Screen::Match)));
}

pub fn spawn(root: &mut ChildSpawnerCommands, ctx: &Ctx) {
    root.spawn(Node {
        position_type: PositionType::Absolute,
        top: px(TOP),
        left: percent(22),
        width: percent(56),
        height: px(CLOCK_HEIGHT),
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|row| {
        plate(row, ctx, Team::Blue);
        row.spawn((
            Node {
                width: px(116),
                height: px(CLOCK_HEIGHT),
                flex_shrink: 0.,
                border: UiRect::all(px(3)),
                border_radius: BorderRadius::all(percent(50)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            frame(chrome::GOLD),
            ZIndex(2),
            crate::input::Blocks,
        ))
        .with_children(|c| {
            c.spawn((title("0:00", 29., &ctx.fonts), Readout::Clock));
            c.spawn(chrome::muted(ctx.lang.t(Key::Battle), 8.));
        });
        plate(row, ctx, Team::Red);
    });
    // The Arena chip under the clock, as in the 1v1 battle.
    root.spawn(Node {
        position_type: PositionType::Absolute,
        top: px(TOP + CLOCK_HEIGHT + 4.),
        width: percent(100),
        justify_content: JustifyContent::Center,
        ..default()
    })
    .with_children(|row| {
        row.spawn((
            Node {
                padding: UiRect::axes(px(12), px(3)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            frame(Color::srgb_u8(70, 90, 110)),
        ))
        .with_children(|chip| {
            chip.spawn((
                Text::new(ctx.lang.t(Key::Valley)),
                TextFont::from_font_size(FontSize::Px(14.)),
                TextColor(chrome::CREAM),
            ));
        });
    });
}

fn plate(parent: &mut ChildSpawnerCommands, ctx: &Ctx, team: Team) {
    let blue = team == Team::Blue;
    parent
        .spawn((
            Node {
                flex_grow: 1.,
                flex_basis: px(0),
                min_width: px(0),
                height: px(PLATE_HEIGHT),
                padding: UiRect::axes(px(8), px(4)),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(7)),
                column_gap: px(10),
                align_items: AlignItems::Center,
                flex_direction: if blue {
                    FlexDirection::Row
                } else {
                    FlexDirection::RowReverse
                },
                ..default()
            },
            frame(chrome::GOLD),
            crate::input::Blocks,
        ))
        .with_children(|plate| {
            plate
                .spawn((
                    Node {
                        width: px(38),
                        height: px(44),
                        flex_shrink: 0.,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::bottom(px(7)),
                        ..default()
                    },
                    ImageNode::new(ctx.assets.load(format!(
                        "battle/native/crest-{}.webp",
                        if blue { "ally" } else { "enemy" }
                    ))),
                ))
                .with_children(|crest| {
                    let name = ctx.lang.t(team_key(team));
                    let initial = name
                        .rsplit(' ')
                        .next()
                        .and_then(|w| w.chars().next())
                        .unwrap_or('?');
                    crest.spawn(title(initial.to_string(), 20., &ctx.fonts));
                });
            plate
                .spawn(Node {
                    flex_grow: 1.,
                    flex_basis: px(0),
                    min_width: px(0),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(4),
                    ..default()
                })
                .with_children(|body| {
                    body.spawn(Node {
                        width: percent(100),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        flex_direction: if blue {
                            FlexDirection::Row
                        } else {
                            FlexDirection::RowReverse
                        },
                        ..default()
                    })
                    .with_children(|line| {
                        line.spawn((
                            title(ctx.lang.t(team_key(team)), 14., &ctx.fonts),
                            TextLayout::no_wrap(),
                            Readout::Name(team),
                        ));
                        line.spawn((chrome::muted("", 9.), Readout::Towers(team)));
                    });
                    strip(body, team);
                });
        });
}

/// Where the 1v1 plate has its segmented HP bar: a team-coloured strip
/// carrying the team's Gold and kills.
fn strip(parent: &mut ChildSpawnerCommands, team: Team) {
    let (top, bottom) = match team {
        Team::Blue => (Color::srgb_u8(95, 212, 255), Color::srgb_u8(10, 111, 180)),
        Team::Red => (Color::srgb_u8(255, 128, 133), Color::srgb_u8(180, 20, 44)),
    };
    parent
        .spawn((
            Node {
                width: percent(100),
                height: px(17),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(3)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(Color::BLACK.with_alpha(0.6)),
            gradient(top, bottom),
        ))
        .with_children(|s| {
            s.spawn((
                Text::new(""),
                TextFont::from_font_size(FontSize::Px(12.)),
                TextColor(Color::WHITE),
                TextShadow {
                    offset: Vec2::new(0., 1.),
                    color: Color::BLACK,
                },
                Readout::Stats(team),
            ));
        });
}

fn team_key(team: Team) -> Key {
    match team {
        Team::Blue => Key::BlueTeam,
        Team::Red => Key::RedTeam,
    }
}

fn update(feed: Res<Feed>, lang: Res<Lang>, mut texts: Query<(&Readout, &mut Text)>) {
    let Some(world) = &feed.curr else { return };
    let mine = feed.seat.map(|s| s.team());
    for (readout, mut t) in &mut texts {
        let value = match readout {
            Readout::Clock => clock(world.tick),
            Readout::Name(team) if mine == Some(*team) => {
                format!("{} · {}", lang.t(team_key(*team)), lang.t(Key::You))
            }
            Readout::Name(team) => lang.t(team_key(*team)).to_string(),
            Readout::Towers(team) => format!(
                "{} {}",
                lang.t(Key::Towers),
                world.towers_left[team.index()]
            ),
            Readout::Stats(team) => format!(
                "{} {}  ·  {} {}",
                lang.t(Key::Gold),
                short(world.team_gold[team.index()]),
                lang.t(Key::Kills),
                world.score[team.index()]
            ),
        };
        if t.0 != value {
            t.0 = value;
        }
    }
}
