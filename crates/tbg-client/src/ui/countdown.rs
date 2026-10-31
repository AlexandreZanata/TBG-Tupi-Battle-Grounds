//! Large pre-match countdown centered on the BattleLine for every viewport.

use bevy::prelude::*;
use tbg_core::{content::TICK_HZ, Phase};

use crate::{sync::LatestView, GameState};

#[derive(Component)]
struct Countdown;

#[derive(Component)]
struct CountdownNumber;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::InMatch), spawn)
        .add_systems(Update, update.run_if(in_state(GameState::InMatch)));
}

fn spawn(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Countdown,
            Visibility::Hidden,
            ZIndex(10),
            DespawnOnExit(GameState::InMatch),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: if cfg!(target_os = "linux") {
                        px(208)
                    } else {
                        Val::Auto
                    },
                    height: if cfg!(target_os = "linux") {
                        px(186)
                    } else {
                        Val::Auto
                    },
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: if cfg!(target_os = "linux") {
                        JustifyContent::Center
                    } else {
                        JustifyContent::FlexStart
                    },
                    padding: UiRect::axes(px(28), px(12)),
                    border_radius: BorderRadius::all(px(14)),
                    ..default()
                },
                BackgroundColor(if cfg!(target_os = "linux") {
                    Color::srgba(0.12, 0.11, 0.08, 0.73)
                } else {
                    Color::srgba(0.04, 0.08, 0.12, 0.88)
                }),
                children![
                    super::label(
                        "STARTING",
                        if cfg!(target_os = "linux") { 28. } else { 20. }
                    ),
                    (
                        Text::new(""),
                        TextFont {
                            font: if cfg!(target_os = "linux") {
                                super::kit::display_font()
                            } else {
                                default()
                            },
                            font_size: FontSize::Px(if cfg!(target_os = "linux") {
                                100.
                            } else {
                                72.
                            }),
                            ..default()
                        },
                        TextColor(Color::srgb_u8(0xF5, 0xDF, 0xB6)),
                        TextShadow {
                            offset: Vec2::splat(2.0),
                            color: Color::BLACK
                        },
                        CountdownNumber,
                    ),
                ],
            ));
        });
}

fn update(
    view: Res<LatestView>,
    mut roots: Query<&mut Visibility, With<Countdown>>,
    mut numbers: Query<&mut Text, With<CountdownNumber>>,
) {
    let remaining = view
        .0
        .as_ref()
        .filter(|v| v.phase == Phase::Countdown)
        .map(|v| v.phase_ticks_left.div_ceil(TICK_HZ).max(1));
    for mut visibility in &mut roots {
        *visibility = if remaining.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if let Some(remaining) = remaining {
        for mut text in &mut numbers {
            text.0 = remaining.to_string();
        }
    }
}
