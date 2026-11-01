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

## 2026-11-01 — Laboratory reduced-motion toggle

Linux reference: Troops opens with ordinary playback even if the saved Result
reduced-motion preference is enabled. The laboratory toggle is temporary to that
visit, labels its on/off state explicitly, and never reads or changes the Result
preference. Exiting Troops resets the toggle; changing its internal tabs does not.
Manually enabling reduced motion still samples the existing static pose; disabling
it resumes the existing animation. No champion clip, dance, projectile, combat
behavior or timing is modified by this correction.

Shared propagation: `training::{Catalog, runtime, preview, catalog::viewer}` and
both locale catalogs are shared by Linux, Windows and desktop web. No target-specific
presentation is introduced. Required platforms are listed explicitly in the JSON.
Source sharing does not establish rendered correspondence.

Matrix additions: `preview-motion-default`, `preview-motion-reduced`,
`preview-motion-resumed`, `preview-motion-reentry`, each with the contract's motion
frames. Cover Dance/Victory and Attack, default and evolved forms, pt-BR and EN,
with a previously saved global reduced-motion value of 1. Check button state,
motion before/after toggling, reentry, and unchanged global preference bytes.

Evidence at implementation: Linux native build and focused motion observations
pending. Windows and desktop web observations are queued for consolidation, pending
same-source packaged comparison. No cross-platform parity is claimed.
