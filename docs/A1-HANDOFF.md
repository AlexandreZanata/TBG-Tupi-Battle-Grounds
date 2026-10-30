# A-1 execution handoff — 2026-10-29

## Resume here

The owner requested a handoff to Antigravity because their Codex credits
are running out. **Do not consider A-1 implemented.** Step 0 passed; step 1
is partially audited; steps 2–14 have not started. No permanent client
behavior change has been made.

Work only in `/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/tbg-a1`, branch
`phase/a1-visual-polish`, based on main
`077abe7e4cdd0ac06b8bddc64d849bdb69d053c0`.

The owner's original checkout is
`/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds` on
`phase/p40-arcana-trees`. It has unrelated edits. Do not implement there.
At execution start it was at `6ee3495`, with an edited ROADMAP, an untracked
A-1 spec and `.claude/`; the planning-time claim about uncommitted balance
changes was stale. Nothing there was modified.

Read the latest owner instructions in the original checkout's `AGENTS.md`,
then this worktree's `docs/phases/A1-visual-polish-troops-strongholds.md`,
then only the spec's needed Read first sources. This handoff preserves the
recognition work; there is no need to scan the repository again. The
original detailed executor prompt is also available at
`/home/iiii/.codex/attachments/978c34a8-93f0-40de-9d59-6bab2adf12d6/Pasted text.txt`.

## Boundaries and workflow

- Desktop browsers only. No phone layouts or touch work.
- No edits to `tbg-core`, `tbg-server`, `tbg-balance`, or `Cargo.lock`.
- No rules, values, event/protocol changes; `PROTOCOL_VERSION` on this base
  is 9. No new dependencies, glTF, skinning, physics or general IK.
- Targets come from `Event::AttackStart { from, to }`, never proximity.
  Do not change `lane_z`, logical positions, targeting, ranges or cooldowns.
- Strongholds: visual `StrongholdLook` only. Live mapping stays `Classic`.
  The four planned looks are Treasury, Redoubt, Ballista and Banner, with
  shared Walls/Arms branches. Do not import the competing P17-A system.
- English code, comments, docs and commits; module responsibility comments.
- Steps and checks in order; one commit per step and per Faction at step 8.
  Never push, merge, deploy, rewrite history, or touch `archive/`.
- Same error twice: stop that step, record exact `> blocked:` evidence,
  continue only independent steps. The step-7 prototype gate is mandatory.
- Copy Bevy 0.20 API patterns from the examples installed on disk.
- No agents were delegated. Do not run multiple agents on the same files.

## Completed verification and repository state

Commit `53aa523 feat(a1): record isolated preflight and baseline environment`
contains the supplied spec and step-0 report. The spec was untracked in the
original checkout and absent from main, so it was copied into this branch.
The additional handoff commit contains documentation/evidence only.

Untouched-source commands passed:

```text
cargo build --workspace
Finished `dev` profile [unoptimized + debuginfo] target(s) in 4m 05s

cargo check -p tbg-client --target wasm32-unknown-unknown
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 46s
```

Temporary audit builds also passed `trunk build --release` from
`crates/tbg-client/`: the first Rust release build took 4m 20s; an improved
probe rebuild took 1m 57s. Trunk exited 0 after packaging in both cases.
A first invocation from the workspace root failed with
`ERROR could not find the root package of the target crate`; correcting
the working directory resolved it.

The four temporarily instrumented source files were restored byte-for-byte
to HEAD after saving [the probe patch](media/a1/temporary-audit-probe.patch).
**The ignored `dist/` and release build artifacts still contain the probe;
rebuild after restoring source before collecting an untouched baseline.**

Not run: final fmt/clippy/test suite, aim tests, final WASM verification,
brotli size measurement, Firefox gameplay, online two-client/reconnect
validation, baseline performance matrix, final performance matrix.

## Environment and browser procedure

- `rustc 1.99.0 (b940084d7 2026-09-28)`; `trunk 0.22.1`.
- Google Chrome 156.0.8078.73; Mozilla Firefox 158.0.1 (version only).
- Node 26.3.1 is installed. `playwright`, `puppeteer`, and
  `@playwright/test` were not resolvable from this checkout. The saved
  browser helpers use Node's built-in fetch/WebSocket and Chrome DevTools;
  no npm dependency is required.
- Headless Chrome defaulted to **software** SwiftShader and initially ran
  near 1 FPS. Those observations are not a performance baseline.
- A normal desktop Chrome under X11 uses **real hardware**:
  `ANGLE (Intel, Mesa Intel(R) Graphics (RPL-P), OpenGL 4.6)`.
  Both Intel Raptor Lake-P and NVIDIA RTX 4060 devices are installed;
  the tested renderer was Intel, not NVIDIA.
