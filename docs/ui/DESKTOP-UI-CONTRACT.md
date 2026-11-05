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

### 2026-11-01 — Laboratory Practice reference proportions

Linux reference: match the owner's 1366x768 laboratory screenshot. Difficulty
choices use the standard 38-pixel laboratory controls instead of the compressed
30-pixel rows. The Practice column uses 8-pixel gaps and a 10-pixel note inset.
This gives the controls above the arena the reference proportions and reduces
the flexible picture's height. The centered cover crop, full pane fill, bottom
arena selector and Play button remain unchanged.

Shared propagation: `training::catalog` is the same implementation on Linux,
Windows and desktop web. Cover all eight arenas, both locales and viewport/DPI
cases; check difficulty selection, proportional arena cropping, bottom controls
and resize without clipping. Add `training-practice-reference-layout` and
`training-practice-reference-resize`, with motion frames for the resize case.
Linux release `e0d8f46` built and installed. Celestial Practice at 1366x768 was
visually compared with the owner's reference; the difficulty rows, gaps and
smaller arena picture follow its proportions. The 1920x970 layout and the actual
installed executable at 1366x768 were also inspected, with both bottom controls
visible. See [the work-front report](../phases/TRAINING-augment-polish.md).
Other arenas, locales, viewport/DPI and resize cases, and Windows/web same-source
packaged comparisons remain pending consolidation. No parity claim is made.

### 2026-11-01 — Augment cards without glow or rarity labels

Linux reference: remove the card's BoxShadow entirely, including the selected
green glow that extended below the illustrated frame. Selection remains visible
through the existing thin outline. Remove the rarity text node from every card;
Bronze/Silver/Gold/Diamond frame artwork and price remain the visual identifiers.
Name, description, price, art placement, availability, keyboard focus and
two-step authoritative confirmation retain their existing behavior.

Shared propagation: `native_battle::council::{options, texts, layout}` supplies
one shared presentation on Linux, Windows and desktop web. Add
`war-council-no-rarity-labels` and `war-council-selection-no-shadow` to cover
all four frame rarities, selection changes, hover/keyboard focus, unavailable
options and resolved choices; include motion frames for selection changes.
Linux release `e0d8f46` built and installed. Actual War Council pointer selection
changed between two cards; the outline and enabled confirmation remained visible
without an external glow. Diamond and Silver frames, names, descriptions and
prices were inspected with no rarity labels. See [the work-front report](../phases/TRAINING-augment-polish.md).
Other rarities, keyboard focus, availability/resolved states and Windows/web
same-source packaged comparisons remain pending consolidation.

### 2026-11-01 — P73 Active players, invitations, rematch and match history

- Reference: Linux native is canonical. P73 adds the Active players directory
  (search, filters, paging, availability, loading/empty/failed/stale/partial
  states), incoming/outgoing invitation notices, Result actions (Players, My
  history, recorded Pending/Stored/Delayed status, rematch offer in every
  online PvP mode), owner history list/detail/settings and the public profile
  history preview. No Arena, champion, combat VFX or Victory dance changed.
- Propagation: one shared Bevy implementation (`ui/history`, `ui/social`,
  `ui/shell/live`, `ui/result`) with shared pt-BR/en catalogs for Linux,
  Windows and web. The only target split is the timestamp service adapter:
  native reads the system zone through Jiff and the browser through `Intl`;
  both render the same `dd/mm/yyyy HH:MM ±hh:mm` (pt-BR) or
  `mm/dd/yyyy HH:MM ±hh:mm` (en) text, so layout and visible output match.
- Coverage: `desktop-ui-contract.json` adds `live-players`, `player-invites`,
  `result-p73-actions`, `result-rematch`, `history-owner-private`,
  `history-owner-public`, `history-detail` and `history-public-profile`.
  The contract still lists only the original five Arena screens; extending it
  to Celestial, Clockwork and Desert belongs to the eight-Arena front.
- Validation: source gate `python3 scripts/check-desktop-ui.py` passed on the
  consolidated tree. Seven native Linux fixture captures exist from the P73
  front at a pre-final revision (`docs/planning/evidence/p73/`). Windows and
  browser captures, and final-source Linux captures, are **pending**;
  correspondence is unverified, not waived.

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

## Ring-only champion ownership — owner amendment, 2026-11-01

All 68 registered kinds and their five forms remove body affiliation plaques,
ribbons and generic tabards. Real equipment uses fixed intrinsic materials;
body meshes are cached once and shared by both seats. Blue/red ground rings
carry ownership, including during Root/Slow; those effects use HUD badges.
This amendment applies equally to Linux, Windows and desktop web, portraits,
Shop/Troops/Training/Practice, carried units, summons, Victory and Death.
The owner explicitly replaced the older body-team-color instruction in
ART-DIRECTION §2. New physical badges or seat-dependent body recoloring are
forbidden. Updated Linux captures/installation are in progress; packaged
Windows/desktop-web and both-seat motion comparisons remain pending.


### 2026-11-01 — PR44 laboratory baseline in CC17 local preview

The owner selected PR44 `0b56bb4` as the authoritative current laboratory
presentation. The local champion preview now includes its skin, tab controls,
responsive desktop layout, model preview, training report surfaces, Evolution
dialog and Augment cards, plus P71 measured overhead/occupancy and independent
status outline. All 68 kinds retain identical bodies for both seats with no
affiliation plates. The seventeen added champions remain selectable in the
updated 63-entry catalog and local training. Linux recapture/install checks
are pending on the integrated source; Windows/desktop-web visual and motion
parity remain pending. Old screenshots do not certify the updated UI.

### 2026-11-01 — Stable friendly-target support posture

All ten conventional Support kinds and their five forms use one shared
friendly-action torso/head look across Idle, Walk, Cast and Hit. Feet keep
the BattleLine heading while the authoritative patient can be behind or
across the rear angle boundary. Incoming hits never retarget the heal or
reset the support's look toward an attacker. Patient loss releases the look;
hostile attacks and authored life/celebration clips remain independent.

Shared paths: art/anim.rs, art/anim/support_facing.rs and sync/events.rs.
Coverage: both seats, all forms, 30/120 Hz, rear/side/front patients, target
loss, repeated hits and casts; paired grips and projectile timing. The
[focused report](../phases/SUPPORT-FACING-FIX.md) records the failing-before/
passing-after evidence. Linux preview 0.1.2-cc.2 (code c1d3e20) is installed:
the actual rear-patient witness passed with incoming damage, 60 observed
frames and maximum local foot yaw 0. Both original and new support kits were
observed; the final installed witness used Nectar Nurse. Six unique focused
tests include 200 form/seat/frame-rate cases and walking aim-fade continuity.
Windows and desktop-web actual motion comparison remain pending.

### 2026-11-01 — OG-01 Ogre faction, equipment clearance and catalog

Shared native kits cover nine new champions × five forms on Linux, Windows
and desktop web: distinct shapes, intrinsic material progression, weapons,
projectiles, action/life clips and celebrations. New Ogres join the existing
Ogre Archer; Rivet Ballista replaces its Machine roster slot. Foot rings alone
identify ownership; no body plaques or presentation pedestal on Ogre captures.

SiegeWalker/Ogre Archer and new kits reserve measured body, arms and held
weapon envelopes across Idle/Walk/attack. Overhead labels still exclude the
weapon height. Rotated clearance feeds the existing cosmetic crowd solver;
server position/range/collision rules remain authoritative. Cases: five Ogre
Archer forms, both seats, frontal/oblique aim, fire/reload, mixed adjacent
figures and Evolution growth; stable rear-patient support posture also covers
Hide Mender. Existing bounded lane offsets apply; dense-lane clearance needs
actual observation and is never certified only by compilation.

Landing: 72 actual-model portrait cards, eight filters with nine units each,
English/Portuguese titles and explicit preview-only availability. Counts derive
from core roster sources. Desktop Linux/browser observation, Windows and
same-source web motion comparisons are recorded in
[the expansion report](../phases/OGRE-FACTION.md); pending until observed.

