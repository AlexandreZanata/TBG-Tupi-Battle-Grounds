//! The immutable per-revision catalog stored once in `ranked_rulesets`
//! (P61 §3): stable IDs for kinds, Factions, Archetypes, Strongholds,
//! Augments, Arcana nodes and Arenas, so old rows decode with their own
//! mapping rather than today's. Built once at startup.

use serde_json::json;

use tbg_core::content::{def, AUGMENTS, TICK_HZ};
use tbg_core::types::{ALL_KINDS, FULL_POOL};
use tbg_core::{Arena, StrongholdKind};

pub(crate) fn catalog_json() -> String {
    let kinds: Vec<_> = ALL_KINDS
        .iter()
        .enumerate()
        .map(|(i, &k)| {
            let d = def(k);
            json!([
                i,
                d.name,
                FULL_POOL.contains(&k),
                k.is_summon(),
                d.faction.index(),
                format!("{:?}", d.archetype),
            ])
        })
        .collect();
    let augments: Vec<_> = AUGMENTS.iter().map(|a| json!([a.id, a.name])).collect();
    json!({
        "tick_hz": TICK_HZ,
        "kinds": kinds,
        "forms": ["base", "A1", "A2", "B1", "B2"],
        "strongholds": StrongholdKind::SELECTABLE.map(|s| [s.index() as u64]),
        "augments": augments,
        "arcana_nodes": tbg_core::arcana::NODES,
        "arenas": Arena::ALL.map(|a| a.slug()),
    })
    .to_string()
}
