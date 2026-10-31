# System updates — owner request 2026-10-31

Base: GitHub `main`, `9008f11`. Work branch: `phase/system-champions-crowd-shop`.
The original dirty native-client checkout is preserved; implementation is in
an isolated worktree. P55 kits are already on this base (owner directs main).

## Scope and checkpoints

Implement the four supplied briefs: champion changes, client crowd,
performance investigations and Shop Reserve/Reroll. New balance values are
**Proposed**. Crowd Frontage/Ghost mechanics and performance phase 8 remain
outside the authorized implementation checkpoints. CANON Shop stays disabled;
practice/lab preview permits owner testing. Blob shadows require owner look
approval before adoption. No production deployment or merge is requested.

## Crowd

Started with the supplied pure solver verbatim: all five reference tests
passed. English comments, retained sort scratch and two boundary tests were
then added. ECS writes display transforms after snapshot sync and before aim;
projectile sockets and targets already read rendered transforms. `?crowd=0`
restores the three original lanes. No core/server changes belong to this step.
Carried units are a known limitation: main's `unit_view` does not pack carrier
state; no phantom protocol bit is invented. Dying units are nonblocking.

## Verification and measurements

Baseline core tests: `/tmp/tbg-baseline-tests.log` (record final counts below).
Reference crowd tests: 5 passed; extended crowd tests: 7 passed.
Full build, browser evidence, lab comparisons and outstanding checks follow.
