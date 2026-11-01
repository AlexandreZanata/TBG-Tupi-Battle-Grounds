# Ogre Archer reload follow-up — 2026-11-01

Owner request: improve SiegeWalker's reload because the supporting arm crossed
inside the torso. Keep simulation stats, shot cadence and Release at 0.18 s.

The shoulder gun stays in the carrying hand. Its supporting grip moves forward,
and the old chest-side crank is replaced by a short external charging lever.
The left hand transfers outward from its support, grips the lever, pulls it,
pauses, returns it and settles back on the forward grip. The elbow pole points
forward/outward; partial support blends use quaternion interpolation. The lever
uses the existing Nock accessory slot parented to the gun, so its visible slide
follows the hand instead of staying frozen. All five forms use the same path.

Work-front validation: affected-package formatting and `git diff --check`
passed. Shared-lock `cargo check --offline --locked -p tbg-client --bin
tbg-client` passed in 5.02 s (`ogre-reload-check.log`, two Cargo jobs, existing
root target cache). No dependencies, protocol or deterministic rules changed.

Consolidation queued: existing `the_bombard_points_along_its_shells_flight`
regression, a native debug build, and an authentic SiegeWalker firing/reload
recording. No full suite, release/package build or gallery matrix is requested.
The owner-authorized delivery remains commit, push, then native Linux reinstall.

## Consolidated result

Runtime commit: `bc8ef321fb1bfda4852ccf7d7e927c1ae945d5a1`.
`cargo test --offline --locked -p tbg-client --bin tbg-client
the_bombard_points_along_its_shells_flight -- --test-threads=1`: 1 passed
(aim directions across all five forms). The native debug build passed in
19.61 s with the embedded runtime commit; same nonblocking host lock and
two-job limit. No full suite or additional platform build ran.

Two real 12-second training engagements were recorded with the ordinary core
and normal Animator: Base/seat A and A2/seat B, initial distance5500, no allied
frontline. The recordings observed 9 and 8 total real attack starts respectively
(subject plus opponent); these counts are not a claim about subject cadence.
Sampled reload frames at video PTS1.60,1.95,2.20 were inspected for both. The
forearm stays in front of the torso, with an outward elbow and a small visible
charging-handle stroke. The A2 armor/large barrel variant retains that clearance
in the inspected frames. These are bounded native observations, not an
exhaustive collision or performance guarantee.

The Shop portrait was regenerated from the actual model; original and asset
hashes are in P71-PORTRAITS.json. The local evidence index below preserves
recording, build and review-frame hashes. Bulk video/private data stay ignored.
WASM runtime/browser appearance, a full all-form temporal matrix, Low quality
and full-suite checks remain deferred to future scoped consolidation.

Delivery: push the follow-up commits, then install the verified executable and
current portrait assets as `0.1.0-native.2-p71-bc8ef32`, preserving previous
installations and existing user data. The binary source is bc8ef32; subsequent
commits only update the portrait and this report.
