# Champion completion — seventeen authored kits and local Linux preview

**Latest laboratory update:** the owner-selected PR44 head 9fb7be0 is now
installed with the support-facing fix as preview 0.1.2-cc.2. See
[the support-facing report](SUPPORT-FACING-FIX.md) for current installed code
and [the PR44 update report](CHAMPION-COMPLETION-PR44.md) for laboratory
integration, screenshots and validation. The original delivery below
is retained as historical evidence.

**Original delivery (2026-11-01): implemented and installed as a separate Linux
preview; all 17 champions and 85 forms are available in Troops/Training and
the 63-champion local Practice pool. Blue/red foot rings identify teams; body
affiliation plates are removed across all 68 registered kinds. Earlier
checkpoint notes below are historical; final installed evidence is at the end.
Windows/web parity, subjective visual/fun approval and competitive balance
remain pending. No push or public deployment was performed.

**Owner request, 2026-11-01.** Keep all seventeen concepts, complete every
champion's appearance and behavior, show the initial low-poly base and four
Evolution references, then implement, validate and install locally on Linux.
The owner explicitly confirmed continuing through local installation.

Branch: `phase/champion-completion`, based on main `0e23a1a`. No push,
history rewrite, public deployment or production roster activation.

## Read first

- AGENTS.md and docs/VALIDATION.md.
- ART-DIRECTION sections 2–5 and 11; ARCHITECTURE section 7.
- docs/ui/DESKTOP-UI-CONTRACT.md.
- docs/champion-completion/catalog.json and the affected champion sheet.
- Existing core special/hit/healing contracts and art/kits patterns only
  when their work item needs them.

## Deliverables and order

1. Preserve the original seventeen visual references; serve them and the
   production gallery on loopback. Generate one five-form low-poly reference
   sheet per champion. English and Portuguese names cover the base skin,
   champion, both Evolution steps on both branches and signature ability.
2. Finish each character's authored model and motion brief: five forms,
   head/body/prop silhouette, weapon, projectile or melee impact, idle,
   walk/facing, ordinary attack, special, Spawn/Hit/Evolve, unique Victory
   dance and Death. Cosmetics remain presentation only.
3. Append seventeen kinds without reordering old wire tags. Register initial
   **Proposed** numbers and competitive semantics in GAME-DESIGN. Preserve
   the 46-kind online pool; the new 63-kind pool is a local preview until
   owner look/fun and competitive acceptance. No economy, Synergy or old
   champion balance changes.
4. Implement deterministic authoritative abilities and meaningful targeted
   regression scenarios. Healing, shields, reductions, status priorities,
   seat ownership and Stronghold exclusions follow existing contracts.
   New per-unit state remains bounded and replay-visible.
5. Implement seventeen distinct art kits through the shared rig/material,
   cached joint meshes and named palette. Each Evolution changes scale,
   a major material surface and silhouette. The reference sheet is concept
   evidence; only an actual engine capture proves the implemented model.
6. Measure posed bounds and space for all five forms, including attacks,
   Victory and Death. Keep crowd offsets cosmetic and within current
   limits. Fold wings/antennas/chain/fan props rather than widening the
   BattleLine or changing competitive range/targets.
7. Integrate local Troops/Training/Practice and bilingual display names.
   Register projectile, special and status cues with reconnect snapshots;
   stage production protocol changes coherently if view shapes change.
8. Run the union of affected tests once under the shared nonblocking lock
   with CARGO_BUILD_JOBS=2. Record actual Linux evidence and exact pending
   Windows/desktop-web cases in the visual contract. No full workspace
   suite or heavy balance gate is inferred from this request.
9. Build the authorized Linux preview once under the lock and install it
   in a separate versioned local slot, preserving the currently installed
   client and user data. Verify the launcher/version and new catalog.

## Presentation coverage

All seventeen champions and all 85 forms require one shared implementation
for Linux, Windows and desktop web. Every pose, palette, prop, timing, status,
shot, impact and effect carries the same meaning. No platform quality waiver.
Targets that have not been observed remain pending, never parity-verified.

The default skin is the authored appearance and four Evolution forms. This
phase does not invent a paid Store, unlock or Inventory requirement.

## Execution report

- Scope: seventeen confirmed by the owner; user also confirmed implementation
  through local Linux installation.
- Baseline: isolated worktree created from main 0e23a1a. Existing user changes
  in the main workspace are preserved.
- Visual references: original seventeen PNG files preserved in the ignored
  local artifact folder; loopback server on port 8963.
- Catalog: seventeen individual briefs, bilingual champion/default-skin/four
  Evolution names, unique weapon/projectile/motion/space plans written.
