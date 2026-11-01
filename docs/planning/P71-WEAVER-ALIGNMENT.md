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
