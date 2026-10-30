# Ranking UI delivery report

## Result

A shared desktop ranking presentation is integrated into the game lobby and public `/ranking/` page: a 2/1/3 podium, ten-row paginated table, player search, public profile links, current account record and five recent ranked results. Six new illustrations follow the Arcana/Augment direction. See [art provenance and measurements](art/RANKING.md).

The server remains authoritative: BR-R07 ordering, ranked eligibility, Rating and BR-P05 public statistics are unchanged. No Rust source, protocol, schema, game rules or dependency changed. The public page remains plain HTML/CSS/JS without WASM. In-game identity comes from the existing shell projection, never account keys. Public `?player=Name` is labeled Player Record and is not authentication. Navigation cancels requests; identity changes invalidate older responses. Missing places, empty data, request errors and rate limits have honest states.

## Deviations and scope

The reference includes season and region dropdowns. These features are deferred by the canon, so the actual All time / Global / Ranked 1v1 scope appears as informational fields. There are no invented filters or synthetic production players. Ten-row paging uses the existing HTTP API; the Bevy fallback stays unchanged. Desktop browsers only, per owner rule.

## Verification

Synthetic local visual fixtures use Fixture* names and never enter a server database. No production account is created for testing.

- `cargo fmt --all -- --check`: pass (empty output).
- `cargo clippy --workspace -- -D warnings`: pass, finished dev profile in 25.78 s.
- `cargo test --workspace`: the unchanged balance contract passed (7/7, 627.09 s). The initial online batch failed 3 of 31 tests; 230 earlier tests passed, with the existing one full realtime equivalence test ignored. See the exact retry evidence below.
- `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`: pass, dev profile finished in 13.29 s.
- `node --test crates/tbg-client/tests/*.test.js`: 21 passed, 0 failed. Includes server rank order, bounds, public W/L/D, all six asset integrations/budget, and late-response identity cancellation.
- `trunk build --release` in the client: pass. Rust release was cached (0.67 s); Trunk also completed WASM optimization and output staging. Served WASM remains SHA-256 `af370582afe9815c26bee7a49f1e762800712d7a98cf0ce297751c88450cef57` because Rust source is unchanged.
- Strict main synchronization: `git fetch origin && git merge origin/main` returned `Already up to date.` Baseline main already contains PR #10 Strongholds and PR #13 art; this branch started from main, not `phase/p17-strongholds`.

Observed in the browser: the real game build opens Ranking from the header and `?panel=ranking`; Back to lobby and Escape close it; Guest CTA opens the existing account form and Cancel returns; Arcanas still opens. The public page shows ten rows, stable global podium across pages, correct first/last controls, a public player record and recent Win/Loss/Draw deltas; profile links retain the existing full public profile. Empty, HTTP 429 and invalid-name API fixtures show honest empty/error states and a usable refresh action. All images load, and the final game build has zero error-level console messages.

At 1920x1080 all ten rows fit. At 1366x768 the table/recent list scroll inside the frame; there is no document overflow. Podium cards remain square and every stat panel stays within its hero area. Gallery:

- [Public fixture, 1920x1080](media/ranking/public-fixture-1920.jpg)
- [Public fixture, 1366x768](media/ranking/public-fixture-1366.jpg)
- [Real local game, empty DB, 1920x1080](media/ranking/game-empty-1920.jpg)
- [Real local game, empty DB, 1366x768](media/ranking/game-empty-1366.jpg)

The local fixture server and all test data are private `.local` tooling and are not distributed. No game server/core source changed, so multiplayer authority/reconnection behavior is covered by the unchanged existing tests; deployment additionally validates one live two-bot casual match.

## Online verification blocker

> blocked: `cargo test --workspace` initially failed `reconnect_resumes_match`, `records_insert_and_count`, and `two_bots_finish_a_match`. The first and third reported `Some("timeout") / Some("timeout")`; persistence reported `assertion failed: a.completed && b.completed` at `crates/tbg-server/tests/online.rs:503:5`. Retried the complete online suite with `cargo test -p tbg-server --test online -- --test-threads=1`, preserving all code, assertions and deadlines: 30 passed, 1 failed in 248.70 s. Reconnection and the two-bot match passed. `records_insert_and_count` repeated the exact assertion above. Suspected cause: local CPU contention against the existing 60-second bot deadline; this is an inference, not a proven diagnosis. AGENTS.md requires stopping the repeated step. No third attempt, test weakening, merge or deploy was performed.

The remaining `cargo test -p tbg-server --test ranked` passed 6/6, and workspace doc tests passed (0 cases). Both GitHub `check` jobs on f56ba52 passed (4m03s and 5m01s), including the full unchanged Rust workspace. The implementation, artwork and desktop gallery are complete. PR #14 remains draft while the local persistence test is unresolved. Local test evidence covers 266 passing cases and the existing one ignored case; one persistence test remains failed. User authorization is required to override the explicit repeated-error stop rule for an isolated release-mode retry. No player database is used by the tests.