## AB-01 / OG-02 — Marine faction, organic Ogres and healing casks (2026-11-01)

Affected: all nine Ogre champions and five Evolution forms (Hook Brawler
reference anatomy/coat/hooks; Barrel Keeper replacing Gust Drummer); eight
new Abyssals plus existing Trident and the Pod Snapper Plant replacement;
roster filters, bilingual names/facts, Shop portraits, gallery lineups,
locomotion/aim/actions, specials, impacts, spawn/evolve/hit/death/victory,
actual mesh equipment clearance and support rear-target posture.

Shared presentation: one native kit/rig/palette/animation path for Linux,
Windows and desktop web. Only foot rings identify seats; no body plaques or
team recolors. All marine figures remain grounded. Rolling barrels use the
same authoritative snapshot interpolation as units, including both seats,
reconnect, source death, interception, area burst, reload visibility and pause.
No local collision/heal decision or platform-specific effect is permitted.

Matrix and motion coverage: the same five forms × ten life/action clips per
affected champion, close/far/rear aim, both seats, pause/reset/reconnect and
30/120 Hz support posture. Linux is the reference; native captures and scoped
motion witnesses will be recorded in the phase report. Windows and desktop
game-web comparison are pending. A shared implementation or native build is
not parity evidence. Static landing review is separate from game-web parity.

AB-01/OG-02 evidence: 38 ready Linux engine captures (95 forms, three poses), 11 targeted client cases and a post-blade geometry follow-up. Native Training witnesses and installed-build receipt are recorded separately in the phase report. Static landing checks passed for 81 entries; actual browser inspection is pending because the browser session reports `browser.user is unavailable`. This is not game-web parity evidence.

Final AB-01/OG-02 Linux witness: installed source `5553393`, all 379 asset hashes verified, five real Training scenarios passed (both cask contacts, mixed Abyssals, two attacked rear-healers with foot yaw 0). See `docs/abyssal-faction/linux-validation.json`. Four nonfatal entity commands after reset need cleanup investigation. Windows/game-web and browser-observed landing appearance remain pending.

Rivet Ballista retains its pre-refinement head and arm proportions. The shared Ogre-module walking accent also reaches this Machine; include its Walk clip in the same pending cross-platform motion comparison. Its prior Ogre-delivery reference sheet remains historical.


## TW-01 / PK-01 / GD-01 — Gods and authoritative passage (2026-11-02)

Affected shared presentation: Colossus and all five forms, one back-carried
obelisk, heavy walk, draw/slam, stone ground impact, spawn/evolve/hit,
kneeling death and monumental victory salute; Gods palette, bilingual names,
portraits and filters. No body team markers. The native review gallery must
space monumental forms without changing gameplay scale.

Widen the common BattleLine floor to match five usable tiles; move meadow
dressing outside it and extend the Command aura arc to its edges.
All 87 kinds use server-provided lateral positions when the authoritative
physical-lanes candidate flag is enabled, interpolated on the existing
timeline. Those matches use no competing repulsion solver. Legacy canonical
matches and stress tools retain their prior cosmetic solver while the
candidate awaits competitive acceptance. The same flag travels to spectators. Show the local
deployment count in HUD/Training and localize LaneFull. Rolling casks use
launch origin/target and exact burst lane to follow shifted figures.
Stronghold preview/Fortification panels share the new HP, damage, costs,
Bastion armor, Artillery splash, Treasury spoils and Command aura facts.

Coverage matrix additions: Linux / Windows / desktop game-web, both seats,
Colossus five forms × Idle/Walk/attack/Hit/Spawn/Evolve/Death/Victory,
obelisk recovery at the authoritative hit cue; crowded Archers, mixed widths,
full-width blockage, gradual free-lane bypass, stationary pause, queue release,
rejected Evolution, cask interception and all four Fortification branches.
Linux engine captures and targeted witnesses are the reference evidence;
Windows and desktop game-web observation remain pending until recorded.
Shared code and native compilation never certify correspondence.


GD-01 gallery framing uses the actual posed vertices of all five forms in the
three lineup rows, including forward obelisk depth. It fits the projected bounds
with the shared camera FOV and margins for the title/footer. Cases include the
highest A2 crown/obelisk and the B2 forward slam. Linux native capture is the
reference; Windows/game-web observation of this same framing remains pending.


Final GD-01 Linux evidence: actual 400x400 portrait and unclipped 1600x1000
vertex-fitted five-form sheet, plus native packing/allied-cask/interception
witnesses. The installed debug preview is source `a1f6446`, wire 36, with all
380 asset hashes verified. Exact renderer revisions, observations and pending
Windows/desktop game-web cases are in the TW-01/PK-01/GD-01 phase report.
The new catalog window uses separate data and preserves the earlier session.


## RESTORE-01 — Legacy movement on the wide BattleLine (2026-11-02)

Shared Linux/Windows/desktop game-web path: all normal matches, private Practice,
Training and spectators keep legacy authoritative progression and immediate
unit entry, with cosmetic pushing only. The five-tile floor, stronger towers,
all champion models and foot-ring identification remain. No gameplay capacity
queue is active in those modes. The visible window revision identifies R12.

Crowd offsets use fixed 60 Hz relaxation, stable anchor/ID order and eased,
speed-bounded presentation offsets; authoritative anchors remain immediate.
Aim changes do not reorder figures by moving a virtual anchor. Cases: repeated
purchases into a crowded line, twelve mixed large/ranged figures, free advancement,
Evolution among allies, crowded idle/turning, 30/60/120 Hz, pause and both seats.
Targeted tests and the Linux native witness are recorded in the restoration
report. Windows/desktop game-web visual observation remains pending.

RESTORE-01 Linux receipt: source `ef0975c`, R12 installed with all 380 asset
hashes verified. One native witness observed twelve allied root entities plus
one opponent, no queue and 30 stable paused frames. Ten Archers share an
artificial initial anchor and can occlude each other; this evidence certifies
entity entry and bounded motion, not universal non-overlap. Core tests cover
immediate purchases and continued advancement. Exact commands, hashes and
remaining platform observations are in `docs/phases/LEGACY-CROWD-RESTORATION.md`.


## RESTORE-02 / ARENA-CLEAR-01 — Original pushing and road clearance (2026-11-02)

Supersede RESTORE-01 display easing, fixed stepping, pair ordering and anchor
changes. Linux, Windows and desktop web again use the pre-PK-01 crowd solver and
direct offsets; only the lateral bound expands to 2.1 tiles. Normal movement and
purchase entry stay unblocked. Coverage: both seats, pushing/overtaking, crowded
attack/turn/walk, immediate purchases and Evolution. Towers and models are unchanged.

All eight Arenas keep the five-tile floor with its full paved border clear of
scenery. Common brazier origins place both base and fire outside the road; fence
posts and animated cloth share side origins. Move complete scenery groups from
their authored origins, preserving snow caps, crystals, trim and glow alignment.
Coverage: Forest, Tundra, Volcanic, Ruins, Storm, Celestial, Clockwork and Desert;
near/far road edges and corners, trees/canopies, rocks, palms, columns, temple
steps, fire flicker, cloth sway and gears. Stronghold-attached banners are excluded
from the decorative clearance volume. Shared code/tests do not certify platform
parity: Linux observations and Windows/desktop web pending cases belong to the
corresponding phase receipt.

RESTORE-02 / ARENA-CLEAR-01 Linux receipt: source `e868927`, nine targeted
client tests passed, eight real Practice scenes captured and individually
inspected, and all 380 installed asset hashes matched. Fire sits on each shared
brazier and the widened road remains clear. The old solver/direct-offset source
is restored apart from its lateral bound. Per-image provenance and pending
Windows/desktop web cases: `docs/phases/ARENA-CLEARANCE-AND-ORIGINAL-PUSHING.md`.


