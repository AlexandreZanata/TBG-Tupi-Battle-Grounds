# P70 visual execution review

**Status: S0 in progress; no art acceptance or completed phase is claimed.**
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
generated image can substitute for renderer evidence. Captures are not yet
present at the initial tooling commit; index entries remain pending.

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

## Deferred acceptance and decisions

- Baseline/final renderer campaigns, all 46 individual art passes, blue
  artifact runtime reproduction, lineup, live abilities, Troops/training,
  P63 and temporal transition checks remain pending at this checkpoint.
- Source inspection is not a confirmed root cause of the reported blue
  square/hole. The shared tabard is only one of several independent leads.
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
