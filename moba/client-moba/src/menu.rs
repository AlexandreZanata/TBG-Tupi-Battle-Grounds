//! Title screen over the orbiting valley: role choice, practice or online.

use bevy::prelude::*;
use tbg_moba_core::Role;

use crate::app::Screen;
use crate::config::Launch;
use crate::hud::{despawn, panel, text, title};
use crate::i18n::{Fonts, Key, Lang};
use crate::input::Action;
use crate::palette;

#[derive(Component)]
struct MenuRoot;
#[derive(Component)]
struct RoleButton(usize);

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Menu), spawn)
        .add_systems(OnExit(Screen::Menu), despawn::<MenuRoot>)
        .add_systems(Update, highlight.run_if(in_state(Screen::Menu)))
        .add_systems(Startup, autostart);
}

/// `--play` / `--capture` skip the menu.
fn autostart(launch: Res<Launch>, mut next: ResMut<NextState<Screen>>) {
    if launch.play {
        next.set(Screen::Match);
    }
}

fn spawn(mut commands: Commands, lang: Res<Lang>, fonts: Res<Fonts>, launch: Res<Launch>) {
    let lang = *lang;
    commands
        .spawn((
            MenuRoot,
            Node {
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.25)),
        ))
        .with_children(|root| {
            root.spawn(panel(
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(16),
                    padding: UiRect::axes(px(48), px(30)),
                    ..default()
                },
                palette::GOLD,
            ))
            .with_children(|p| {
                p.spawn(title(lang.t(Key::Title), 56.0, palette::GOLD, &fonts));
                p.spawn(text(lang.t(Key::Subtitle), 18.0, palette::MUTED));
                p.spawn(text(lang.t(Key::ChooseRole), 18.0, palette::TEXT));
                p.spawn(Node {
                    column_gap: px(8),
                    ..default()
                })
                .with_children(|row| {
                    for (i, role) in Role::ALL.into_iter().enumerate() {
                        row.spawn((
                            Node {
                                width: px(118),
                                height: px(44),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(px(2)),
                                border_radius: BorderRadius::all(px(8)),
                                ..default()
                            },
                            BackgroundColor(palette::PANEL_LIGHT),
                            BorderColor::all(palette::GOLD_DARK),
                            Action::PickRole(i),
                            RoleButton(i),
                        ))
                        .with_children(|b| {
                            b.spawn(text(lang.role(role), 18.0, palette::TEXT));
                        });
                    }
                });
                p.spawn(Node {
                    column_gap: px(12),
                    margin: UiRect::top(px(10)),
                    ..default()
                })
                .with_children(|row| {
                    crate::hud::notices_button(
                        row,
                        lang.t(Key::PlayBots),
                        Action::PlayOffline,
                        &fonts,
                    );
                    let leave = if launch.practice_only {
                        Key::BackToLobby
                    } else {
                        crate::hud::notices_button(
                            row,
                            lang.t(Key::PlayOnline),
                            Action::PlayOnline,
                            &fonts,
                        );
                        Key::Quit
                    };
                    crate::hud::notices_button(row, lang.t(leave), Action::Quit, &fonts);
                });
            });
        });
}

fn highlight(
    launch: Res<Launch>,
    mut buttons: Query<(&RoleButton, &mut BorderColor, &mut BackgroundColor)>,
) {
    for (RoleButton(i), mut border, mut background) in &mut buttons {
        let on = Role::ALL[*i] == launch.role;
        let wanted = BorderColor::all(if on {
            palette::GOLD
        } else {
            palette::GOLD_DARK
        });
        if *border != wanted {
            *border = wanted;
        }
        let shade = if on {
            Color::srgba(0.35, 0.27, 0.08, 0.95)
        } else {
            palette::PANEL_LIGHT
        };
        if background.0 != shade {
            background.0 = shade;
        }
    }
}
