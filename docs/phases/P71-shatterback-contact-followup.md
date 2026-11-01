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
