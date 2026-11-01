# In-game visual correspondence contract: Linux is the reference

**Core owner rule, 2026-11-01:** [AGENTS.md, non-negotiable 8](../../AGENTS.md#non-negotiables)
requires every in-game visual change to look and behave exactly the same on
Linux, Windows and desktop web, always using Linux as the canonical reference.
This is a permanent requirement for every phase, feature, fix, refactor, asset
change and optimization; it is not limited to P68, P72 or menu UI.

The scope includes every screen, component, modal, HUD, card, typography/wrapping,
geometry/spacing, color, image, icon, loading/error/empty state and transition;
every Arena, camera, model, Evolution form, material, light/shadow and VFX;
and all animations and visible champion behavior: spawn, idle, movement, facing,
attack/ability presentation, hit feedback, death, preview, dance and Victory.
Timing, trajectories, poses, transitions and cleanup belong to this contract,
including repeated/dead celebration participants and both player seats.
The server and game canon still own actual targeting, combat, values and outcomes.

Linux quality must not be reduced to accommodate Windows or web. No platform,
renderer, GPU, driver or browser difference grants an automatic exception. No
pixel tolerance, mask, partial formation/screen, substitute animation or separate
platform theme may be accepted as exact correspondence. Only an explicit owner
instruction can change this contract; record that decision rather than silently
updating a baseline. Unresolved differences block the affected visual acceptance
on Windows or web. Historical nonzero similarity measurements are diagnostic,
not proof of exact correspondence.

## Every visual change updates this contract

Each visual change must add a change record below describing the intended Linux
reference, affected surfaces/champions/forms/states/motion, Windows and web
propagation, matrix/case changes, and passed/failed/deferred evidence per target.
Extend `desktop-ui-contract.json` with every newly affected scenario, including
motion frames for visible behavior. Existing cases must remain covered; a static
UI screenshot does not cover an attack, death or dance. Keep implementation,
shared assets and the contract update in the same change.

Implement presentation once for all three targets. Platform services may use
separate adapters internally, but cannot justify an altered in-game appearance
or visible behavior. A Linux-only implementation is not complete cross-platform
delivery. Follow [VALIDATION](../VALIDATION.md): minimum per work front, targeted
checks once at consolidation, no automatic full suite or extensive gallery.
Deferred observations stay pending and cannot support a parity claim.

## One presentation implementation

`crates/tbg-client/build.rs` emits `tbg_desktop` for the actual compilation
target's Linux/Windows OS or wasm32 browser target. The audited Linux
presentation paths use this single condition, including native battle, War
Council, card locks, details,
loading, ranking, all menu pages, lobby combat demo and opt-in capture journeys.
The browser is separately compiled but uses that same Bevy presentation, not a
second HTML implementation. Art inventory is generated for the shared targets,
sorted and normalized to forward-slash relative paths.
Fonts, design tokens and asset bytes are shared. The migration inventory is
`P68-DESKTOP-GATES.json`; the original hashed P68 planning inventory is retained.

Presentation code must not introduce Linux/Windows/web-specific layout, styling,
animation or visible-behavior branches. OS services remain separate: secure vault
providers, Unix file modes,
window/process metadata, clipboard and filesystem adapters. An OS-specific
rendering fix may live in a narrowly documented platform adapter, but cannot
waive the comparison gate. Changing this contract requires owner approval.

## Source gate and packaged visual gate

Run `python3 scripts/check-desktop-ui.py` for the cheap source gate. It rejects
OS-specific presentation conditions and checks the target-side desktop/art
selector. This gate runs in the required `desktop-ui` CI lane. Passing it
proves shared routing only, never visual equivalence or Windows/web support.

`desktop-ui-contract.json` declares Linux, Windows and web as required platforms.
The current `--compare` implementation compares Linux and Windows only; it does
not verify web. Browser evidence must cover the same required cases and exact
criteria separately. Extending this runner to consume web capture indexes is
pending; a successful Windows comparison cannot substitute for browser evidence.

The comparator and its directed tests require Pillow (validated with 13.0.0).
It is a development tool dependency, not a game runtime dependency. The small
workflow installs that pinned version and runs negative comparison cases plus
the synthetic browser coordinator regressions; it does not compile the game.

The strict comparison command is:

```sh
python3 scripts/check-desktop-ui.py --compare /path/linux-captures /path/windows-captures --output /path/comparison.json
```

Each capture directory contains an `index.json` matching the JSON contract and
its packaged-build screenshots. The browser counterpart uses the final deployed
WASM bundle and game viewport, not browser toolbar/window chrome. The command
fails on missing scenarios,
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

The required matrix must cover both seats for battle/result, pt-BR and English,
the specified DPI/viewport scenarios, account/restore/failure/conflict, every menu
page, all current and future Arenas, training/preview, card/details/Evolution/
Fortification, War Council, loading, result/Victory and next-match cleanup. Motion cases contain matched
frames, not a single convenient still. Do not substitute an incomplete matrix
and report the frontend as identical.

## Current evidence and rollout

This Linux-first delivery prepares the implementation and enforceable contract.
Actual same-source packaged Windows captures, Wine, a real Windows GPU session
and matching desktop-browser observations remain delivery gates. Until those
pass, equivalence remains
**unverified**, regardless of Linux compilation or a successful source gate.
No Windows/web parity, Windows support or signing approval follows from this
document. The broader policy is mandatory now; existing comparison tooling and
historical captures do not certify every required surface or animation.

## Change records

### 2026-11-01 — Core correspondence policy across all in-game presentation

- Reference: Linux remains canonical; the intended game appearance and behavior
  are unchanged by this documentation update.
- Affected scope: every current/future screen, visual state, champion/form,
  Arena, animation, VFX and visible behavior, across Linux, Windows and web.
- Propagation: the shared presentation remains required for all three targets;
  this update changes no runtime code, assets or gameplay rules.
- Coverage: the JSON now explicitly lists all required platforms. Its historical
  scenario list must be extended for newly affected content/motion; the existing
  comparator still does not consume web captures.
- Validation: policy/link/JSON inspection and `git diff --check` only. No builds,
  new captures or cross-platform runtime validation were run for this policy
  change. Full visual/motion correspondence remains unverified, not waived.

### 2026-11-01 — Laboratory reduced-motion toggle

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

Evidence: Linux native release `547d7aa` built and installed; the Bone Brute Base
Dance toggle/entry/reentry check passed with the saved global setting preset to 1,
which remained unchanged. Normal / reduced / resumed / reentry stage recordings
had 14 / 1 / 15 / 15 distinct frames out of 15 each. These are scoped diagnostic
observations, not the full-frame correspondence gate. Installed UI state was also
inspected. Other champion/form/clip cases, Windows and desktop web observations
remain queued for consolidation and same-source packaged comparison. No
cross-platform parity is claimed. See [the report](../phases/LABORATORY-motion-toggle.md).

### 2026-11-01 — Laboratory Practice arena cover and shared propagation audit

Linux reference: the training arena picture absorbs the Practice pane's available
height and preserves the source aspect ratio. A centered source crop fills the
picture without squashing the arena; the arena selector and Play button follow it
at the bottom of the pane. Update the crop after layout/image loading or resizing.
No blank spacer is reserved beneath those two controls.

Coverage: add `training-arena-cover-forest`, `training-arena-cover-volcanic`,
`training-arena-cover-celestial` and `training-arena-cover-resize` for both languages
and the contract viewport/DPI cases. Check source proportions, center framing,
side crop, selector changes, button visibility and pane fill. The resize case also
uses the contract motion frames for before/after layout transitions. The 768x382 original
thumbnails and 640x360 newer arena thumbnails must both work without asset edits.

Propagation audit: build.rs selects the same `tbg_desktop` presentation for Linux,
Windows and wasm32. The entire training catalog, courtyard, model/camera/animation
controls, local reduced-motion state and this arena crop have one implementation
and shared assets. There are no platform-specific laboratory versions to port.
The changes in this chat already propagate at source level to those three targets;
no duplicate platform code or unrelated target rebuild is needed for propagation.
This audit is not evidence of exact rendered parity.

Linux native release `ac50ba6` built and installed. Forest at 1920x1080, Volcanic
and Celestial at 1920x970, live selector clicks and resizing to 1366x768 were
inspected: proportional centered lateral crops, no bottom spacer, both controls
visible. The installed executable was separately opened at 1920x970. See
[the work-front report](../phases/LABORATORY-arena-cover.md). Same-source
Windows and desktop-web full-frame/motion observations remain pending consolidation,
including the previously registered laboratory and reduced-motion cases. No waiver
or visual-parity claim follows from shared routing.

### 2026-11-01 — Champion and Stronghold upgrade modal consistency

Linux reference: first-tier Evolution retains two equal side-by-side choices.
Only the second-tier remaining branch spans the entire content row. Fortification
uses the champion's exact shared option skin, text size, heading/close geometry,
spacing and outer frame, including the single full-width remaining second-tier
choice. Choice names, price, next step and effects are inline instead of separate
A/B medallions and coin boxes. Unavailable/pending feedback and authoritative
purchase checks remain intact; purchases retain focus for the next tier.

Shared propagation: `native_battle::{chrome, unit_panel, fortify}` uses one
presentation on Linux, Windows and desktop web. The shared choice bundle grows
into the available row, so hiding the unchosen branch removes the empty half.
No platform-specific style or game-rule/protocol/price change is introduced.

Matrix additions: `evolution-tier0-two`, `evolution-tier1-a-wide`,
`evolution-tier1-b-wide`, `fortification-tier0-two`, `fortification-tier1-a-wide`,
`fortification-tier1-b-wide`, `upgrade-dialog-disabled`, and
`upgrade-dialog-focus-transition` (matched motion frames). Cover A/B, both languages,
viewport/DPI cases, price/availability states, retained focus and dismissal.

Evidence: native Linux release `ba65ac8` built and installed. Actual champion
and Stronghold tier 0/1/2 purchases passed on branch A at 1920x970 and branch B
at 1366x768. Two-choice widths changed from `[193, 193]` to one `[392]` and
from `[152, 152]` to one `[310]`, respectively. Focus remained through
purchase; outside dismissal completed. Representative modal screenshots were
visually inspected. The packaged installed executable separately passed the
branch-A flow using its own assets. See [the work-front report](../phases/UPGRADE-modal-consistency.md).
These observations cover representative Linux cases only; disabled states,
the remaining locale/viewport/DPI cases and Windows/web same-source packaged
full-frame/motion comparisons remain pending consolidation. Source sharing alone
is not proof of visual parity.
