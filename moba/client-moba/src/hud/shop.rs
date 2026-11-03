//! Bottom of the screen in the 1v1 battle layout: the framed resource strip
//! over the three role-locked Shop cards (same frame, portraits, texts and lock as the
//! 1v1 cards), the Troop Destination and the player card.

use bevy::prelude::*;
use tbg_moba_core::content::{self, MAX_TIER, OFFERS};
use tbg_moba_core::Destination;

use super::chrome::{self, frame, title};
use super::Ctx;
use crate::app::Screen;
use crate::i18n::{Key, Lang};
use crate::input::{Action, Selection};
use crate::link::Feed;

const CARD_W: f32 = 166.;
const CARD_H: f32 = 225.;
const GAP: f32 = 8.;

#[derive(Component)]
enum Readout {
    Gold,
    Essence,
    Upgrade,
    Army,
    Reroll,
    Seat,
    Who,
    Line,
}

#[derive(Component)]
enum Card {
    Surface(usize),
    Portrait(usize),
    Name(usize),
    Role(usize),
    Rules(usize),
    Cost(usize),
    Lock(usize),
    LockFrame(usize),
}

#[derive(Component)]
struct DestinationRow(usize);
/// The shield on the player card, in the player's team colour.
#[derive(Component)]
struct Emblem;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (readouts, cards, destinations, emblem).run_if(in_state(Screen::Match)),
    );
}

pub fn spawn(root: &mut ChildSpawnerCommands, ctx: &Ctx) {
    let width = CARD_W * OFFERS as f32 + GAP * (OFFERS as f32 - 1.);
    root.spawn(Node {
        position_type: PositionType::Absolute,
        bottom: px(20),
        width: percent(100),
        justify_content: JustifyContent::Center,
        ..default()
    })
    .with_children(|row| {
        row.spawn(Node {
            width: px(862.),
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            ..default()
        })
        .with_children(|column| {
            resources(column, ctx);
            column
                .spawn(Node {
                    column_gap: px(GAP),
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|cards| {
                    for i in 0..OFFERS {
                        card(cards, ctx, i);
                    }
                });
        });
    });
    root.spawn(Node {
        position_type: PositionType::Absolute,
        bottom: px(20),
        right: percent(50),
        margin: UiRect::right(px(width / 2. + 14.)),
        ..default()
    })
    .with_children(|slot| destination(slot, ctx));
    player_card(root, ctx);
}

fn resources(column: &mut ChildSpawnerCommands, ctx: &Ctx) {
    column
        .spawn((
            Node {
                width: percent(100),
                height: px(43),
                padding: UiRect::axes(px(15), px(5)),
                column_gap: px(14),
                align_items: AlignItems::Center,
                ..default()
            },
            ImageNode {
                image_mode: NodeImageMode::Stretch,
                visual_box: bevy::ui::VisualBox::BorderBox,
                ..ImageNode::new(ctx.assets.load("battle/native/resources.webp"))
            },
            crate::input::Blocks,
        ))
        .with_children(|bar| {
            bar.spawn(title(ctx.lang.t(Key::Gold), 17., &ctx.fonts));
            bar.spawn(chrome::coin(18.));
            bar.spawn((
                Text::new("0"),
                TextFont::from_font_size(FontSize::Px(17.)).with_font(ctx.fonts.display.clone()),
                TextColor(Color::srgb_u8(255, 220, 120)),
                Readout::Gold,
            ));
            chrome::divider(bar);
            bar.spawn(title(ctx.lang.t(Key::Essence), 17., &ctx.fonts));
            bar.spawn((
                Text::new("0"),
                TextFont::from_font_size(FontSize::Px(13.)),
                TextColor(Color::srgb_u8(223, 156, 255)),
                Readout::Essence,
            ));
            bar.spawn((
                chrome::button(Node {
                    min_width: px(58),
                    height: px(26),
                    padding: UiRect::horizontal(px(6)),
                    ..default()
                }),
                Action::Upgrade,
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new(""),
                    TextFont::from_font_size(FontSize::Px(11.)),
                    TextColor(chrome::CREAM),
                    Readout::Upgrade,
                ));
            });
            chrome::divider(bar);
            bar.spawn(chrome::icon(&ctx.assets, "friends", 27., chrome::CREAM));
            bar.spawn((title("", 17., &ctx.fonts), Readout::Army));
            bar.spawn(Node {
                flex_grow: 1.,
                ..default()
            });
            bar.spawn((
                chrome::button(Node {
                    min_width: px(150),
                    height: px(30),
                    padding: UiRect::horizontal(px(8)),
                    ..default()
                }),
                Action::Reroll,
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new(""),
                    TextFont::from_font_size(FontSize::Px(13.)),
                    TextColor(chrome::CREAM),
                    Readout::Reroll,
                ));
            });
        });
}

