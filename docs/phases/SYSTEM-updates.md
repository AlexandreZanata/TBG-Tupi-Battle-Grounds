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
The champion presentation follow-up packs the already allocated carrier bit
14, so carried passengers and dying units are nonblocking. Stress scenes
also use neutral anchors; `?crowd=0` retains the original three lanes.
The solver bounds take precedence in very dense stacks: separation is not
a guarantee of disjoint bodies for an unlimited army in a bounded area.

## Verification and measurements

Validation level at delivery: **work front**, following the owner's later
2026-10-31 addendum and the updated primary checkout's AGENTS.md and
`docs/VALIDATION.md`. Their changes belong to the active native front and
are preserved there, rather than copied into this branch. This policy
supersedes the broad command lists in the attached briefs.

Reference crowd tests: 5 passed; extended crowd tests: 7 passed. Before the
policy update, baseline core tests passed 135 tests; the revised core with
lab enabled passed 215 tests, including the nine Siphon scenarios, summoner
and Shop tests. Workspace Clippy passed at its recorded intermediate tree.
These are historical results, not a claim that later edits were retested.
The final movement assertion, detached-plate budget assertion and reservation
journal regression are queued for consolidation. See `docs/media/system-updates/verification.txt`.

After the addendum, only hot-cache focused checks were started, with
`flock -n -E 75 /tmp/tbg-validation-$(id -u).lock`,
`CARGO_BUILD_JOBS=2`, and the existing caches:

- `cargo check -p tbg-core --lib`: passed, 5.65 s.
- `cargo check -p tbg-balance --lib`: passed, 5.75 s.
- `cargo check -p tbg-client --bin tbg-client --target wasm32-unknown-unknown`: passed, 5.30 s (existing cache with nonblocking guards).
- Affected-package formatting, both changed JavaScript files' syntax, catalog
  JSON/key parity and `git diff --check`: passed.

Older release/workspace jobs already active when the policy changed were
left undisturbed. No new full suite, release build, lab campaign or gallery
campaign was started under the new policy. Deferred checks are not passes.

## Shop preview (D-69)

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

> blocked: Browser build initially reused the baseline worktree's generated
> catalog: `error[E0599]: no variant, associated function, or constant named
> RejectionStaleOffer found for enum Msg` (also ReserveFull, AlreadyHeld,
> NoReserve and StalePrice). The second attempt ran before the build-script
> correction due to a wrong shell working directory. The failed build step
> was stopped; an independent build-script fix now tracks absolute catalog
> paths before corrected validation resumes. No catalog entries were removed.

## Champion presentation

Bloodweaver fires a short blood bolt that lands at the normal Hit; the
separate stolen-blood stream travels from the enemy to the healed ally.
Old Finn uses a fish-specific 80 ms flight after the existing 0.22 s release,
with a minimum 2.0-world-unit apex and spinning water trail. No visual Hit
delay or global WINDUP change was introduced; the short arc still needs
owner look review.

Longaxe has a 2.40-unit haft (was 1.45), a larger head and a ground cut,
with a fan on branch B. Leechroot has a broader body, sap veins and up to
seven visible Essence sprouts (a saturated display, not a simulation cap).
Smuggler carries a crate in every form, aims/fires with one hand and lowers
cargo using the free arm without replacing its walking clip. Crate drops
now announce the existing Spawn event so the cue can actually run; their
previous lack of that event was an additional integration finding.
Blood Furnace's recovery is shorter while its strike stays at 0.30 s.
Nester's glowing eggs follow quantized authoritative timer progress and its
hatch gesture follows the timer reset. Shatterback is a low turtle with a
beak, thick feet, soft inner shell and four separately cached plate meshes;
plate loss hides only that plate and spawns a falling copy. The triangle
budget test now includes those separate meshes.

The existing UnitView.fx layout carries plate count (8–10), saturated growth
or Nester maturity (11–13) and carried state (14). UnitView/protocol shapes
are unchanged. No gameplay Ghost or Frontage was added.

