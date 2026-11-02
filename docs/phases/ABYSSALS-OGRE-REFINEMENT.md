# Abyssals and Ogre visual refinement

Owner request, 2026-11-01: refine all nine Ogre champions against the attached
Hook Brawler five-form reference; replace Gust Drummer with a long-reload
rolling healing-barrel carrier; move Trident from Plants to Abyssals, add eight Abyssals and replace the
Plants slot with Pod Snapper. Continue the established
concept/native-model/Linux-preview/GitHub delivery workflow.

Implementation on `codex/ogre-faction`, retaining PR44/CC17/OG-01.
Original checkout and owner edits remain untouched. No merge or public deployment.

Read first: AGENTS, VALIDATION, ART-DIRECTION sections 2–5 and 11,
DESKTOP-UI-CONTRACT, GAME-DESIGN OG-01, existing completion/kit/preview pipelines.

AB-01 and OG-02 numeric values are Proposed. Nine new kinds and a ninth
Faction are appended; 81 authored conventional champions. New kinds remain
private-preview-only pending competitive acceptance. Online FULL_POOL is unchanged.
The ninth Faction has the existing Trident Tank, three new Dps, two new Ranged, two new Control
and one new Support. Plants gains a Ranged replacement for its former Tank.
GustDrummer keeps its stable code id but becomes Barrel Keeper / Mestre do Barril
and Support; Ogres therefore deliberately have two Supports in this requested kit change.

Art target: organic tapered muscle volumes, broad brows/jaws, fitted layered
clothing, long readable Hook Brawler hook blades and separate A/B equipment.
All figure bodies are seat-independent; foot rings carry blue/red identity.
The reference is art direction, not a promise of pixel-identical illustration/mesh.

Validation: minimum per work front, one union of targeted tests at consolidation,
shared nonblocking host lock and at most two Cargo jobs. No full workspace suite,
release packaging or full balance gate is authorized. Windows/game-web visual
correspondence remains pending until actually observed.

## Results

Source implementation complete. Linux render, preview installation and local landing evidence in progress.

### Work-front checks

Core library check passed in 1.85 s. The first native check failed because
ALSA pkg-config was unset. One approval review timed out without executing
its command; the permitted single retry ran using the existing native sysroot.
An explicit-target cache miss caused an unintended dependency recheck; no
new target directory was created. This is recorded as wasted validation work,
not evidence of an additional supported platform.

> blocked: native work-front check repeated E0063, missing `casks` in the
> VfxAssets initializer. The intended patch had been inside the timed-out
> command and did not execute; a later script assertion stopped before
> repairing it. The initializer is now repaired with an inspected patch.
> Stop this work-front compile step. Continue independent core regression,
> documentation and art work; queue the integrated native check for scoped
> consolidation instead of repeatedly compiling this front.

The initial client check also found a missing ninth palette row and a u32
TICK_HZ conversion; both were corrected. Final results remain pending.

Core kit consolidation compiled after the new test's CANON import was corrected.
31 selected cases passed, including all six rolling-cask scenarios. Two old
catalog assertions still expected 72 champions and 85 Training forms; actual
execution covered 81 champions and 175 forms. The roster assertion was updated
and passed. A first automated edit missed the `covered` variable in the final
form-count assertion, repeating that one fixture failure.

> blocked: stop the repeated Training-count assertion step after 175 != 85.
> All 175 per-form functional assertions executed before the obsolete final
> count failed. The literal is now corrected to 175 by an inspected patch;
> do not call this individual assertion rerun a pass. Continue independent
> protocol/roster contracts, bounded balance evidence and native art work.

A separate protocol contract run passed 18 cases and found its literal still
pinned to wire 34. A multiline replacement missed that literal once; the
repeated version-assertion step is stopped under the same policy. An inspected
regex correction now pins wire 35 in the AB-01/OG-02 canon change. Do not count
that assertion as rerun-passed; independent protocol layout/roster cases and
bounded measurements continue. No gameplay threshold was changed to fix a test.

### Targeted consolidation (completed source tree)

All commands used the shared nonblocking host lock, two Cargo jobs and the
existing cache; tests used one test thread. No workspace suite ran.

- Core kit selection: 32 distinct passing cases across the initial run and
  corrected roster check; stopped count assertion remains separately disclosed.
- `cargo test -p tbg-core --features lab,training --lib og02_first_allied_contact`:
  one pass, including two wounded allies healed after real swept contact.
- `abyssal_roster`: one pass. `ogre_roster`, `contract_protocol_server`,
  `contract_public_view`, `contract_roster`: 15 passes combined.
- `contract_protocol`: 18 functional passes; stopped wire literal assertion
  remains corrected by inspection, not counted as rerun-passed.
- `cargo test -p tbg-core --features lab --test contract_determinism
  --test contract_lab --test contract_ruleset_version`: seven passes. All five
  tick traces match the new revision. Wire 35 / ruleset 10 / `523044fc973bac39`.
- `cargo test -p tbg-client --bin tbg-client -- ogre_ abyssal_ ring_only_
  support_ completion_all_ability_templates --test-threads=1`: 11 passes in
  1.43 s. 45 new forms, 45 Ogre-module forms, all 430 registered forms with
  seat-independent bodies, 260 healer-facing cases, bazooka/arms and all
  rolling-cask form clearance samples. This closes the earlier missing-casks
  native work-front compile issue through integrated consolidation evidence.
- `cargo check -p tbg-server --bin tbg-server`: passed in 3.38 s. One harmless
  core import warning appears without the lab feature; it is not a test failure.
- Bounded D-33 screening: 126 seat-swapped duels, including all five Trident
  forms before/after transfer, eight non-healer additions, and two measured
  follow-ups. CSVs are in `docs/abyssal-faction/`. Final Pearl Slinger wins
  Shield/Longaxe and loses Archer; Nacre Duelist wins Shield and loses the
  other two. These limited solo results are not a competitive balance gate.

Native Linux images and training witnesses are next. Windows and desktop game
web observation, team/all-form balance coverage and the full balance gate remain
pending. The stopped count/wire assertion reruns are deferred under AGENTS rule 6.
