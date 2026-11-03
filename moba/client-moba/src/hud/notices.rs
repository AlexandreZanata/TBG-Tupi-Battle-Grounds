//! Kill feed, short toasts, connection state and the end-of-match overlay.

use bevy::prelude::*;
use tbg_moba_core::state::{EndReason, Event};
use tbg_moba_core::{Reject, Team};

use super::{panel, text, title, Ctx};
use crate::app::Screen;
use crate::i18n::{Key, Lang};
use crate::input::Action;
use crate::link::{FailReason, Feed, Status};
use crate::palette;

#[derive(Component)]
struct FeedList;
#[derive(Component)]
struct Toast;
#[derive(Component)]
struct Hint;
/// Seconds left before a feed line or toast disappears.
#[derive(Component)]
struct Ttl(f32);
#[derive(Component)]
struct Overlay;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (events, expire, overlay).run_if(in_state(Screen::Match)),
    );
}

pub fn spawn(root: &mut ChildSpawnerCommands, ctx: &Ctx) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(90),
            right: px(220),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexEnd,
            row_gap: px(4),
            ..default()
        },
        FeedList,
    ));
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(92),
            width: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(4),
            ..default()
        },
        Toast,
    ));
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(10),
            bottom: px(196),
            max_width: px(372),
            padding: UiRect::axes(px(10), px(5)),
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.04, 0.05, 0.08, 0.72)),
        Hint,
    ))
    .with_children(|h| {
        h.spawn(text(ctx.lang.t(Key::Hint), 13.0, palette::MUTED));
    });
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        Overlay,
    ));
}

fn line(lang: Lang, event: &Event, mine: Option<tbg_moba_core::Seat>) -> Option<(String, Color)> {
    let team_color = |t: Team| palette::glow(t);
    match *event {
        Event::CommanderKilled { victim, killer, by } => {
            let who = |s: tbg_moba_core::Seat| {
                if Some(s) == mine {
                    lang.t(Key::You).to_string()
                } else {
                    format!(
                        "{} {}",
                        lang.role(s.role()),
                        if s.team() == Team::Blue { "A" } else { "V" }
                    )
                }
            };
            let killer = killer.map_or_else(
                || {
                    lang.t(if by == Team::Blue {
                        Key::BlueTeam
                    } else {
                        Key::RedTeam
                    })
                    .to_string()
                },
                who,
            );
            Some((format!("{killer}  ×  {}", who(victim)), team_color(by)))
        }
        Event::TowerDestroyed { team, .. } => Some((
            lang.t(Key::TowerFallen).to_string(),
            team_color(team.enemy()),
        )),
        _ => None,
    }
}

fn reject_key(reject: Reject) -> Option<Key> {
    match reject {
        Reject::NotEnoughGold => Some(Key::NotEnoughGold),
        Reject::TroopCap => Some(Key::TroopCap),
        Reject::NotEnoughEssence => Some(Key::NotEnoughEssence),
        Reject::MaxTier => Some(Key::MaxTier),
        Reject::MatchOver | Reject::InvalidSeat => None,
    }
}

fn events(
    mut commands: Commands,
    mut feed: ResMut<Feed>,
    lang: Res<Lang>,
    list: Single<Entity, With<FeedList>>,
    toast: Single<Entity, With<Toast>>,
) {
    let mine = feed.seat;
    for event in std::mem::take(&mut feed.events) {
        if let Some((value, color)) = line(*lang, &event, mine) {
            let row = commands
                .spawn((
                    Node {
                        padding: UiRect::axes(px(10), px(3)),
                        border_radius: BorderRadius::all(px(6)),
                        ..default()
                    },
                    BackgroundColor(palette::PANEL),
                    Ttl(6.0),
                ))
                .with_children(|r| {
                    r.spawn(text(value, 15.0, color));
                })
                .id();
            commands.entity(*list).add_child(row);
        }
    }
    for reject in std::mem::take(&mut feed.rejects) {
        let Some(key) = reject_key(reject) else {
            continue;
        };
        let row = commands
            .spawn((
                Node {
                    padding: UiRect::axes(px(14), px(5)),
                    border_radius: BorderRadius::all(px(8)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.35, 0.05, 0.05, 0.9)),
                Ttl(2.2),
            ))
            .with_children(|r| {
                r.spawn(text(lang.t(key), 18.0, palette::TEXT));
            })
            .id();
        commands.entity(*toast).add_child(row);
    }
}

