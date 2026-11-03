//! Bottom bar: player card, the five Troop cards, Destination and "Buy and Send".

use bevy::prelude::*;
use tbg_moba_core::content::{self, MAX_TIER};
use tbg_moba_core::{Destination, TroopKind};

use super::{coin, gem, panel, text, title, Ctx};
use crate::app::Screen;
use crate::i18n::{Key, Lang};
use crate::input::{Action, Selection};
use crate::link::Feed;
use crate::palette;

#[derive(Component)]
enum Readout {
    Gold,
    Essence,
    Level,
    Who,
    Upgrade,
    Troops,
}

#[derive(Component)]
struct Card(usize);
#[derive(Component)]
struct Cost(usize);
#[derive(Component)]
struct DestinationRow(usize);
/// The shield on the player card, painted in the player's team colour.
#[derive(Component)]
struct Emblem;

const CARD_W: f32 = 132.0;
const CARD_H: f32 = 176.0;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (readouts, cards, destinations, emblem).run_if(in_state(Screen::Match)),
    );
}

pub fn spawn(root: &mut ChildSpawnerCommands, ctx: &Ctx) {
    root.spawn(Node {
        position_type: PositionType::Absolute,
        left: px(10),
        bottom: px(10),
        align_items: AlignItems::FlexEnd,
        column_gap: px(12),
        ..default()
    })
    .with_children(|row| {
        player_card(row, ctx);
        row.spawn(Node {
            column_gap: px(8),
            ..default()
        })
        .with_children(|cards| {
            for (i, troop) in TroopKind::ALL.into_iter().enumerate() {
                card(cards, ctx, i, troop);
            }
        });
        destination(row, ctx);
        buy(row, ctx);
    });
}

fn player_card(row: &mut ChildSpawnerCommands, ctx: &Ctx) {
    row.spawn(panel(
        Node {
            width: px(372),
            height: px(118),
            align_items: AlignItems::Center,
            column_gap: px(14),
            padding: UiRect::all(px(10)),
            ..default()
        },
        palette::GOLD_DARK,
    ))
    .with_children(|p| {
        p.spawn((
            Node {
                width: px(70),
                height: px(84),
                border: UiRect::all(px(3)),
                border_radius: BorderRadius {
                    top_left: px(10).into(),
                    top_right: px(10).into(),
                    bottom_left: px(35).into(),
                    bottom_right: px(35).into(),
                },
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(palette::BLUE_DARK),
            BorderColor::all(palette::GOLD),
            Emblem,
        ))
        .with_children(|s| {
            s.spawn((title("1", 32.0, palette::TEXT, &ctx.fonts), Readout::Level));
        });
        p.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            flex_grow: 1.0,
            ..default()
        })
        .with_children(|c| {
            c.spawn((
                title(ctx.lang.t(Key::You), 21.0, palette::TEXT, &ctx.fonts),
                Readout::Who,
            ));
            c.spawn(Node {
                align_items: AlignItems::Center,
                column_gap: px(8),
                ..default()
            })
            .with_children(|r| {
                coin(r, 22.0);
                r.spawn((text("0", 24.0, palette::GOLD), Readout::Gold));
                r.spawn(Node {
                    width: px(10),
                    ..default()
                });
                gem(r, 22.0);
                r.spawn((text("0", 24.0, palette::ESSENCE), Readout::Essence));
            });
            c.spawn((text("", 14.0, palette::MUTED), Readout::Troops));
        });
        p.spawn((
            Node {
                width: px(46),
                height: px(46),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(8)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(palette::GOLD),
            BorderColor::all(palette::GOLD_DARK),
            Action::Upgrade,
        ))
        .with_children(|u| {
            u.spawn(title("+", 26.0, palette::SKY, &ctx.fonts));
            u.spawn((text("", 11.0, palette::SKY), Readout::Upgrade));
        });
    });
}

fn card(row: &mut ChildSpawnerCommands, ctx: &Ctx, i: usize, troop: TroopKind) {
    let (name, kind) = ctx.lang.troop(troop);
    row.spawn((
        panel(
            Node {
                width: px(CARD_W),
                height: px(CARD_H),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(px(5)),
                row_gap: px(2),
                ..default()
            },
            palette::GOLD_DARK,
        ),
        Action::SelectTroop(i),
        Card(i),
    ))
    .with_children(|c| {
        c.spawn((
            Node {
                width: px(CARD_W - 14.0),
                height: px(100),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            ImageNode::new(ctx.portraits.troops[i].clone()),
        ));
        c.spawn(title(name, 19.0, palette::TEXT, &ctx.fonts));
        c.spawn(text(kind, 13.0, palette::MUTED));
        c.spawn(Node {
            align_items: AlignItems::Center,
            column_gap: px(5),
            ..default()
        })
        .with_children(|r| {
            coin(r, 16.0);
            r.spawn((
                title(
                    content::troop_cost(troop).to_string(),
                    19.0,
                    palette::GOLD,
                    &ctx.fonts,
                ),
                Cost(i),
            ));
        });
        c.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(6),
                left: px(9),
                ..default()
            },
            text((i + 1).to_string(), 13.0, palette::TEXT),
        ));
    });
}