## GD-02 — Colossus arms in live crowd clearance (2026-11-02)

Shared Linux/Windows/desktop web: the existing cosmetic crowd solver now reads
Colossus's measured form envelope, including both arms/hands, armor and obelisk,
rotated with its current aim. Measurement and live selection share one predicate
to prevent a measured champion being omitted from crowd separation again.
Retain original direct pushing, wide road, immediate births and no normal queues.
Coverage: Base/A1/A2/B1/B2, Idle/Walk/Attack, near/far aim, both seat facings and
a neighboring Archer. GD-02 initially deferred for a busy validation slot; all
three Colossus tests subsequently passed in ARENA-REFINE-01 consolidation. Linux
reinstall remains pending and the installed source remains `e868927`.
All GD-02 runtime/platform observations are pending; see
`docs/phases/COLOSSUS-ARM-CLEARANCE.md`.


## ARENA-REFINE-01 — Separate independent scenery silhouettes (2026-11-02)

Shared Linux/Windows/desktop web: optional trees and scattered props are placed
after authored architecture, checking actual transformed mesh footprints and
previously accepted sites. The layout uses the same seeded sites on all targets.
Remove redundant rear coastal boulder groups that cut into trees/temples; keep
foreground geology and whole fitted snow/plant/stone assemblies intact. Separate
Forest rock gardens, Tundra foreground firs, Celestial statue surroundings and
Clockwork boilers/fences. No road width, movement, unit balance or camera changes.
Coverage: all eight Arenas; rock/canopy intersections, landmark silhouettes,
boiler/fence separation, compound attachments, original road-clearance cases.
The phase receipt records actual validation and observed platforms; unobserved
Windows/desktop web appearance remains pending.

ARENA-REFINE-01 current evidence: both road/animation tests and all three
Colossus cases passed on the initial combined tree. The new scenery test found
insufficient Forest density; nine curated anchors restore the intended silhouette
in source without relaxing the test. Its focused rerun returned exit 75. Final
Arena tests, native install and new visual observations remain pending. No
ARENA-REFINE-01 or GD-02 runtime parity is claimed.


ARENA-REFINE-01 final consolidation (SIDE-01): all three Arena tests now pass,
covering every Arena, the animated attachments and retained Forest density.
The bounds check also exposed a remaining Tundra foreground fir/coastal-rock
overlap; moving that complete fir/crystal group to z=3.4 fixes it without
narrowing the road. Runtime installation and platform observations follow in
the SIDE-01 receipt; tests alone do not establish visual parity.


## SIDE-01 — Absolute blue/red sides and common camera (2026-11-02)

Linux/Windows/desktop web share the same camera transform, viewport framing
and authoritative world projection: blue A left, red B right for participants
and spectators. No per-player coordinate reflection or negative mesh scaling.
Actual ownership independently controls Shop, Evolution, Fortification, result
and private UI; red humans command the right Stronghold. Foot rings/materials,
figure/weapon yaw, crowd direction, tower looks/fire/aura/wear, barrel motion and
bursts, HP fills, crests, names and ratings use the absolute side. Intrinsic
champion body colors remain unchanged.

Arena chip and five-second introduction identify Blue side/Red side in both
languages; viewers get a neutral legend. All online modes/random fresh rematches
and local Practice assign sides without dependence on arrival order. Frozen
identity survives reconnect; replay/history/statistics retain canonical seats.
Coverage: both human sides, bot on either side/difficulty, all tower kinds,
units/Evolution forms, movement/aim/heal/area/casks, command ownership, reconnect,
spectating and ranked settlement. Native captures are reference observations,
not Windows/web parity evidence. The SIDE-01 phase receipt records actual checks
and leaves all unobserved platforms/cases pending.

### 2026-11-01 — Canonical cross-platform source and asset bytes

- Linux remains the reference. Text inputs use UTF-8/LF on every checkout;
  binary artwork is preserved verbatim. Git's Windows CRLF conversion must not
  change shipped catalogs, the dependency-lock digest or hashed rule sources.
- `.gitattributes` and the native input preflight enforce these build inputs
  before compilation. The historical archive remains excluded from conversion.
- The complete native promotion guard still requires identical assets, lock,
  features and MatchContract across the final Linux/Windows package pair. It
  rejects differences rather than normalizing away evidence after a build.
- Local synthetic Windows-autocrlf checkout tests and input guards passed.
  Same-source final packages and the existing pixel/motion matrix remain the
  acceptance evidence; line-ending tests do not certify visual parity.

## RESTORE-03 — Departure before contact

Linux/Windows/desktop web use the same original 1.1 preferred formation scale
inside the retained 2.1 lateral boundary. A departing unit that does not contact
another cannot repel it merely because the road grew. Real contacts still push,
allow overtaking and regroup after separation. Test both sides near each tower,
ordinary bodies and measured equipment. The focused numerical regression is
mechanism evidence; all runtime/platform observations remain pending.



## CROWD-EXIT-01 — Tower-exit contact fidelity, natural regrouping and bounded arm/bazooka clearance (2026-11-02)

Shared Linux/Windows/desktop web path (`crates/tbg-client/src/crowd.rs`,
`crates/tbg-client/src/sync/crowd_ecs.rs`, `crates/tbg-client/src/art/overhead.rs`):
`Solver::solve` keeps `Params::half_width = 2.1` for the outer road clamp while
restoring `HOME_HALF_WIDTH = 1.1` (`[-0.66, +0.66]`) for initial tower-exit
birth offsets and the lateral regrouping spring target. Units no longer spawn
or drift toward `±1.26` without contact, yet dense pileups can still push out
to `±2.1` on the widened road and regroup into the center band afterward.

Live collision envelopes respect real visual volume instead of worst-case
multi-target weapon swings: `overhead::reserves_equipment` is scoped to
champions with explicit equipment/arm contracts (`Colossus`, `SiegeWalker`,
`GustDrummer`); `Colossus` measures its upper-arm, forearm and hand vertices
rather than its slammed obelisk; and `crowd_ecs::live_body` uses proportional
walking radii while advancing unengaged (`aim.is_none()`) and directional
envelopes when engaged (`aim.is_some()`). Exiting units do not displace
separated allies ahead of them before contact, collide and push naturally on
real contact, and regroup smoothly. Coverage: both seats and spectators, tower
exit with and without contact, overtaking, dense pileups, and all five
Evolution forms of `Colossus`, `SiegeWalker`, `GustDrummer`, `Shatterback`,
Ogres and Abyssals. Linux targeted regressions and native preview installation
are verified; Windows and desktop game-web visual observations remain pending.

### CROWD-EXIT-01 consolidation receipt (2026-11-02)

Normal merge of exact source 73c7a07c28833d6663a11dda65332a578a8c4a3a
retains HOME_HALF_WIDTH and all live-body/overhead changes. The earlier
RESTORE-03 regressions remain alongside both new tower-exit regressions.
Source routing and patch-note checks passed. The requested local focused
union was deferred: the shared nonblocking validation lock returned 75
before compilation; no test ran or passed in that attempt. Hosted CI on
the consolidated head remains required. Windows/browser rendered parity
is still pending, independently of shared source and numerical tests.


## GD-03 — Colossus body turn and 3D obelisk-tip strike alignment (2026-11-02)

Shared Linux/Windows/desktop web path (`crates/tbg-client/src/art/kits/colossus/motion.rs`,
`crates/tbg-client/src/art/anim.rs`):
- `Animator::observe` treats active combat action clips (`AttackMelee`,
  `AttackRanged`, `Cast`) with an active `aim_at` as engaged immediately, even
  while walk-stop hysteresis (`STOP_SECS`) is still settling or when crowd
  offsets nudge root `x`.
