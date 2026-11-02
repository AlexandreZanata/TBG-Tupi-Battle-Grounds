# RESTORE-01 — immediate pieces and legacy pushing

Owner correction, 2026-11-02: keep the widened BattleLine and strong towers,
restore the former advancing/pushing format, remove the visible delay and
stationary units caused by PK-01, and only optimize presentation flicker.

Practice and Training now use the same legacy movement default as canonical
matches. Every affordable purchase enters immediately. Friendly body width
never queues a normal purchase or blocks authoritative movement/Evolution.
The explicit PK-01 lab experiment and its historical evidence remain isolated;
normal constructors do not enable it. No match is hot-migrated from an old
running executable. The new installed window identifies itself as R12.

The shared cosmetic solver keeps pushing/overtaking, uses the widened floor,
updates at fixed 60 Hz, orders pairs by stable anchors/IDs, and expands weapon
clearance without moving the sort anchor. Display offsets are eased with a
speed cap and stationary deadband; server anchors and new figures are not
smoothed or delayed. Towers, prices, champion stats, five forms and body art
are unchanged. See GAME-DESIGN RESTORE-01 and the desktop visual contract.

Targeted consolidation (shared nonblocking lock, two build jobs, existing
native cache, offline/locked, one test thread):

- Core `legacy_deployment`: two passed, including 12 immediate mixed purchases,
  both seats, correct Energy spending, free progression, Evolution among allies,
  and twelve active giants advancing in Training without a queue.
- Ruleset/determinism/lab identity: seven passed. Canonical state and trace
  checksums are unchanged from revision 11; ruleset 12 identifies preview defaults.
- Client `crowd::tests restore01_`: ten passed, including pushing/overtaking,
  authoritative offset bounds, ghosts, frame-delta bounds, 60/120 Hz equality,
  stable aim anchors and immediate new figures with bounded cosmetic motion.
- Formatting/diff/static web checks and Linux native observation/install receipts
  are recorded below. No full suite, balance gate or additional platform build.

Windows/desktop game-web visual observation and a long dense interactive game
remain pending owner review. Linux is the reference. Shared code and scoped
unit tests do not certify full visual parity or every possible overlap case.

## Consolidated Linux receipt

Source: `ef0975cf46827559443380bd3200bdb01b11f99a`.
Wire 36, ruleset 12, contract `w36-r12-8f5a8ebb3e4f88ec`.
The package remains `0.1.4-colossus.1`; the window suffix `R12` distinguishes
this restoration. The subsequent receipt commit changes documentation only.

Exact targeted commands, each compiling under
`flock -n -E 75 /tmp/tbg-validation-1000.lock`, `CARGO_BUILD_JOBS=2` and the
existing shared target directory:

```sh
cargo test --offline --locked -p tbg-core --features lab,training --test legacy_deployment --test contract_ruleset_version --test contract_determinism --test contract_lab -- --test-threads=1
cargo test --offline --locked -p tbg-client --bin tbg-client -- crowd::tests restore01_ --test-threads=1
cargo build --offline --locked -p tbg-client --bin tbg-client
```

Results: core 9 passed, client 10 passed, native debug build passed in 14.84 s.
The native build used `TBG_SOURCE_COMMIT` above and the existing local native
dependency sysroot. The client tests emitted the existing unused `Layout::clear`
warning. Affected-package formatting, `git diff --check` and
`python3 web/scripts/check.py` passed (82 catalog entries, 156 landing files).

One isolated native X11 capture used `TBG_CAPTURE_PANEL=training`,
`TBG_CAPTURE_LEGACY_CROWD=1`, `TBG_CAPTURE_SIZE=1600x1000` and
`TBG_CAPTURE_AFTER=8`, with two renderer CPUs/threads and separate capture data.
Exit 0. The renderer observed 13 active entities (12 allies), zero queued units
and 30 bounded paused frames, maximum root step 0. Loading was complete with
zero failed images or pending text. The screenshot was inspected. The fixture
deliberately starts ten Archers at one anchor: entity presence does not certify
that every model is unoccluded, and this saturated fixture still has overlaps.
Progression is covered by the two core scenarios, not by the paused image.

The Linux installation is the source-specific `0.1.4-colossus.1-ef0975cf4682`
slot. Binary SHA-256:
`8c3ba31d6924dd7558cceb8f806d4787af8c2a946f2d0fb1cbbfb83779528c55`.
All 380 installed asset hashes and the exact installed build information matched.
The immutable staged executable and installation share a hard link to avoid a
redundant 118 MB copy on the nearly full disk. The launcher/desktop entry now
select this slot; a new catalog window uses separate data. Existing matches are
not migrated. Local gallery <http://localhost:8966/> now describes restoration
and replaces the old queue proof with the new native scene. The landing at
<http://localhost:8604/#champions> reports the R12 contract. HTTP responses were
checked; actual browser appearance remains unobserved.

Logs, original PNG, installation manifest and per-file verification are retained
locally in `.local/legacy-crowd-2026-11-02/` at the owner checkout.

Deferred consolidation: compare crowded purchases, pushing, turning, pause and
Evolution on Windows and the desktop web client, then review a long dense Linux
Practice match for appearance/fun. Browser automation remains unavailable
(`browser.user is unavailable`). No additional target build, full suite, balance
gate, release build, merge or public deployment was run for RESTORE-01.