- Initial images: all seventeen five-form sheets generated, inspected and
  copied to the local evolution gallery (85 concept appearances). They are
  reference illustrations, not claimed engine renders.
- Actual models, scoped tests, parity checks and installation remain pending.
- Core work-front check was attempted once under the shared lock and returned
  exit 75 (busy), with no compilation started. Deferred to consolidation;
  no waiting, polling or lock bypass was performed.

### Validation and deferred work

Work-front mode. Documentation/art inspection and git diff --check at this
checkpoint. Compilation and tests wait for the corresponding implementation;
full suite and heavy matrices are not authorized by this phase.

### Implementation checkpoint

- `cargo check -p tbg-core --lib`: passed (2.10 s) under the shared lock.
- `cargo check -p tbg-client --bin tbg-client`: passed (3.95 s) after using
  the repository's existing local ALSA SDK. Initial attempt failed because
  the host's default pkg-config path did not contain alsa.pc; no dependency
  was installed and audio support was not removed.
- `cargo test -p tbg-core --lib completion_ -- --test-threads=1`:
  20 passed, 0 failed, 50 filtered out, 0.02 s execution.
- `cargo test -p tbg-client --bin tbg-client completion_ --
  --test-threads=1 --nocapture`: two passed, 0 failed, 162 filtered out.
  All 85 forms built and sampled through ten life/action clips; maximum
  measured cost 2938 triangles (limit 6000). All additional insect feet
  retained their world contact in the targeted stance scenario.
- The same client test command had earlier returned 75 while another build
  held the shared slot. No lock was removed or active build interrupted.
- `python3 scripts/check-desktop-ui.py`: passed; this proves routing only.
- The runtime Training allowlist integration was found during review and is
  being covered by an additional 85-form session scenario. The latest
  triangular prism and footprint integration require final focused checks.
- Known unrelated test-only warning: `visuals/layout.rs::Layout::clear` is
  unused in this build. A new unused test import was removed.

Pending at this checkpoint: schema-only deterministic hashes; registry
contract consolidation; final art/UI/session checks; actual Linux captures;
separate local installation. No full suite, balance gate, Windows or
desktop-web parity claim is made.

### Consolidated implementation evidence

- Core completion with `--features lab,training`: 21 passed, including all 85
  private Training forms, healing attribution and immediate Root cancellation.
- Legacy determinism: all three winner/reason/tick guards passed, then the
  schema-only checksum migration passed the frozen contract.
- Protocol: 15 passed, including bounded completion facts, short windup and
  preservation of old enum tags. Public-view privacy: three passed.
- Final client completion focus: six passed, including all 85 life/geometry
  samples, exact two-hand shaft/rim bindings, extra planted insect feet,
  three-second Victory loop seams, bilingual ability templates and exact
  0.30-second contact timing for six projectile families.
- Art uses cached per-joint meshes, named shared palette/materials, no new
  dependency or simulation floats. Extra insect legs/wings reuse fixed nodes.
- Measured resting widths drove compact poses and per-kind crowd profiles;
  see the visual correspondence contract. Production balance is still Proposed.
- Existing registry Evolution rows were retained verbatim; 68 new rows were
  appended. Remaining registry/ruleset/view/training consolidation was deferred
  once on exit 75 and remains queued for the final targeted batch.
- New localized mathematical wording was changed to font-supported prose
  after the build script correctly rejected U+2265. No font waiver was added.
- Actual engine portraits/lineups and installation are the next checkpoints.


### Owner body-identity amendment

The owner rejected the blue hip squares in the runtime portraits. The older
ART-DIRECTION instruction requiring body team colors is explicitly superseded:
remove affiliation geometry and trim, share one body mesh set for both seats,
and preserve blue/red foot rings during control effects. All 68 registered kinds
and five presentation forms are covered, including retained fallback paths.
Real shields, organs, garments and faction colors remain intrinsic and fixed.
The new control HUD badge updates its material when Root changes to Slow.

Focused regression coverage includes all 340 authored forms, shared body mesh
handles for both seats, reserved ownership colors, status-badge transitions,
all-kind form distinction/budgets and the six new-kit motion/binding checks.
The first amended-source targeted batch returned exit 75 (shared slot busy),
without compiling or waiting. Updated captures, portrait assets, final binary
and Linux installation are pending at this source checkpoint.

Amended-source consolidation: all eleven client-focused scenarios passed:
`ring_only_` (2), `every_kit_` (2), `completion_` (6) and
`every_kind_has_its_own_figure` (1), with `--test-threads=1` under the shared
host lock and two Cargo jobs. This covers 340 presentations, both-seat body
mesh identity, reserved colors, retained fallback materials, Root/Slow badge
reuse, budgets and distinct Evolutions. The first actual compile exposed two
empty affiliation-only builders and one test fixture needing the added badge;
these were removed/updated before the successful batch. Unused removed-flag
imports were also cleaned. Source-routing and `git diff --check` passed.
Logs are in the ignored artifact directory `validation/ring-only-checks.json`.


