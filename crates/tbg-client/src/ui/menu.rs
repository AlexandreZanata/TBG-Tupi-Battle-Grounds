//! Main menu: practice and casual play, plus ranked play for accounts.
//! Rebuilt when the account changes while the menu is open.

use bevy::prelude::*;

use super::account::{Account, AccountScreen};
use super::ranking::Ranking;
use super::{button, label, screen, UiAction, PANEL};
use crate::audio::Sounds;
use crate::GameState;

/// The mute toggle; its label follows `Sounds.muted`.
#[derive(Component)]
struct SoundButton;

#[derive(Component)]
struct MenuRoot;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Menu), spawn)
        .add_systems(
            Update,
            (
                sound_label,
                hide_under_panels,
                refresh.run_if(resource_changed::<Account>),
            )
                .run_if(in_state(GameState::Menu)),
        );
}

/// Rebuild the menu when the profile arrives or leaves mid-menu.
fn refresh(mut commands: Commands, account: Res<Account>, roots: Query<Entity, With<MenuRoot>>) {
    if roots.is_empty() {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }
    build(&mut commands, &account);
}

/// Modal panels (ranking, save your key) hide the menu: taps hit-test the
/// first visible button, so two overlapping buttons must never both show.
fn hide_under_panels(
    ranking: Res<Ranking>,
    account: Res<Account>,
    mut roots: Query<&mut Visibility, With<MenuRoot>>,
) {
    let modal = ranking.open || matches!(account.screen, AccountScreen::SaveKey { .. });
    let want = if modal {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    for mut v in &mut roots {
        if *v != want {
            *v = want;
        }
    }
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

fn spawn(mut commands: Commands, account: Res<Account>) {
    build(&mut commands, &account);
}

fn panel() -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(px(24.0)),
            row_gap: px(12.0),
            border_radius: BorderRadius::all(px(16.0)),
            ..default()
        },
        BackgroundColor(PANEL),
    )
}

fn build(commands: &mut Commands, account: &Account) {
    // P11 profile chip: guest actions, or name · rating plus log out.
    // P12 ranked section: your rating on the button, or the locked hint.
    let mut root = commands.spawn((
        screen(JustifyContent::Center),
        DespawnOnExit(GameState::Menu),
        MenuRoot,
    ));
    match &account.profile {
        // Logged in: name · rating, log out, ranked with your rating.
        Some(p) => {
            let title = format!("{} · {}", p.name, p.rating);
            let ranked = format!("Ranked Match ({})", p.rating);
            root.with_children(|root| {
                root.spawn((
                    panel(),
                    children![
                        label("Tupi Battle Grounds", 34.0),
                        label("Real-time 1v1 lane strategy", 16.0),
                        label(title, 14.0),
                        button("Log out", UiAction::Logout, 200.0),
                        label("Casual", 20.0),
                        button("Practice vs Bot", UiAction::Practice, 260.0),
                        button("Quick Match", UiAction::Quick, 260.0),
                        button("Play with a friend", UiAction::CreateRoom, 260.0),
                        label("Ranked", 20.0),
                        button(&ranked, UiAction::Ranked, 260.0),
                        button("Ranking", UiAction::OpenRanking, 260.0),
                        (
                            button("Sound: on", UiAction::ToggleSound, 260.0),
                            SoundButton
                        ),
                    ],
                ));
            });
        }
        // Guest: create-account entries, ranked locked.
        None => {
            root.with_children(|root| {
                root.spawn((
                    panel(),
                    children![
                        label("Tupi Battle Grounds", 34.0),
                        label("Real-time 1v1 lane strategy", 16.0),
                        label("Guest", 14.0),
                        button("Create account", UiAction::CreateAccount, 200.0),
                        button("I have a key", UiAction::HaveKey, 200.0),
                        label("Casual", 20.0),
                        button("Practice vs Bot", UiAction::Practice, 260.0),
                        button("Quick Match", UiAction::Quick, 260.0),
                        button("Play with a friend", UiAction::CreateRoom, 260.0),
                        label("Ranked", 20.0),
                        button(
                            "Create an account to play ranked",
                            UiAction::Disabled,
                            260.0
                        ),
                        button("Ranking", UiAction::OpenRanking, 260.0),
                        (
                            button("Sound: on", UiAction::ToggleSound, 260.0),
                            SoundButton
                        ),
                    ],
                ));
            });
        }
    }
}
