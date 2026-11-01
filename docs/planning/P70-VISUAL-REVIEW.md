# P70 visual execution review

**Status: S0 pose baseline recorded; S1 candidate committed; S2 pilot candidates in progress. No art acceptance or completed phase is claimed.**
Owner request: 2026-10-31. Execution is local only on
`phase/p70-champion-visual-polish` in `.local/p70-champion-visual-polish`.

## Frozen scope and source

- Required content base: `55510c5363e3f1e555fa5133208a5faf20c5825b`.
- Planning parent: `60416ae` (the required base plus P70 planning files).
- Owner policy preservation: `3dc532b`; current root-worktree AGENTS.md and
  docs/VALIDATION.md were copied into this isolated branch. Owner edits in
  the original worktree remain untouched.
- The fixed [manifest](P70-VISUAL-MANIFEST.json) still contains exactly
  46 champions / 230 forms / 2,300 clip/form slots per source revision.
- [Evidence index](P70-VISUAL-EVIDENCE.json): source applicability resolves
  1,835 applicable pose slots and 465 inapplicable slots per revision.
  Inapplicability is explained from actual cue/targeting/kit paths. Specials
  remain separate event channels; generic Cast samples do not prove them.
- [Individual dossiers](P70-DOSSIERS.json) preserve every existing identity
  and proposed color/motion direction. They are preparation, not completed
  model passes or visual approvals.
- [Pilot source audit](P70-PILOT-SOURCE-AUDIT.md) records source-supported
  defects, remaining visual questions and exact proposed regression scope.

## S0 tooling

The existing frame gallery incorrectly fixed branch 0 and used the shared
clip duration. It now preserves the requested branch, includes actual
strike/release and terminal poses, and uses the selected kit and P63
Victory durations. `Animator::present` provides the existing event-free
preview path. No kit/model/pose/effect/combat content changed in this step.

`gallery/polish.rs` reuses 30 figures per champion to render the five forms
across up to six deduplicated sample times, sequentially covering all ten
clip IDs. Captures have fixed camera geometry, explicit front/profile/back/
three-quarter orientation, selectable team and nameless framing. Static
secondary attachments are deliberately sampled at rest equilibrium and
labeled as such; temporal spring/transition inspection remains required.

`capture/polish.rs` records renderer originals and per-sheet metadata,
refuses overwrite, waits for loading readiness and settled frames, and
advances only on the renderer's screenshot callback. The orchestration
script uses one process per champion/view, isolated preferences, bounded
timeouts, immutable original paths, source/binary provenance, PNG decoding,
hashes and resumable completed units. It does not touch Shop portraits.

Raw captures belong under ignored `.local/p70-evidence/`. No paintover or
generated image can substitute for renderer evidence. All 46 champions now
have ten original before sheets (460 originals), covering all five forms.
An additional 120 enemy-team sheets cover Archer, Bone Brute and Trident
from three-quarter, front, profile and back for S1. Their three hashed
manifests are linked from the evidence index. The renderer was Vulkan on
NVIDIA GeForce RTX 4060 Laptop GPU, NVIDIA 595.104.02, at 1920 × 2160,
standard quality and isolated native preferences. Each original was decoded
and hashed at capture time. These are Linux event-free poses, not temporal,
live-effect, web or Windows acceptance.

## S1 and S2 candidates

The [blue-artifact investigation](P70-BLUE-ARTIFACT.md) identifies the shared
opaque team tabard and its incompatible hip/chest attachment as one reproduced
cause. Changing teams changed the patch from blue to red while the background
remained blue. Closed geometry and opaque materials do not support a universal
hole/culling explanation. Four callers now use fitted chest heraldry; distinct
rag/ribbon/skeletal surfaces still require individual review.

[Old Finn](P70-OLD-FINN.md), [Pyre Warden](P70-PYRE-WARDEN.md) and Trident
have candidate authored model/motion work. Pilot after renders and live
combat inspection remain pending. The new live recorder drives ordinary
training actions and records real event/HP observations at unchanged combat
speed; it does not synthesize combat events.


## Validation actually run

Level: **work front**, with the explicitly requested baseline capture build.
The owner did not request a full suite, release build or balance gate.

