//! Waiting screen: searching for an opponent, or a private room code with
//! a share link. After 30 s of searching, practice is offered.

use bevy::prelude::*;

use super::{button, label, screen, UiAction, PANEL};
use crate::sync::QueueInfo;
use crate::GameState;

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct PracticeOffer;

#[derive(Resource)]
struct Entered(f64);

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Queue), spawn)
        .add_systems(Update, update.run_if(in_state(GameState::Queue)));
}

fn spawn(mut commands: Commands, time: Res<Time>) {
    commands.insert_resource(Entered(time.elapsed_secs_f64()));
    commands.spawn((
        screen(JustifyContent::Center),
        DespawnOnExit(GameState::Queue),
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(px(24.0)),
                row_gap: px(12.0),
                border_radius: BorderRadius::all(px(16.0)),
                ..default()
            },
            BackgroundColor(PANEL),
            children![
                (label("Connecting...", 22.0), StatusText),
                button("Cancel", UiAction::Cancel, 220.0),
            ],
        )],
    ));
}

fn update(
    mut commands: Commands,
    time: Res<Time>,
    entered: Res<Entered>,
    info: Res<QueueInfo>,
    mut text: Single<&mut Text, With<StatusText>>,
    offers: Query<(), With<PracticeOffer>>,
) {
    let waited = (time.elapsed_secs_f64() - entered.0) as u32;
    let clock = format!("{}:{:02}", waited / 60, waited % 60);
    text.0 = match (&info.room, info.searching) {
        (Some(code), _) => format!(
            "Room {}\nShare the link with a friend\nWaiting {clock}",
            String::from_utf8_lossy(code)
        ),
        (None, true) => format!("Searching for an opponent... {clock}"),
        (None, false) => "Connecting...".into(),
    };
    let wants_offer = info.room.is_some() || (info.searching && waited >= 30);
    if wants_offer && offers.is_empty() {
        let (text, action) = if info.room.is_some() {
            ("Copy link", UiAction::CopyLink)
        } else {
            ("Practice while you wait", UiAction::Practice)
        };
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: percent(12.0),
                justify_self: JustifySelf::Center,
                ..default()
            },
            PracticeOffer,
            DespawnOnExit(GameState::Queue),
            children![button(text, action, 260.0)],
        ));
    }
}
