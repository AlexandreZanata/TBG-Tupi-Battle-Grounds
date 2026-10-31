//! Linux preferences arranged into readable language, audio, video and account panels.

mod controls;
use super::*;
use crate::ui::shell::catalog;
use bevy::window::{PresentMode, WindowMode};

pub fn spawn(
    root: &mut ChildSpawnerCommands,
    account: &Account,
    sounds: &Sounds,
    window: &Window,
    lang: Lang,
    volume: f32,
    assets: &AssetServer,
) {
    controls::heading(root, assets, lang);
    root.spawn(Node {
        grid_template_columns: RepeatedGridTrack::flex(2, 1.),
        ..catalog::grid()
    })
    .with_children(|grid| {
        language(grid, assets, lang);
        audio(grid, assets, sounds, volume, lang);
        video(grid, assets, window, lang);
        account_panel(grid, assets, account, lang);
    });
}

fn words(lang: Lang, en: &str, pt: &str) -> String {
    if lang == Lang::PtBr {
        pt.into()
    } else {
        en.into()
    }
}

fn state(lang: Lang, on: bool) -> String {
    lang.key(if on {
        "lobby.sound.on"
    } else {
        "lobby.sound.off"
    })
}

fn language(parent: &mut ChildSpawnerCommands, assets: &AssetServer, lang: Lang) {
    controls::section(
        parent,
        assets,
        lang.key("lobby.settings.language"),
        "scroll",
        |card| {
            card.spawn(kit::muted(
                words(
                    lang,
                    "Choose your interface language.",
                    "Escolha o idioma da interface.",
                ),
                13.,
            ));
            card.spawn(controls::group()).with_children(|choices| {
                for (name, value) in [("English", Lang::En), ("Português (Brasil)", Lang::PtBr)] {
                    choices.spawn((
                        controls::button(name.into(), lang == value),
                        UiAction::SetLang(value),
                    ));
                }
            });
            card.spawn(kit::muted(
                words(
                    lang,
                    "You can change this whenever you return to Settings.",
                    "Você pode mudar esta preferência quando quiser nas Configurações.",
                ),
                11.,
            ));
        },
    );
}

fn audio(
    parent: &mut ChildSpawnerCommands,
    assets: &AssetServer,
    sounds: &Sounds,
    volume: f32,
    lang: Lang,
) {
    controls::section(
        parent,
        assets,
        words(lang, "Audio", "Áudio"),
        "sound",
        |card| {
            controls::row(
                card,
                words(lang, "Game sound", "Som do jogo"),
                words(
                    lang,
                    "Combat and interface effects",
                    "Efeitos de combate e da interface",
                ),
                (
                    controls::button(state(lang, !sounds.muted), !sounds.muted),
                    UiAction::ToggleSound,
                ),
            );
            let percent_value = (volume.clamp(0., 1.) * 100.).round() as u8;
            card.spawn(Node {
                width: percent(100),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|level| {
                level.spawn(kit::muted(
                    words(lang, "Master volume", "Volume geral"),
                    12.,
                ));
                level.spawn(kit::heading(format!("{percent_value}%"), 18.));
            });
            card.spawn((
                Node {
                    width: percent(100),
                    height: px(6),
                    border_radius: BorderRadius::MAX,
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(kit::INK),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        width: percent(percent_value),
                        height: percent(100),
                        ..default()
                    },
                    BackgroundColor(kit::GOLD),
                ));
            });
            card.spawn(controls::group()).with_children(|choices| {
                for value in [0, 25, 50, 75, 100] {
                    choices.spawn((
                        controls::button(format!("{value}%"), percent_value == value),
                        UiAction::Disabled,
                        Action::Volume(value),
                    ));
                }
            });
        },
    );
}

fn video(parent: &mut ChildSpawnerCommands, assets: &AssetServer, window: &Window, lang: Lang) {
    controls::section(
        parent,
        assets,
        words(lang, "Display & performance", "Tela e desempenho"),
        "settings",
        |card| {
            let fullscreen = window.mode != WindowMode::Windowed;
            controls::row(
                card,
                lang.key("native.fullscreen"),
                "F11 / Alt+Enter".into(),
                (
                    controls::button(
                        words(
                            lang,
                            if fullscreen { "Fullscreen" } else { "Windowed" },
                            if fullscreen { "Tela cheia" } else { "Janela" },
                        ),
                        fullscreen,
                    ),
                    UiAction::Disabled,
                    Action::Fullscreen,
                ),
            );
            let vsync = window.present_mode != PresentMode::AutoNoVsync;
            controls::row(
                card,
                "VSync".into(),
                words(
                    lang,
                    "Synchronize frames with your display",
                    "Sincronizar os quadros com a tela",
                ),
                (
                    controls::button(state(lang, vsync), vsync),
                    UiAction::Disabled,
                    Action::Vsync,
                ),
            );
            let shadows = !crate::config::load_flag("video.low");
            controls::row(
                card,
                lang.key("native.shadows"),
                words(
                    lang,
                    "Turn off to reduce graphics load",
                    "Desligue para reduzir o uso gráfico",
                ),
                (
                    controls::button(state(lang, shadows), shadows),
                    UiAction::Disabled,
                    Action::Quality,
                ),
            );
        },
    );
}

fn account_panel(
    parent: &mut ChildSpawnerCommands,
    assets: &AssetServer,
    account: &Account,
    lang: Lang,
) {
    controls::section(
        parent,
        assets,
        words(lang, "Account & app", "Conta e aplicativo"),
        "account-user",
        |card| {
            if let Some(profile) = &account.profile {
                card.spawn(kit::heading(profile.name.clone(), 18.));
                card.spawn(kit::muted(lang.key("lobby.settings.key_note"), 12.));
                card.spawn(controls::group()).with_children(|actions| {
                    actions.spawn((
                        controls::button(lang.key("lobby.settings.copy"), false),
                        UiAction::CopyKey,
                    ));
                    actions.spawn((
                        controls::button(lang.key("lobby.settings.download"), false),
                        UiAction::DownloadKey,
                    ));
                    actions.spawn((
                        controls::button(
                            if account.logout_armed {
                                words(lang, "Confirm log out", "Confirmar saída")
                            } else {
                                lang.key("lobby.settings.logout")
                            },
                            false,
                        ),
                        UiAction::Logout,
                    ));
                });
            } else {
                card.spawn(kit::muted(
                    words(
                        lang,
                        "Save your identity and compete in Ranked.",
                        "Salve sua identidade e participe das partidas ranqueadas.",
                    ),
                    12.,
                ));
                card.spawn(controls::group()).with_children(|actions| {
                    actions.spawn((
                        controls::button(lang.key("lobby.account.create"), false),
                        UiAction::CreateAccount,
                    ));
                    actions.spawn((
                        controls::button(lang.key("lobby.account.key"), false),
                        UiAction::HaveKey,
                    ));
                });
            }
            card.spawn(Node {
                flex_grow: 1.,
                min_height: px(0),
                ..default()
            });
            card.spawn(kit::muted(words(lang, "APPLICATION", "APLICATIVO"), 10.));
            card.spawn(controls::group()).with_children(|actions| {
                actions.spawn((
                    controls::button(lang.key("native.downloads"), false),
                    UiAction::Disabled,
                    Action::Download,
                ));
                actions.spawn((
                    controls::button(lang.key("native.kit"), false),
                    UiAction::Disabled,
                    Action::Open(Panel::Kit),
                ));
            });
        },
    );
}