- `cargo fmt -p tbg-client -- --check`: passed.
- `git diff --check`: passed.
- Python AST parsing and `capture_roster.py --help`: passed.
- `python3 scripts/p70/validate_evidence.py --json`: bookkeeping valid,
  phase incomplete; initially 0 before / 0 after verified pose slots.
- The first locked offline development build failed because `alsa.pc` was
  outside the default pkg-config path. The existing local dependency path
  documented in NATIVE-client.md was reused; no packages were installed.
- Corrected scoped command:

  ```sh
  flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
    env CARGO_BUILD_JOBS=2 \
    CARGO_TARGET_DIR=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/target \
    PKG_CONFIG_PATH=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/.local/native-deps/root/usr/lib/x86_64-linux-gnu/pkgconfig \
    cargo build --offline --locked -p tbg-client --bin tbg-client
  ```

  Result: exit 0, `Finished dev profile [unoptimized + debuginfo] ... in
  56.63s`. This compiled the S0 capture/gallery working diff against the
  unchanged required content, using the existing shared target cache.

The combined pilot development build was attempted once with the same
scoped command and returned **75 (shared validation lock busy)** before
compilation. It is deferred; no waiting, polling, lock bypass, new target
directory or claim of candidate compilation followed. Continue independent
source review and consolidate after the front is ready.

## Deferred acceptance and decisions

- The 460-sheet static before campaign and 120-sheet S1 enemy diagnostic
  campaign completed. Temporal before reconstruction from the frozen tree,
  all final captures, remaining 43 art passes, lineup, live abilities,
  Troops/training, P63 and transition checks remain pending.
- The reproduced tabard issue is documented separately; there is no claim
  that every blue patch shares that cause or that the final regression
  campaign has passed.
- Trident height, weapon reach and proposed integer range remain unmeasured
  and unselected. Core range remains 1,100; no balance acceptance is claimed.
- Queue the new scoped regression at consolidation, under the host lock:
  `cargo test -p tbg-client --bin tbg-client
  p70_pose_sheets_include_event_and_final_without_duplicate_samples --
  --test-threads=1`. It was written but not run on the work front.
- Existing kit event-time and fish trajectory checks, pilot-specific new
  regressions, WASM/browser validation and Windows runtime are deferred to
  the relevant final changes. Native compilation does not establish parity.
- D-33 baseline/candidate lab and gate need the separately authorized heavy
  validation round. Owner look/fun approval remains pending for all 46.
- No push, merge, deployment, packaging or changes to other active worktrees.

## Deferred pilot consolidation and independent continuation

> blocked: The scoped pilot development build and later combined pilot
> consolidation both returned exit 75 (shared validation slot busy), before
> Cargo started. The latter attempted the deduplicated client filters `p70_`,
> `fish`, and `pyre_jet`, followed by the capture executable. No test, build
> or after capture ran. The nonblocking lock was preserved; compilation
> attempts stopped. The suspected cause is another validation owning the
> host slot. No owner session/build was terminated or changed.

Three independent source-review fronts continue the remaining 43 champion
art passes using the already captured authentic baseline. This is a recorded
sequence deviation: S2 runtime acceptance is still pending, so those passes
must remain implemented candidates and cannot be signed off from source.
The owner policy permits deferring contested compilation and continuing
independent work; Trident range calibration and actual after evidence remain
dependent and deferred. No arbitrary range increase substitutes for that
calibration. The corresponding competitive change is not yet implemented.

The unchanged frozen executable also produced 50 auxiliary pose sheets
(Beetling, Crate, Microdrill, Duodrill and Megadrill). These five entities
remain outside the 46 champion count. Their preview columns do not imply
playable Evolution; spawning, cargo and fusion need parent-event evidence.
The index now links 630 authentic before originals, including the original
460 champion sheets and 120 S1 enemy diagnostics. Each new auxiliary PNG
was fully decoded and hashed at capture time.

The live recorder now accepts the fixed roster through `FULL_POOL` and the
manifest, retains the three pilot compositions, and derives a conservative
ordinary engagement from the existing kind/form stats for other champions.
It records the subject's real ID, separates its attack/ability events from
battle-wide counts, and retains healing and summon attribution. A generic
engagement is not proof of conditional abilities, all clips or Victory.
Syntax/format checks passed; runtime remains deferred with the client build.
