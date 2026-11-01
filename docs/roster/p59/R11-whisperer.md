# R11 — Whisperer delivery

Implemented: a floating bone bell, single eye/mask, six short tendrils, five forms
and own float/pulse/collapse motion. Fear Pulse uses independent readiness and a
locked six-tick windup while the ordinary basic remains (D-71). Protocol 23 exposes
full cooldown/cast snapshots and appends explicit cues. The cosmetic special
channel cannot decide a hit. The shared venom slot is reserved for R18; public
Shop/lab pools remain unchanged.

## Local evidence

Level: work-front delivery and combined R08–R11 pilot consolidation. Every
compiling command used the nonblocking shared lock and CARGO_BUILD_JOBS=2 with
existing caches. Preview packaging also used BINARYEN_CORES=2 and cached tools.

- Affected-package formatting and `git diff --check`: passed.
- `cargo test -p tbg-core --lib r0 -- --test-threads=1`: 8 passed.
- `cargo test -p tbg-core --lib r1 -- --test-threads=1`: 9 passed, including all
  five R11 scenarios: full cooldown, waiting readiness, locked target/no range
  recheck, Root/death cancellation without refund or retarget, independent basics,
  Guard, ward, no special damage, Augment pause and seat/public reconnect facts.
- `cargo test -p tbg-core --test contract_roster --test contract_protocol -- --test-threads=1`:
  5 roster and 10 protocol tests passed.
- `cargo test -p tbg-core --test contract_determinism -- --test-threads=1`:
  initially failed only the serialized-layout hash comparison after all three
  unchanged winner/reason/tick guards passed. D-71 records the observed checksums;
  after the measured re-freeze, 2/2 passed in 0.43s. No outcome guard was weakened.
- `trunk build --release --offline --locked`: exit 0 on the combined pilot using
  /tmp/tbg-pr20-web-target. Two unused imports were removed afterwards; final
  `cargo check -p tbg-client --bin tbg-client --target wasm32-unknown-unknown`
  passed without warnings in 4.59s using /tmp/tbg-pr20-target.
- Actual 1280×720 browser lineup: Base/A1/A2/B1/B2, idle/walk/attack anticipation.
  Triangles: 1,588 / 1,588 / 1,612 / 1,788 / 1,788. A changes the pale bell,
  crest and wide mask rim; B segments the dark bell and enlarges the eye socket.

![Whisperer actual five-form lineup](../../media/p59/whisperer/lineup.jpg)

R09's earlier preview slot was busy (exit 75). Independent R10/R11 work preceded
this new combined consolidation; no lock polling or bypass. The screenshot is
from the combined pilot build; the only later client edit removed unused imports.
Full-clip playback, live match ability VFX/reconnect, performance, balance gates
and the full suite remain deferred. Native checks alone were not used as proof
of browser compatibility; the real WASM preview and screenshot are separate evidence.
