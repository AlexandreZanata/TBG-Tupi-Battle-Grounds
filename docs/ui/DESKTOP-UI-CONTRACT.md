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