- `DISPLAY=:1`, `WAYLAND_DISPLAY=wayland-1` were available.
- Match evidence used a CDP override of **1280 × 800, DPR 1, mobile false**.
  Keep the controlling CDP session connected. On detach the override is
  removed; the browser window's remaining viewport was only 1280 × 550,
  which hid the HTML battle HUD and broke the purchase script. Adding the
  override to the targeted script fixed this. No app layout was changed.
- This manually requested sibling checkout is not app-managed. An attempted
  app attachment returned `The checkout exists but is not a managed
  worktree.` Use the path directly in Antigravity.

To repeat the local browser setup (ordinary shell, separate terminals):

```bash
cd /home/iiii/PESSOAL-PROJETOS-ALEXANDRE/tbg-a1/crates/tbg-client
trunk build --release
python3 -m http.server 8131 --bind 127.0.0.1 --directory dist

google-chrome --user-data-dir=/tmp/tbg-a1-chrome-gpu \
  --remote-debugging-port=9333 --no-first-run \
  --disable-background-networking --ozone-platform=x11 \
  --window-size=1280,800 about:blank
```

Do not compare hardware results against SwiftShader. Verify the renderer
and foreground/background behavior for every benchmark run. The screenshots'
FPS overlays are transient readings, **not** the required 60-second,
three-run medians. No p50/p95/p99, memory, CPU or asset-count baseline exists.

Saved helpers are in [docs/media/a1/tools](media/a1/tools/):

- `tbg-a1-cdp.mjs`: CDP connection, evaluate, screenshot. Default port 9331;
  the two audit scripts explicitly use the hardware browser at 9333.
- `tbg-a1-audit-browser.mjs`: `?autoplay=practice&audit&measure`, console
  capture and first 12 arrow-adjacent screenshots. It loops up to 900 s;
  it does not automatically detect the match result. Stop it when done.
- `tbg-a1-targeted-audit.mjs`: starts Practice through the normal UI and
  buys available Rampart/Sawbot cards through normal button clicks. It
  supplies missing melee coverage without changing gameplay rules.
- Scripts save working results under `/tmp`; copy needed evidence before
  cleanup. Run scripts from their saved directory so relative imports work.

## Recognized source facts (correct the planning audit with these)

All references below are relative to this worktree's unchanged source.

