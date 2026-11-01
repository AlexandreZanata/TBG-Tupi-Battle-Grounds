# Bone Brute cleaver walk clearance — 2026-11-01

Owner request: keep Bone Brute's cleaver from entering the floor while walking.
The inherited ±28-degree upper-arm swing compounded with the hunched trunk
and wrist angle, lowering the blade at the backward part of the stride.

Walking now carries the cleaver forward of the knee with a bent elbow and
lifted point, retaining a smaller weight-bearing swing. Butcher B1 receives
the same treatment on its second cleaver. Shield poses and the Reaper B2
scythe guard remain unchanged. No weapon geometry, other clips, movement
speed, damage or attack event timing changes. Idle portraits need no refresh.

Formatting and diff inspection are the work-front checks. The authorized
native reinstall will consolidate this fix with the already committed Pyre
ground-line removal, using the existing cache, shared nonblocking validation
lock and two Cargo jobs. No new tests/full suite for this cosmetic pose edit.
Native build and bounded renderer observations are pending below.

## Actual validation and delivery status

Formatting and `git diff --check` passed. The consolidated native build exited75:
**deferred: shared validation slot busy**. No automatic retry, polling, waiting,
lock bypass or alternate target cache was attempted. Compilation, native
recordings and reinstall are therefore pending, not passing. Existing installed
version `bc8ef32` is preserved; the Pyre ground-line removal is still pending
installation with this change.

An independent noncompiling numerical inspection sampled257 phases of the
authored walk, with both the native planted-hip bob and preview hip offsets.
Conservative bounds of the cleaver primitives were transcribed from the model,
and transformed through the authored arm/torso chain. It reproduced negative
clearance before the change and positive clearance afterwards for Base/A1/A2
and both B1 cleavers (minimum lower bound0.338 world units). This supports
the authored pose choice but is not compiled code or renderer evidence.
Local result: `.local/p71-evidence/bone-brute-clearance.json`, SHA-256
`5542cc0bf975ea5e551f1355b3e5c1177b0a513818f3f23607eab6d586a56456`. No new Cargo tests or full suite ran.

The next authorized native delivery must build the current committed tree
under the standard shared lock, CARGO_BUILD_JOBS=2, existing root target and
native pkg-config path, then freeze the binary, inspect a bounded walk and
reinstall with executable/asset hash verification. The exact build wrapper
is recorded in P71-PYRE-LINE-REMOVAL.md. No simulation or other champion
animation source was modified by this follow-up.
