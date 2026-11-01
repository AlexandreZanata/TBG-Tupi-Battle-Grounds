# P71 execution review — in progress

Owner authorized implementation, local commits, GitHub push, then Linux native
installation on 2026-11-01. This also applies to subsequent requested targeted
adjustments: commit, push and reinstall each completed adjustment. No automatic
merge, force push, public deployment or full-suite validation is authorized.

Execution branch: `phase/p71-combat-visual-polish`, base `6d5d2a2`.
Owner worktrees, installed fallback and P70 original evidence are preserved.

## Blocked Trident contact refinement

> blocked: New Trident raised-load/downward-contact motion failed the existing
> grip regression twice. `consolidation-fixed.log` reports `form (0, 0): grip
> left the hand by 0.4311485`; after a post-blend IK repair and revised guard,
> `trident-refined.log` reports the same assertion at `0.06513715` (required
> `<0.05`). Suspected cause: unreachable/support-wrist targets or pose blends
> across the larger raised-guard rotation; exact clip diagnosis remains open.
> Following AGENTS.md, this step stopped. Motion/Animator/test changes were
> restored to the P70 baseline; no failed candidate thrust is installed.
> Trident's body-marker removal and form-aware HP bar are independent.

Other steps continue. This report will record actual validation, renderer
coverage, remaining acceptance, GitHub reference and final installed version.

## Resumed consolidation (2026-11-01)

The interrupted working tree contained only the final tower-collapse and
debris-surface changes and removal of the obsolete tabard helper. They are
retained. `collapse-check.log`: `cargo check --offline --locked -p tbg-client
--bin tbg-client` passed in 3.13 s under the shared validation lock (2 jobs,
existing native cache). Collapse now eases over 0.65 s; debris originates
on the visible exterior. These are presentation changes only.

Owner follow-ups queued for execution: stable Bloomkeeper support facing;
Farwatch Stronghold targeting; separate top-center Evolution/Fortification
choices with retained focus and outside dismissal; independent Tick pistol
and brother-specific Evolution branches; larger humanoid-bee Hornet with a
venom pistol connected to its abdomen. Gameplay changes require Proposed
canon entries and targeted authoritative regression evidence.

### Support facing and Farwatch follow-up

Bloomkeeper now twists the upper body for healing rather than repeatedly
turning the locomotion Root. A regression covers rear targets on either side
of the ±180-degree boundary. Farwatch uses ordinary Guard/nearest-target
selection and can attack an unguarded Stronghold within its unchanged range.
This owner rule is marked Proposed in §C.2; revision 4 records the changed
ruleset source. Native `followup-check.log` passed in 1.79 s. Focused tests
are queued for the final consolidation; no broad balance gate ran.

### Evolution and Fortification focus

Linux keeps the two information cards at their original anchors. Purchase
choices occupy a shared 390 px wide, minimum 106 px high card at y=134,
below the timer. Arena introduction uses the same slot and is suppressed
while a purchase card is open. Outside actions dismiss focus; purchases
retain it until selection changes, the unit disappears, or the user closes
it. The non-Linux Evolution card rebuilds only when identity/form/language
changes, so the next tier replaces stale choices without reopening. Browser
Fortification choices are also separated from the information card. Browser
arena banner now dismisses on outside pointer presses.

### Independent Tick pistol

Authoritative special timer now targets troops at medium range independently
of the club; its single-target pistol replaces the old area bomb/Slow. Branch
A remains Rattle HP/club; B improves only Tick cadence/damage. Static branch A
damage is explicitly excluded from Tick scaling. Three new core regressions
passed (both seats, independent movement/basic windup/counter, no collateral,
branch ownership). Two Farwatch targeting regressions passed, including Guard.
`followup-targeted.log`: 3 core + 2 Farwatch + 6 P71 client tests passed.
The client check initially found E0624 on pose sampling visibility; it was
corrected before the successful test build. D-33 balance gate and broad
multiplayer validation remain deferred; the values are Proposed playtest data.