- `colossus::motion::pose` turns `Joint::Root` toward `m.aim` during
  `AttackMelee`, `AttackRanged`, `Cast` and `Hit` (`tools::face`), centers the
  obelisk on the body midline during the strike (`at.x = 0` at `Strike`),
  places both hands symmetrically on the shaft, and solves the chest-space
  strike pitch and longitudinal slide so the obelisk ray and its pyramidion tip
  (`Vec3::new(0.0, 1.27, 0.0)`) align directly with `m.aim.target` in 3D across
  all five Evolution forms.
- Coverage: all five `Colossus` forms (`Base`, `A1`, `A2`, `B1`, `B2`), both
  seats, on-axis and off-axis lateral targets on the widened road, close and
  max-range combat targets, and arm/hand clearance envelopes (`gd01_colossus`,
  `gd02_colossus`, `gd03_colossus`). Linux targeted regressions and native
  preview installation are verified; Windows and desktop web visual
  observations remain pending.

### 2026-11-02 — GD-04 Gods expansion: Colossus refinement, Volthar and Astreon

- Reference: Linux native client. Shared presentation only: kit figures,
  motion, the storm/rupture VFX and the gallery framing live in
  `crates/tbg-client/src/art/kits/{colossus,volthar,astreon}`,
  `vfx/{shots,completion}.rs`, `sync/{actions,special_cues}.rs` and
  `gallery/*`; no OS or target condition is introduced.
- Affected cases: Colossus Base/A1/A2/B1/B2 (refined geometry, Idle/Walk/
  Victory weight; GD-03 strike alignment and GD-02 clearance unchanged), new
  Volthar and Astreon five forms in every clip (Idle, Walk, Attack, charged
  fourth bolt / pierce, Hit, Spawn, Evolve, Death, Victory), `StormArc`
  links, StormBolt/StormSurge projectiles, Volthar and Astreon impact and
  Evolution bursts, training/catalog cards and shop portraits for both kinds,
  both seats and spectators.
- Matrix: `training-gods-{colossus,volthar,astreon}-forms`,
  `battle-gods-two-colossus`, `battle-gods-volthar-chain` and
  `battle-gods-astreon-pierce` join the screens; all but the two-Colossus
  still also join `motion_screens` (matched frames 0/10/20/40).
- Evidence: see `docs/phases/GODS-faction-expansion.md` for the Linux captures
  actually taken and the focused tests run. Windows and desktop-web
  observations are **pending**; no parity is claimed from shared code or
  compilation.

## P74 — War Auxiliaries (`Mineiro` & `Mercador`) HUD Panel, 3D Precincts & Construction Animations (2026-11-02)

Shared Linux/Windows/desktop web path (`crates/tbg-client/src/ui/native_battle/auxiliaries.rs`,
`crates/tbg-client/src/ui/native_battle/resources.rs`,
`crates/tbg-client/src/arena/auxiliaries/`):
- Bottom-right `Auxiliares (0/2)` / `Auxiliaries (0/2)` navy-and-gold framed
  panel (`right: 14px`, `bottom: SHOP_BOTTOM`, `SHOP_SCALE = 0.9`) with two
  dedicated auxiliary cards (`Mineiro · Cost 6` and `Mercador · Cost 4`), 3D
  portraits (`aux_miner.webp`, `aux_merchant.webp`), and live phase/timer/progress
  badges, keeping the five conventional Shop cards and the
  `Energy / Interest / Army / Reroll` bar in their exact coordinates.
- Free-reroll readout on the `Reroll` button (`Reroll · GRÁTIS (N)` /
  `Reroll · FREE (N)`) enabled at `0` Energy when `free_rerolls > 0`.
- Symmetric Left (`Seat::A` / Blue, `x = -2.55`) and Right (`Seat::B` / Red,
  `x = +18.55`) off-lane 3D precincts for `Mineiro` (`z = -3.45`: rock outcrop,
  portal shoring, lantern, rails, ore cart, pickaxe mining loop) and `Mercador`
  (`z = +3.45`: timber pier on pilings, red-and-cream striped market stall,
  balance-scales banner, crates/sacks, brass spyglass lookout, and wave-bobbing
  wooden Merchant Caravel ship), with 3-stage procedural construction animations
  and theme-adapted palettes across all eight Arenas (`Forest`, `Tundra`,
  `Volcanic`, `Ruins`, `Storm`, `Celestial`, `Clockwork`, `Desert`).
- Linux targeted regressions and native preview installation are verified;
  Windows and desktop web visual observations remain pending.

### INT-06 integration coverage (2026-11-02)

Shared P74/GD-04 source is integrated at wire 38/R15. Construction reads the
authoritative remaining ticks and pauses with the Augment window; idle
work/delivery loops use low-motion static poses when requested. The P74 HUD
remains private to the owning seat; precinct phases are public on both sides
and to spectators. CROWD-EXIT-01 HOME_HALF_WIDTH, walking envelopes and
original pushing remain intact alongside the refined Colossus and both Gods.

Focused numerical/source coverage passed on Linux: 35 client cases including
all eight Arena precinct/road geometry, Gods forms/grips/clearance, both side
orientations and live contact envelopes. Real WebSocket auxiliary ownership,
private free currency, quote roundtrips and reconnect passed independently.
Packaged integrated Linux capture and Windows/browser rendered observations
remain pending; no parity is inferred from these tests or shared code.


### 2026-11-02 — Accurate session warnings and persistence recovery

Linux reference: a confirmed signed-in account shows no temporary-storage
warning. Actual vault failures retain the existing temporary-session notice.
A failed HTTP renewal/confirmation instead says the account is connected and
session renewal/recovery could not be confirmed, with the existing localized
Retry control. Queue/socket admission cannot clear a persistence failure;
successful coordinator recovery clears it. Both Bevy and legacy HTML controls
use one Rust status-to-message selector. No champion motion is affected.

Matrix additions: account-confirmation-pending, account-confirmation-retry,
account-renewal-network-outage and account-queue-return-persistence, alongside
the existing account-restoring/temporary/conflict cases. Shared state, messages
and controls propagate to Linux, Windows and desktop web. Runtime evidence and
remaining target observations are recorded in
[the focused report](../phases/AUTH-SESSION-PERSISTENCE-FIX.md). Native-only tests
or synthetic JavaScript regressions do not prove cross-platform visual parity.


Evidence for the session correction: 21 focused native auth/socket/UI tests and
five synthetic JavaScript regressions passed. The installed Linux release was
observed with an injected confirmation failure (account still logged in, correct
warning and Retry visible) and on restart after recovery (same account,
confirmed persistence, no warning). Both 1280x720 Linux captures were inspected.
Real interactive queue-return observation and actual Windows/desktop-browser
captures remain pending; synthetic JS and Linux observations do not prove parity.

### 2026-11-02 — Native settings, ordered notes and in-app update controls

Linux reference: patch history merges installed and online notes by identity
and displays descending date, ruleset and semantic version, preserving every
entry. Latest installed notes remain visible when the backend has an older
catalog. The account panel offers masked one-time import of the existing
manual recovery key; copying/downloading that protected backup does not change
the active account. Downloads export explicitly to the user's Downloads folder.

Downloads and updates opens the shared in-game modal, including checking,
current, download, staged/restart, recoverable errors and the automatic toggle.
Automatic native updates download and restart only at the existing safe idle
boundary. A Ranked request cancels/defers the download and retains priority;
no updater request changes the game backend endpoint or match contract.
The stable launcher and signed repository supply platform services. Browser
uses its reload adapter; OS vaults and explicit export/clipboard remain platform
services. Typography, colors, spacing, forms and modal rendering stay shared.

Matrix additions: patch-notes-newest-first; settings-key-backup-import/export;
settings-update-check/current/download/idle-restart. No champion motion changes.
Focused evidence and pending Windows/real-browser observations are in
[the settings delivery report](../phases/NATIVE-SETTINGS-UPDATES.md). Shared code
and Linux checks cannot certify cross-platform visual or behavioral parity.


