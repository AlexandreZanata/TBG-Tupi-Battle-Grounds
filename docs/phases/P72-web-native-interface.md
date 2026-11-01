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

- 2026-11-01: native reference captured from the rc.1 Linux build; desktop
  presentation enabled for wasm, `bevy_ui()` constant, `index.html` reduced,
  DOM assets and their tests removed. Web build pending (shared validation
  lock held by another session).