fn card(parent: &mut ChildSpawnerCommands, ctx: &Ctx, i: usize) {
    parent
        .spawn((
            Node {
                width: px(CARD_W),
                height: px(CARD_H),
                flex_shrink: 0.,
                ..default()
            },
            ImageNode {
                image_mode: NodeImageMode::Stretch,
                ..ImageNode::new(ctx.assets.load("battle/native/shop-card.webp"))
            },
            Action::BuySlot(i),
            Card::Surface(i),
            crate::input::Blocks,
        ))
        .with_children(|card| {
            card.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: px(9),
                    left: px(7),
                    width: px(152),
                    height: px(137),
                    ..default()
                },
                ImageNode::default(),
                Card::Portrait(i),
            ));
            card.spawn(Node {
                position_type: PositionType::Absolute,
                left: px(12),
                right: px(10),
                top: px(147),
                height: px(22),
                overflow: Overflow::clip(),
                ..default()
            })
            .with_children(|t| {
                t.spawn((
                    title("", 16., &ctx.fonts),
                    TextLayout::no_wrap(),
                    Card::Name(i),
                ));
            });
            card.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(12),
                    top: px(167),
                    ..default()
                },
                chrome::muted("", 11.),
                Card::Role(i),
            ));
            card.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(12),
                    top: px(181),
                    ..default()
                },
                chrome::muted("", 9.),
                Card::Rules(i),
            ));
            card.spawn(Node {
                position_type: PositionType::Absolute,
                left: px(9),
                right: px(9),
                bottom: px(8),
                height: px(29),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(7),
                ..default()
            })
            .with_children(|cost| {
                cost.spawn(chrome::coin(16.));
                cost.spawn(chrome::muted(ctx.lang.t(Key::Cost), 12.));
                cost.spawn((title("", 19., &ctx.fonts), Card::Cost(i)));
            });
            // The locked-slot outline and the lock button of the 1v1 Shop.
            card.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(7)),
                    ..default()
                },
                BorderColor::all(chrome::GOLD),
                Visibility::Hidden,
                Card::LockFrame(i),
            ));
            card.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: px(8),
                    top: px(12),
                    width: px(27),
                    height: px(28),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(5)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(chrome::INK),
                BorderColor::all(chrome::GOLD),
                Action::ToggleLock(i),
                ZIndex(4),
            ))
            .with_children(|lock| {
                lock.spawn((
                    Node {
                        width: px(15),
                        height: px(16),
                        ..default()
                    },
                    ImageNode::new(ctx.assets.load("battle/native/shop-lock-open.webp")),
                    Card::Lock(i),
                ));
            });
        });
}

fn destination(parent: &mut ChildSpawnerCommands, ctx: &Ctx) {
    parent
        .spawn((
            Node {
                width: px(190),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(10)),
                row_gap: px(6),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(7)),
                ..default()
            },
            frame(chrome::GOLD),
            crate::input::Blocks,
        ))
        .with_children(|p| {
            p.spawn(title(ctx.lang.t(Key::Destination), 16., &ctx.fonts));
            for (i, (to, hotkey)) in Destination::ALL
                .into_iter()
                .zip(["Z", "X", "C", "V"])
                .enumerate()
            {
                p.spawn((
                    Node {
                        height: px(34),
                        padding: UiRect::horizontal(px(10)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(px(5)),
                        ..default()
                    },
                    BackgroundColor(chrome::INK),
                    BorderColor::all(Color::srgb_u8(60, 72, 80)),
                    Action::SetDestination(i),
                    DestinationRow(i),
                ))
                .with_children(|r| {
                    r.spawn(title(ctx.lang.lane(to), 15., &ctx.fonts));
                    r.spawn(chrome::muted(hotkey, 11.));
                });
            }
        });
}

