# GD-02 — Colossus arms participate in visual spacing

Owner request, 2026-11-02: neighboring champions must respect the Colossus arms.

The model's posed envelope already measured the arms and hands, but the live ECS
equipment list omitted Colossus. Share that list with measurement so all five
Colossus forms use their actual cached body/equipment extent during pushing.
No solver, smoothing, authoritative collision, unit stats, road width or queue
behavior changes. Other champions retain their previous selection and envelope.

Regressions sample actual upper-arm, forearm and hand mesh vertices in all five
forms across Idle/Walk/Attack, 33 clip samples, three target distances and five
yaws. A second scenario checks neighboring Archer separation with the original
solver for all five forms on both facings. Validation receipts follow below.

## Work-front validation and pending consolidation

Affected-package `cargo fmt -p tbg-client -- --check` and `git diff --check`
passed. Source review confirms the existing envelope sampler is unchanged and
live crowd selection gains only Colossus; the original solver is unchanged.

The following targeted consolidation attempt returned **exit 75**, shared
validation slot busy, before any compilation or test ran:

```sh
flock -n -E 75 /tmp/tbg-validation-1000.lock env CARGO_BUILD_JOBS=2 \
  CARGO_TARGET_DIR=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/target \
  cargo test --offline --locked -p tbg-client --bin tbg-client -- colossus --test-threads=1
```

No polling/retry, lock bypass or interference with the active build. The
disk-space preparation inside the same lock also did not run; no build output
was removed. The three Colossus cases (one existing art contract and two new
arm/neighbor regressions) are written but **not executed for GD-02**. Earlier
passing tests/captures are historical and do not validate this change.

Deferred to the next authorized consolidation: run that exact filter once
with the existing native dependency sysroot; build the committed Linux client
once under the same lock with two jobs and `TBG_SOURCE_COMMIT=<source SHA>`;
install with `scripts/install-champion-preview.py` and verify its executable
and 380 asset hashes. Review arm clearance in a normal crowd on Linux, then
compare Windows/desktop web. Do not repeat the eight-arena gallery or unrelated
core tests. No full suite, balance gate, release build or extra target build.

The installed Linux version is still source `e868927` (RESTORE-02/ARENA-CLEAR-01).
GD-02 is not installed or visually verified yet. Existing sessions remain intact.

## Later consolidation update — ARENA-REFINE-01

After the owner's subsequent scenery request, one new consolidation acquired the
shared slot for the combined changed fronts. All three Colossus tests passed,
including both GD-02 regressions. Compilation passed; this supersedes the earlier
not-executed test status. No Colossus source changed afterward. Linux reinstall
and runtime/Windows/desktop web observations remain pending. The scene test's
minimum-density failure and later busy-slot deferral are recorded in
[the scenery receipt](ARENA-SCENERY-REFINEMENT.md).

## CROWD-EXIT-01 refinement (2026-11-02)

To eliminate premature lateral displacement at the tower exit while preserving
arm and bazooka clearance:
- `overhead::height` measures `Colossus`'s upper-arm, forearm and hand vertices
  (`UpperArmL/R`, `ForearmL/R`, `HandL/R`) rather than its slammed obelisk
  (`Socket::WeaponR`), and scopes `reserves_equipment` to `Colossus`,
  `SiegeWalker` and `GustDrummer`.
- `crowd::Solver::solve` keeps `Params::half_width = 2.1` for the widened road
  boundary clamp while restoring `HOME_HALF_WIDTH = 1.1` for initial birth
  offsets and lateral regrouping.
- `crowd_ecs::live_body` applies compact walking radii while advancing without
  an active aim target and full directional envelopes when engaged.
- All 20 targeted client tests (`crowd`, `gd01_colossus`, `gd02_colossus`,
  `ogre_`, `shatterback`, `p71_body_envelopes`, `side01_`, and the new
  non-contact vs contact tower-exit regressions) passed. Windows and desktop
  web visual observation remain pending.



## GD-03 — Colossus body turn and obelisk-tip strike alignment (2026-11-02)

- `Animator::observe` keeps `engaged = true` whenever an attack action clip is
  active and `aim_at` is present, preventing walk-stop hysteresis or crowd nudges
  from suppressing aim weight during combat.
- `colossus::motion::pose` rotates `Joint::Root` toward `m.aim` during attack
  and hit clips, centers the obelisk laterally at impact (`Strike`), places both
  hands symmetrically on the shaft, and solves the strike pitch and longitudinal
  slide so the obelisk tip (`Vec3::new(0.0, 1.27, 0.0)`) aims and lands directly
  on the target across all five forms while preserving GD-02 arm/hand clearance.
- Verified by `gd03_colossus_turns_body_and_aligns_obelisk_tip_with_target_at_strike`
  alongside `gd01_colossus` and `gd02_colossus`. Windows and desktop web visual
  observations remain pending.
