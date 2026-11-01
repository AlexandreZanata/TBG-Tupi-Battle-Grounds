# P72 — The browser runs the native (Linux) interface

**Status:** executing (2026-11-01) on `phase/p72-web-native-ui`, stacked on
P71 (`phase/p71-match-continuity`).

## Owner request and approval (2026-11-01)

Capture every native Linux screen (lobby, bot match, arenas, ranking,
settings, character cards and evolutions, …), compare each with the web,
and make the web look exactly like native Linux; "even if we must remove the
CSS, JS, everything, and leave only the native". This approves the migrated
Bevy screens for the browser that `config::bevy_ui()` was waiting for
("Keep the HTML path until the owner approves the migrated screens").

## Decision

- The browser uses the **desktop presentation**: `build.rs` sets
  `cfg(tbg_desktop)` for `wasm32` as well as Linux and Windows, so the web
  compiles the same Bevy shell, battle HUD (`native_battle`), ranking
  (`native_ranking`) and loading screen (`native_loading`) as native Linux.
  Presentation still never depends on the OS; platform services (vault,
  files, updater, process lock) keep their own `target_arch` gates.
- `config::bevy_ui()` is always true.
- `index.html` keeps only the language resolver, icons, the Trunk links, the
  canvas and the attribution footer. The HTML lobby, ranking overlay, battle
  HUD, account form and their CSS/JS are removed, with the JS tests that only
  covered them (`tests/{account,arcana,arenas,art,factions,live,lobby-art,
  lobby,portraits,stronghold}.test.js`) and the DOM wiring test
  `ui/battle_wiring_tests.rs`.
- Kept: `assets/auth/session.js` (browser sign-in, tested by
  `auth-session.test.cjs` in CI), `assets/i18n/i18n.js`,
  `assets/ranking/*` and `assets/lobby/icons.svg` (the public `/ranking/`
  pages use them), `diagnostics/measure.js` (opt-in `?measure`/`?perf`).
- Android keeps the non-desktop presentation (`target_os = "android"`); its
  `cfg(not(tbg_desktop))` code stays.

## Verification

The same scenario parameters drive both platforms: native reads
`TBG_<NAME>` environment variables, the browser the `?name=` query
(`config::url_param`). Each scenario is captured by the client's own Bevy
screenshot at 1440x900 in pt-BR (native: `/data/tbg-p72-visual/
native-matrix.sh`; web: `web-matrix.cjs`, where the screenshot arrives as a
download), then compared by mean absolute difference and side by side
(`compare.py`).

Scenarios: lobby; the Troops, Arenas, Arcanas, Stronghold, Settings, Patch
notes and Live panels; Ranking; the training lab; a running Practice match
(shop cards with portraits, Energy/Interest/Army bar); War Council; Shop
Reserve; Fortification; unit details (evolutions); arena training (Forest);
victory.

## Status log

- 2026-11-01: native reference captured from the rc.1 Linux build (17
  scenarios); desktop presentation enabled for wasm, `bevy_ui()` constant,
  `index.html` reduced, DOM assets and their tests removed (`0061723`).
- 2026-11-01: the lobby battle demo panicked on wasm ("time not implemented
  on this platform": `std::time::SystemTime` in `sync/lobby_battle.rs`); the
  browser now seeds it from `Date.now()` (`b2c2d9e`).
- 2026-11-01: first parity result. With the same Bevy screenshot on both
  platforms (web build with `--features ui-acceptance`, comparison only, never
  deployed), the lobby differs from native by a mean of 5.5 (0–255 scale): the
  same header, menu, demo battle, cards and news. Page screenshots of the
  production web build already matched the Troops, Patch notes, Arcanas,
  Stronghold, Arenas and Settings panels within 2–7. Remaining scenarios
  (ranking, training, battle, War Council, Reserve, Fortification, unit
  details, arena training) are being recaptured; the first web run used a
  percent-encoded `?server=` that `config::url_param` reads raw (a harness
  mistake, not a product issue: production has no `?server=`).
- Validation: `cargo test -p tbg-client --bin tbg-client` 160 passed (2
  ignored); clippy `-p tbg-client -D warnings` clean; `cargo check --target
  wasm32-unknown-unknown -p tbg-client -p tbg-core` clean; release wasm and
  Trunk builds succeed. Known limit: phones lose D-20's DOM keyboard input
  (out of the desktop-only scope).
- 2026-11-01: **parity measured** with the client's own Bevy screenshot on
  both platforms (native: rc.1 Linux build; web: comparison-only build with
  `--features ui-acceptance`, never deployed), 1440x900, pt-BR. Mean
  absolute difference per pixel (0–255):

  | Scenario | Difference | Scenario | Difference |
  |---|---|---|---|
  | Troops panel | 0.07 | Unit details (evolutions) | 1.07 |
  | Ranking | 0.15 | Shop Reserve | 1.67 |
  | Stronghold panel | 0.28 | Fortification | 2.16 |
  | Patch notes panel | 0.30 | Live panel | 2.71 |
  | Arcanas panel | 0.41 | Lobby | 3.03 |
  | Settings panel | 0.47 | Arena training (Forest) | 4.00 |
  | Training lab | 0.70 | Running battle (shop cards) | 4.69 |

  The remaining differences come from dynamic content: the lobby's demo
  battle, random Shop offers and unit positions per match, live counters.
  Not captured on the web within the time limits of software (SwiftShader)
  rendering: War Council (waits for an Augment round), victory and the
  Arenas panel; the Arenas panel's page screenshot shows the same cards.
- 2026-11-01: **deployed** to the test environment as the web-only release
  `p72w-b2c2d9e17509` (code `b2c2d9e`; later commits are docs) on top of the
  P71 server: the public page loads the Bevy canvas (no DOM lobby, no page
  errors), Casual opens `wss://…/ws/slot-a?contract=w30-…`, and the browser
  sign-in module still issues the `__Host-tbg_refresh` cookie (Secure,
  HttpOnly, SameSite=Strict), restores the same family and logs out.