### Final integrated desktop observations (2026-11-02)

The released shared P74/GD-04 presentation is wire 38/R15. Actual integrated
Linux PR-package captures cover Colossus/Volthar/Astreon five-form Idle/Walk/
Attack boards and Forest Practice; all 395 package files were verified. Official
colossus.3 source 6d96245 also captured first-ready-Running Red-side Forest
Practice at 1600x1000, with boot_ready=true and failed_images=0. The existing
P74 and SIDE-01 screen coverage therefore has a concrete integrated Linux smoke,
not just numerical/source tests.

Published desktop web at 1280x720 was observed on Red-side Forest Practice:
84-champion catalog, own Red tower on the right and Blue tower on the left,
side notice, both Auxiliary cards, Merchant hire, three purchases -> FREE(1)
reroll -> consumed, Miner preparation -> active countdown, and Colossus/Astreon
shop portraits. The existing P74 HUD/construction and SIDE-01 cases have this
scoped browser observation. No shared presentation code changed during delivery.

This does not close the Windows column, all-eight-Arena rendered coverage,
spectator/Blue-side browser coverage, or matched-frame GD-04/P74 motion rows.
Those remain pending; no pixel parity or renderer tolerance is inferred.
See [the production report](../phases/DELIVERY-consolidation-production-20261102.md)
for actual sources, evidence and limitations.

## 2026-11-02 — P69 main integration and mobile adaptation

Owner explicitly requests landscape Android composition adapted to compact
phone dimensions while retaining Linux art, information, rules and motion.
Desktop Linux/Windows/web exact presentation requirements remain unchanged.
Android uses the same presentation modules/assets with a mobile composition;
its geometry is compared by information, accessibility and style, not desktop
pixel dimensions. Lobby scenery fills the window with controls inside real
lateral safe bounds; match scenery is edge-to-edge and controls stay safe.

The shared bin-to-lib extraction carries all current main plugins and #57
account/update fixes. Desktop and web retain their existing presentation.
Affected cases: loading, lobby/navigation, account/error/forms, profile/ranking/
history, news/notes, Troops/forms/abilities/previews, Arcanas/Strongholds/Arenas,
Practice/Training, battle/HUD/Shop/auxiliaries/details, War Council, settings,
confirmations/results/Victory/reconnect and return cleanup. All observations
for this integrated source are pending; shared routing/builds never prove parity.
See [P69 current execution](../phases/P69-mobile-20261102.md).

P69 candidate 16 uses the engine's direct CPU mesh submission on Android
following corrupted gfxstream geometry; no mesh, material, lighting rule or
animation asset is replaced. The GLES point-shadow specialization no longer
runs on Vulkan. Linux/Windows/web use their unchanged submission paths.
Android visual comparison of the corrected path remains pending.

P69's next direct-submission candidate also selects the public render-world
`GpuPreprocessingMode::None`; phase batches must match the CPU builder.
Candidate 16 failed this runtime requirement; its screenshot is not accepted.

P69 manual CPU-builder overrides failed twice and are removed. The next
Android capability profile derives the complete uniform-buffer rendering path
from WebGL2 device limits on both native backends; sRGB Vulkan presentation
is retained in the emulator. This does not waive art/motion correspondence.

## 2026-11-02 — P69 compact navigation, laboratory and safe controls

Android's adaptive composition retains the shared Linux heraldic icons,
mode actions, collection art and kit typography. The owner-authorized mobile
header is 56dp; the page fills its measured remainder. The same five modes
(including Ranked account gating), collection pages, live directory and notes
remain reachable. Laboratory controls move into a scrollable drawer while the
model keeps its own viewport. Scroll/cancel handling follows BR-AN05 and
48dp effective controls; it never changes purchases or simulation rules.
Lobby insets distinguish visible bars/cutouts from mandatory gesture strips;
battle scenery and loading backgrounds cover the complete surface. No Android
creator strip is reintroduced. Shared desktop geometry/behavior is retained.

Coverage: lobby, header, every menu page, laboratory catalog/forms/clips/editor/
Practice, touch cancellation/scroll, native text input, loading/safe-area
transitions. Android runtime checks and same-source Linux/Windows/web
observations for this new composition remain pending.

P69 candidate 19's separate full-bleed UI root failed to present content on the
emulator. The next candidate restores the previously observed single UI-root
hierarchy; loading-edge coverage remains pending while geometry/UI visibility
are diagnosed. No desktop root or presentation path changes.

## 2026-11-02 — P69 session-only manual backup and shared key masking

Android consumes the shared form masking state for login and BackupKey entry,
retaining native paste/reveal/Done/Back behavior. Its manual verified-key cache
is process-only; the mobile prompt explains that lifetime. Linux/Windows/web
retain #57's existing OS-vault/browser behavior and text. Native key masking,
reveal/cancel, copy and account/logout states are affected. Emulator journeys
and desktop correspondence remain pending for this source.

## 2026-11-02 — P69 coalesced scrolling and compact battle controls

Android preserves every touch move delivered between rendered frames and picks
the foremost shell target, including disabled modal shields. A BR-AN05
regression covers accumulated motion and cancellation. Its auxiliary cards move
above the Shop; all five offers and Reroll retain independent touch regions.
The resource strip uses compact mobile dimensions and localized labels.
Android Settings retains language/audio/account controls and adds the existing
shadow preference; app version and manual APK update information replace the
inactive desktop updater entry. Desktop geometry and updater remain unchanged.

Coverage: lobby modal ownership, clipped/scrolling Practice controls, battle
Shop/reservation/Reroll/auxiliary controls and settings. Candidate 21 on API36
shows intact geometry in inspected lobby/Practice/match frames but exposes
auxiliary overlap and lost scroll distance. Corrected runtime and same-source
Linux/Windows/web correspondence remain pending.

## 2026-11-02 — P69 explicit Android document export

Android Download recovery key uses the system document picker and acknowledges
the backup only after a successful write. Back/cancel leaves the saved state
unchanged, and stale account-epoch completions are ignored. Clipboard Copy
remains a separate explicit action. Desktop/browser export and #57 updater
behavior remain unchanged. Coverage: account backup/import/save confirmation,
Android picker return/focus/immersive restoration and failure/cancel states.
Compilation and observed journeys for this adapter remain pending.

## 2026-11-02 — P69 landscape IME and mobile match labels

Android text entry keeps the native dialog visible instead of using the IME's
fullscreen extract editor. The dialog resizes with its keyboard; shared
validation/masking/reveal and submission remain authoritative. Mobile match
phase, Stronghold and paid Reroll labels use the active language; desktop
text/geometry remain unchanged. An internal loopback-only gameplay intention
trace records sequence/type without auth/room/resume data, enabling one-touch
and inspection/scroll no-purchase observations. Emulator validation is pending.

## 2026-11-02 — P69 menu density and protected form layout

Android uses smaller panel spacing/type, three Arcana family columns instead
of full-width banner stacks, and independent unshrunk Arena title/theme/effect
regions. Settings actions wrap long localized labels and retain48dp minima.
The saved-key panel owns a high-layer shield and a scrollable body; temporary
session information moves to Settings instead of covering menu content.
Toasts remain readable above the mobile modal. Native text forms have a short
localized heading, a scrollable full prompt, and a reveal control beside the
field so the software keyboard cannot hide it below the input.

Shared native Arcana family/node and Stronghold name/summary/branch/advantage
copy now uses the existing English/PT-BR catalogs, matching browser content
instead of showing raw English catalog fields. This affects corresponding
Linux, Windows and desktop web copy; their geometry stays unchanged.
Coverage: Arcanas, Strongholds, Arenas, menu kit panels, Settings account
actions, temporary account states, save-key modal, toast stacking and native
IME/reveal/Done/Back. Corrected emulator journeys and updated same-source
desktop observations remain pending.