fn destination(row: &mut ChildSpawnerCommands, ctx: &Ctx) {
    row.spawn(panel(
        Node {
            width: px(196),
            height: px(CARD_H),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(8)),
            row_gap: px(5),
            ..default()
        },
        palette::GOLD_DARK,
    ))
    .with_children(|p| {
        p.spawn(text(ctx.lang.t(Key::Destination), 17.0, palette::TEXT));
        for (i, (to, hotkey)) in Destination::ALL
            .into_iter()
            .zip(["Z", "X", "C", "V"])
            .enumerate()
        {
            p.spawn((
                Node {
                    height: px(29),
                    padding: UiRect::horizontal(px(10)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(6)),
                    ..default()
                },
                BackgroundColor(palette::PANEL_LIGHT),
                BorderColor::all(Color::NONE),
                Action::SetDestination(i),
                DestinationRow(i),
            ))
            .with_children(|r| {
                r.spawn(text(ctx.lang.lane(to), 16.0, palette::TEXT));
                r.spawn(text(hotkey, 12.0, palette::MUTED));
            });
        }
    });
}

fn buy(row: &mut ChildSpawnerCommands, ctx: &Ctx) {
    row.spawn((
        Node {
            width: px(172),
            height: px(96),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: px(4),
            border: UiRect::all(px(3)),
            border_radius: BorderRadius::all(px(10)),
            ..default()
        },
        BackgroundColor(palette::GOLD),
        BorderColor::all(Color::srgb(1.0, 0.9, 0.55)),
        Action::Buy,
        crate::input::Blocks,
    ))
    .with_children(|b| {
        // A simple cart: basket and two wheels.
        b.spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|cart| {
            cart.spawn((
                Node {
                    width: px(30),
                    height: px(14),
                    border_radius: BorderRadius::bottom(px(4)),
                    ..default()
                },
                BackgroundColor(palette::SKY),
            ));
            cart.spawn(Node {
                column_gap: px(10),
                margin: UiRect::top(px(2)),
                ..default()
            })
            .with_children(|w| {
                for _ in 0..2 {
                    w.spawn((
                        Node {
                            width: px(7),
                            height: px(7),
                            border_radius: BorderRadius::all(percent(50)),
                            ..default()
                        },
                        BackgroundColor(palette::SKY),
                    ));
                }
            });
        });
        b.spawn((
            title(ctx.lang.t(Key::BuyAndSend), 19.0, palette::SKY, &ctx.fonts),
            TextLayout::new(Justify::Center, LineBreak::WordBoundary),
        ));
        b.spawn(text("Enter", 11.0, Color::srgb(0.3, 0.22, 0.05)));
    });
}

fn readouts(feed: Res<Feed>, lang: Res<Lang>, mut texts: Query<(&Readout, &mut Text)>) {
    let (Some(world), Some(me)) = (&feed.curr, &feed.me) else {
        return;
    };
    let seat = &world.seats[me.seat.0 as usize];
    for (readout, mut t) in &mut texts {
        let value = match readout {
            Readout::Gold => me.gold.to_string(),
            Readout::Essence => me.essence.to_string(),
            Readout::Level => seat.level.to_string(),
            Readout::Who => format!("{} ({})", lang.t(Key::You), lang.role(me.seat.role())),
            Readout::Upgrade => {
                if me.tier >= MAX_TIER {
                    "MAX".into()
                } else {
                    content::upgrade_cost(me.tier).to_string()
                }
            }
            Readout::Troops => format!(
                "{} {}/{} · {} {}",
                lang.t(Key::Troops),
                me.troops,
                content::TROOP_CAP,
                lang.t(Key::Tier),
                me.tier
            ),
        };
        if t.0 != value {
            t.0 = value;
        }
    }
}

fn cards(
    feed: Res<Feed>,
    selection: Res<Selection>,
    mut cards: Query<(&Card, &mut BorderColor, &mut BackgroundColor)>,
    mut costs: Query<(&Cost, &mut TextColor)>,
) {
    let gold = feed.me.map_or(0, |m| m.gold);
    for (Card(i), mut border, mut background) in &mut cards {
        let affordable = gold >= content::troop_cost(TroopKind::ALL[*i]);
        let wanted = BorderColor::all(if selection.troop == *i {
            palette::GOLD
        } else {
            palette::GOLD_DARK
        });
        if *border != wanted {
            *border = wanted;
        }
        let shade = if affordable {
            palette::PANEL
        } else {
            Color::srgba(0.03, 0.03, 0.05, 0.95)
        };
        if background.0 != shade {
            background.0 = shade;
        }
    }
    for (Cost(i), mut color) in &mut costs {
        let wanted = if gold >= content::troop_cost(TroopKind::ALL[*i]) {
            palette::GOLD
        } else {
            palette::HP_ENEMY
        };
        if color.0 != wanted {
            color.0 = wanted;
        }
    }
}

fn destinations(
    selection: Res<Selection>,
    mut rows: Query<(&DestinationRow, &mut BorderColor, &mut BackgroundColor)>,
) {
    for (DestinationRow(i), mut border, mut background) in &mut rows {
        let on = selection.destination == *i;
        let wanted = BorderColor::all(if on { palette::GOLD } else { Color::NONE });
        if *border != wanted {
            *border = wanted;
        }
        let shade = if on {
            Color::srgba(0.35, 0.27, 0.08, 0.95)
        } else {
            palette::PANEL_LIGHT
        };
        if background.0 != shade {
            background.0 = shade;
        }
    }
}

fn emblem(feed: Res<Feed>, mut shields: Query<&mut BackgroundColor, With<Emblem>>) {
    let Some(seat) = feed.seat else { return };
    for mut background in &mut shields {
        let wanted = palette::team_dark(seat.team());
        if background.0 != wanted {
            background.0 = wanted;
        }
    }
}