Interpretations I-1–I-8 are adopted: lowest exact HP fraction with distance/id
ties; unit hits only; living conventional recipients including self; Finn
keeps damage 8; Cargo timers always advance; revolver can hit Strongholds;
Longaxe matches Rootcaller's base range; faster Furnace attacks also burn
its HP faster. Shared Blood uses up to two eligible recipients.

## Balance evidence and limits

Seven reference rows completed on the preserved main binary, 2,000 paired
seeds each, starting at 9000: Longaxe 53.1%, Leechroot 46.4%, Blood Furnace
52.9%, Nester 53.8%, Old Finn 49.7%, Smuggler 54.3%, Bloodweaver 47.2%.
These are favor-policy win rates, not unit ownership win rates, and show
that not every requested champion was weak in that particular baseline.
Raw run/roster files and binary provenance are under `docs/media/system-updates/lab/`.
The original 20,000-pair attempt ended with process status 143 and produced
no completed result; it is not evidence. After-change rows, ownership/share
analysis, seat bias and the final 20,000-pair comparisons remain pending.
No balance gate or threshold was weakened; the numbers remain Proposed.

## Performance

`?bench=30|100|200&perf=1` selects a fixed roster/form cycle and stationary
camera, with a 3-second warmup and 20-second browser sample. Metrics expose
frame percentiles/max, FPS, entities, meshes, materials, actual WebGL draw
calls and triangle submissions, viewport and DPR. The draw/triangle counts
include shadow passes. Browser scene measurements and CPU/GPU attribution
are deferred; no integrated-GPU target is claimed.

A `release-server` candidate (opt-level 3, fat LTO, unwind) was built before
the policy change and compared with release on mixed 30/100-unit fixtures,
600 ticks, seeds 1/7/42, CPU 2. It showed no demonstrated >=5% gain in this
noisy, non-interleaved exploratory run, so the profile was removed and deploy
remains unchanged. Raw CSV/metadata: `docs/performance/system-updates/`.
This is not proof that size optimization is universally faster.

Deferred behind an explicitly scheduled baseline/measurement round: server
capacity at 1,000/2,000 matches; native-CPU/allocator variants; WASM size and
CI budget calibration; shader prewarming; dynamic resolution/quality;
blob-shadow comparison and owner approval; VFX pooling, animation LOD,
HUD update profiling, and WASM profile/wasm-opt variants. The brief forbids
retaining optimizations without measured gain, so no unmeasured optimization
was adopted. Phase-8 architecture experiments remain outside scope.

## Consolidation backlog (do not automatically run)

All compiling commands below require the nonblocking shared lock and
CARGO_BUILD_JOBS=2; tests additionally use --test-threads=1. A busy lock means
defer, with no waiting or automatic retry. Reuse the current cache.

- `cargo test -p tbg-core --features lab --test champion_siphon --test summoners --test shop_reserve_reroll --test contract_determinism --test contract_lab --test contract_roster --test contract_protocol --test contract_public_view -- --test-threads=1`
- `cargo test -p tbg-balance shop_metrics::tests --lib -- --test-threads=1`
- In one sequential client batch: `cargo test -p tbg-client --bin tbg-client art::kits::tests -- --test-threads=1`, `cargo test -p tbg-client --bin tbg-client crowd::tests -- --test-threads=1`, and `cargo test -p tbg-client --bin tbg-client fish_tests -- --test-threads=1`. This includes total Shatterback triangles.
- Required desktop observations: Shop pending/ack races and rejections,
  reserve expiry/release/purchase, turtle plate loss, fish timing, cargo
  placement, Essence and hatch maturity, and player/spectator crowd alignment.
- Existing v17 online intentions must remain unavailable and Shop controls
  hidden. Feature-online reconnect/ack scenarios wait for owner checkpoint 9,
  the protocol bump and removal of serde(skip); they cannot be claimed now.
- Heavy acceptance needs a separate explicit request: release/size builds,
  full gallery screenshots at 1920x1080 and 1280x800, hardware FPS/profile,
  paired balance comparisons/gates and constrained capacity runs.
- The full workspace suite is not authorized by this delivery request.

