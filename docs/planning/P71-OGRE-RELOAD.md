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
