//! Shared native arena descriptions, including arenas with a symmetric tradeoff.

use crate::i18n::Lang;
use tbg_core::{
    content::{ARENA_RULES, ENERGY_SCALE},
    Arena,
};

pub(super) fn describe(arena: Arena, lang: Lang) -> String {
    let rule = ARENA_RULES[arena.index()];
    let mut lines = Vec::new();
    for (key, bp) in [
        ("arenas.effect.move", rule.move_bp),
        ("arenas.effect.attack", rule.atk_bp),
        ("arenas.effect.regen", rule.regen_bp),
    ] {
        if bp != 0 {
            lines.push(
                lang.key(key)
                    .replace("{value}", &format!("{:+}%", bp / 100)),
            );
        }
    }
    if rule.first_evolve_discount != 0 {
        lines.push(lang.key("arenas.effect.evolve").replace(
            "{value}",
            &(rule.first_evolve_discount / ENERGY_SCALE).to_string(),
        ));
    }
    if lines.is_empty() {
        lang.key("arenas.effect.none")
    } else {
        lines.join(" · ")
    }
}
