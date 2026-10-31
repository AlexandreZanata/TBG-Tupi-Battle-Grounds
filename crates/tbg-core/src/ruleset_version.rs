//! Online ruleset identity for ranked records and statistics (P61 §3,
//! shared with P60 step 3): a monotonic revision plus a source hash over
//! every gameplay source file. Computed once at server startup, never per
//! tick or per match. Not the protocol version and not a selectable ruleset.

/// Gameplay revision. Bump it, and append a line to
/// `tests/fixtures/ruleset-versions.txt`, whenever a watched source changes.
pub const RULESET_VERSION: u32 = 1;
/// Version of the per-match summary JSON (P61 §4).
pub const SUMMARY_VERSION: u16 = 1;

/// Watched gameplay sources, sorted by path (`ruleset-sources.txt`).
pub const WATCHED: [(&str, &str); 19] = [
    ("afflictions.rs", include_str!("afflictions.rs")),
    ("arcana.rs", include_str!("arcana.rs")),
    ("augment.rs", include_str!("augment.rs")),
    ("combat.rs", include_str!("combat.rs")),
    ("combat/formation.rs", include_str!("combat/formation.rs")),
    ("content.rs", include_str!("content.rs")),
    ("hits.rs", include_str!("hits.rs")),
    ("intent.rs", include_str!("intent.rs")),
    ("rng.rs", include_str!("rng.rs")),
    ("ruleset.rs", include_str!("ruleset.rs")),
    ("shop.rs", include_str!("shop.rs")),
    ("sim.rs", include_str!("sim.rs")),
    ("state.rs", include_str!("state.rs")),
    ("stronghold.rs", include_str!("stronghold.rs")),
    ("summon.rs", include_str!("summon.rs")),
    ("support.rs", include_str!("support.rs")),
    ("synergy.rs", include_str!("synergy.rs")),
    ("targeting.rs", include_str!("targeting.rs")),
    ("types.rs", include_str!("types.rs")),
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
