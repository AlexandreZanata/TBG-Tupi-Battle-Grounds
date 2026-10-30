# Ranking UI delivery report

## Result

A shared desktop ranking presentation is integrated into the game lobby and public `/ranking/` page: a 2/1/3 podium, ten-row paginated table, player search, public profile links, current account record and five recent ranked results. Six new illustrations follow the Arcana/Augment direction. See [art provenance and measurements](art/RANKING.md).

The server remains authoritative: BR-R07 ordering, ranked eligibility, Rating and BR-P05 public statistics are unchanged. No Rust source, protocol, schema, game rules or dependency changed. The public page remains plain HTML/CSS/JS without WASM. In-game identity comes from the existing shell projection, never account keys. Public `?player=Name` is labeled Player Record and is not authentication. Navigation cancels requests; identity changes invalidate older responses. Missing places, empty data, request errors and rate limits have honest states.

## Deviations and scope

The reference includes season and region dropdowns. These features are deferred by the canon, so the actual All time / Global / Ranked 1v1 scope appears as informational fields. There are no invented filters or synthetic production players. Ten-row paging uses the existing HTTP API; the Bevy fallback stays unchanged. Desktop browsers only, per owner rule.

## Verification

Pending final verification and desktop gallery. Synthetic local visual fixtures use Fixture* names and never enter a server database. No production account is created for testing.