Capture review rejected loading overlays in part of the first native run.
Original images/logs remain preserved. The opt-in capture now waits for actual
boot/scene readiness, not only elapsed time. The capture script checks readiness
and content hashes, reuses only valid captures with per-image build provenance,
and recaptures rejected images. Siege Walker's affiliation-only hip plate and
hem were removed entirely after visual review; its portrait is invalidated.


### Native capture checkpoint

The final union contains 84 ready Linux native captures: 67 roster portraits
and 17 five-form lineups (Idle/Walk/Attack), including the owner's three
examples. Each PNG content hash and loading-ready log was verified. Seventy
valid captures were reused with their original build metadata; fourteen were
new/missing, rejected-loading or invalidated-geometry replacements. Seven
loading overlays and the first blank Needlewing capture remain preserved
in the original artifact directory, excluded from delivered portraits.
The gallery links the per-image provenance manifest.

The final Siege Walker change passed the 340-form ownership regression,
all-kind budgets and distinct-form checks (two scenarios), and all-kind
figure uniqueness: four focused scenarios, zero failures. Existing unrelated
Layout::clear test-only warning remains. No core rules changed in this amendment.
The six completion scenarios and HUD-badge transition test remain valid from
the prior amended-source consolidation. Native debug build passed in 7.67 s
with source c50ee5b; its opt-in capture waits for actual scene readiness.

Installation is the next action; its receipt and installed-build smoke evidence
will be stored beside the captures. Windows/desktop-web visual and motion
comparison, dense-formation runtime review, subjective look/fun approval,
D-33 measured competitive balance, full suite and production activation remain
unverified. No production server/client deployment is included.


### Installed Linux delivery

- Installed code/assets revision: `4bce5e93c055c5752a905bb50857cf22a2536cbd`.
  Version `0.1.1-cc.1`, protocol 32, contract `w32-r3-0c28263061714c3f`,
  native debug target `x86_64-unknown-linux-gnu`.
- Executable SHA-256: `f697d95e551202772b31cc9ef7c8c1c8220262cf9ef6c32c4779f7fd782cd6ea`.
- Installer: `python3 scripts/install-champion-preview.py --binary
  <artifact-root>/engine-install-renderer/tbg-client`. Installed separately
  at `~/.local/opt/tupi-battle-grounds-champion-preview/0.1.1-cc.1-4bce5e93c055`,
  with launcher `~/.local/bin/tbg-champions` and desktop entry
  `~/.local/share/applications/tbg-champions-preview.desktop`. Existing game
  installation, data and root checkout changes are preserved.
- Final locked native build passed in 2.21 s; installed binary, Cargo.lock
  receipt and every copied asset hash matched the committed installation.
- All 67 portrait WebPs were independently compared with their ready PNGs:
  pixel difference zero, each within 40 KiB.
- Actual installed-build captures in Portuguese passed: Troops selected
  Prismancer and displayed 63 champions; the native range showed blue/red
  foot disks with unmarked bodies; Training passed pause, exact one-step,
  reset identity and stale-command rejection. All three scenes reported
  boot ready, no pending images/texts and no failed images.
- The first Training launch used an unsupported hyphenated debug flag and
  opened Lobby. That attempt is preserved; the corrected native environment
  parameter `TBG_CAPTURE_PANEL=training` passed the lifecycle journey.
- The preview launcher was started for owner validation with isolated data.
  Captures and receipt: `.local/champion-concepts-2026-11-01/installed-linux/`
  and `linux-installed-verification.json` in the original checkout.
- Browser inspection confirmed the localhost gallery's seventeen sections,
  bilingual names, actual 85 forms, concept disclosures and download link.
  Fixed image dimensions preserve anchor positions while lazy images load.

Validation level: targeted consolidation, not full suite. All compilation,
Rust tests and native scene batches used the shared nonblocking host lock;
Cargo jobs and rendering CPU affinity were capped at two. No balance gate,
benchmark, workspace suite, production match activation or deployment ran.

Deferred: Windows packaged same-source captures plus strict comparison via
`scripts/check-desktop-ui.py --compare ...`; desktop-web build and matching
actual browser motion/effect observations; dense formations and owner look/fun
review. The D-33 competitive balance gate and full workspace suite need a
separate explicit owner request. This documentation/review-gallery-only delivery report
commit does not change the installed game source/assets revision above.
