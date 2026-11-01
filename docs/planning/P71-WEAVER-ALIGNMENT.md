# Weaver alignment and connected head — 2026-11-01

Owner request: Weaver must walk straight, like the corrected Drillnest and
Bonehorn, and its head must connect to the body instead of floating.

Live Weaver figures now use the same lane-aligned ±90-degree base heading
as those units. Walking retains the alternating eight-leg gait and small
body roll, with no body/head yaw sway. Catalog camera angles are unchanged.

The head now inherits the Chest transform; its relative rest offset preserves
the original world rest position. A low chitin cephalothorax and overlapping
head collar bridge the abdomen/head gap and keep the joint covered while the
head casts or tilts. Eyes, nozzle identity and existing clip timing stay intact.
No simulation, movement speed, range, damage, protocol or dependency changes.

Work-front validation: affected-package formatting and `git diff --check`.
Native build and renderer inspection will use the shared nonblocking lock,
two Cargo jobs and existing cache. The next successful authorized Linux
reinstall must also include the pending Pyre and Bone Brute corrections.
No new unit tests or full-suite validation for this isolated visual change.

## Current validation and delivery status

Formatting and `git diff --check` passed. The native build exited75:
**deferred: shared validation slot busy**. No retry, lock polling, waiting,
cache bypass or interruption of the current validation was attempted. Native
renderer verification, the refreshed Weaver portrait and local installation
remain pending. No compile or renderer pass is claimed.

Independent analytic inspection confirmed that the changed parent/rest offsets
preserve the head rest position, and that selected points overlap the abdomen,
cephalothorax and head-collar ellipsoids through the authored small head-rotation
and translation envelope, including doubled cast shifts. This manually
transcribed numerical inspection does not verify faceted meshes or runtime
poses. Local result: `.local/p71-evidence/weaver-connection-inspection.json`,
SHA-256 `3958333ad6ba3f320ddc6c41cf30899d60dd3f1e6cd33c1f6a9932b5debe01ec`.

After three consecutive follow-ups encountered the busy shared validation slot,
the owner was asked whether to authorize waiting for the slot without stopping
its current holder. This is an explicit exception request to the AGENTS.md
no-wait/no-poll rule, not permission to bypass the lock or raise build jobs.
Until answered, native delivery remains pending and installed versionbc8ef32
is preserved. The source change is committed for GitHub delivery.

## Owner-authorized consolidated wait

The owner explicitly answered “Wait and complete installation” on2026-11-01.
For this consolidated native delivery only, waiting on the existing shared
lock is authorized, without stopping its current holder, bypassing the lock,
increasing Cargo jobs or changing target caches. This supersedes the pending
approval above. The original nonblocking policy remains the default for
unrelated future work. Build and installation outcomes follow below.
