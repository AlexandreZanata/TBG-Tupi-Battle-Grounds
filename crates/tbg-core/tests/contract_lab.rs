//! Balance-lab contracts (D-33, docs/phases/P16C-balance-lab.md step 10).
//! Append-only: change a test here only in the same commit as the canon
//! change it guards. C1: the simulation reads no clock, float, OS
//! randomness, or hash-ordered container, so a match played back to back
//! equals the same match ticked every 50 ms. C2: per-tick hash traces.

use std::path::Path;

use tbg_core::bot::Bot;
use tbg_core::headless::{self, Moment};
use tbg_core::MatchState;

/// Modules outside the simulation, each with the reason it may differ.
const ALLOWED: [(&str, &str); 3] = [
    (
        "account.rs",
        "account names and keys, never part of a match",
    ),
    ("rating.rs", "Elo after a match, outside the simulation"),
    ("protocol.rs", "wire encoding, outside the simulation"),
];

const FORBIDDEN: [&str; 13] = [
    "Instant",
    "SystemTime",
    "std::time",
    "thread",
    "sleep",
    "f32",
    "f64",
    "HashMap",
    "HashSet",
    "thread_local",
    "static mut",
    "std::env",
    "getrandom",
];

#[test]
fn c1_simulation_sources_have_no_hidden_inputs() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut checked = 0;
    let mut stack = vec![src];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read src") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.ends_with(".rs") || ALLOWED.iter().any(|(a, _)| *a == name) {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("read source");
            for word in FORBIDDEN {
                assert!(
                    !text.contains(word),
                    "{}: `{word}` would make offline matches differ from the server's (C1)",
                    path.display()
                );
            }
            checked += 1;
        }
    }
    assert!(checked >= 15, "only {checked} simulation files found");
}

/// FNV-1a over the state hash of every tick, so a divergence that later
/// converges still changes the trace.
fn trace(seed: u64) -> (u64, u64, u32) {
    let mut s = MatchState::new(seed);
    let mut bots = [Bot::new(seed * 2 + 1), Bot::new(seed * 2 + 2)];
    let mut acc = 0xcbf2_9ce4_8422_2325_u64;
    let ended = headless::play(
        &mut s,
        |seat, view| bots[seat.index()].decide(view),
        |m| {
            if let Moment::Stepped { state } = m {
                acc = (acc ^ state.hash()).wrapping_mul(0x0100_0000_01b3);
            }
        },
    );
    assert!(ended, "seed {seed} did not end");
    (acc, s.hash(), s.tick)
}