P69 internal loopback diagnostics now report allocated/text dimensions and
positions for overflowing UI labels, without recording their contents. This
measures Arena/Ranking layout defects before another geometry change. Guest
Android Settings no longer claims an authenticated session lifetime; signed-in
session information remains in Settings. Desktop presentation is unchanged.

P69 measured Arena theme labels with57px allocated versus76px of glyphs,
and an effect with60px versus109px. Android's theme/effect text now uses
intrinsic height instead of the desktop minimum-height reservation. Desktop
geometry is unchanged. BR-AP05 result waiting/closed feedback now remains
visible for private/casual rematch offers on every target, matching their
existing server eligibility; a focused regression covers the non-Challenge
state. Neither change alters a rule, cost, deadline or outcome. Runtime
remeasurement and same-source desktop observations remain pending.

P69 continuation: Android War Council uses four horizontal choices with original
frame/illustration proportions, local selection plus a scrollable full-effect
readout and explicit 48dp confirmation. Ranking compacts its header/podium and
keeps the public table/sidebar available through scroll. Desktop geometry is
retained; Council copy now uses existing locale keys on native targets, and
eligible non-Challenge rematches show shared waiting/closed feedback. Arena
mobile body copy uses intrinsic height. Runtime coverage remains pending until
recorded in `docs/phases/P69-mobile-20261102.md`; Windows/web parity is pending.

P69 details consolidation: Android Training toolbar consumes intrinsic space
before measurement panels; catalog stat labels wrap in their actual cells,
Ranking sidebar names wrap, and Android unit details combine full ability/effect
copy and Evolution choices in a scrollable safe panel. Loading scene decoration
fills the surface separately. Native shared unit/Stronghold labels and form names
use existing locales plus equivalent stat-label keys. Desktop geometry is retained;
all affected runtime coverage is recorded separately in the P69 report.

P69 small-height pass: Android header and catalog labels wrap within their own
bounds, the catalog list keeps a reachable scroll viewport, and Ranking supports
whole-page scrolling for short landscape surfaces. The Android system splash
uses the existing dark background. Desktop geometry remains unchanged. Actual
source/profile evidence and failed intermediate captures stay in the P69 report.

P69 laboratory refinement: Android-only status/actions toolbar with equivalent
short labels, wrapped participant navigation/headings and bounded generic
button text. Desktop layout/copy remains unchanged; see P69 runtime evidence.

P69 nested-scroll pass: Android scroll residuals propagate only to ancestor
viewports, respecting visible action shields. Mobile catalog removes the duplicate
heading and scrolls its hint with bounded thumbnail/name rows. Desktop unchanged;
actual runtime coverage is recorded separately in the P69 report.

P69 shared menu lifecycle: idle backup polling no longer marks Account changed;
legitimate Ranking refreshes retain page/table/sidebar scroll. Presentation and
backup semantics are unchanged across native targets. The App-level regression
and actual Linux/Android scroll observations are recorded in the P69 report.

P69 copy bounds: Android readouts retain intrinsic height and use word wrapping
with a character fallback for individual long names/labels. Ranking sidebar
stat labels are bounded; Browser button copy is shortened equivalently on Android.
Desktop geometry/copy remains unchanged. See the P69 per-profile observations.

P69 short battle viewport: Android Shop cells bound compact portraits/names/roles,
Energy label/value share a column, and the48dp auxiliary drawer button flows in
the resource bar beside Reroll. The dismissible drawer retains both choices and
all server-derived prices/progress while shielding background actions. Scroll
ownership permits a viewport's own buttons and blocks unrelated modal branches.
Desktop geometry and all motion/gameplay remain unchanged. Runtime evidence and
pending targets are recorded in the P69 continuation report.

P69 native IME keeps essential actions together above short landscape keyboards;
Android OS Back follows the existing Training editor return action. Native
Fortification tier and Evolution headings use equivalent localized copy, with
desktop geometry retained. Actual observations are in the P69 continuation report.

P69 Android catalog camera uses the measured central preview rectangle to fit
and center the unchanged model envelope below the tabs and above controls.
Desktop camera, models and motion remain unchanged; zoom remains available.
Per-model/form runtime coverage is recorded in the P69 continuation report.

## P75 shared identity and match-social presentation — 2026-11-02

Registration gains a private email step. Shared Bevy overlays implement account email verification/change/hold, lost-key request/code/save-key, suspension, reports, three-slot emoji library/upload states, and chat mode. The battle HUD reserves its right column for text, input and emoji buttons, with seat bubbles lasting exactly 2.5 seconds and sprite-sheet animation driven by the same frame delays on every target. Mute and spectator exclusion share one implementation; private controls carry `Hud::PrivateControls`. Enter requires explicit Tab focus before activating a battle control and typing consumes game keys.

The `p75-*` cases in `desktop-ui-contract.json` cover the states listed by P75-CONTRACTS section 5. Linux is the reference; Windows and desktop web must match the same states, layout, imagery and motion. Native picker dialogs adapt to XDG/Win32/browser services behind the same visible button. Real Linux, Windows and web captures, keyboard journeys, upload errors and animation timing are **pending S14/owner device evidence**. Shared code or compilation does not establish parity.


P75 S14 consolidation adds modal-only focus, PageUp/PageDown scrolling, private preview cache invalidation on identity changes, unavailable chat states, guest report guards, the existing localized history date formatter for hold/suspension deadlines, persistent All/Emojis/Off modes and immediate mute filtering through the same shared presentation. Native and WASM checks passed; every corresponding in-game capture and Windows comparison remains pending. The console-only EN/pt-BR captures in the P75 execution report are outside this game's parity evidence.

## AX-T — Tiered War Auxiliaries: compact cards, real portraits, red side and per-Arena precincts (2026-11-02)

Shared Linux/Windows/desktop web path, no platform branch
(`crates/tbg-client/src/ui/native_battle/auxiliaries.rs`,
`crates/tbg-client/src/arena/auxiliaries/`):

- **HUD:** the two auxiliary cards are 112×152 (two thirds of a Shop card) in
  a panel anchored inside the scaled Shop column, `10 px` right of the fifth
  card, bottom-aligned, sharing the Shop scale. Each card shows three tier
  pips, the status line, and `Custo`/`Cost`, `Evoluir`/`Evolve` with the next
  price, or `Nível III`/`Tier III` when maxed.
- **Portraits:** renders of the in-game low-poly `Mineiro`/`Mercador` figures
  in the viewer's team colors (`aux_{miner,merchant}_{blue,red}.webp`),
  captured with `--gallery=auxiliary --aux=… --side=… --portrait`.
- **Red side:** Seat B precincts are mirrored with a negative X scale; their
  material renders both faces so the outward faces keep correct lighting
  instead of the grey interiors previously shown.
- **Placement:** per-Arena offsets and Merchant sea lanes (`placement.rs`
  `LAYOUTS`) keep every tier, platform, bridge and ship course clear of the
  road, Stronghold footprints and existing scenery
  (`br_ax15_every_tier_bridge_and_ship_course_clears_existing_arena_scenery`).
  Offshore pads stand on stilts; a plank bridge is laid plank by plank from
  the shore during construction.
- **Tier motion:** tier 2 adds a winch with a bucket and an ore heap
  (`Mineiro`), goods and pennants (`Mercador`); tier 3 adds a headframe with a
  spinning sheave, a crystal crown, a banner and an apprentice (`Mineiro`), a
  pavilion and a coin chest that opens on payouts (`Mercador`). Swing speed,
  sparks and ship size grow per tier; a golden ring sweeps out on each
  Evolution. Low motion keeps static poses and instant reveals.

Affected cases: battle HUD shop row (both sides), every Arena × both sides ×
tiers 1–3 precinct rendering and motion. Linux native captures are the
reference; Windows and desktop web rows for these cases remain **pending**
until captured on those targets.


