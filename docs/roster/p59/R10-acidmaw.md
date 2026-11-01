# R10 — Acidmaw delivery

Implemented: a dedicated long-neck/four-jaw kit, five forms, breathing sac,
high-kneed motion, spit recoil/snap, collapsing neck and Evolution jaw fan.
The shot owns its turquoise mesh/trail/impact and secondary connections.
Corrosive Spit preserves primary damage and selects a bounded set of distinct
legal secondary targets. Content owns branch values (D-70). Protocol 22 appends
its public cue; public Shop/lab pools remain unchanged.

## Local evidence

Level: work-front delivery, with one combined R08–R11 pilot consolidation.
All compiling commands used the shared nonblocking host lock, two Cargo jobs and
existing caches; Trunk also used BINARYEN_CORES=2 and the cached matching tools.

- `cargo fmt -p tbg-core -p tbg-client`, `git diff --check`: passed.
- `cargo test -p tbg-core --lib r1 -- --test-threads=1`: 9 passed, including all
  four Acidmaw scenarios: ordinary third primary damage, no Stronghold counting,
  best-k exclusions/order, killing primary/enemy summons, B2 and static-only
  auxiliary damage without poison or recursive counts.
- `cargo test -p tbg-core --test contract_roster --test contract_protocol -- --test-threads=1`:
  5 roster and 10 protocol tests passed on the combined pilot tree.
- `trunk build --release --offline --locked` in crates/tbg-client: exit 0 using
  CARGO_TARGET_DIR=/tmp/tbg-pr20-web-target. Only two unused-import warnings;
  removed them, then `cargo check -p tbg-client --bin tbg-client --target wasm32-unknown-unknown`
  passed without warnings in 4.59s using CARGO_TARGET_DIR=/tmp/tbg-pr20-target.
- Actual browser at 1280×720: Base/A1/A2/B1/B2, idle/walk/attack anticipation.
  Triangles: 1,280 / 1,408 / 1,422 / 1,440 / 1,482. A shows throat guards and
  the final sight horn; B has a wider sac shell and final three-lobed collar.

![Acidmaw actual five-form lineup](../../media/p59/acidmaw/lineup.jpg)

The preceding R09 preview was deferred on exit 75 (shared slot busy); independent
R10/R11 work continued before the combined consolidation. No lock polling or
bypass. Screenshot evidence is from that combined pilot build; the only subsequent
client edit removed unused imports. The historical character commit was not
rebuilt separately. Full-clip playback, ability VFX in a real match, live reconnect,
performance, balance gates and the full suite remain deferred; none is a pass.
