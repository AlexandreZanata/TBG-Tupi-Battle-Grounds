# Support friendly-target facing regression

Owner report: a healing support taking hits while its allies are behind it
keeps turning on the spot. This is a presentation defect, not permission to
change combat target selection, healing, damage, cooldowns or simulation rules.

## Cause and correction

Kit Idle turned the whole Root toward the engaged patient. Walk/Hit returned
it to the lane heading; Bloomkeeper's Cast already used a partial torso turn.
Alternating those clips repeatedly rotated the entire figure. Rear-facing
angle wrap could also switch the chosen shoulder on tiny crowd offsets.

The authoritative AttackStart now supplies the action target's ally relation.
All Support kinds casting at allies keep that same target for projectiles and
FX, while their combat poses share one eased torso/head look with the existing
65-degree Bloomkeeper limit. Whole-body target rotation is suppressed for
those poses. Rear look direction is latched across the rear angle boundary;
missing targets ease out. Hostile attacks, other Archetypes, authored Spawn,
Evolution, Death and Victory motions keep their normal presentation.
Legacy full-figure aiming is removed for friendly support casts as well.

The implementation is shared across Linux, Windows and desktop web. No
simulation/protocol/ruleset source changed. Preview version is 0.1.2-cc.2.

## Focused verification

The regression failed on the prior behavior: Bloomkeeper base, opposite side,
30 Hz, frame 19, local forward z = -0.22337 while the feet should keep lane
heading. The corrected five-test batch passed: 200 cases (ten conventional
Support kinds × five forms × two sides × 30/120 Hz), repeated Cast/Hit/Idle/
Walk transitions, unchanged authoritative target, rear-boundary jitter and
release after target loss, preserved life/non-support poses, paired grips and
projectile contact timing. Execution 0.50 s; compile 4.35 s. Existing unrelated
Layout::clear test-only warning remains.

Exact test filters: support_friendly_, support_rear_branch_cut_,
completion_two_hand_grips, completion_projectiles, with --test-threads=1.
Logs: .local/champion-concepts-2026-11-01/support-facing-fix/validation/.

## Native witness and install plan

The opt-in Training capture sets a valid exact-team scenario through the
existing catalog: chosen support at 8000, Blood Furnace at 6000, enemy
Shieldbearer at 8600. The Furnace wounds itself through real rules; no HP is
injected. The enemy attacks the front support while its patient stays behind.
A post-animation witness requires actual received damage and at least 60
rear-patient frames with stable local foot yaw before accepting a screenshot.
Run it for Bloomkeeper and Nectar Nurse, then install the committed binary
and verify its source/assets. All builds/native checks use the shared
nonblocking host lock and at most two Cargo jobs/render CPUs.

Native witness/install are pending at this source checkpoint. Windows and
browser visual/motion correspondence, unrelated suites and full competitive
balance remain pending; no full suite, release packaging or deployment is
included.

### Walking continuity follow-up

The shoulder latch also survives a zero aim-blend weight during walking
while the same patient still exists. A focused full-second Walk → Idle
regression crosses the rear angle boundary without switching shoulders.
Four support tests passed after this refinement (0.42 s); paired-grip and
projectile timing checks from the preceding batch remain unchanged. Six
unique focused scenarios passed across the two batches.

The first native witnesses passed for Bloomkeeper and Nectar Nurse: real
incoming damage, 60 rear-patient frames each, maximum local foot yaw 0.
The installed Bloomkeeper witness also passed; final-source installation
and its new source-stamped witness are being consolidated below.
