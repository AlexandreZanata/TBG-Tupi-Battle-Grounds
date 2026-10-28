//! Main menu: practice now, online modes arrive in P4.

use bevy::prelude::*;

use super::{button, label, screen, UiAction, PANEL};
use crate::GameState;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Menu), spawn);
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
                button("Quick Match (online soon)", UiAction::Disabled, 260.0),
                button(
                    "Play with a friend (online soon)",
                    UiAction::Disabled,
                    260.0
                ),
            ],
        )],
    ));
}
