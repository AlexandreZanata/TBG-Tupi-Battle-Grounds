//! Main menu: practice and casual play, plus ranked play for accounts.
//! Rebuilt when the account changes while the menu is open.

use bevy::prelude::*;

use super::account::{Account, AccountScreen};
use super::ranking::Ranking;
use super::{button, label, screen, UiAction, PANEL};
use crate::audio::Sounds;
use crate::i18n::{Lang, Msg};
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
                refresh.run_if(resource_changed::<Account>.or_eager(resource_changed::<Lang>)),
            )
                .run_if(in_state(GameState::Menu)),
        );
}

/// Rebuild the menu when the profile or the language changes mid-menu.
fn refresh(
    mut commands: Commands,
    account: Res<Account>,
    lang: Res<Lang>,
    roots: Query<Entity, With<MenuRoot>>,
) {
    if roots.is_empty() {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }
    build(&mut commands, &account, *lang);
}

/// Modal panels (ranking, save your key) hide the menu: taps hit-test the
/// first visible button, so two overlapping buttons must never both show.
fn hide_under_panels(
    ranking: Res<Ranking>,
    account: Res<Account>,
    desktop: Res<super::desktop::DesktopLobby>,
    mut roots: Query<&mut Visibility, With<MenuRoot>>,
) {
    let modal =
        desktop.0 || ranking.open || matches!(account.screen, AccountScreen::SaveKey { .. });
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
    lang: Res<Lang>,
    buttons: Query<&Children, With<SoundButton>>,
    mut texts: Query<&mut Text>,
) {
    for children in &buttons {
        for child in children.iter() {
            if let Ok(mut t) = texts.get_mut(child) {
                let want = lang.t(if sounds.muted {
                    Msg::MenuSoundOff
                } else {
                    Msg::MenuSoundOn
                });
                if t.0 != want {
                    t.0 = want.into();
                }
            }
        }
    }
}

fn spawn(mut commands: Commands, account: Res<Account>, lang: Res<Lang>) {
    build(&mut commands, &account, *lang);
}

fn panel() -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(px(16.0)),
            row_gap: px(8.0),
            border_radius: BorderRadius::all(px(16.0)),
            ..default()
        },
        BackgroundColor(PANEL),
    )
}

/// Two side-by-side children (keeps the menu short enough for a
/// 1280×800 screen at UI scale 1.5).
fn pair(a: impl Bundle, b: impl Bundle) -> impl Bundle {
    (
        Node {
            column_gap: px(12.0),
            ..default()
        },
        children![a, b],
    )
}

fn build(commands: &mut Commands, account: &Account, lang: Lang) {
    if crate::config::arena_preview() {
        return;
    }
    // P11 profile chip: guest actions, or name · rating plus log out.
    // P12 ranked section: your rating on the button, or the locked hint.
    // P13: Ranking panel next to the sound toggle.
    let mut root = commands.spawn((
        screen(JustifyContent::Center),
        DespawnOnExit(GameState::Menu),
        MenuRoot,
    ));
    let (chip, actions, ranked) = match &account.profile {
        Some(p) => (
            format!("{} ({})", p.name, p.rating),
            None,
            button(
                &lang.tf(Msg::MenuRankedMatch, &[("rating", &p.rating)]),
                UiAction::Ranked,
                260.0,
            ),
        ),
        None => (
            lang.t(Msg::LobbyGuestName).to_string(),
            Some(()),
            button(lang.t(Msg::MenuRankedLocked), UiAction::Disabled, 260.0),
        ),
    };
    root.with_children(|root| {
        root.spawn(panel()).with_children(|p| {
            p.spawn(label(lang.t(Msg::LobbyBrandName), 34.0));
            p.spawn(label(lang.t(Msg::MenuTagline), 16.0));
            p.spawn(label(chip, 14.0));
            match actions {
                Some(()) => p.spawn(pair(
                    button(
                        lang.t(Msg::LobbyAccountCreate),
                        UiAction::CreateAccount,
                        180.0,
                    ),
                    button(lang.t(Msg::MenuHaveKey), UiAction::HaveKey, 180.0),
                )),
                None => p.spawn(button(
                    lang.t(Msg::LobbySettingsLogout),
                    UiAction::Logout,
                    200.0,
                )),
            };
            p.spawn(label(lang.t(Msg::LobbyModeCasualTitle), 20.0));
            p.spawn(button(
                lang.t(Msg::LobbyModePracticeTitle),
                UiAction::Practice,
                260.0,
            ));
            p.spawn(button(lang.t(Msg::MenuQuick), UiAction::Quick, 260.0));
            p.spawn(button(
                lang.t(Msg::MenuChallenge),
                UiAction::CreateChallenge,
                260.0,
            ));
            p.spawn(button(
                lang.t(Msg::MenuPrivateRoom),
                UiAction::CreateRoom,
                260.0,
            ));
            p.spawn(label(lang.t(Msg::LobbyModeRankedTitle), 20.0));
            p.spawn(ranked);
            p.spawn(pair(
                button(lang.t(Msg::LobbyTabsRanking), UiAction::OpenRanking, 124.0),
                (
                    button(lang.t(Msg::MenuSoundOn), UiAction::ToggleSound, 124.0),
                    SoundButton,
                ),
            ));
        });
    });
}
