# Desktop UI contract: Linux is the approved reference

The owner requires identical Linux and Windows frontend appearance, not merely
similar themes. This contract covers every game screen and transient state,
including typography/wrapping, geometry, spacing, colors, images, iconography,
models, materials, lighting/shadows, camera, VFX, animation and loading states.
Linux quality must not be reduced to accommodate Windows. No OS, renderer or
driver difference is an automatic exception. Unresolved visible differences
block P68 W07/W08 and Windows visual acceptance.

## One presentation implementation

`crates/tbg-client/build.rs` emits `tbg_desktop` for the actual compilation
target's Linux or Windows OS. The audited Linux presentation paths now use this
single condition, including native battle, War Council, card locks, details,
loading, ranking, all menu pages, lobby combat demo and opt-in capture journeys.
The browser branch remains separately compiled. Art inventory is generated for
both desktop targets, sorted and normalized to forward-slash relative paths.
Fonts, design tokens and asset bytes are shared. The migration inventory is
`P68-DESKTOP-GATES.json`; the original hashed P68 planning inventory is retained.

Presentation code must not introduce Linux/Windows-specific layout or styling
branches. OS services remain separate: secure vault providers, Unix file modes,
window/process metadata, clipboard and filesystem adapters. An OS-specific
rendering fix may live in a narrowly documented platform adapter, but cannot
waive the comparison gate. Changing this contract requires owner approval.

## Source gate and packaged visual gate

Run `python3 scripts/check-desktop-ui.py` for the cheap source gate. It rejects
OS-specific presentation conditions and checks the target-side desktop/art
selector. This gate also runs in its own small GitHub workflow. Passing it
proves shared routing only, never visual equivalence or Windows support.

The comparator and its directed tests require Pillow (validated with 13.0.0).
It is a development tool dependency, not a game runtime dependency. The small
workflow installs that pinned version and runs negative comparison cases plus
the synthetic browser coordinator regressions; it does not compile the game.

The strict comparison command is:

```sh
python3 scripts/check-desktop-ui.py --compare /path/linux-captures /path/windows-captures --output /path/comparison.json
```

Each capture directory contains an `index.json` matching the JSON contract and
its packaged-build screenshots. The command fails on missing scenarios,
mismatched source/assets/Cargo.lock/settings, wrong image dimensions or any
decoded pixel difference. It has no threshold override, masks, baseline-update
shortcut or driver waiver. PNG compression/metadata may differ; decoded RGBA
pixels must match. Review overlays and motion sequences in addition to the
mechanical gate; static images cannot establish animation timing or fun.

Use packaged binaries from the **same final source SHA**, identical compiled
features, language, physical viewport, effective UI scale, graphics quality,
scene seed and simulation tick/animation frame. Record binary/package hashes,
OS/GPU/driver/backend, scene identifiers and effective DPI. OS window chrome is
outside the game framebuffer; game text and icons inside it are included.
Never compare a source-checkout debug executable to a packaged release.

The matrix covers both seats for battle/result, pt-BR and English, the specified
DPI/viewport scenarios, account/restore/failure/conflict, every menu page, all
five Arenas, training/preview, card/detail/Evolution/Fortification, War Council,
loading, result/Victory and next-match cleanup. Motion cases contain matched
frames, not a single convenient still. Do not substitute an incomplete matrix
and report the frontend as identical.

## Current evidence and rollout

This Linux-first delivery prepares the implementation and enforceable contract.
Actual same-source packaged Windows captures, Wine and a real Windows GPU
session are subsequent delivery gates. Until those pass, equivalence remains
**unverified**, regardless of Linux compilation or a successful source gate.
No Windows-ready/support claim or signing approval follows from this document.

## CC17 — Champion completion visual correspondence

The seventeen new champion kits, 85 forms and bilingual local catalog use
the same FigureLibrary, rig-v2 mesh cache, palette, animation sampling,
projectile and public-snapshot cue paths on Linux, Windows and desktop web.
The [coverage manifest](CHAMPION-COMPLETION-VISUAL.json) extends this contract's
matrix and motion coverage. Existing crowd offsets and competitive geometry
are unchanged; folded appendages and authored poses constrain visual volume.

Affected cases: all seventeen base/A1/A2/B1/B2 models, default skin, weapons,
loaded needles, healing projectiles, impacts, Evolution bursts, Idle, Walk,
Spawn, Hit, Death, basic attacks, independent specials, Victory, Troops/card
portraits/details, bilingual Evolution names, Training/Practice, both seats,
reduced motion, pause/reset, reconnect and next-match cleanup. Solar spark,
echo, regrowth, interrupt immunity, charged pounce, Nectar channel and fixed
Prismancer warning derive exclusively from authoritative public facts.
The additional insect feet reuse fixed spare nodes and shared world foot
anchors. No per-platform shape, animation duration, styling or quality fork.

Initial generated five-form illustrations are direction references only;
they are not engine output or evidence of platform parity. At this checkpoint
Linux runtime review is pending; Windows and desktop-web matched captures
and motion comparisons are also pending. Compilation or shared code cannot
close those gates. Update the manifest only with actual recorded observations.

CC17 uses per-character cosmetic lateral radii (0.29–0.54 at base, +8%
per Evolution tier) for the seventeen added kinds only. The exact values are
in the coverage manifest and catalog. Existing-kind radii and solver limits
are preserved. These offsets never affect server positions, targeting or
range. They must be reviewed against actual models and dense formations.

CC17 measurement: model-driven idle/walk half-width samples across all 85
forms set each new cosmetic base radius, with reserved clearance and tier
growth. Compact arm poses reduced Gravecutter's normalized half-width from
0.6077 to 0.4228, Lastlight Cantor from 0.6491 to 0.4141, Cragclaw from
0.6685 to 0.4694, and Dawnmender from 0.4474 to 0.3751. These are sampled
geometry measurements, not runtime crowd/overlap acceptance or FPS evidence.
All new Victory loops last three seconds, stay eventless, and close without
a sampled pose seam. Shared weapon-shaft and lantern-rim bindings are tested.
