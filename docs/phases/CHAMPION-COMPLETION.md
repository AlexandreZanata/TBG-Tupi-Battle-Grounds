# Champion completion — seventeen authored kits and local Linux preview

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