| Finding | Verified recognition and source |
|---|---|
| F1 | `state.rs:83` carries both attack targets; `Hit` has no attacker. `content.rs:48` has WINDUP=6. `combat.rs:44–67` handles unit windup; `combat.rs:97–119` emits Stronghold AttackStart and damage in the same tick. No protocol extension is needed. |
| F2 | `sync.rs:95` spreads units across z = -0.8, 0, +0.8 by id. `world_x` mirrors x for seat B. The simulation's lane remains one-dimensional. |
| F3 | `visuals.rs` fixes figure yaw at +65/-65 degrees. `visuals/update.rs:31` rebuilds the transform with the same yaw on Evolution. No attack-target yaw update exists. |
| F4 | Base melee ranges: Shieldbearer/Sawbot/Rampart 0.8, BoneBrute 1.0, Beetling 0.6 tiles (`content.rs:167–245`). Lane separation alone can reach 1.6. Preliminary live distances exceed the 1-D ranges, but the captured probe requires the correction below before its samples can be accepted. |
| F5 | `art/arrows.rs:34` follows socket rotation at fixed range/0.12; `sync/events.rs:166–172` also spawns a generic projectile on AttackStart. Runtime logs confirm native Archer generic emissions followed by native bow releases. Native arrows have a constant horizontal direction near (-0.9063, 0, +0.4226) for the enemy side. |
| F6 | Planning audit was inaccurate: live non-native units are colored legacy bodies adapted to rig v2, not gray blockouts. `art/mod.rs:109` makes Archer base, A1 and A2 native. Archer B1/B2 and every other kind remain adapted. Gray native blockouts are separate review assets. See `art/roster.rs:101–114`; Nester/Rampart/Beetling recolor Bloomkeeper/Shieldbearer/Rootcaller. |
| F7 | Shared ten clips, plus native Archer pose overrides. AttackMelee/Ranged=0.5 s, Cast=0.6, Strike=0.30, Release=0.18/0.22. Death=0.6 plus corpse linger=0.9. Hit is tint + optional nondirectional clip. Native-only foot locking and Archer bow IK already exist. |
| F8 | No live Stronghold kind/tier data. `arena/towers.rs` has three HP groups using constant STRONGHOLD_HP: crown hidden at <=2/3, middle at <=1/3. |
| F9 | Planned names/tree are GAME-DESIGN §10.4–10.5, at lines 472–517 on main (different from the spec's planning checkout). The glossary is §2, not §3. Competitive activation is not part of A-1. |
| F10 | Generic Stronghold shots originate at logical x=0/16, y=1.2, z=0. The visible current towers are actually at x=-0.9/16.9 (`arena/mod.rs`). Account for this when adding muzzle/hit sockets. |
| F11 | `gallery/settings.rs`, `stress.rs`, `overlay.rs` and `diagnostics/measure.js` exist on main. `?stress=N` means total units, split by parity, so use 2N for N per side. Existing JS measure counts actual WebGL calls and reports FPS/draws, but does not provide all required performance metrics. |

Further art recognition:

- `FigureLibrary::FORMS` is **five** forms: (0,0), (1,0), (2,0), (1,1),
  (2,1). Eleven kinds therefore require 55 form rows, not 33.
- `FigureLibrary` caches meshes **per joint per form**, with one shared
  material. Native Archer also caches enemy vertex colors. The spec's
  literal one-mesh-per-form wording should be reconciled with its rigid
  articulated rig requirement; do not destroy articulation to flatten it.
- Baseline mesh/triangle counts for every form were actually logged at
  startup and are saved in `baseline-form-triangles.txt`. Native Archer
  bodies are 1178/1274/1406 triangles, plus 74 bow/arrow triangles.
- Fresh live adapted visuals measured 31 entities; native Archer measured
  34. The full per-form/history matrix has not been measured. A BowRig
  attached before Evolution can remain when switching to a legacy branch.
- Archer A1 currently has no size increase; A2 grows only 7%, contrary to
  the A-1 +8/+16% target. See `art/roster/humans/archer_v2.rs:19`.
- Current Shieldbearer base/A weapon is a sword, not the requested mace;
  B has a spear. BoneBrute base/A has a club, B1 a cleaver, B2 a scythe.
  SiegeWalker currently has torso-mounted cannon barrels, not an arbalest.
  Nester/Rampart/Beetling do not have independent silhouettes yet.
- Existing gallery frame sheets support only A branch and four quarters;
  permanent aim sheets must add the requested cases/branch selection and
  the 100% frame. Standard roster gallery excludes summon Beetling.

## Preserved evidence and audit caveat

- [Autoplay result](media/a1/autoplay-result.jpg): Chromium desktop match
  reached DEFEAT / Stronghold destroyed, duration 1:52, 25 deployments,
  1 Evolution. This validates the audit build, not future A-1 changes.
- [Preliminary diagonal scene](media/a1/preliminary-diagonal.jpg): live
  scene with fixed-yaw troops on different lanes. It is not yet a labeled
  causal contact sheet. Both committed JPEGs are below 300 KB.
- `preliminary-autoplay.json` and `preliminary-targeted.json`: filtered
  actual console evidence. The targeted match bought Rampart and Sawbot
  through the UI and observed their hits. These files are preliminary.
- `baseline-form-triangles.txt`: actual startup counts, suitable for the
  form audit; this is not the performance baseline of step 2.

**Do not use the preliminary F4 distances as accepted measurements.**
The initial probe queued only unit AttackStart events and popped on Hit.
It did not consume Heal events or queue Stronghold attacks. This could
misattribute hits (the logs even contain healing Bloomkeeper entries as
attacker). The saved patch improves the probe by queuing all AttackStart
sources, consuming Hit and Heal, expiring entries after 20 ticks and
checking minimum windup age. It compiled and packaged, but was **not run**
after this correction because the owner requested the handoff. Validate
its FIFO association, canceled windups and batched-view timing before
accepting its distances. The exact attacker cannot be reconstructed for
all unmatched/status/splash events; do not guess one by proximity.

Apply the saved probe only temporarily if useful:

```bash
cd /home/iiii/PESSOAL-PROJETOS-ALEXANDRE/tbg-a1
git apply docs/media/a1/temporary-audit-probe.patch
cd crates/tbg-client
trunk build --release
```

Remove it before the step-1 commit; rebuild the pristine client for step 2.
Do not commit the probe as production animation/targeting code. There are
no golden/contract test edits or competitive source edits.

## Next actions, in order

1. Finish step 1: corrected live F4 measurements for all five melee kinds,
   duplicate-shot evidence, 55-form audit, entity/triangle audit and an
   unambiguous diagonal screenshot. Fill the report and commit the audit.
2. Step 2: standalone old-behavior aim lab and baseline **before changing
   behavior**. 10/40/100/200 per side (`stress=20/80/200/400`), plus practice;
   60 s each, three runs, median; same browser/GPU. Add only the minimum
   diagnostic support needed to expose entity/assets/memory/frame metrics.
3. Step 3 ART-DIRECTION §11; step 4 pure aim/FIFO math and contract tests.
4. Steps 5–6 Archer/Shieldbearer prototypes and directed projectiles/melee.
   Read rendered target Transforms via the entity map; never move roots.
5. Step 7 gate before all-Faction art. Then continue steps 8–14 as specified.
   No prototype, native roster replacement, Stronghold models or tier kit
   have been implemented yet.

Owner decisions: O-A1-1 look-only/live Classic; O-A1-2 default supersession
on successful completion, not now; O-A1-3 no protocol changes; O-A1-4 owner
look sign-off pending and required before merge; O-A1-5 no lane convergence.
The phase report remains partial. Do not mark the roadmap implemented or
supersede the old phases until the work actually replaces them.
