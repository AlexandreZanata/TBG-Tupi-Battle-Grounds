# SIDE-01 — Random sides with a common competitive camera

Owner request, 2026-11-02: implement random blue/red sides online and versus
bots, identify the side like the Arena, validate Ranked and deliver to GitHub.
The owner explicitly confirmed blue left/red right for every viewer.

Online pairing randomizes the whole token order once from an unbiased server
bit, before account/name/connection/Arcanas/Stronghold freezing. Existing seat
authority, protocol, canonical coordinates, summaries and Elo remain. Practice
uses a separate deterministic side stream and correctly maps human/bot commands,
loadout/views and disclosed bot assistance. The latter helper is in the already
excluded local-only practice-bot source; watched gameplay/identity is unchanged.

The desktop scene retains its common camera and removes viewer-relative world
reflection. Actual side drives figure yaw, foot rings, crowd direction, tower
geometry/fire/aura/wear, HP/crests/names and barrel placement. Actual ownership
continues to drive actions, private cards, statistics and Fortification selection.
A localized Arena/side chip and introduction announce the human side; spectators
see a neutral legend. Existing original pushing/wide road are retained.

Tests cover balanced side assignment over every draw byte, both Practice human
sides and loadout/commands, seat-aware Impossible assistance, shared coordinates/
facing/camera, side announcement, and a real local Ranked WebSocket match with
two registered accounts, purchases, enemy-command rejection, own Fortification,
reconnect and correct Elo/account settlement. Invitation rematch/preset tests
are updated to identify accounts through their assigned seats.

Targeted consolidation and Linux installation/visual receipts follow below.
No full suite, balance gate, public ranked traffic, merge or deployment.

## Targeted consolidation

All compiling commands used `/tmp/tbg-validation-1000.lock` nonblockingly,
`CARGO_BUILD_JOBS=2`, the existing shared target/native dependency sysroot,
`--offline --locked` and `--test-threads=1`. No full suite was run.

```sh
cargo test --offline --locked -p tbg-core --test practice_bot --test contract_ruleset_version -- --test-threads=1
cargo test --offline --locked -p tbg-server --lib -- side01_ ranked_matches_are_listed_with_ratings_and_their_end_unlists_them --test-threads=1
cargo test --offline --locked -p tbg-server --test ranked side01_ranked_assignment_commands_reconnect_and_account_settlement -- --test-threads=1
cargo test --offline --locked -p tbg-server --test online challenge_freezes_arcanas_through_reconnect_and_rematch -- --test-threads=1
cargo test --offline --locked -p tbg-client --bin tbg-client -- side01_ arena_clear01_ arena_refine01_ colossus --test-threads=1
```

Final distinct results: six core cases (four bot, two identity), two lobby
cases, one real Ranked scenario, one invitation/rematch scenario, four client
side/camera/Practice cases, three Colossus cases and three Arena cases passed.
The listing fixture was corrected to assert both ratings independent of order;
its synthetic session names are unrelated to the account profile names. The
Ranked harness initially attempted an anonymous reconnect for an authenticated
account and was stopped; it now uses the original family Access credential and
an overall 20-second deadline. The final scenario passed in 0.30 s.

Initial client compilation exposed incorrect casts and test-only field access;
these were corrected against the concrete types. The combined client run then
passed nine cases and exposed a remaining Tundra rock/fir overlap. The complete
fir/crystal group was moved and only the three affected Arena cases rerun:
3 passed, 0 failed. Practice's loadout fixture was subsequently strengthened
with the existing valid non-default invitation preset; only that affected case
was rerun. Logs retain all attempts, including a rejected invalid test preset;
no gameplay rule or test assertion was relaxed. The existing unused
`Layout::clear` warning remains.

Artifacts: `.local/blue-red-sides-2026-11-02/`. Two obsolete derived executables
from this worktree were losslessly archived with verified SHA-256 under the
same host lock to recover build space. Active executables, dependency cache
directories, installed versions and sessions were preserved.

Linux installation and native observations are recorded after building the
committed source. Windows and desktop game-web visual parity remain pending;
shared source, Linux screenshots and native tests do not establish parity.
