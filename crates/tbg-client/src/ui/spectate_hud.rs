//! P47 spectator chrome: the battle HUD for a watcher. The seat-only controls
//! (Shop, Energy, Fortify, War Council, Surrender) are hidden, a LIVE badge
//! shows the watcher count, and a Leave button stops watching. Players see the
//! same badge as "N watching" (BR-LV05), without the spectator-only Leave.

use bevy::prelude::*;

use super::fortify::FortifyControl;
use super::hud::Hud;
use super::surrender::SurrenderButton;
use super::{button, UiAction};
use crate::i18n::{Lang, Msg};
use crate::sync::{LatestView, MatchInfo};
use crate::GameState;

/// The top-centre badge: `LIVE · N watching` for a watcher, `N watching` for a player.
#[derive(Component)]
struct LiveBadge;

/// The Leave button; shown only while watching.
#[derive(Component)]
struct LeaveWatch;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::InMatch), spawn)
        .add_systems(
            Update,
            (hide_seat_controls, badge)
                .chain()
                .run_if(in_state(GameState::InMatch)),
        );
}

fn spawn(mut commands: Commands, lang: Res<Lang>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(10.0),
                left: px(0.0),
                right: px(0.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: px(12.0),
                ..default()
            },
            // Above the HUD, never blocks clicks on the arena around it.
            GlobalZIndex(5),
            DespawnOnExit(GameState::InMatch),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new(String::new()),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                LiveBadge,
            ));
            root.spawn((
                button(lang.t(Msg::SpectateLeave), UiAction::LeaveWatch, 120.0),
                LeaveWatch,
                Visibility::Hidden,
            ));
        });
}

/// Hide what a watcher cannot use. The Energy bar, Shop cards and War Council
/// sit on the same column as the badge; hiding each one keeps the layout.
fn hide_seat_controls(
    info: Res<MatchInfo>,
    mut hud: Query<(&Hud, &mut Visibility), Without<LeaveWatch>>,
    mut seat_only: Query<
        &mut Visibility,
        (
            Or<(With<SurrenderButton>, With<FortifyControl>)>,
            Without<Hud>,
            Without<LeaveWatch>,
        ),
    >,
) {
    let hidden = if info.spectating {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    for (element, mut visibility) in &mut hud {
        if element.seat_only() {
            visibility.set_if_neq(hidden);
        }
    }
    for mut visibility in &mut seat_only {
        visibility.set_if_neq(hidden);
    }
}

fn badge(
    info: Res<MatchInfo>,
    latest: Res<LatestView>,
    lang: Res<Lang>,
    mut badge: Query<(&mut Text, &mut Visibility), (With<LiveBadge>, Without<LeaveWatch>)>,
    mut leave: Query<&mut Visibility, (With<LeaveWatch>, Without<LiveBadge>)>,
) {
    let watchers = latest.0.as_ref().map_or(0, |v| v.spectators);
    let choosing = latest.0.as_ref().is_some_and(|v| v.augment_round.is_some());
    let line = if info.spectating {
        let mut line = lang.tf(
            Msg::SpectateLive,
            &[("count", &watchers as &dyn std::fmt::Display)],
        );
        if choosing {
            line.push_str(" · ");
            line.push_str(lang.t(Msg::SpectatePlayersChoosing));
        }
        Some(line)
    } else if watchers > 0 {
        Some(lang.tf(
            Msg::SpectateWatchingBadge,
            &[("count", &watchers as &dyn std::fmt::Display)],
        ))
    } else {
        None
    };
    for (mut text, mut visibility) in &mut badge {
        let shown = line.clone().unwrap_or_default();
        if text.0 != shown {
            text.0 = shown;
        }
        visibility.set_if_neq(if line.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    let watching = if info.spectating {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut leave {
        visibility.set_if_neq(watching);
    }
}
