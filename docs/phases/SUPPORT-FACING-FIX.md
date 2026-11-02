# Support friendly-target facing regression

**Delivered:** Linux preview **0.1.2-cc.2**, installed code source
`c1d3e204618ba5506ce0ada4caeca90a1c6e90db`. Six unique focused tests
passed, including 200 support/form/seat/frame-rate cases. Actual native
rear-patient witnesses passed under incoming damage. Windows and desktop-web
motion comparison remain pending; shared implementation is not parity evidence.

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

## Native witness and installation

The opt-in Training capture sets a valid exact-team scenario through the
existing catalog: chosen support at 8000, Blood Furnace at 6000, enemy
Shieldbearer at 8600. The Furnace wounds itself through real rules; no HP is
injected. The enemy attacks the front support while its patient stays behind.
A post-animation witness requires actual received damage and at least 60
rear-patient frames with stable local foot yaw before accepting a screenshot.
It passed for Bloomkeeper and Nectar Nurse, then for the installed binary.
All builds/native checks used the shared
nonblocking host lock and at most two Cargo jobs/render CPUs.

Native witness/install were pending at the initial source checkpoint. Windows and
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
The installed Bloomkeeper witness also passed. Final-source evidence follows.

## Final installed evidence

Validation level: scoped consolidation of this client correction and the
owner-authorized Linux installation. Final native build passed in 4.84 s;
four final support tests passed in 0.42 s (10.12 s compile). The unchanged
paired-grip and projectile timing evidence from the first batch was reused.
Formatting, `git diff --check` and `python3 scripts/check-desktop-ui.py`
passed; the last check verifies routing, not cross-platform appearance.

Commands used, inside the shared nonblocking host lock with
`CARGO_BUILD_JOBS=2` and the existing target cache:

```bash
cargo test --offline --locked -p tbg-client --bin tbg-client support_ -- --test-threads=1
cargo build --offline --locked -p tbg-client --bin tbg-client
python3 scripts/install-champion-preview.py --binary <frozen-final-renderer>/tbg-client
```

The earlier five-test selection used filters `support_friendly_`,
`support_rear_branch_cut_`, `completion_two_hand_grips` and
`completion_projectiles`. Logs retain the actual executed test names and output.
Final build metadata used `TBG_SOURCE_COMMIT=c1d3e204618ba5506ce0ada4caeca90a1c6e90db`.
The native witness used `TBG_CAPTURE_PANEL=training`,
`TBG_CAPTURE_SUPPORT_REAR=1`, `TBG_CAPTURE_AFTER=12`,
`TBG_CAPTURE_SIZE=1600x1000`, Xvfb and two render CPUs.

Final installed Nectar Nurse witness output:

```text
Training journey passed: pause, exact single step, reset identity, stale action rejection and independent local rendering
Support rear-heal visual regression passed: kind=NectarNurse, rear_frames=60, attacked=true, max_foot_yaw=0
```

The installed directory is
`/home/iiii/.local/opt/tupi-battle-grounds-champion-preview/0.1.2-cc.2-c1d3e204618b`.
Launcher: `/home/iiii/.local/bin/tbg-champions`. Binary SHA-256:
`96679896f3b1497f0d37cfb18f917f6eba62f0e2297ee81683713b046ad6299a`.
All 361 installed assets matched the receipt. Protocol 33 and contract
`w33-r7-a9fa44b55cdea10b` are unchanged. The desktop preview was reopened;
the host process executable matched this installation. Later documentation
commits do not change the installed code revision.

Evidence directory, relative to the original checkout:
`.local/champion-concepts-2026-11-01/support-facing-fix/`.
`validation/regression-before.log` records the failing baseline;
`validation/regression-after.log` and `validation/regression-final.log`
record the passing batches. `final/linux-verification.json`,
`final/linux-install-receipt.json`, `final/validation/linux-build.log` and
`final/installed-rear-heal.log` identify the exact final installed witness.
The screenshot is an end-of-trial still; the frame-by-frame witness, rather
than that still alone, verifies stable orientation.

Deferred to cross-platform consolidation: repeat the same live rear-patient
Training scenario on Windows and desktop web from this source, with incoming
damage, repeated casts/hits, walking, target loss, both seats and all forms;
compare torso/feet/projectile timing against Linux. Those platforms were not
observed here. Full workspace tests, release packaging, multiplayer campaigns
and balance gates were outside this correction's scope and were not run.