fn expire(mut commands: Commands, time: Res<Time>, mut lines: Query<(Entity, &mut Ttl)>) {
    for (entity, mut ttl) in &mut lines {
        ttl.0 -= time.delta_secs();
        if ttl.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// What the overlay should say: the result, or the link state while not live.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Shown {
    None,
    Connecting,
    Failed(FailReason),
    Ended(Option<Team>, EndReason),
}

#[allow(clippy::too_many_arguments)] // builds one modal from feed, language and fonts
fn overlay(
    mut commands: Commands,
    feed: Res<Feed>,
    lang: Res<Lang>,
    fonts: Res<crate::i18n::Fonts>,
    launch: Res<crate::config::Launch>,
    mut root: Single<(Entity, &mut Node), With<Overlay>>,
    mut last: Local<Option<Shown>>,
    children: Query<&Children>,
) {
    let shown = match (&feed.status, feed.curr.as_ref().and_then(|w| w.outcome)) {
        (Status::Connecting | Status::Idle, _) => Shown::Connecting,
        (Status::Failed(reason), _) => Shown::Failed(*reason),
        (Status::Live, Some(o)) => Shown::Ended(o.winner, o.reason),
        (Status::Live, None) => Shown::None,
    };
    if *last == Some(shown) {
        return;
    }
    *last = Some(shown);
    let (entity, node) = &mut *root;
    if let Ok(kids) = children.get(*entity) {
        for kid in kids.iter() {
            commands.entity(kid).despawn();
        }
    }
    node.display = if shown == Shown::None {
        Display::None
    } else {
        Display::Flex
    };
    let mine = feed.seat.map(|s| s.team());
    let (headline, color) = match shown {
        Shown::None => return,
        Shown::Connecting => (lang.t(Key::Connecting), palette::TEXT),
        Shown::Failed(FailReason::Full) => (lang.t(Key::RoomFull), palette::HP_ENEMY),
        Shown::Failed(FailReason::Lost) => (lang.t(Key::ConnectionLost), palette::HP_ENEMY),
        Shown::Ended(None, _) => (lang.t(Key::Draw), palette::TEXT),
        Shown::Ended(Some(w), _) if Some(w) == mine => (lang.t(Key::Victory), palette::GOLD),
        Shown::Ended(Some(_), _) => (lang.t(Key::Defeat), palette::HP_ENEMY),
    };
    let again = matches!(shown, Shown::Ended(..)) && !launch.online;
    commands.entity(*entity).with_children(|o| {
        o.spawn(panel(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(18),
                padding: UiRect::axes(px(56), px(30)),
                ..default()
            },
            palette::GOLD,
        ))
        .with_children(|p| {
            p.spawn(super::title(headline, 64.0, color, &fonts));
            if let Some(world) = &feed.curr {
                p.spawn(text(
                    format!(
                        "{}  {}  ×  {}  {}",
                        lang.t(Key::BlueTeam),
                        world.score[0],
                        world.score[1],
                        lang.t(Key::RedTeam)
                    ),
                    20.0,
                    palette::TEXT,
                ));
            }
            p.spawn(Node {
                column_gap: px(14),
                ..default()
            })
            .with_children(|buttons| {
                if again {
                    button(buttons, lang.t(Key::PlayAgain), Action::PlayAgain, &fonts);
                }
                button(buttons, lang.t(Key::BackToMenu), Action::ToMenu, &fonts);
            });
        });
    });
}

pub fn button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: Action,
    fonts: &crate::i18n::Fonts,
) {
    parent
        .spawn((
            Node {
                padding: UiRect::axes(px(26), px(10)),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(8)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(palette::GOLD),
            BorderColor::all(Color::srgb(1.0, 0.9, 0.55)),
            action,
        ))
        .with_children(|b| {
            b.spawn(title(label, 22.0, palette::SKY, fonts));
        });
}
