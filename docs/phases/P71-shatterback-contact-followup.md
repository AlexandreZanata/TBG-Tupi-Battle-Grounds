# P71 Shatterback plate and contact follow-up — 2026-11-01

## Owner request and scope

Remove the yellow-looking stripe above Evolution1 Shatterback plates and keep
its head from entering adjacent champions by including its body in space blocking.
Read AGENTS.md, VALIDATION.md, ART-DIRECTION's rig/motion requirements,
GAME-DESIGN's D-57/P71 cosmetic occupancy clarification and P71 S1e. The current
native baseline is d4436e0 with the three new arenas. Preserve those arenas and
all unrelated champion changes. Use the standing commit/push/native-install workflow.

The authored orange top stripe belongs to B1/B2 plate meshes. Remove it only
from B1, retaining the B2 mark and the other forms. Reuse cached figure envelopes
and the existing client crowd solver. For Shatterback, sample Idle/Walk/melee
body motion, include the head and beak, project the envelope at current yaw and
reserve the existing lunge. The flesh envelope stays stable after plates detach.
No core rule, combat position, range, stat, plate HP/count or protocol change.

## Validation and delivery

Work-front formatting/diff inspection, followed by one targeted consolidation
of the Shatterback contact regression and existing crowd tests. Use the shared
host lock and at most two Cargo jobs with the existing target cache. Native
playtest build is authorized by the standing owner delivery rule; no release
packaging, full suite, balance gate or broad roster gallery. Capture the affected
plate forms and ordinary close contact on both seats with the genuine renderer.
Browser runtime, dense multi-unit saturation and owner look/fun remain separate
acceptance; no exhaustive collision guarantee is implied by finite samples.

Results and installed-source identity will be appended before completion.

### Owner-approved lock exception

The initial nonblocking targeted check exited75 because the shared validation
slot was busy. On2026-11-01 the owner explicitly authorized waiting in this and
future requested character adjustments, without interrupting other processes,
so the commit/push/native-reinstall workflow can finish. This is a standing
owner exception to AGENTS.md's lock-busy deferral for these requested adjustments;
it does not authorize extra build jobs, bypasses, cache replacement, unrelated
checks, full suites, release packaging or terminating another work front.

### Targeted consolidation results

- `cargo fmt -p tbg-client` and `git diff --check`: passed.
- `cargo test --offline --locked -p tbg-client --bin tbg-client shatterback -- --test-threads=1`: 1 passed (6.68s compilation,0.07s test).
- `cargo test --offline --locked -p tbg-client --bin tbg-client crowd::tests -- --test-threads=1`: 7 passed (0.01s tests).

Both test commands ran sequentially under the shared lock, with CARGO_BUILD_JOBS=2,
the existing ROOT/target cache and native dependency pkg-config path. The only
warning was the existing unused Layout::clear method. The new regression builds
real Shatterback head meshes in all five forms, samples Idle/Walk/melee at four
mirrored facing angles, reserves the existing lunge and checks settled contact
with allied/opposing ordinary bodies. It also keeps the existing displacement
bounds. Core sources, stats, plate durability and protocol30 are unchanged.

The original five-form Idle sheet from frozen installed source d4436e0 is kept
locally under `.local/shatterback-contact-evidence/before-idle/`, together with
its build receipt and renderer log. Visual inspection confirms the orange/gold
stripe on B1 plates; the B2 stripe is intentionally retained. Native after-captures
and installation follow the source commit. No full suite, release packaging,
browser runtime or dense-crowd guarantee is claimed.

### Side-facing broad-phase refinement

Review found that the existing pair solver uses at least a circular longitudinal
reach. A turtle turned across the lane can project wider than it is long, so its
new footprint must keep half_length at least radius for the unchanged broad-phase
search to see the same pairs. The clamp is local to the new envelope method;
existing crowd parameters and other champions' footprints are unchanged.

The real-head regression now samples six mirrored yaw angles (±25, ±65, ±90),
all five forms, both allied/opposing contact and Idle/Walk/melee with lunge.
It asserts that broad-phase reach invariant too. A focused rerun after this
refinement passed1 test,0 failed (2.62s compile,0.10s test), under the same lock
and two-job limit. The seven generic crowd regressions remain valid unchanged.
The final installation binary is rebuilt after this source correction; the prior
10.04s native build was an intermediate candidate, not the delivered artifact.

### Final renderer validation and Linux delivery — completed

Runtime source: `ce0e4060446a03eeda0549fb1e3df21513363f10`. The final debug
native build passed in3.54s under the shared lock with CARGO_BUILD_JOBS=2 and
the existing cache. Its immutable binary/receipt are local in
`.local/shatterback-contact-evidence/candidate-ce0e406/`.

A matching after-Idle sheet was captured with the same1920x1080 real renderer
configuration as the baseline. Visual comparison confirms B1's four plate-top
stripes are removed and the B2 stripes remain. The comparison crop is local at
`.local/shatterback-contact-evidence/b1-before-after.png`; both original sheets,
sidecars and logs are retained. No Shop portrait regeneration was needed because
Base artwork did not change.

Two real B1 laboratory engagements used seed70, initial distance1500, no added
frontline and12 seconds of ordinary simulation. Each was recorded by the existing
`scripts/p70/record_trial.py` against the frozen source and receipt at1920x1080,
30 recording frames/s. These are not playback-speed or rendering-FPS claims.

- Seat A:241 active ticks (12.05 simulated seconds),10 subject AttackStart events;
  Shatterback's plates absorbed400 HP and observed plate counts covered4,3,2,1,0.
- Seat B:240 active ticks (12.00 simulated seconds),10 subject AttackStart events;
  Shatterback's plates absorbed400 HP and observed plate counts covered4,3,2,1,0.

Both recordings completed successfully with genuine AttackStart/Hit engagement.
Frames at3,7 and11 recording seconds on both seats were inspected, including
intact and depleted shell states; they show separated head/body contact without
shrinking the flesh footprint after plate loss. Original videos, event journals,
metadata, metrics and hashes are retained locally in `contact-seat-a/` and
`contact-seat-b/` under the same evidence directory. The cropped contact sheet
is `contact-samples.png`. Finite sampled frames and the targeted geometry
regression do not prove every crowded battle or every animation transition.

The final runtime commit was pushed to `origin/phase/p71-shatterback-contact`
before installation. `python3 /tmp/tbg-install-shatterback.py` succeeded and
verified the binary and all362 assets. Installed executable:
`/home/iiii/.local/opt/tupi-battle-grounds/0.1.0-native.2-shatterback-ce0e406/tbg-client`.
The TBG launcher and desktop entry now select it; `--version` confirms source
ce0e406 and unchanged protocol30. The previous arena build and user data remain
intact. Installation receipt: `.local/shatterback-contact-evidence/installed-linux.json`.

Binary SHA-256:
`091f607e4d74c2258cb9a34a8ed29c9b4e1e48a35d2e20e7da7d3cfbb50e2af4`.

Deferred: owner look/fun approval; desktop browser runtime and a scoped WASM
check (`cargo check --offline --locked -p tbg-client --bin tbg-client --target wasm32-unknown-unknown`);
dense multi-unit saturation and full suite. Native was the requested installed
target; no additional platform build, public deployment or balance gate ran.
