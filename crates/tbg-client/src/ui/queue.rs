//! Waiting screen: quick match, private room code, or Quick Challenge link.
//! After 30 s of searching, practice is offered.

use bevy::prelude::*;

use super::{button, label, screen, UiAction, PANEL};
use crate::config;
use crate::sync::QueueInfo;
use crate::GameState;

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct LinkText;

#[derive(Component, Clone, Copy)]
enum QueueButton {
    Copy,
    Share,
    Practice,
}

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
                (
                    label("", 12.0),
                    Node {
                        max_width: px(360.0),
                        ..default()
                    },
                    LinkText,
                ),
                (
                    Node {
                        column_gap: px(8.0),
                        ..default()
                    },
                    children![
                        (
                            button("Copy link", UiAction::CopyLink, 150.0),
                            QueueButton::Copy
                        ),
                        (
                            button("Share", UiAction::ShareChallenge, 120.0),
                            QueueButton::Share
                        ),
                        (
                            button("Practice while you wait", UiAction::Practice, 220.0),
                            QueueButton::Practice
                        ),
                    ],
                ),
                button("Cancel", UiAction::Cancel, 220.0),
            ],
        )],
    ));
}

fn update(
    time: Res<Time>,
    entered: Res<Entered>,
    info: Res<QueueInfo>,
    mut text: Single<&mut Text, (With<StatusText>, Without<LinkText>)>,
    mut link_text: Single<&mut Text, (With<LinkText>, Without<StatusText>)>,
    mut buttons: Query<(&QueueButton, &mut Node)>,
) {
    let waited = (time.elapsed_secs_f64() - entered.0) as u32;
    let clock = format!("{}:{:02}", waited / 60, waited % 60);
    let (status, link, copy, share, practice) = if let Some(token) = info.challenge {
        (
            format!("Challenge created\nWaiting for your friend {clock}"),
            config::challenge_link(&token),
            true,
            true,
            false,
        )
    } else if info.joining_challenge {
        (
            "Joining challenge...".into(),
            String::new(),
            false,
            false,
            false,
        )
    } else if let Some(code) = info.room {
        (
            format!(
                "Room {}\nShare the link with a friend\nWaiting {clock}",
                String::from_utf8_lossy(&code)
            ),
            config::room_link(&String::from_utf8_lossy(&code)),
            true,
            false,
            false,
        )
    } else if info.searching {
        (
            format!("Searching for an opponent... {clock}"),
            String::new(),
            false,
            false,
            waited >= 30,
        )
    } else {
        ("Connecting...".into(), String::new(), false, false, false)
    };
    text.0 = status;
    link_text.0 = link;
    for (kind, mut node) in &mut buttons {
        let visible = match kind {
            QueueButton::Copy => copy,
            QueueButton::Share => share,
            QueueButton::Practice => practice,
        };
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
    }
}
