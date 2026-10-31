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
    // W0 (D-62): `UnitFx` grew with the bleed, weakening, queued-Slow, mark,
    // self-reduction and ability heal-cut slots; traces and hashes moved, with
    // end ticks, winners and reasons identical on all five seeds (verified).
    // W1 (P58 §13.5, D-64): the Synergy array grew to seven Factions; traces and
    // hashes moved, with end ticks, winners and reasons identical on all five
    // seeds (verified against the pre-W1 commit).
    let goldens = [
        (1, 0xe0d4_1ba9_f7f3_42e2, 0x5179_ac0a_c414_6409, 3_336),
        (7, 0xf0db_7322_c047_445a, 0xd9a7_27fc_c92d_bfdb, 2_286),
        (42, 0x6846_b8b3_81ee_9782, 0xb202_0ad9_8ef8_03ed, 6_295),
        (1000, 0xaf23_2be9_944f_66dd, 0x4e2f_0851_9ee3_42dd, 6_683),
        (65535, 0xda3b_1789_38e6_ce41, 0x5870_7bf0_8eb0_52fe, 5_531),
    ];
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
