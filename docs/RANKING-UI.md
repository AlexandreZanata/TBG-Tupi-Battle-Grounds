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
- `cargo test --workspace`: pending the long balance contract test.
- `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`: pending the workspace test sequence.
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
