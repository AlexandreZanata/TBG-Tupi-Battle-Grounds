//! User-visible audio, language, video and account preferences, plus kit review.

use super::{button, kit, Account, Action, Lang, Panel, Sounds, UiAction};
use bevy::prelude::*;

pub fn spawn(
    root: &mut ChildSpawnerCommands,
    account: &Account,
    sounds: &Sounds,
    window: &Window,
    lang: Lang,
) {
    root.spawn(kit::panel()).with_children(|p| {
        p.spawn(kit::heading(lang.key("lobby.settings.title"), 32.));
        p.spawn(kit::heading(lang.key("lobby.settings.language"), 20.));
        p.spawn(kit::row()).with_children(|r| {
            r.spawn(kit::button("English", UiAction::SetLang(Lang::En), 180.));
            r.spawn(kit::button(
                "Português (Brasil)",
                UiAction::SetLang(Lang::PtBr),
                220.,
            ));
        });
        p.spawn(kit::button(
            format!(
                "{} {}",
                lang.key("lobby.sound.label"),
                lang.key(if sounds.muted {
                    "lobby.sound.off"
                } else {
                    "lobby.sound.on"
                })
            ),
            UiAction::ToggleSound,
            320.,
        ));
        p.spawn(kit::row()).with_children(|r| {
            for volume in [0, 25, 50, 75, 100] {
                r.spawn(button(format!("{volume}%"), Action::Volume(volume), 70.));
            }
        });
        p.spawn(button(
            format!("{} · F11 / Alt+Enter", lang.key("native.fullscreen")),
            Action::Fullscreen,
            320.,
        ));
        p.spawn(button(
            format!("VSync · {:?}", window.present_mode),
            Action::Vsync,
            320.,
        ));
        p.spawn(button(
            format!(
                "{}: {}",
                lang.key("native.shadows"),
                lang.key(if crate::config::load_flag("video.low") {
                    "lobby.sound.off"
                } else {
                    "lobby.sound.on"
                })
            ),
            Action::Quality,
            320.,
        ));
        if account.profile.is_some() {
            p.spawn(kit::heading(lang.key("native.account"), 22.));
            p.spawn(kit::muted(lang.key("lobby.settings.key_note"), 16.));
            p.spawn(kit::button(
                lang.key("lobby.settings.copy"),
                UiAction::CopyKey,
                320.,
            ));
            p.spawn(kit::button(
                lang.key("lobby.settings.download"),
                UiAction::DownloadKey,
                320.,
            ));
            p.spawn(kit::button(
                lang.key("lobby.settings.logout"),
                UiAction::Logout,
                320.,
            ));
        }
        p.spawn(button(lang.key("native.downloads"), Action::Download, 320.));
        p.spawn(button(
            lang.key("native.kit"),
            Action::Open(Panel::Kit),
            320.,
        ));
    });
}

pub fn gallery(root: &mut ChildSpawnerCommands, lang: Lang) {
    root.spawn(kit::panel()).with_children(|p| {
        p.spawn(kit::heading(lang.key("native.kit"),32.));
        p.spawn(kit::muted(lang.key("native.kit_note"),18.));
        p.spawn(kit::row()).with_children(|r| {
            r.spawn(kit::button(lang.key("native.enabled"),UiAction::ToggleSound,180.));
            r.spawn(kit::button(lang.key("native.disabled"),UiAction::Disabled,180.));
            r.spawn(kit::button(lang.key("lobby.account.key"),UiAction::HaveKey,240.));
        });
        p.spawn((Node{width:px(480),height:px(24),..default()},BackgroundColor(kit::INK),children![(Node{width:percent(65),height:percent(100),..default()},BackgroundColor(kit::GOLD))]));
        p.spawn(kit::heading("Aa Bb Çç Ãã Éé 0123456789",28.));
        p.spawn(kit::muted("BattleLine · PlayerIntention · Archetype · Faction · Energy · Shop · Evolution · Synergy · Stronghold",16.));
    });
}
