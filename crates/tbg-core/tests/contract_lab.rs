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
    // 2026-10-29 (uncapped Energy, Interest, the Augment pause with its
    // paused steps traced, purchase-only Shop refills, BR-S04, and the 20 s
    // selection window, BR-AG13: only the traces moved, final hashes did not).
    let goldens = [
        (1, 0x501b_cc9f_bdf0_e458, 0x7242_2aae_5147_d2e7, 3_664),
        (7, 0x058c_86bb_1dd4_2d38, 0x7d5a_9954_45c7_a322, 2_146),
        (42, 0x6865_c895_84b7_7f7b, 0xf3a6_1b6c_8581_3910, 2_342),
        (1000, 0x19e8_b049_e90e_d7e4, 0xa634_79ae_6844_aefe, 4_195),
        (65535, 0xe465_ccbb_34a4_e900, 0xc6f2_5831_ee16_2fcd, 2_079),
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