Owner review is still needed for look/fun, balance acceptance and enabling
Shop online. This branch is delivered as a draft PR, not release-ready.


### Legacy job outcomes (not retried)

The release build reached `Finished release profile` but its artifact request
then reported `error: could not compile tbg-client (bin tbg-client) due to 15
previous errors` and `bad status returned from cargo artifacts request`.
The log did not retain those 15 primary diagnostics. The later focused WASM
check passed after the cache fix, but this is not a successful release build.

The old workspace test job failed with `error: couldn't find file
crates/tbg-core/examples/update_probe.rs` after the temporary probe was
removed while Cargo still had it in its target graph. The probe has been
restored as a small maintained rule-update evidence tool. This was an agent
cleanup mistake; the unrelated active front's files/builds were untouched.
The full suite was not restarted under the new policy. Final focused tests
and geometry assertions remain in the consolidation backlog.

## PR #21 consolidation (2026-10-31)

Owner requested merge to main and a TBG-only production deployment. The
integration includes the already deployed PR #20 head `3287410`, preserving
protocol 20, carrier identities/boarding, unit shields, seven Factions,
arenas and spectator/ranking fixes. Independent native work is excluded.
Decision IDs from this front are renumbered D-67/D-68/D-69 to preserve the
existing deployed D-60 through D-66 references. Historical v17 evidence
above describes the original front, not the integrated release.

The owner chose Smuggler range **6.0**, replacing this front's 6.4 proposal
to obey the existing Stronghold range invariant. No invariant, test gate or
CI step is weakened. Deterministic hashes must be recorded again because
the integrated serialized state includes both the roster slots and the
preview-only Shop state. Shop Reserve/Reroll remains Practice-only.

Validation level: targeted consolidation. The first compilation attempt
returned exit 75 (shared validation slot busy), so it was deferred without
retry. The owner then explicitly authorized one new attempt after the
adjustments, under the same lock and at most two Cargo jobs.

Deferred: full workspace suites and Clippy, C7/C8, repeated balance gates,
paired 20,000-seed champion measurements, complete gallery and mobile
device tests. These require the explicit heavy/full validation request or
remain owner look/fun acceptance; no pass is claimed.

Consolidation evidence before the deploy build:

- `cargo check -p tbg-core --lib`: passed (1.92 s, reused cache).
- `cargo run -p tbg-core --features lab --example update_probe`: five seeds
  completed; measured traces are in `docs/media/system-updates/integration-goldens.txt`.
- `cargo test -p tbg-core --features lab --test champion_siphon --test summoners --test shop_reserve_reroll --test contract_determinism --test contract_lab --test contract_roster --test contract_protocol --test contract_public_view --test contract_cargo_crab -- --test-threads=1`: 53 distinct final scenarios passed. The stale Smuggler 6.4 expectation was corrected to the owner-selected Stronghold range; only that target and the added public-view regression were rerun.
- `cargo test -p tbg-balance --test contract c5_every_tunable_number_is_registered -- --test-threads=1`: passed, fixing the original CI failure without changing the invariant.
- `cargo test -p tbg-balance --lib shop_metrics::tests -- --test-threads=1`: passed.
- Client filters `crowd::tests` and `art::kits::`: 18 distinct scenarios
  passed after correcting merge-only struct initializers and keeping the
  Leechroot B branch distinct from its newly veined base. The mesh-budget
  regression passed with the added wrist nodes; no useful test was removed.
- `node --test crates/tbg-client/tests/lobby.test.js crates/tbg-client/tests/live.test.js crates/tbg-client/tests/portraits.test.js`: 3 files passed.
- Server `live_api` / `spectate` tests require local TCP ports. The sandbox
  attempt failed with `Operation not permitted`; an authorized unsandboxed
  rerun under the same shared lock is in progress. Release builds, browser
  observation and final remote CI are still pending at this commit.
- `cargo fmt -p tbg-core -p tbg-client -p tbg-server -p tbg-balance` and
  `git diff --check`: passed. All compiling commands used the shared
  nonblocking lock, `CARGO_BUILD_JOBS=2` and serial test execution.