P75/AX-T consolidation (2026-11-02): the compact auxiliary cards remain children
of the shared Shop scale. The P75 chat column stays above that row. Backup-key
forms retain four keyboard stops; login-key forms include the fifth lost-key
stop. Newly issued recovery keys replace cached exports and pending pre-rotation
export results on every desktop target. All device observations remain
explicit in the consolidation report; these shared paths alone prove no parity.

BR-AX15 consolidation preserves all static land props even at a precinct anchor;
only sea-surface floaters below the existing harbor threshold may yield. The
unfiltered scenery collision contract remains mandatory for every Arena/side.

P75/AX-T HUD integration: a compact two-row social dock at the upper right
keeps both precinct flanks visible. Activating Chat opens the log in the central
rectangle; guests can read it, while only the authoritative Allowed policy
enables typing. Escape/close restores the dock and captures battle keys while
open. Both rectangles are included in the eight-Arena precinct visibility test.
Linux is the reference; actual Windows/web presentation and motion comparison
remain pending until observed.

The 1280×720 web observation exposed clipping from full-height kit controls in
the compact dock. Its shared controls now use two 32 px rows; mute remains in
the expanded log. Standard emoji handles stay resident and bind after Update
command application, before layout/render extraction, so slow HUD rebuild
frames never display an unbound placeholder. The first-frame regression is
covered by a scheduled Bevy test; actual device correspondence remains pending.

P69 current-main integration retains P75 and three-tier auxiliary content.
Android adapts the shared tier cards into its explicit drawer, gives P75 dialogs
owned touch/scroll/Back, puts account/library/recovery entries in Settings, and
uses native IME/document services for chat and emoji selection. Desktop/web
retain main's layout, keyboard and picker. All actual new-source target
observations remain separate and are recorded in the P69 continuation report.

P69 Android configuration handling preserves the native loop across font and
related resource changes, refreshes insets, schedules IME after window focus,
and gives the explicit key-save panel an owned scroll viewport. No desktop/web
layout or motion changes. Profile failures and repeat evidence are tracked
separately in the P69 continuation report.

P69 native overlays: Android explicitly handles assetsPaths configuration
changes through the shared host callback. Model preview rotation consumes an
owned touch drag confined to its measured viewport and excludes its controls
drawer; real mouse behavior and all desktop motion/assets remain unchanged.
Actual cancellation/rotation/configuration observations are in the P69 report.


P69 identity forms: Android verification uses the shared alphanumeric code
contract and Unicode scalar field limits. Account/Library responses clear the
loading notice only after actual data arrives on all targets. Native IME
geometry and desktop field geometry are unchanged. Candidate40 lifecycle/drag
and five actual Linux/Android canonical traces are recorded separately in the
P69 continuation report; verification-form runtime repetition remains pending.


P69 battle visibility: the Android scene remains edge-to-edge while its camera
fits the actual horizontal safe width and centers bodies between measured HUD
bounds. A framed Army control opens complete scrollable Synergy/Augment details;
shared desktop readouts and camera are preserved. Android Council names occupy
the full card width above the original art/price. On all targets, Council has
global modal priority and chat closes throughout its authoritative pause.
Actual runtime and correspondence observations are pending the next candidate.


P69 Result: Android keeps confirmed outcome/reason/duration/purchase/Evolution
summary and every action in a bounded scroll panel. Compact action and
celebration rows retain48dp controls and leave scene space; keyboard hints stay
on desktop. Cast, animation, page size and Linux/Windows/web geometry remain
unchanged. The overflowing candidate40 Result is a recorded failure; new-source
winner/loser/celebration observations remain pending.


P69/P71 retry correspondence: all clients retain an active seat's retry path
through the canonical connection pause, reconnect window and terminal replay
retention. Queue/probe timeout and the existing banner layout remain unchanged;
the numeric banner now reflects the longer active-seat retry limit. Only the
server determines continuation/forfeit. Actual Android40 early-abandonment and
Linux pause/result evidence, plus the focused regression and pending corrected
runtime repetition, are recorded in the P69 continuation report.


P69 social dock: Android places the collapsed chat/count in a48dp header slot,
reserves the remaining score width, and bounds the two-step surrender label so
it cannot grow across the HP plates. Expanded chat and policies are shared;
desktop score/social/surrender geometry remains unchanged. Actual candidate41
body visibility, drawer/Back and one-confirm Council evidence are recorded in
the P69 continuation; the compact dock repetition is pending candidate42.


P69 tier readouts: Android permits two wrapped lines in the existing auxiliary
subtitle region, so Merchant T3 progress and remaining charges stay complete.
Desktop subtitle geometry, every tier's art/motion and all authoritative values
are preserved. The actual T1/T2/T3 candidate41 observations and clipped status
failure are recorded in the P69 continuation; wrapped-runtime repetition remains
pending the next package.


P69 score readability: Android stacks the existing name/rating and Stronghold
labels on separate full-width lines above HP, within the same plate height.
The Arena side chip retains ownership identification; desktop name suffix,
plate layout and every authoritative value remain unchanged. The overlapping
candidate42 failure and actual41 Evolution/Fortification evidence are recorded
in the continuation report. Corrected header observation remains pending.


P69/P75 pointer capture: expanded chat captures gameplay while permitting its
own compose/close/mute/emoji actions on all targets. Actual P75 dialogs retain
their dedicated handler; native forms retain their existing guard. No geometry,
chat permission or authority change. Candidate42's inert chat controls are a
recorded failure; corrected delivery and background-gameplay exclusion remain
pending actual runtime repetition.


P69 celebration visibility: Android fits the unchanged formation envelope into
the actual safe space above the measured Result panel by changing only the
camera. Dancers are not rebuilt on panel-height changes; cast, world slots,
page size, clips and desktop camera/motion are preserved. Mobile paging gets a
wider range label and hide/show/motion captions omit desktop keyboard hints.
The actual42 complete-summary pass and occluded-cast failure remain distinct;
corrected full-cast/motion/cleanup observations are pending.


P69/P75 retained chat: expanded logs show the existing bounded recent messages
for native touch users and guest readers regardless of keyboard focus. Compact
docks and emoji-bubble timing remain unchanged. Actual native45 IME/delivery to
Linux passed; its empty older-message drawer is a recorded failure. Corrected
later reading remains pending on the next package.


P69 terminal recovery: shared server cleanup honors the existing unexpired
terminal replay window after a long P71 pause. All clients require actual
MatchStart/MatchEnd recovery before clearing the original retry deadline;
Welcome alone cannot leave a frozen battle marked ready. No window, game value,
seat authority or layout changes. The45s recovery passed; the120s terminal
failure, targeted regressions and pending rebuilt-harness repetition are
recorded separately in the P69 continuation.


P69 Result cause correspondence: shared Result copy uses the existing
server-provided disconnect/moderation cause when present, preserving manual
surrender/local fallback copy. Geometry, outcomes and authority are unchanged.
The34 focused client/two server passes and actual45 chat/pointer negatives are
recorded in the continuation. Disconnect-label runtime repetition is pending
the rebuilt local backend.


P69 large-roster Result: after an actual82-purchase/83-entry Android result,
winning Result uses a bounded scrollable right sidebar with two-column actions.
The unchanged64-entry cast page fits the measured safe region to its left, and
cosmetic roots face that fitted camera without restarting clips. The range
label fits both real pages. Nonwinning Result and desktop geometry remain
unchanged. Actual counts/paging and the too-small above-panel failure are
recorded separately; corrected sidebar/cast/motion/cleanup observation is pending.


P69 emoji selection: the shared standard library uses art-only controls with
enough width for the existing32px icon and padding. Numeric IDs no longer
overlap the emoji. Selection, slots, permissions and server authority are
unchanged. Actual Android47 library/cancel evidence exposed this issue;
corrected Android/Linux/web observations are queued.


