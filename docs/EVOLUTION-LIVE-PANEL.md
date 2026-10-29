# Live Evolution choices and centered countdown

Owner bug fix on the existing `phase/p16a-desktop-lobby-arena` branch.
Changes are limited to the client UI, separate from P16b art/animation work.

## Cause and fix

The Evolution panel rebuilt only when `Selected` changed. Its buttons stored
either an Evolution action or `Disabled` at that moment, so opening below the
cost left the choices disabled after Energy regenerated. Reopening recreated
the buttons with the newer Energy value, hiding the stale-state bug.

Each choice now keeps its unit and branch identity and refreshes its action
and color from `LatestView`. The panel and button entities remain in place.
Regeneration enables the choice at its current tier's actual cost; spending
Energy disables it again. Ownership, phase, branch eligibility, and maximum
tier are checked for presentation. The server still validates the intention
and applies Evolution (BR-E02, BR-E03, BR-V01, BR-V02, BR-A02).

A regression scenario opens the panel one sub-energy below the tier-1 cost,
reaches the exact cost, and drops below it again. It checks the same existing
buttons throughout, including their actions and enabled/disabled colors.

## Countdown

The three-second pre-match countdown uses a large number and a `STARTING`
label centered on the BattleLine. It is a separate Bevy overlay, so it also
works when the desktop HTML HUD hides the native HUD. The HP header shows the
canonical regulation clock instead of the countdown text. The overlay hides
when the authoritative phase leaves Countdown and despawns on leaving the
match; the duration and simulation remain unchanged.

## Validation

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace -- -D warnings`: passed.
- `cargo test --workspace`: 97 passed, 0 failed, including the new regression.
- `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`: passed.
- `trunk build --release`: passed, using an isolated output directory under
  `.local/previews/evolution-live-panel/`.
- `git diff --check`: passed.

On 2026-10-29, two browser clients played against an isolated server on
127.0.0.1:18081. The Rootcaller panel stayed open with disabled choices at
1.4 Energy; the same choices became enabled at 3.1 Energy. Clicking Deep
Roots succeeded without reopening: the panel closed, Energy became 0.2,
and the unit showed its Evolution ring. Reloading resumed the online match;
two-step surrender ended both clients with opposite Defeat/Victory results.

The centered countdown and the HP-header clock were inspected at 1280x800
and 390x844. The latter is a desktop browser at phone dimensions, not a
physical-device performance measurement. No browser warnings or errors were
observed in the final phone verification.

Verification output and browser captures are in
`docs/media/evolution-live-panel/`. The existing shared `dist/` directory,
art/animation files, simulation, protocol, and server source were not changed.
