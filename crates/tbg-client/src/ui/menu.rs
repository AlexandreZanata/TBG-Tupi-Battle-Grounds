//! Main menu: practice vs bot, quick match, or a private room with a friend.

use bevy::prelude::*;

use super::{button, label, screen, UiAction, PANEL};
use crate::audio::Sounds;
use crate::GameState;

/// The mute toggle; its label follows `Sounds::muted`.
#[derive(Component)]
struct SoundButton;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Menu), spawn)
        .add_systems(Update, sound_label.run_if(in_state(GameState::Menu)));
}

fn sound_label(
    sounds: Res<Sounds>,
    buttons: Query<&Children, With<SoundButton>>,
    mut texts: Query<&mut Text>,
) {
    for children in &buttons {
        for child in children.iter() {
            if let Ok(mut t) = texts.get_mut(child) {
                let want = if sounds.muted {
                    "Sound: off"
                } else {
                    "Sound: on"
                };
                if t.0 != want {
                    t.0 = want.into();
                }
            }
        }
    }
}

fn spawn(mut commands: Commands) {
    commands.spawn((
        screen(JustifyContent::Center),
        DespawnOnExit(GameState::Menu),
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(px(24.0)),
                row_gap: px(14.0),
                border_radius: BorderRadius::all(px(16.0)),
                ..default()
            },
            BackgroundColor(PANEL),
            children![
                label("Tupi Battle Grounds", 34.0),
                label("Real-time 1v1 lane strategy", 16.0),
                button("Practice vs Bot", UiAction::Practice, 260.0),
                button("Quick Match", UiAction::Quick, 260.0),
                button("Play with a friend", UiAction::CreateRoom, 260.0),
                (
                    button("Sound: on", UiAction::ToggleSound, 260.0),
                    SoundButton
                ),
            ],
        )],
    ));
}
