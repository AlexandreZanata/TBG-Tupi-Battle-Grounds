//! Four-column Linux Arena cards with full-width art and direct Practice selection.

use super::*;
use crate::ui::shell::catalog;

pub fn spawn(root: &mut ChildSpawnerCommands, assets: &AssetServer, lang: Lang) {
    root.spawn(kit::panel()).with_children(|p| {
        p.spawn(kit::heading(lang.key("arenas.title"), 32.));
        p.spawn(kit::muted(lang.key("arenas.intro"), 18.));
        p.spawn(kit::muted(lang.key("arenas.online"), 15.));
    });
    root.spawn(catalog::grid()).with_children(|grid| {
        for arena in tbg_core::Arena::ALL {
            grid.spawn(catalog::card()).with_children(|card| {
                let slug = arena.slug();
                card.spawn(catalog::banner(
                    assets,
                    format!("arenas/{slug}.webp"),
                    16. / 9.,
                ));
                card.spawn(catalog::body())
                    .with_children(|body| details(body, arena, lang));
            });
        }
    });
}

fn details(body: &mut ChildSpawnerCommands, arena: tbg_core::Arena, lang: Lang) {
    let slug = arena.slug();
    body.spawn((
        kit::heading(
            lang.key(&format!("arenas.{slug}.name")),
            if cfg!(target_os = "android") {
                18.
            } else {
                24.
            },
        ),
        Node {
            min_height: px(if cfg!(target_os = "android") { 44 } else { 30 }),
            #[cfg(target_os = "android")]
            flex_shrink: 0.,
            ..default()
        },
    ));
    body.spawn((
        kit::muted(lang.key(&format!("arenas.{slug}.theme")), 14.),
        Node {
            min_height: px(38),
            #[cfg(target_os = "android")]
            flex_shrink: 0.,
            ..default()
        },
    ));
    body.spawn((
        crate::ui::label(effect(arena, lang), 15.),
        Node {
            min_height: px(40),
            #[cfg(target_os = "android")]
            flex_shrink: 0.,
            ..default()
        },
    ));
    body.spawn(Node {
        flex_grow: 1.,
        ..default()
    });
    let chosen = crate::config::load_arena() == arena;
    body.spawn(button(
        lang.key(if chosen {
            "arenas.chosen"
        } else {
            "arenas.choose"
        }),
        Action::Arena(arena),
        0.,
    ))
    .insert(catalog::button_node());
    body.spawn(kit::button(
        lang.key("arenas.train"),
        UiAction::PracticeArena(arena),
        0.,
    ))
    .insert(catalog::button_node());
}
