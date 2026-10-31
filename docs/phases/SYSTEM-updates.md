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

## Shop preview (D-62)

Practice uses Ruleset::preview with two free reservations, three new-Shop
charges, Reroll base/step 600 sub-energy, no cap. CANON.shop remains None.
Online controls remain hidden and the v17 View bytes keep their old shape:
preview MatchView fields use serde(skip), intentionally removed only with
the owner-approved activation/version bump. New intention indices append.

Nine Shop integration scenarios passed, plus 10,000 kept-slot redraws per
mask and exact RNG equivalence for keep=0. The appended private state moved
hash bytes; all five champion-adjusted outcomes/end ticks remained identical
before refreezing (3261, 2266, 7649, 2785, 24689). Preview-but-unused also
matches CANON offers, RNG, players, units and final state hash directly.

The lab accepts reserve-bot/reroll-bot/shop-bot and corresponding favor/faction
policies; defaults remain the prior control policies. Five Shop knobs are
registered; shop.csv reports reservations/conversions/wait, Reroll Energy,
streak, expensive purchases and the 60-second frontline checkpoint.
HUD locks carry the visible kind; Reroll carries the displayed price.
Rust and the HTML controls both guard pending mutations; rejection clears
pending state. R is the Reroll shortcut. Phone controls remain unchanged.

Deviation: Shop fields stay local-only on the wire until activation instead
of silently changing v17 with no protocol bump. This is necessary to keep
main-client compatibility while honoring the explicit checkpoint.
