//! Public live-match browser and online-account list with watch/share controls.

use super::{button, data::Data, kit, Action, Lang, UiAction};
use bevy::prelude::*;

pub fn spawn(root: &mut ChildSpawnerCommands, data: &Data, lang: Lang) {
    root.spawn(kit::panel()).with_children(|p| {
        p.spawn(kit::heading(lang.key("live.dialog.title"), 32.));
        if data.unavailable {
            p.spawn(kit::muted(lang.key("live.unavailable"), 16.));
        }
        let Some(live) = &data.live else {
            #[cfg(tbg_native_ui)]
            {
                if data.live_pending() {
                    crate::ui::native_loading::rows(p, 3);
                }
                return;
            }
            #[cfg(not(tbg_native_ui))]
            {
                p.spawn(kit::muted(lang.key("live.loading"), 18.));
                return;
            }
        };
        if !live.complete {
            p.spawn(kit::muted(lang.key("p73.partial"), 16.));
        }
        if live.stale || data.unavailable {
            p.spawn(kit::muted(lang.key("p73.stale"), 16.));
        }
        p.spawn(kit::muted(
            format!("{} · {} · {}", live.players, live.battle, live.watching),
            16.,
        ));
        if live.matches.is_empty() {
            p.spawn(kit::muted(lang.key("live.none"), 20.));
        }
        for battle in &live.matches {
            p.spawn(kit::panel()).with_children(|card| {
                card.spawn(kit::heading(
                    format!("{}  vs  {}", battle.names[0], battle.names[1]),
                    24.,
                ));
                card.spawn(kit::muted(
                    format!(
                        "{} · {} · {}:{:02} · {} {}",
                        lang.key(&format!("live.mode.{}", battle.mode)),
                        lang.key(&format!("arenas.{}.name", battle.arena)),
                        battle.elapsed / 60,
                        battle.elapsed % 60,
                        battle.spectators,
                        lang.key("native.spectators")
                    ),
                    15.,
                ));
                card.spawn(kit::muted(
                    format!("HP  {}% / {}%", battle.hp[0] / 100, battle.hp[1] / 100),
                    15.,
                ));
                card.spawn(kit::row()).with_children(|r| {
                    r.spawn(kit::button(
                        lang.key("live.watch"),
                        UiAction::Watch(battle.id),
                        180.,
                    ));
                    r.spawn(button(
                        lang.key("native.copy_link"),
                        Action::CopyWatch(battle.id),
                        180.,
                    ));
                });
            });
        }
    });
}
