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
            #[cfg(target_os = "linux")]
            {
                if data.live_pending() {
                    crate::ui::native_loading::rows(p, 3);
                }
                return;
            }
            #[cfg(not(target_os = "linux"))]
            {
                p.spawn(kit::muted(lang.key("live.loading"), 18.));
                return;
            }
        };
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
        p.spawn(kit::heading(
            lang.key("live.accounts")
                .replace("{count}", &data.online_total.to_string()),
            24.,
        ));
        #[cfg(target_os = "linux")]
        if data.online_pending() {
            crate::ui::native_loading::rows(p, 3);
            return;
        }
        if data.online.is_empty() {
            p.spawn(kit::muted(lang.key("live.accounts_none"), 16.));
        }
        for row in &data.online {
            p.spawn(super::super::label(
                format!(
                    "{} · {} · {}",
                    row.name,
                    row.rating.map_or_else(|| "—".into(), |r| r.to_string()),
                    lang.key(&format!("live.status.{}", row.status))
                ),
                16.,
            ));
        }
    });
}
