//! Four equal Linux Stronghold cards with edge-to-edge, proportionate banner art.

use super::*;
use crate::ui::shell::catalog;

pub fn spawn(root: &mut ChildSpawnerCommands, assets: &AssetServer, lang: Lang) {
    let data = stronghold_data();
    root.spawn(kit::panel()).with_children(|p| {
        p.spawn(kit::heading(lang.key("lobby.tabs.stronghold"), 32.));
        p.spawn(kit::muted(text(&data["copy"], "intro"), 18.));
        p.spawn(kit::muted(text(&data["copy"], "scope"), 15.));
    });
    root.spawn(catalog::grid()).with_children(|grid| {
        for v in data["kinds"].as_array().into_iter().flatten() {
            let Some(kind) = crate::config::parse_stronghold(&text(v, "kind")) else {
                continue;
            };
            grid.spawn(catalog::card()).with_children(|card| {
                card.spawn(catalog::banner(
                    assets,
                    format!("stronghold/{}", text(&v["banner"], "display")),
                    2.5,
                ));
                card.spawn(catalog::body())
                    .with_children(|body| details(body, v, data, kind, lang));
            });
        }
    });
}
