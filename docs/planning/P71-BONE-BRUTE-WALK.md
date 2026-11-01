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
