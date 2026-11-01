# P71 planning report — 2026-11-01

**Delivered:** an executable visual phase and reconciled design references.
**Status:** planned, implementation not started. No Rust, assets, binary,
installer, saved preferences, combat values or running game were changed.

## Input and baseline

Owner request: HP bar clearance for Trident, Whisperer and Bonehorn; remove body team bands; brighter
outlined yellow Evolution stars; white/red/black Pyre Warden and orange/red
flame; polish all Stronghold performances, diagnose buggy shots and add
cumulative breakage, fragments and fire for every 5% HP lost.

The inspected owner screenshot is 1920 × 1080 at
`/home/iiii/Pictures/Screenshots/Screenshot_2026-11-01_07-28-59.png`, SHA-256
`29359afaf162a5a9eef744fdea30fe2e43a1e53e8044dc930c8f1ccd0512bcc6`.
Its source file was read only. It confirms the bar overlap but cannot prove
a firing animation fault or the exact simulation state.

Owner follow-up adds selected Base Whisperer in
`/home/iiii/Pictures/Screenshots/Screenshot_2026-11-01_07-46-37.png`,
1920 × 1080, SHA-256
`f606ea8f2c650df016791d61361cf76b5d8c7a19d787a4b3a8d6e4f98d07c19e`.
The HP bar crosses the floating upper body beneath its name. S1 explicitly
requires both-seat, five-form Whisperer clearance through hover and primary
motion, with stable placement and no name/star overlap. This remains a
planning addition; no runtime fix is claimed.

Planning branch `codex/p71-combat-visual-plan` starts at P70 `b2680c3`, whose
runtime source is `c481369`. P70 is already installed for local playtesting;
this plan does not change that installation. Owner edits in the main worktree
and historical P70 evidence are preserved.

Owner follow-up also reports Rattle & Tick's weapon penetrating the ground:
`/home/iiii/Pictures/Screenshots/Screenshot_2026-11-01_07-48-11.png`,
1920 × 1080, SHA-256
`632c529801fb534e15ee66eefa26dc0c9f44594cacbfa4ebbdcb4ac1148b9e42`.
The selected unit is Base. New step S1b requires reproducing the motion,
correcting weapon/grip/pivot geometry, checking all five forms and clips on
both sides, and preserving combat timing and values. A generated-vertex
clearance regression and ordinary-speed before/after are queued. The exact
animation cause is not diagnosed from a single screenshot; no runtime fix
is claimed in this planning update.

Owner follow-up reopens Trident's close-range thrust: the spear enters the
opponent before the extension reads clearly. New S1c requires earlier visible
ready/load preparation, a raised/retracted guard and a more downward,
target-directed thrust at close melee distance. Both hands stay attached,
with distance-aware visual extension and no penetration of Trident's own
body or the floor. The existing 0.30 s Strike, basic/special event timing,
range, movement and damage remain unchanged. Actual approach/aim/blend
recordings on both sides and all five forms are mandatory; fixed-target pose
samples alone cannot establish the correction. This is planned, not a
claimed runtime fix or a new mechanical cast window.

Owner follow-up adds Bonehorn's HP-bar overlap and diagonal-looking walk:
`/home/iiii/Pictures/Screenshots/Screenshot_2026-11-01_07-53-42.png`,
1920 × 1080, SHA-256
`e21aa4676033ae7341a99b37093ce7dfe8e782ef4bdda8b640bfa2a8018d1104`.
S1 now includes its complete back-armor envelope and walking bob. New S1d
requires visual body/foot alignment with actual travel, both sides and all
five forms, including stop/attack transitions. The shared oblique figure yaw
is a source lead to inspect; the screenshot does not prove diagonal gameplay
movement. A kit-specific presentation repair must preserve authoritative
movement, speed and spacing. These changes remain planned, not implemented.

## Findings that determine the plan

- Spawn and Evolution update both use a generic scaled bar height, not
  figure bounds. A one-off Trident offset would leave the shared cause.
- Tier markers are actual branch-gold/silver cubes. New stars need a shared
  layout with names/status badges and readable unlit fill/outline.
- Individual kit markers coexist with shared tabard helpers. Enemy variants
  currently use ALLY-color equality substitution; audit intrinsic colors too.
- Status effects overwrite foot ring color today. Keeping allegiance visible
  is necessary before removing body team identification.
- Pyre's green jet and danger strip are isolated in `vfx/flame.rs`; recolor
  them without changing the event/socket/segment contract.
- Tower pieces already support coarse damage loss. Damage updates currently
  rebuild children, and Classic flags have a separate threshold path.
  Twenty-step damage must preserve live emitters, aim and recoil, reconcile
  actual healing/Fortification and avoid VFX accumulation on repeated views.
- Current shot code has aim/settle, muzzle calculation and separate moving
  parts. The phase requires reproducing each bug before identifying its
  cause; none is labeled confirmed merely from this static screenshot.
- ART-DIRECTION still described obsolete cosmetic tower styles and compulsory
  body team heraldry. Planning now reconciles it with active D-46 kinds and
  this explicit owner override; old implementation evidence remains historical.

## Decisions and Proposed tuning

Base has no stars, tier 1 one, tier 2 two, preserving existing tier meaning.
Both branches/seats use yellow with a thin dark edge. Intrinsic colors do not
change by team; foot rings and architectural banners identify allegiance.

The 5% step is owner-requested. The phase defines safe integer boundaries
and 21 states including intact/destroyed. Specific material colors, star
pixel size, smoke/fire onset bands and damaged-region compositions are
Proposed art tuning for rendered review. Damage has no mechanical penalty.

Use the existing cached procedural art, pieces and VFX; no new dependency,
physics engine, persistent history protocol or general animation framework.
A phase-local evidence index and bug ledger cover genuine renderer originals
and exact deferred cases. The final installed candidate preserves rollback
and saved data. Owner subjective acceptance comes after the concrete result.

## Files and validation

- [Execution phase](../phases/P71-combat-readability-and-stronghold-polish.md).
- [Roadmap](../ROADMAP.md), [art direction](../ART-DIRECTION.md),
  [cosmetic contract](../GAME-DESIGN.md), [decision follow-up](../DECISIONS.md).

Level: documentation-only work front. Actual checks: `git diff --check`,
changed Markdown local-link/source-path checks and review of the step formula
at thresholds against varied max HP. Results: passed, with no build/test run.

Deferred to execution/consolidation: all renderer captures, named `p71_`,
Stronghold and Pyre regressions, native playtest installation and desktop
browser/Windows runtime checks as scoped by the phase. No deferred check is
reported as passed. No full suite, D-33 gate, release, packaging, benchmark,
push, merge or deployment occurred. P70's unrelated open items remain open.