#[test]
fn c2_per_tick_traces_are_frozen() {
    // (seed, per-tick trace, final hash, end combat tick): owner revision
    // 2026-10-29 (uncapped Energy, Interest as a regen boost, the Augment
    // pause with its paused steps traced, the 20 s selection window,
    // P40's fifteen-rank Arcana state, a full Shop redraw per purchase,
    // BR-S04, and the D-46 Stronghold fields: same Classic matches and end
    // ticks, only the serialized state grew).
    // P43: the zeroed `UnitFx` block moved the hashes
    // with tick, winner and reason identical (verified). P47:
    // `march_ignore_slow` grew `UnitFx`; hashes moved once more, behavior
    // identical (verified).
    // P48-A: `surge_until` grew `UnitFx`; hashes moved once more,
    // behavior identical (verified).
    // P48-B: `betray_bp`/`betray_stumble` grew `UnitFx` and `cargo_rng`
    // joined `PlayerState`; hashes moved once more, behavior identical
    // (verified).
    // Merge of the roster waves with D-46: both grew the serialized state;
    // hashes refrozen once, tick, winner and reason identical (verified).
    // D-50: the Shop draws all 25 kinds; traces and end ticks moved.
    // D-55 (ARENAS-G): the arena fields joined the serialized state; the
    // Forest is neutral: traces and hashes moved, end ticks identical.
    // D-68: owner champion changes in GAME-DESIGN, collected with update_probe.
    // D-69: appended private Shop state; all D-68 outcomes and ticks identical.
    // D-68/D-69 integration: protocol-20 slots + Shop state; owner range correction.
    // 46-kind CANON activation (GAME-DESIGN consolidation amendment);
    // collected with examples/trace_canon.rs, same server cadence and Bot.
    let goldens = [
        (1, 0x77fd9e4abe880db1u64, 0x78908734d3dc7ec1u64, 3844),
        (7, 0xca07e68322bdf61fu64, 0x7ab50dfdd5735803u64, 15586),
        (42, 0x5af8901a3d72b530u64, 0xc4377231f72a0c57u64, 5049),
        (1000, 0x8423d39086a7e075u64, 0x74b875d065a507a1u64, 7559),
        (65535, 0x2c3c3a735f7b3712u64, 0x528f18fd57983ffau64, 7737),
    ];
    // D-101: revision 2 remains frozen above. Revision 6 includes the owner's
    // documented P70/P71 candidates; unknown source identities fail closed.
    let goldens = match (
        tbg_core::ruleset_version::RULESET_VERSION,
        tbg_core::ruleset_version::ruleset_source_hash(),
    ) {
        (2, 0x3d56_1560_49e6_26c5) => goldens,
        (6, 0x0973_85fe_e9a1_0b0f) => [
            (1, 0x73626121cf33e5c9, 0xe318aa87b746f36c, 4044),
            (7, 0xa4cbc8bd6d9434b8, 0xb796fd4f3eb12d9f, 9982),
            (42, 0x4e31f157f00823a3, 0x52594b93cce38cf5, 10382),
            (1000, 0x691f7d391aeaf998, 0x5916e7f9018b83fd, 8869),
            (65535, 0x3a8c4b25c812f504, 0xac5524f22bc861e9, 8613),
        ],
        // OG-01: explicit canonical trace for the requested Ogre revision.
        (8, 0x3d07d8f3e73671ef) => [
            (1, 0xb28190b76fda00fb, 0xdc6ddee0ab7abe7e, 4044),
            (7, 0x1c8f1dee7e671e96, 0x981a89b67d681f3d, 9982),
            (42, 0x7cf4b2ac39262edd, 0x750b2a389290829d, 10382),
            (1000, 0x2f5fcbb3660cdd4c, 0xc80e2d0725d28b4d, 8869),
            (65535, 0x322f254d7eb09476, 0xa8b07cb5b9434daf, 19759),
        ],
        // OG-01 candidate 9 changes staged kinds only; canonical traces remain frozen.
        (9, 0x89698dcd4bcb5ddd) => [
            (1, 0xb28190b76fda00fb, 0xdc6ddee0ab7abe7e, 4044),
            (7, 0x1c8f1dee7e671e96, 0x981a89b67d681f3d, 9982),
            (42, 0x7cf4b2ac39262edd, 0x750b2a389290829d, 10382),
            (1000, 0x2f5fcbb3660cdd4c, 0xc80e2d0725d28b4d, 8869),
            (65535, 0x322f254d7eb09476, 0xa8b07cb5b9434daf, 19759),
        ],
        // AB-01/OG-02: appended state layout and unchanged established trace behavior.
        (10, 0x523044fc973bac39) => [
            (1, 0xc3480c1818db1fbd, 0x4b854f968429a97c, 4044),
            (7, 0x06a09918e8635c48, 0x70c562418d530b55, 9982),
            (42, 0x704873a9ca6f4feb, 0xcea96d41f552661d, 10382),
            (1000, 0x1aac595c5725aaba, 0xfe7914eff893639d, 8869),
            (65535, 0x5212bdb98903fc74, 0x8279f6a0fad3e661, 19759),
        ],
        // TW-01/PK-01/GD-01: exact tower candidate, with physical lanes staged privately.
        (11, 0xa0a1d508c59e5841) => [
            (1, 0x7d33b1664f3b9824, 0xae47594e85732bbc, 16625),
            (7, 0x84d4030142b4f1c5, 0x34994c75252c5c39, 14265),
            (42, 0x92d4133c4f4351b1, 0xec212d8ee38450d7, 23274),
            (1000, 0x8598d51bed033353, 0x8a17aae489e3424a, 58307),
            (65535, 0x1045dd3c5fc0a20f, 0x2f93bf36167ed04e, 24291),
        ],
        // RESTORE-01: canonical behavior unchanged; only private constructors restore legacy movement.
        (12, 0x8f5a8ebb3e4f88ec) => [
            (1, 0x7d33b1664f3b9824, 0xae47594e85732bbc, 16625),
            (7, 0x84d4030142b4f1c5, 0x34994c75252c5c39, 14265),
            (42, 0x92d4133c4f4351b1, 0xec212d8ee38450d7, 23274),
            (1000, 0x8598d51bed033353, 0x8a17aae489e3424a, 58307),
            (65535, 0x1045dd3c5fc0a20f, 0x2f93bf36167ed04e, 24291),
        ],
        revision => panic!("register an explicit canonical trace for {revision:?}"),
    };
    for (seed, t, h, tick) in goldens {
        assert_eq!(trace(seed), (t, h, tick), "seed {seed}: simulation changed");
    }
}

/// Every `.rs` file under `dir`, recursively.
fn sources(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d).expect("read dir") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    out
}

#[test]
fn c10_online_crates_only_play_the_canonical_ruleset() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for name in ["tbg-server", "tbg-client"] {
        let dir = crates.join(name);
        let manifest = std::fs::read_to_string(dir.join("Cargo.toml")).expect("manifest");
        let core_line = manifest
            .lines()
            .find(|l| l.trim_start().starts_with("tbg-core"))
            .unwrap_or_else(|| panic!("{name}: no tbg-core dependency line"));
        assert!(
            !core_line.contains("lab"),
            "{name}: enables tbg-core's `lab` feature (C10)"
        );
        for path in sources(&dir.join("src")) {
            let text = std::fs::read_to_string(&path).expect("read source");
            for word in ["with_rules", "lab_spawn", "Ruleset {", "Box::leak"] {
                assert!(
                    !text.contains(word),
                    "{}: `{word}` could play a non-canonical ruleset online (C10)",
                    path.display()
                );
            }
        }
    }
}
