# GD-04 — Gods expansion: Colossus refinement, Volthar and Astreon

> Owner request, 2026-11-02. Branch `phase/gods-faction-expansion`, based on
> PR #48 head `747e2ad` (`codex/ogre-faction`) plus the two local Colossus
> commits that were not pushed yet: CROWD-EXIT-01 (`73c7a07`, tower-exit
> clearance) and GD-03 (`e18657e`, body turn and obelisk tip at the strike).
> Both are preserved as cherry-picks; CROWD-EXIT-01's width constant collapses
> onto main's equivalent RESTORE-03 `FORMATION_HALF_WIDTH`.

Rules canon: [GAME-DESIGN GD-04](../GAME-DESIGN.md#gd-04--proposed-gods-expansion-volthar-and-astreon-2026-11-02),
[D-105](../DECISIONS.md#d-105--gods-expansion-reuses-counted-hit-hooks-and-one-public-link-event--2026-11-02-proposed-gd-04).
Art: [ART-DIRECTION GD-04](../ART-DIRECTION.md). Visual contract:
[DESKTOP-UI-CONTRACT, 2026-11-02 GD-04 record](../ui/DESKTOP-UI-CONTRACT.md).

## 1. Scope and decisions

| Front | Delivered | Owner decision still needed |
|-------|-----------|-----------------------------|
| Colossus | Visual/motion refinement only; stats, abilities, cost, scale, obelisk core, GD-02/GD-03 unchanged | Look of the five forms |
| Volthar | Rules (`volthar.rs`), five-form kit, lightning VFX, i18n, tests | Values (Proposed), A1 fourth share, look/fun |
| Astreon | Rules (`astreon.rs`), five-form kit, lance pierce VFX, i18n, tests | Values (Proposed), look/fun |

- **Poster names vs canon.** The Colossus poster's defensive A / offensive B
  is the reverse of the catalog (A = damage/range/splash, B = +HP). Canonical
  names stay; visual direction follows the *role*: canon A wears the offensive
  "Pilar da Ira / Obelisco Destruidor" language, canon B the defensive
  "Bastião de Mármore / Fortaleza Divina" language **in dark charcoal**, as the
  owner required.
- **Scale.** All Gods use the Colossus monument body plan (Large × 1.2 ×
  size): Colossus 1.85, Volthar 1.66, Astreon 1.64, so the Colossus remains
  the largest. No gameplay scale was reduced.
- **Values.** Owner numbers converted exactly to simulation units; 0.75 tile/s
  is not an integer per tick, so Astreon moves at 37 milli-tiles/tick (0.74).
  The request gave four Volthar A1 jumps but three shares: the fourth is an
  agent proposal of 25 %. A2's "+10 points" applies to all four shares.
- **Authority.** Clients draw `StormArc` links, `AbilityResolved` impacts and
  the counted bolt; the counters ride the existing public progress bits.
  No client targeting, damage or timing decision was added.
- **Preview scope.** Both join the local Practice/Training preview pool
  (`COMPLETION_POOL`, 84 kinds), like Colossus. FULL_POOL (online Shop) stays
  46; online activation remains an owner/D-33 decision.

## 2. Files

- Core: `crates/tbg-core/src/{volthar,astreon}.rs` (+ `tests.rs`), `content.rs`
  (abilities, rows), `types.rs` (kinds, pools), `hits.rs`/`combat.rs`/`view.rs`
  hooks, `state.rs` + `view/visibility.rs` (`StormArc`), `content/occupancy.rs`,
  identity (`protocol.rs` 37, `ruleset_version.rs` 14, `ruleset-sources.txt`,
  fixture), contracts (`contract_roster`, `contract_protocol`, `ogre_roster`,
  `abyssal_roster`).
- Client: `art/kits/{volthar,astreon}/`, `art/kits/colossus/{model,motion}.rs`,
  `art/palette.rs`, `art/anim.rs` (grip restore after blends), `art/gods_tests.rs`,
  `vfx/{shots,completion}.rs`, `sync/{actions,special_cues}.rs`,
  `ui/ability_gods.rs`, `ui/ability_details.rs`, `i18n/content.rs`, catalogs,
  `gallery/*` (monument framing for all Gods).
- Lab: `crates/tbg-balance/examples/gods_screen.rs`.

## 3. Validation (minimal policy, focused checks under the shared lock)

Every compiling command ran under `/tmp/tbg-validation-1000.lock` with
`CARGO_BUILD_JOBS=2` and this worktree's own `CARGO_TARGET_DIR`
(`/data/tbg-gods-faction-target`). Another front had started writing into
the first target directory this branch used, so the cache was copied and all
workspace sources touched before reuse (stale cross-worktree crates avoided).

| Check | Result |
|-------|--------|
| `cargo test -p tbg-core --lib --features lab -- gd04` | 12 passed (Volthar 6, Astreon 6) |
| `cargo test -p tbg-core --features lab` (lib + all 44 integration targets) | all pass after extending the explicit revision selectors to R14 (see below) |
| `cargo test -p tbg-client --bin tbg-client` | 224 tests: all pass (2 count fixtures and one purchase-energy fixture updated for the 84-kind preview pool) |
| `cargo check -p tbg-server --bins --tests` | passes |
| `cargo build -p tbg-balance --release --example gods_screen` + run | CSV in `docs/media/gd04/gods-screen.csv` |
| `git diff --check`, `cargo fmt --all -- --check` | clean |

**Canonical identity.** The first R14 run moved canonical bot traces: the
Smuggler's legendary cargo draws from every registered conventional kind, so
the new Gods would have changed another champion's crates. GD-04 excludes them
from cargo; R14 (`f354f651c056d615`) now reproduces every R13 canonical trace,
outcome and tick (`contract_determinism`, `contract_lab`, `CC-ID03`).

Not run (deferred, owner/consolidation): workspace clippy, `tbg-server` test
suite, browser/WASM build, D-33 balance gate, Windows/Wine packages, browser
and Windows visual captures, live online match captures.

## 4. First-order value analysis (Proposed values, not a gate)

Per cost, before Synergy/Arcanas/Augments, simulation units (20 ticks/s):

| Champion | Cost | HP | Single-target DPS | Extra |
|----------|------|----|-------------------|-------|
| Colossus | 18 | 2400 | 170 / 5.5 s = 30.9 | full frontal splash 1.8 tiles |
| Volthar | 21 | 2200 | 130 / 1.5 s = 86.7 at 5.5 tiles | every 6 s: 1.55 × 130 = 201.5 spread (+33.6/s) when ≥ 3 enemies are chained |
| Astreon | 23 | 1900 | 190 / 2 s = 95 at 2.2 tiles | every 8 s: +5 % max HP ≤ 240 (+15/s vs a 2400-HP Colossus; +30/s cap) |

- Volthar's value scales with enemy count and spacing (≤ 2.2 tiles per link);
  against one isolated target it is a plain 86.7-DPS ranged champion.
- Astreon's pierce caps at 240 (base) — against a 2400-HP Colossus it adds
  ~16 % damage; A1/A2 raise the frequency and cap (300). Single target only.
- Both have higher raw DPS per Energy than the Colossus, which trades DPS for
  HP, area and reach; the bounded duel screen below measures real outcomes.

## 5. Known limitations

- PK-01 lab footprints for the new kinds are authored estimates; the native
  footprint exporter has not measured them.
- Windows and desktop-web presentation are **not observed**; parity is pending.
- No Gods Synergy row: Gods keeps zero Synergy bonuses (owner decision).
- Balance values are Proposed; the bounded screen is not a D-33 gate.

## 6. Execution evidence

Linux captures, native debug client of this branch, private Xvfb + Mesa
lavapipe and an isolated `HOME` (no owner window, preference or vault touched).
Software rendering: no FPS or GPU performance conclusion is drawn from them.

| File (`docs/media/gd04/`) | Shows |
|---------------------------|-------|
| `colossus-forms.webp`, `colossus-lineup.webp` | Five refined forms; idle/walk/strike rows (GD-03 strike still toward the target) |
| `volthar-forms.webp`, `volthar-lineup.webp`, `volthar-death.webp`, `volthar-victory.webp` | Five forms, hammer in hand in every row, face clear of the hammer |
| `astreon-forms.webp`, `astreon-lineup.webp`, `astreon-death.webp`, `astreon-victory.webp` | Upright guard beside the body, diagonal only in the thrust, kneeling death on the lance |
| `battle-two-colossus.webp` | Two Colossi (A2, B2) in an authoritative Training battle |
| `battle-volthar-16.webp`, `battle-volthar-19.webp` | Two Volthars versus seven Archers; StormArc links drawn from server events; allied win |
| `battle-astreon.webp` | Two Astreons versus Colossus and Bone Brute |
| `portrait-*.webp` | 400×400 Shop portraits (encoded into `assets/battle/portraits/*.webp`) |

Issues found in captures and fixed before commit: Astreon's diagonal guard
read as the lance crossing the chest (now upright at the right side, diagonal
only during the stroke); Volthar B-form hammer spikes crossed the face (head
turned edge-on in guard, smaller B head); Colossus B buttresses read as gold
planks (slim stone with gold caps); A-crown rays shortened. Automated checks
also caught the left hand leaving the lance in the thrust and in death.

Observed limitation: in the close Training camera two giants of one kind can
overlap visually while engaged (presentation crowd solver only). Arena-camera
readability with several Gods, and Windows/web parity, still need review.

### Bounded duel screen (canonical movement, towers off, regen off)

`score` is the mean of both seat orders; Evolution cost is inside the budget,
so an evolved God at 48 Energy faces a full budget of cheap units.

- **Volthar base** wins against melee swarms at 24 and 48 Energy (Sawbot,
  Ravager, Bloomkeeper, Mantis) and loses to ranged swarms (Archer, Farwatch)
  and Shieldbearers until they are evolved. It **beats Colossus and Astreon in
  every pairing tried** — the strongest signal for owner tuning.
- **Astreon base** beats Colossus (its intended counter) at 24 and 48, loses to
  the cheap tank swarms (Bone Brute, Rampart, Bonehorn, Shieldbearer: many
  bodies, one target per thrust) and to Volthar.
- Evolved Gods at 48 Energy lose most panels: one evolved God against 48 Energy
  of base units is a budget shape, not a verdict on the forms.

This is a hypothesis screen, not D-33 acceptance; no value was changed from
the owner's proposal.
