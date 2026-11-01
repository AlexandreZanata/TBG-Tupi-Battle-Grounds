//! Arena catalog and Practice selection, using canonical modifiers.

use super::{button, kit, Action, Lang, UiAction};
use bevy::prelude::*;

#[cfg(tbg_desktop)]
mod native;
#[cfg(tbg_desktop)]
pub use native::spawn;

#[cfg(not(tbg_desktop))]
pub fn spawn(root: &mut ChildSpawnerCommands, assets: &AssetServer, lang: Lang) {
    root.spawn(kit::panel()).with_children(|p| {
        p.spawn(kit::heading(lang.key("arenas.title"), 32.));
        p.spawn(kit::muted(lang.key("arenas.intro"), 18.));
        p.spawn(kit::muted(lang.key("arenas.online"), 15.));
    });
    root.spawn(Node {
        flex_wrap: FlexWrap::Wrap,
        column_gap: px(16),
        row_gap: px(16),
        ..default()
    })
    .with_children(|grid| {
        for arena in tbg_core::Arena::ALL {
            grid.spawn(kit::panel()).with_children(|p| {
                let slug = arena.slug();
                p.spawn(kit::image(
                    assets,
                    format!("arenas/{slug}.webp"),
                    300.,
                    150.,
                ));
                p.spawn(kit::heading(lang.key(&format!("arenas.{slug}.name")), 24.));
                p.spawn(kit::muted(lang.key(&format!("arenas.{slug}.theme")), 14.));
                let effect = effect(arena, lang);
                p.spawn(super::super::label(effect, 16.));
                let chosen = crate::config::load_arena() == arena;
                p.spawn(button(
                    lang.key(if chosen {
                        "arenas.chosen"
                    } else {
                        "arenas.choose"
                    }),
                    Action::Arena(arena),
                    280.,
                ));
                if chosen {
                    p.spawn(kit::button(
                        lang.key("arenas.train"),
                        UiAction::PracticeSetup,
                        280.,
                    ));
                }
            });
        }
    });
}

fn effect(arena: tbg_core::Arena, lang: Lang) -> String {
    crate::ui::arena_effect::describe(arena, lang)
}
