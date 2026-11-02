//! Online ruleset identity for ranked records and statistics (P61 §3,
//! shared with P60 step 3): a monotonic revision plus a source hash over
//! every gameplay source file. Computed once at server startup, never per
//! tick or per match. Not the protocol version and not a selectable ruleset.

/// Gameplay revision. Bump it, and append a line to
/// `tests/fixtures/ruleset-versions.txt`, whenever a watched source changes.
pub const RULESET_VERSION: u32 = 13;
/// Version of the per-match summary JSON (P61 §4).
pub const SUMMARY_VERSION: u16 = 2;

/// Watched gameplay sources, sorted by path (`ruleset-sources.txt`).
#[rustfmt::skip] // The client build script reads one literal entry per line.
pub const WATCHED: [(&str, &str); 57] = [
    ("acidmaw.rs", include_str!("acidmaw.rs")),
    ("afflictions.rs", include_str!("afflictions.rs")),
    ("arcana.rs", include_str!("arcana.rs")),
    ("augment.rs", include_str!("augment.rs")),
    ("auxiliary.rs", include_str!("auxiliary.rs")),
    ("barrel.rs", include_str!("barrel.rs")),
    ("bonehorn.rs", include_str!("bonehorn.rs")),
    ("bulwark.rs", include_str!("bulwark.rs")),
    ("capybara.rs", include_str!("capybara.rs")),
    ("celestial_archer.rs", include_str!("celestial_archer.rs")),
    ("combat.rs", include_str!("combat.rs")),
    ("combat/formation.rs", include_str!("combat/formation.rs")),
    ("completion/mod.rs", include_str!("completion/mod.rs")),
    ("completion/specials.rs", include_str!("completion/specials.rs")),
    ("completion/state.rs", include_str!("completion/state.rs")),
    ("completion/strikers.rs", include_str!("completion/strikers.rs")),
    ("completion/supports.rs", include_str!("completion/supports.rs")),
    ("content.rs", include_str!("content.rs")),
    ("content/completion.rs", include_str!("content/completion.rs")),
    ("content/occupancy.rs", include_str!("content/occupancy.rs")),
    ("crowcaller.rs", include_str!("crowcaller.rs")),
    ("demolisher.rs", include_str!("demolisher.rs")),
    ("drillnest/effects.rs", include_str!("drillnest/effects.rs")),
    ("drillnest/mod.rs", include_str!("drillnest/mod.rs")),
    ("hits.rs", include_str!("hits.rs")),
    ("hornet.rs", include_str!("hornet.rs")),
    ("intent.rs", include_str!("intent.rs")),
    ("jawguard.rs", include_str!("jawguard.rs")),
    ("justiciar.rs", include_str!("justiciar.rs")),
    ("luminous_oracle.rs", include_str!("luminous_oracle.rs")),
    ("mantis.rs", include_str!("mantis.rs")),
    ("occupancy.rs", include_str!("occupancy.rs")),
    ("occupancy/deployment.rs", include_str!("occupancy/deployment.rs")),
    ("pyre_warden.rs", include_str!("pyre_warden.rs")),
    ("rattle_tick.rs", include_str!("rattle_tick.rs")),
    ("ravager.rs", include_str!("ravager.rs")),
    ("rng.rs", include_str!("rng.rs")),
    ("roster_slots.rs", include_str!("roster_slots.rs")),
    ("ruleset.rs", include_str!("ruleset.rs")),
    ("secondary_targets.rs", include_str!("secondary_targets.rs")),
    ("shop.rs", include_str!("shop.rs")),
    ("sim.rs", include_str!("sim.rs")),
    ("siphon.rs", include_str!("siphon.rs")),
    ("solar_sentinel.rs", include_str!("solar_sentinel.rs")),
    ("specials.rs", include_str!("specials.rs")),
    ("state.rs", include_str!("state.rs")),
    ("stronghold.rs", include_str!("stronghold.rs")),
    ("stronghold_power.rs", include_str!("stronghold_power.rs")),
    ("summon.rs", include_str!("summon.rs")),
    ("support.rs", include_str!("support.rs")),
    ("synergy.rs", include_str!("synergy.rs")),
    ("targeting.rs", include_str!("targeting.rs")),
    ("trident.rs", include_str!("trident.rs")),
    ("trigger.rs", include_str!("trigger.rs")),
    ("types.rs", include_str!("types.rs")),
    ("weaver.rs", include_str!("weaver.rs")),
    ("whisperer.rs", include_str!("whisperer.rs")),
];

/// FNV-1a 64 over `path, 0x00, source, 0xFF` for each watched file in
/// order. A pure formatting edit changes it too: deliberately conservative.
pub fn ruleset_source_hash() -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut feed = |bytes: &[u8]| {
        for b in bytes {
            h = (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3);
        }
    };
    for (path, source) in WATCHED {
        feed(path.as_bytes());
        feed(&[0x00]);
        feed(source.as_bytes());
        feed(&[0xFF]);
    }
    h
}
