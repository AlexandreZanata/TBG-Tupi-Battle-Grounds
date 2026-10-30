//! Contract: `assets/arenas/arenas.json` (the lobby Arenas page and the
//! in-match banner) names the five arenas in id order and states exactly
//! the modifiers of `tbg-core` (GAME-DESIGN §12, D-55).

use super::json_lite::parse;
use tbg_core::content::{ARENA_RULES, ENERGY_SCALE};
use tbg_core::Arena;

#[test]
fn arena_page_numbers_equal_the_core_rules() {
    let text = include_str!("../../assets/arenas/arenas.json");
    let data = parse(text).unwrap_or_else(|e| panic!("arenas.json: {e}"));
    assert_eq!(data.get("storageKey").str(), Some("tbg.arena"));
    assert_eq!(
        data.get("defaultArena").str(),
        Some(Arena::default().slug())
    );
    assert_eq!(data.get("energyScale").int(), Some(i64::from(ENERGY_SCALE)));
    let list = data.get("arenas").list();
    assert_eq!(list.len(), Arena::ALL.len());
    for (json, arena) in list.iter().zip(Arena::ALL) {
        let rule = ARENA_RULES[arena.index()];
        let slug = arena.slug();
        assert_eq!(json.get("id").int(), Some(arena.index() as i64), "{slug}");
        assert_eq!(json.get("slug").str(), Some(slug));
        assert_eq!(
            json.get("image").str(),
            Some(format!("{slug}.webp").as_str())
        );
        for (key, value) in [
            ("moveBp", i64::from(rule.move_bp)),
            ("attackBp", i64::from(rule.atk_bp)),
            ("regenBp", i64::from(rule.regen_bp)),
            ("firstEvolveDiscount", i64::from(rule.first_evolve_discount)),
        ] {
            assert_eq!(json.get(key).int(), Some(value), "{slug}.{key}");
        }
    }
}
