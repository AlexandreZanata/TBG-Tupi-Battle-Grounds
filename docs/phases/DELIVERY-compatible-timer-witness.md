# Coordinated release and compatible timer-only witness

The owner requested complete consolidation and a second real main delivery
with no manual apply command. Combined protocol 38, ruleset 15, contract
w38-r15-cc459351581b7763 and schema 10 remain unchanged.

PR #48 merged normally as f1ee218564762147df179b0908c0d26316543964
after every required lane and both native factories passed at a035282.
The main push also passed every lane. The tagged colossus.1 publication
correctly required its own complete validation because its promotion check
ran before the main suite completed.

> blocked: immutable colossus.1 publication. Tag test job 125452855873 and
> compatible PR test job 125445829513 found the same P73 history-journal
> fixture assumption: FixtureA must win on Seat A. SIDE-01 now randomly
> assigns that connection to A or B. Do not move the tag, overwrite assets,
> retry for a lucky side or weaken the winner assertion.

PR #53 advances the package to colossus.2, captures both assigned MatchStart
seats, checks the non-surrendering connection wins and both terminal outcomes
are equal despite the deliberately failed history journal. The empty private
history and Delayed status checks remain. A scoped audit found no other
random-side-dependent hardcoded winner assertion in server integration tests.
This changes only the fixture, not authority, sides or combat.

The backend/client/static comments already present in this branch produce
one real same-source candidate per family. Colossus.2 is now the first
attended coordinated activation candidate. Verify complete backend/web/static/
native artifacts before switching the official host; retain its healthy old
generation until existing matches and work drain. Controllers installed from
merged PR #48 are unchanged by this fixture correction. Timers stay paused
until the first complete activation has been observed.

After that activation, a separate colossus.3 compatible witness will keep
this exact wire/ruleset/schema and trigger all four families. After its merge,
do not run an apply CLI or start/restart any deployment controller or timer.
Observe natural firings, receipts, public health/admission, entry digests and
both native formats. Two anonymous private-room guests may witness promotion;
never certify Ranked or rendered platform parity from that probe.

Pre-publication Linux inspection verified all 395 files in the PR #48
merge-ref package 7fe1f3fd176e0335b486a92a232f4164c09f63fa. Four isolated
1600x1000 captures passed with boot_ready=true and failed_images=0: Colossus,
Volthar and Astreon five-form idle/walk/attack boards, plus real Forest
Practice with both Auxiliary cards. Initial Xvfb inherited WAYLAND_DISPLAY;
unsetting it corrected the test environment, without changing game code.
Screenshots remain in /data/tbg-consolidation-evidence/linux-pr48-a035/runtime-x11.
This is pre-publication Linux evidence, not an official main install or parity
proof. Official activation, timer-only delivery and installation remain pending.

Corrected local validation passed under the nonblocking shared host lock with
CARGO_BUILD_JOBS=2 and the existing /data/tbg-ab-target cache:
`cargo test --locked -p tbg-server --test player_challenges -- --test-threads=1`.
Both real-client scenarios passed (2 passed, 0 failed, 23.31 s); compilation
completed in 3m02s. Formatting and git diff --check passed. No unrelated local
suite, cold Bevy build or gate threshold change was used.