fn player_card(root: &mut ChildSpawnerCommands, ctx: &Ctx) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(12),
            bottom: px(20),
            width: px(300),
            padding: UiRect::all(px(10)),
            column_gap: px(12),
            align_items: AlignItems::Center,
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(7)),
            ..default()
        },
        frame(chrome::GOLD),
        crate::input::Blocks,
    ))
    .with_children(|p| {
        p.spawn((
            Node {
                width: px(58),
                height: px(68),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::bottom(px(10)),
                ..default()
            },
            ImageNode::new(ctx.assets.load("battle/native/crest-ally.webp")),
            Emblem,
        ))
        .with_children(|s| {
            s.spawn((title("1", 26., &ctx.fonts), Readout::Seat));
        });
        p.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            ..default()
        })
        .with_children(|c| {
            c.spawn((title(ctx.lang.t(Key::You), 18., &ctx.fonts), Readout::Who));
            c.spawn((chrome::muted("", 12.), Readout::Line));
        });
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
            Readout::Upgrade if me.tier >= MAX_TIER => format!("T{} MAX", me.tier),
            Readout::Upgrade => format!("T{} · +{}", me.tier, content::upgrade_cost(me.tier)),
            Readout::Army => format!("{} {}/{}", lang.t(Key::Army), me.troops, content::TROOP_CAP),
            Readout::Reroll => format!("Reroll · {} {}", content::REROLL_COST, lang.t(Key::Gold)),
            Readout::Seat => (me.seat.0 % 5 + 1).to_string(),
            Readout::Who => format!("{} ({})", lang.t(Key::You), lang.role(me.seat.role())),
            Readout::Line => format!(
                "{} {}/{} · {} {} · {}/{}",
                lang.t(Key::Army),
                me.troops,
                content::TROOP_CAP,
                lang.t(Key::Tier),
                me.tier,
                seat.kills,
                seat.deaths
            ),
        };
        if t.0 != value {
            t.0 = value;
        }
    }
}

#[allow(clippy::type_complexity)] // one card's image, texts, lock and outline
fn cards(
    feed: Res<Feed>,
    assets: Res<AssetServer>,
    mut images: Query<(&Card, &mut ImageNode)>,
    mut texts: Query<(&Card, &mut Text)>,
    mut frames: Query<(&Card, &mut Visibility)>,
) {
    let Some(me) = &feed.me else { return };
    let locked = |i: usize| me.locked & (1 << i) != 0;
    for (card, mut image) in &mut images {
        match *card {
            Card::Surface(i) => {
                // Unaffordable offers dim, exactly like the 1v1 Shop.
                let wanted = if me.gold >= content::troop_cost(me.offers[i]) {
                    Color::WHITE
                } else {
                    Color::srgb(0.65, 0.65, 0.65)
                };
                if image.color != wanted {
                    image.color = wanted;
                }
            }
            Card::Portrait(i) => {
                let handle = chrome::portrait(&assets, me.offers[i]);
                if image.image != handle {
                    image.image = handle;
                }
            }
            Card::Lock(i) => {
                let name = if locked(i) { "closed" } else { "open" };
                let handle = assets.load(format!("battle/native/shop-lock-{name}.webp"));
                if image.image != handle {
                    image.image = handle;
                }
            }
            _ => {}
        }
    }
    for (card, mut t) in &mut texts {
        let def = |i: usize| tbg_core::content::def(me.offers[i]);
        let value = match *card {
            Card::Name(i) => def(i).name.to_string(),
            Card::Role(i) => format!("{:?}", def(i).archetype),
            Card::Rules(i) => {
                use tbg_core::types::Archetype;
                [
                    Archetype::Tank,
                    Archetype::Dps,
                    Archetype::Support,
                    Archetype::Ranged,
                    Archetype::Control,
                ]
                .into_iter()
                .filter(|a| me.slot_rules[i] & (1 << *a as u8) != 0)
                .map(|a| format!("{a:?}"))
                .collect::<Vec<_>>()
                .join(" · ")
            }
            Card::Cost(i) => content::troop_cost(me.offers[i]).to_string(),
            _ => continue,
        };
        if t.0 != value {
            t.0 = value;
        }
    }
    for (card, mut vis) in &mut frames {
        if let Card::LockFrame(i) = *card {
            let wanted = if locked(i) {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            if *vis != wanted {
                *vis = wanted;
            }
        }
    }
}

fn destinations(
    selection: Res<Selection>,
    mut rows: Query<(&DestinationRow, &mut BorderColor, &mut BackgroundColor)>,
) {
    for (DestinationRow(i), mut border, mut background) in &mut rows {
        let on = selection.destination == *i;
        let wanted = BorderColor::all(if on {
            chrome::GOLD
        } else {
            Color::srgb_u8(60, 72, 80)
        });
        if *border != wanted {
            *border = wanted;
        }
        let shade = if on {
            Color::srgb_u8(58, 46, 18)
        } else {
            chrome::INK
        };
        if background.0 != shade {
            background.0 = shade;
        }
    }
}

fn emblem(
    feed: Res<Feed>,
    assets: Res<AssetServer>,
    mut shields: Query<&mut ImageNode, With<Emblem>>,
) {
    let Some(seat) = feed.seat else { return };
    let side = if seat.team() == tbg_moba_core::Team::Blue {
        "ally"
    } else {
        "enemy"
    };
    let handle = assets.load(format!("battle/native/crest-{side}.webp"));
    for mut image in &mut shields {
        if image.image != handle {
            image.image = handle.clone();
        }
    }
}
