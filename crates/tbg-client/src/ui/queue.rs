//! Waiting screen: quick match, private room code, or Quick Challenge link.
//! After 30 s of searching, practice is offered.

use bevy::prelude::*;
use bevy::text::LineBreak;

use super::{button, label, screen, UiAction, PANEL};
use crate::config;
use crate::i18n::{Lang, Msg};
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

fn spawn(mut commands: Commands, time: Res<Time>, lang: Res<Lang>) {
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
                (label(lang.t(Msg::QueueConnecting), 22.0), StatusText),
                (
                    label("", 12.0),
                    TextLayout::new(Justify::Left, LineBreak::AnyCharacter),
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
                            button(lang.t(Msg::QueueCopyLink), UiAction::CopyLink, 150.0),
                            QueueButton::Copy
                        ),
                        (
                            button(lang.t(Msg::QueueShare), UiAction::ShareChallenge, 120.0),
                            QueueButton::Share
                        ),
                        (
                            button(lang.t(Msg::QueuePractice), UiAction::Practice, 220.0),
                            QueueButton::Practice
                        ),
                    ],
                ),
                button(lang.t(Msg::QueueCancel), UiAction::Cancel, 220.0),
            ],
        )],
    ));
}

fn update(
    time: Res<Time>,
    entered: Res<Entered>,
    info: Res<QueueInfo>,
    lang: Res<Lang>,
    mut text: Single<&mut Text, (With<StatusText>, Without<LinkText>)>,
    mut link_text: Single<&mut Text, (With<LinkText>, Without<StatusText>)>,
    mut buttons: Query<(&QueueButton, &mut Node)>,
) {
    let waited = (time.elapsed_secs_f64() - entered.0) as u32;
    let clock = format!("{}:{:02}", waited / 60, waited % 60);
    let (status, link, copy, share, practice) = if let Some(token) = info.challenge {
        (
            [
                lang.t(Msg::QueueChallengeCreated).to_string(),
                lang.tf(Msg::QueueWaitingFriend, &[("clock", &clock)]),
            ]
            .join("\n"),
            config::challenge_link(&token),
            true,
            true,
            false,
        )
    } else if info.joining_challenge {
        (
            lang.t(Msg::QueueJoining).into(),
            String::new(),
            false,
            false,
            false,
        )
    } else if let Some(code) = info.room {
        (
            [
                lang.tf(Msg::QueueRoom, &[("code", &String::from_utf8_lossy(&code))]),
                lang.t(Msg::QueueShareLink).to_string(),
                lang.tf(Msg::QueueWaiting, &[("clock", &clock)]),
            ]
            .join("\n"),
            config::room_link(&String::from_utf8_lossy(&code)),
            true,
            false,
            false,
        )
    } else if info.searching {
        (
            lang.tf(Msg::QueueSearching, &[("clock", &clock)]),
            String::new(),
            false,
            false,
            waited >= 30,
        )
    } else {
        (
            lang.t(Msg::QueueConnecting).into(),
            String::new(),
            false,
            false,
            false,
        )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn challenge_waiting_queries_and_copy_share_controls_work_together() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .init_resource::<Lang>()
            .insert_resource(QueueInfo {
                challenge: Some([7; 16]),
                ..default()
            })
            .add_systems(Startup, spawn)
            .add_systems(Update, update);
        app.update();
        let mut texts = app.world_mut().query::<&Text>();
        assert!(texts
            .iter(app.world())
            .any(|t| t.0.contains("Waiting for your friend")));
        let mut buttons = app.world_mut().query::<(&QueueButton, &Node)>();
        for (kind, node) in buttons.iter(app.world()) {
            assert_eq!(
                node.display,
                if matches!(kind, QueueButton::Practice) {
                    Display::None
                } else {
                    Display::Flex
                }
            );
        }
    }
}
