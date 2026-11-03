//! Top scoreboard: team names, towers standing, team gold, kill score and clock.

use bevy::prelude::*;
use tbg_moba_core::Team;

use super::{clock, coin, panel, short, text, title, tower_glyph, Ctx};
use crate::app::Screen;
use crate::i18n::Key;
use crate::link::Feed;
use crate::palette;

#[derive(Component)]
enum Readout {
    Towers(Team),
    Gold(Team),
    Score(Team),
    Clock,
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, update.run_if(in_state(Screen::Match)));
}

pub fn spawn(root: &mut ChildSpawnerCommands, ctx: &Ctx) {
    root.spawn(Node {
        position_type: PositionType::Absolute,
        top: px(8),
        width: percent(100),
        justify_content: JustifyContent::Center,
        ..default()
    })
    .with_children(|row| {
        row.spawn(panel(
            Node {
                align_items: AlignItems::Center,
                column_gap: px(18),
                padding: UiRect::axes(px(18), px(6)),
                ..default()
            },
            palette::GOLD_DARK,
        ))
        .with_children(|bar| {
            side(bar, ctx, Team::Blue);
            score(bar, ctx);
            side(bar, ctx, Team::Red);
        });
    });
}

fn shield(parent: &mut ChildSpawnerCommands, team: Team, ctx: &Ctx) {
    parent
        .spawn((
            Node {
                width: px(34),
                height: px(40),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius {
                    top_left: px(6).into(),
                    top_right: px(6).into(),
                    bottom_left: px(17).into(),
                    bottom_right: px(17).into(),
                },
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(palette::team(team)),
            BorderColor::all(palette::GOLD),
        ))
        .with_children(|s| {
            // Initial of the team name in the current language.
            let initial = ctx
                .lang
                .t(if team == Team::Blue {
                    Key::BlueTeam
                } else {
                    Key::RedTeam
                })
                .rsplit(' ')
                .next()
                .unwrap_or("")
                .chars()
                .next()
                .unwrap_or(' ');
            s.spawn(title(initial.to_string(), 20.0, palette::TEXT, &ctx.fonts));
        });
}

fn side(bar: &mut ChildSpawnerCommands, ctx: &Ctx, team: Team) {
    let name = ctx.lang.t(if team == Team::Blue {
        Key::BlueTeam
    } else {
        Key::RedTeam
    });
    bar.spawn(Node {
        align_items: AlignItems::Center,
        column_gap: px(12),
        ..default()
    })
    .with_children(|s| {
        let pieces = |s: &mut ChildSpawnerCommands| {
            s.spawn(Node {
                align_items: AlignItems::Center,
                column_gap: px(6),
                ..default()
            })
            .with_children(|t| {
                tower_glyph(t, palette::glow(team));
                t.spawn((text("11", 22.0, palette::TEXT), Readout::Towers(team)));
            });
            s.spawn(Node {
                align_items: AlignItems::Center,
                column_gap: px(6),
                ..default()
            })
            .with_children(|g| {
                coin(g, 18.0);
                g.spawn((text("0", 22.0, palette::GOLD), Readout::Gold(team)));
            });
        };
        if team == Team::Blue {
            shield(s, team, ctx);
            s.spawn(title(name, 24.0, palette::TEXT, &ctx.fonts));
            pieces(s);
        } else {
            pieces(s);
            s.spawn(title(name, 24.0, palette::TEXT, &ctx.fonts));
            shield(s, team, ctx);
        }
    });
}

fn score(bar: &mut ChildSpawnerCommands, ctx: &Ctx) {
    bar.spawn(Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        margin: UiRect::horizontal(px(10)),
        ..default()
    })
    .with_children(|c| {
        c.spawn(Node {
            align_items: AlignItems::Center,
            column_gap: px(14),
            ..default()
        })
        .with_children(|s| {
            s.spawn((
                title("0", 40.0, palette::BLUE_GLOW, &ctx.fonts),
                Readout::Score(Team::Blue),
            ));
            s.spawn(title("×", 30.0, palette::GOLD, &ctx.fonts));
            s.spawn((
                title("0", 40.0, palette::RED_GLOW, &ctx.fonts),
                Readout::Score(Team::Red),
            ));
        });
        c.spawn((
            Node {
                padding: UiRect::axes(px(12), px(1)),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
        ))
        .with_children(|t| {
            t.spawn((text("00:00", 20.0, palette::TEXT), Readout::Clock));
        });
    });
}

fn update(feed: Res<Feed>, mut texts: Query<(&Readout, &mut Text)>) {
    let Some(world) = &feed.curr else { return };
    for (readout, mut t) in &mut texts {
        let value = match readout {
            Readout::Towers(team) => world.towers_left[team.index()].to_string(),
            Readout::Gold(team) => short(world.team_gold[team.index()]),
            Readout::Score(team) => world.score[team.index()].to_string(),
            Readout::Clock => clock(world.tick),
        };
        if t.0 != value {
            t.0 = value;
        }
    }
}
