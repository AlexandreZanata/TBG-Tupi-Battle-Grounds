# P59 character delivery — local validation and deferred acceptance

Scope:21 staged character libraries, five forms each, canonical Proposed abilities,
three finite drill summon ranks and shared inspection/Evolution integration on top
of PR20. CANON.pool stays25, ALL_KINDS27 and UNIT_KINDS10. Protocol27 appends only the reviewed
flame/fusion events after protocol25; simulation state layout is unchanged after
D-71. No activation, merge or production deployment is part of this character PR.

## Policy and actual evidence

Owner's2026-10-31 policy: minimum per front, deduplicated affected tests at local
consolidation, no full suite without an explicit request and no CI wait. Every
compiling command used the shared nonblocking host lock with CARGO_BUILD_JOBS=2;
Rust tests used --test-threads=1. All caches were reused; no other worktree cache,
active process, original workspace work, VPS project or service was changed.

Per-character reports hold the historical blocks and fixes. Counts below describe
each distinct scoped block; do not sum them as unique total coverage:

- Pilot34; R01–R03:58; R04–R07:75; R12/corrected art:15.
- R13–R15:40; Oracle hierarchy/R16:12; corrected poses/Mantis:30.
- Hornet/Weaver:42 after correcting the Guard fixture and restoring Weaver's
  documented4-Energy cost. Existing roster assertions were retained.
- Final R20/R21/core contracts:33 (R20four, R21seven, roster7, protocol13,
  determinism2). Shallow responsibility splits reran only the11 affected cases.
- Final external hostile-Mark fusion regression:one additional passing case.
  The Mark follows the replacement id with its original deadline; no cleanse.

Final command scopes (inside the shared-lock/two-job wrapper):

    cargo test -p tbg-core --features lab --lib r20 -- --test-threads=1
    cargo test -p tbg-core --features lab --lib r21 -- --test-threads=1
    cargo test -p tbg-core --test contract_roster --test contract_protocol --test contract_determinism -- --test-threads=1
    cargo test -p tbg-core --features lab --lib r21_external_hostile_mark -- --test-threads=1
    cargo check -p tbg-client --bin tbg-client --tests --target wasm32-unknown-unknown

The client/test WASM target check passed in2.34s, without warnings, after fixing
an Ability::None/Option::None name collision in the new text formatter. It compiles
tests; it does not execute native Bevy tests. JavaScript syntax passed with
`node --check crates/tbg-client/assets/battle/hud.js`. Package formatting and
`git diff --check` passed. Three frozen bot outcomes/ticks/hashes remain exact.

Actual browser artifacts use:

    trunk build --release --offline --locked

with existing target `/tmp/tbg-pr20-web-target`, BINARYEN_CORES=2 and cached tool
PATH entries for wasm-bindgen0.2.129/wasm-opt131. Hornet/Weaver's Rust release was
3m24s; packaging initially missed that PATH, then exited0 with real gallery output.
The first final release process ended143 before completion; it is not a pass.
Persistent-session resumed packaging and actual final screenshots are recorded
below when completed. Native/test cache stays `/tmp/tbg-pr20-target`.

## Deferred checks — never claimed as passing

- Native Bevy UI tests: `cargo test -p tbg-client ui::ability_details::tests --
  --test-threads=1` and `cargo test -p tbg-client ui::inspect::tests --
  --test-threads=1`. Reason: cold native client cache; WASM is the actual platform.
- Full workspace verification (`cargo clippy --workspace -- -D warnings`,
  `cargo test --workspace -- --test-threads=1`, additional native targets): only
  on explicit owner request. None was requested for this front.
- One full animation-clip/ability/HUD pass for all21; compare quality=low and
  normal, inspect late-join/reconnect mid-special, counter, Mark/focus, DoTs and
  factory/fusion profiles. Gallery captures cover desktop form/gait/basic poses,
  not all clips or real ability impacts. Native compilation is no browser proof.
- Two actual protocol27 clients finish a match with reconnect, plus spectator
  projection during a special/fusion. Reason: this character delivery preserves
  the existing multiplayer front; no complete new-version server scenario ran.
- D-33 staged-pool screening/confirmation matrices, ablations and cap/fusion/
  friendly-fire fun measurements. The unchanged25-kind gate cannot prove a
  46-purchase staged roster. Do not weaken thresholds or activate pools to pass.
  Heavy lab/release gates remain separately authorized owner consolidation work.
- Desktop dense-army CPU/frame/bandwidth/load measurements and owner look/fun
  sign-off. No performance, balance or production-readiness claim is made.

CI belongs to owner consolidation. This report records local evidence and leaves
subjective/W10/W11 acceptance open; deferred checks are not failures or passes.

## Final actual artifact

Persistent-session packaging completed with the full protocol27 candidate,
including the external hostile-Mark fix: native focused scenario passed, core
WASM check1.65s, Rust release2m59s and actual new browser artifact served. Pyre
Warden and Drillnest factory lineups were observed. The generic tiny-drill lineup
was too small for art inspection, so only diagnostic portrait camera framing was
adjusted for those three kinds. One final requested close-up build follows; it
changes no gameplay scale or combat. Combat evidence above is reused without
repeating unchanged cases. Ability copy removes ordinal wording such as3th.

Final accepted captures:21 factory/character lineups plus three base-rank close-ups
in P59-GALLERY.md. The close view found leg gaps and orbiting multi-bore heads;
connected struts/crossmembers and per-bore local axes were corrected before delivery.
Final corrected-axis Rust WASM release compiled without warnings in2m11s;
packaging exited0 and the accepted Duodrill/Megadrill portraits were observed.
Microdrill's accepted geometry and the factory lineup were unchanged and reused.

The mixed-control inheritance clarification reran only
`r21_fusion_never_cleanses` (passed) and cached core WASM check (passed0.50s).
The final corrected-axis artifact includes that correction and shared D-89 UI.
Character source revision:6d65ee6; final shared presentation/report diff follows it.
The34 distinct final scenarios, earlier per-block evidence and WASM checks support
this work-front delivery. Native UI execution, full ability/HUD/Low comparison,
new-version full multiplayer/reconnect, performance, balance and owner look/fun
acceptance remain deferred exactly as listed above. No full suite or CI wait ran.