P69 authenticated recovery: valid same-family reconnect honors the existing
terminal replay deadline after a long pause, through both session cleanup and
authenticated hello. Cross-account, expired Access and controller protections
are retained. Android48 terminal failure and its bounded Menu recovery remain
recorded; corrected signed-in runtime repetition is pending.


P69 Result/social ownership and accessibility: Android's eligible P75 result
report toolbar belongs to the Result scroll panel, so it cannot cover Menu or
history. The catalog stacks full-width filters. Ranking uses localized short
headers with a full explanatory legend, wrapped bounded values and a secondary
rank-status size. Desktop geometry, server eligibility and all data/actions are
preserved. Actual48 clipping/overlay failures remain recorded; corrected
font100/130, report and scroll observations are pending.


P69 body readouts: Android P75 account/recovery/report/library text and retained
chat lines have bounded width, nonshrinking height and word/character wrapping,
including maximum-length unbroken input. Desktop typography/geometry and
shared input/protocol limits remain unchanged. Actual48 report stored500 of
501 attempted ASCII scalars and showed Report sent; overflow is recorded
separately from the successful submission. Corrected rendering is pending.


P69 contextual native input: P75 email fields select the existing Android email
keyboard, and Room selects alphanumeric code input. Their original shared
limits, values, submission and server rules remain intact. Desktop/web editing
geometry/behavior is preserved. Actual52 recovery succeeded; the generic email
keyboard is a recorded presentation issue. Corrected IME observations remain
pending the next package.


P69 final emulator consolidation (2026-11-03): APK54/Linux/web share executable
sourcecd958bd0 and current main177da4a0/w40/R16. Actual scoped screen/input/
profile/cross-play/terminal-recovery observations,612s active/paused reference
measurements and safe original-frame hashes are indexed in
[the consolidated evidence](../phases/P69-emulator-20261103.json). This records
scoped Linux/web runtime, not complete pixel/motion equivalence. Windows visual
coverage, physical ARM64/16KB and owner look/fun remain pending; Windows CI
build/tests/package success does not change that status. Earlier candidate
failures and their corrected repetitions remain distinct in the phase report.

P69 beta publication (2026-11-03): Android's launcher label becomes Tupi Battle
Grounds Beta. Shared game art, layouts and champion motion are unchanged. The
landing reuses eight README Linux frames, adds three labeled Android emulator
frames and refreshes hero/social media and content-hash portrait/Arena URLs.
Desktop/390 x 844 PT/EN landing observations are in
[the media report](../evidence/landing-android-beta-20261103.md). This adds no
in-game parity evidence: Windows pixel/motion and physical ARM64/16 KB remain
pending. Public download compatibility uses the actual running server.

### 2026-11-04 — Verdant Forest Blender study

- The [editable art study](../art/BLENDER-FOREST-ARENA.md) covers Forest
  terrain and coast, the BattleLine, Classic Strongholds, scenery and
  desktop 16:9 match framing. Blender checks cover the Stronghold anchors
  and the unobstructed BattleLine.
- Linux, Windows and desktop web runtime comparisons remain pending for
  the study. Integration must compare match framing, arena transitions
  and Stronghold damage and motion using shared assets.

### 2026-11-04 — Blender Stronghold Fortifications

- The [Stronghold art study](../art/BLENDER-STRONGHOLDS.md) covers Base,
  A1, A2, B1 and B2 for Treasury, Bastion, Artillery and Command. The
  Blender scenes exercise both Forest placements and check the existing
  projectile anchors and rest pitch.
- Future runtime comparison covers both seats in the selection gallery,
  Laboratory, BattleLine, and Fortification transitions. Motion coverage
  includes defender aim, opening hatches, shot and reload, recoil, mint
  and Artillery gearing, cumulative wear and zero-HP collapse. Runtime
  and pixel/motion checks on Linux, Windows and desktop web are pending.

### 2026-11-04 — Animated Blender War Auxiliaries

- The [Forest auxiliary study](../art/BLENDER-WAR-AUXILIARIES.md) covers
  Mineiro and Mercador in both seats, all three tiers, bridge and
  precinct construction, work loops, ship delivery and completion.
  Blender checks cover timed assembly, pick contact, berth, crane and
  chest motion, plus sampled BattleLine clearance.
- Future runtime comparison covers hire and upgrade reconciliation,
  payouts, pause and reduced motion on the Forest BattleLine. Linux,
  Windows and desktop web runtime comparisons remain pending.

### 2026-11-04 — Frozen Tundra Blender study

- The editable study covers the second arena's winter terrain, coast,
  BattleLine, Strongholds and desktop match framing. Both seats have a
  Mineiro and Mercador with winter structures on the shared Forest
  animation timelines. [Frozen Tundra study](../art/BLENDER-FROZEN-TUNDRA.md)
- Linux Blender checks cover the camera, anchors, four shore approaches,
  BattleLine clearance, construction and level reveals, pick contact,
  ship berth, crane line, chest motion and departure.
- Queue runtime presentation checks on Linux, Windows and desktop web at
  integration. Cover arena transitions, both seats, hire and upgrade
  reconciliation, work and delivery triggers, payouts and reduced motion.

### 2026-11-04 — Volcanic Forge Blender study

- The editable study covers the third arena's ash plateau, lava
  shore, Strongholds, BattleLine and desktop match framing. Both
  seats place a Mineiro and a Mercador on the Forest auxiliary
  timeline. [Volcanic Forge study](../art/BLENDER-VOLCANIC-FORGE.md)
- Linux Blender checked the shore approaches, keep-out corridor,
  construction, work, crate lifts, chest motion and both barge
  routes against drifting slag. The shared verifier also passed
  on the saved Tundra project.
- Queue runtime comparisons on Linux, Windows and desktop web for
  arena transitions and both seats of each auxiliary, including
  authoritative work and delivery triggers, upgrades, payouts
  and reduced motion.

### 2026-11-04 — Ancient Ruins Blender study

- The editable study covers the fourth arena's limestone coast,
  BattleLine, Strongholds, sanctuary and desktop match framing.
  Both seats use the shared Forest auxiliary timeline with
  separate quarry portals, reed canopies and carved prows.
  [Ancient Ruins study](../art/BLENDER-ANCIENT-RUINS.md)
- Linux Blender checked the four bridge approaches, BattleLine
  clearance, construction and upgrades, pick contacts, boat
  arrival and departure, crane line, chest and lagoon clearance.
- Queue runtime comparisons on Linux, Windows and desktop web
  for arena transitions, both seats, hire and upgrades, work and
  delivery triggers, payouts and reduced motion.

### 2026-11-04 — Storm Valley Blender study

- The editable study covers the fifth arena's slate coast, BattleLine,
  Strongholds, conductor relay, static sky and desktop match framing.
  Both seats use the shared auxiliary timeline with forged quarry
  supports, standing-seam market roofs and riveted prows.
  [Storm Valley study](../art/BLENDER-STORM-VALLEY.md)
- Linux Blender checked the four shore approaches, lane clearance,
  construction and upgrades, pick contacts, boat approach and
  departure, parcel hoists and chest motion.
- Queue runtime comparisons on Linux, Windows and desktop web for
  arena transitions, both seats, hire and upgrades, work and
  delivery triggers, payouts and reduced motion.

### 2026-11-05 — Celestial Citadel Blender study

- The editable study covers the sixth arena's ivory coast, BattleLine,
  Strongholds, palace, guardians, static cascades and desktop framing.
  Both seats use the shared auxiliary timeline with carved quarry
  portals, gilt azure canopies and ceremonial prows.
  [Celestial Citadel study](../art/BLENDER-CELESTIAL-CITADEL.md)
- Linux Blender checked the four shore approaches, lane clearance,
  construction and upgrades, pick contacts, berths, parcel hoists,
  chest motion, departures and the three waterfall placements.
- Queue runtime comparisons on Linux, Windows and desktop web for
  arena transitions, both seats, hire and upgrades, work and
  delivery triggers, payouts and reduced motion.
