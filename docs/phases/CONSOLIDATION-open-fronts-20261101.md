# Consolidation of the remaining open fronts — 2026-11-01

Level: **consolidation** (AGENTS.md *Verification*, VALIDATION §3). No full
suite was requested. Branch `codex/consolidate-open-fronts`, worktree
`/data/tbg-consolidate-open-fronts`, based on main
`70827a74e7ac66faaf36b23fd204e5b0d9ce1ff1` (merge of #40). No deployment,
installation, publication, timer activation, production migration or flag
change was performed. The native publication front (`0.1.2-rc.1`, branch
`codex/native-downloads-publication`, PR #42) was not touched.

## Pull request inventory

Remote refs refreshed on 2026-11-01 before acting.

| PR | Head branch / SHA | Base | Ancestor of main? | `main...head` paths | Action |
|----|-------------------|------|-------------------|---------------------|--------|
| #22 | `phase/p59-complete-roster` `1d27ae9` | `phase/roster-expansion` | yes | 0 | Absorbed by #40; close as integrated |
| #26 | `codex/native-desktop-preview` `3a95248` | `phase/system-champions-crowd-shop` | yes | 0 | Absorbed by #40; close as integrated |
| #39 | `phase/web-landing-admin-foundation` `2000407` | `phase/backend-main-integration` | yes | 0 | Absorbed by #40; close as integrated |
| #41 | `phase/p73-active-players-challenges` `bf6da86` | `main` | no (9 commits, 96 paths) | 96 | Merged here; superseded by this PR |
| #42 | `codex/native-downloads-publication` `e016d98` | `main` | no (1 commit, 2 docs) | 2 | Owned by the publication front; untouched |
| #43 | `codex/laboratory-desktop-polish` `19f4cfb` | `main` (`70827a7`) | no (15 commits, 26 paths) | 26 | Added on owner request; merged here; superseded by this PR |

The stale descriptions of #22/#26/#39 (25-champion pool, protocol 20/27,
availability notes) are not instructions; main keeps 46 conventional
champions, eight Arenas, protocol 31 and ruleset 6 at `70827a7`.

## P73 merge and conflict resolution

Normal `git merge --no-ff` of `bf6da86` (merge base `0e23a1a`). Main had
162 later commits; 17 paths changed on both sides. Textual conflicts:

| File | Resolution |
|------|------------|
| `crates/tbg-core/src/protocol.rs` | Main's v31 history kept; P73 becomes **v32** at the top of the version log. |
| `crates/tbg-core/tests/contract_protocol.rs` | Version assertion 32 citing D-103; **both** appended tests kept (eight Arena identities and P73 tags/legacy `MatchEnd`). |
| `docs/DECISIONS.md` | D-101 and D-102 of main unchanged; P73's decision appended as **D-103** with the allocation history. |
| `docs/GAME-DESIGN.md` | Main's eight-Arena/auth integration and browser bounds kept; P73 BR-AP01–16 appended. |
| `docs/ROADMAP.md` | Main's backend/auto-deploy entries kept; P73 entry appended with protocol 32. |

Auto-merged overlaps were reviewed semantically: `config.rs` (P73 flags,
doc comment re-attached to `auth_clock`), `lib.rs` (`build_info` +
P73 modules), `matchmaking.rs` (eight-Arena draw unchanged), `store/mod.rs`
and `schema.rs` (P73 opt-in migration next to main's `CURRENT_VERSION`),
client `capture.rs`/`actions.rs`/`ui/mod.rs`/`Cargo.toml`/`Cargo.lock`,
and the i18n catalogs (1549 keys each, identical placeholders, all eight
`arenas.<slug>.name` keys present for history rows).

P63 needs no new reconciliation: it was already in the P73 base and main
did not change `match_task/celebration.rs` or `ui/result.rs` afterwards.
P73 keeps the complete winning cast payload and adds the personal summary
and history status beside, not inside, `MatchEnd`.

## PR #43 merge (laboratory, Evolution dialogs, Augment cards)

Added to this consolidation at the owner's request. Normal merge of
`19f4cfb`, already based on main `70827a7`. Conflicts were adjacent
additions only: both locale catalogs keep the P73 and training keys (1556
each, identical placeholders), `desktop-ui-contract.json` keeps both screen
lists (55 unique) and the contract document keeps both change records.
`ui/shell/mod.rs` auto-merged the Troops clip beside the P73 fixture banner;
the other 21 changed paths (training catalog/stage/preview, native battle
council/fortify/unit panel, captures, five phase reports) do not overlap P73.
#43's own pending acceptance (same-source Windows and desktop-web full-frame
and motion correspondence, remaining locale/viewport/DPI and interaction
states) carries over unchanged.

## Review fixes after integration

An independent read-only review of the P73 server found no high-severity
authorization or privacy defect and four verified issues, fixed in
`50302b3`/`6c1382c` with regressions:

1. A guest that signed into an account kept its directed invitations and
   rematch ticket, so the recipient could be paired with an identity it
   never agreed to (BR-AP03/07). Any sign-in into a different account now
   cancels both (`lobby/auth.rs`).
2. Losing account ownership to another slot kept the rematch ticket
   (`lobby/ownership.rs`).
3. The history journal loaded its own `.tmp-` staging files: one torn file
   hid every pending entry, and a complete one replayed forever and blocked
   slot retirement (`player_history/journal.rs`). Staging files are skipped,
   not deleted.
4. Declining or cancelling spent the per-IP create budget, so a shared
   exhausted address could not close invites (`player_challenges.rs`).

A missing end detail is recorded as `stronghold`; that is the existing
Ranked convention (`settle.rs`: no explicit surrender/disconnect means the
Stronghold fell), not a fabricated value, and was left unchanged.

## Build fixes (PR #41 CI was red)

PR #41's own CI run on `bf6da86` failed `clippy`, `test`, `wasm` and the
required `check`; the first local batch reproduced the same client error.
Fixed in `6f8f31f`:

- `capture` is compiled for every debug build, but the P73 synthetic
  fixtures (`Social`/`History::visual_fixture`) exist only with
  `ui-acceptance`, so plain debug, test and WASM builds did not compile.
  The P73 fixture capture is now gated by `ui-acceptance`, the same feature
  as its "LOCAL VISUAL FIXTURE" label, so synthetic data never renders
  unlabeled.
- Rust 1.99 clippy (`-D warnings`) rejected the history cursor decoder's
  manual parity check and `chunks_exact(2)`; it now uses `is_multiple_of`
  and `as_chunks::<2>()`, with identical behavior.

Later CI runs on this PR exposed more, fixed in `0b56bb4`, `3277d3a` and
`7281c34`:

- Clippy also rejected #43's eight-argument focus capture system (allowed
  with the repository's reason-comment idiom) and the client history entry
  parser's `chunks_exact(2)`.
- The workspace `test` lane failed in four targets the P73 front never ran:
  `protocol/history.rs` was not classified in `ruleset-sources.txt`; the
  Result test lacked the `MatchLink` P73 now reads; Ranked tests did not
  expect the BR-AP05 rematch offer that now follows every online PvP
  result; and the BR-R07 profile tests expected public `recent` results.
- The privacy proxy set `private, no-store` next to, not instead of, the
  server's `public, max-age=10` profile header; the header is now deferred.

## Visible behavior change on deploy

With every history flag off, BR-AP11 already applies: the public profile's
legacy `recent` list is empty for every account (the store treats a
schema-10 database as Only me and the API requires `TBG_HISTORY_PUBLIC`).
After this PR is deployed, profiles therefore stop showing recent matches
until history is migrated and players opt in. This is the documented P73
privacy default, not a regression to hide; the owner should expect it.
The adapted tests assert the private default first and then the original
result/delta/opponent/reason content after an opt-in.

## Identifiers

- **Decision:** D-103 (D-101/D-102 already used on main; nothing renumbered).
- **Protocol:** 32. Client tags 23–26 and server tags 33–36 re-verified:
  main changed no `ClientMsg`/`ServerMsg` variants after the P73 base.
- **Ruleset:** 6, unchanged; no file in the ruleset hash list changed.
- **Match contract (derived):** `w32-r6-097385fee9a10b0f` — wire 32 with the
  unchanged ruleset source hash. Not yet printed by a built binary.
- **Database:** default schema remains **10** (`DATABASE_SCHEMA_VERSION`
  and `--build-info` still report 10). Schema 11 is installed only by
  `TBG_HISTORY_MIGRATE=1`. The four history flags default false.

## Proxy and privacy

The P73 front edited only the legacy `deploy/Caddyfile.template`. The
official domains use `deploy/Caddyfile.domains.template`; it now carries the
same barrier with the existing verified `X-Forwarded-For {client_ip}`:
`/api/internal/*` → 404; `/api/me/*` joins `X-TBG-Slot` routing; public
`/api/players/*` and unknown/encoded `/api/*` go only to slot a with
`private, no-store`; only explicitly safe read APIs keep failover.

Checked with the local `caddy:2.10.2` image (`caddy validate` plus a
routing probe against two loopback stubs on remapped ports):
`/api/%70layers/…`, `/api/players%2F…`, `/API/players/…`, `//api/…` and
`/api/x/../players/…` all reached slot a with `no-store`;
`/api/internal/presence`, `%2F` and `%69nternal` forms returned 404; with
slot a stopped, public profile, unnamed `/api/me` and unknown API returned
502 (no fallback) while `/api/leaderboard` failed over to slot b and
`/api/me` with `X-TBG-Slot: b` reached slot b. Real servers, TLS,
Cloudflare headers and mixed-version slots were not exercised.

## Visual correspondence

`docs/ui/DESKTOP-UI-CONTRACT.md` keeps #43's four change records and adds a
P73 record; `desktop-ui-contract.json` lists eight P73 screens next to #43's
twenty laboratory/dialog/council screens. One shared Bevy
implementation serves Linux, Windows and web; the timestamp adapter differs
internally (Jiff system zone vs browser `Intl`) but renders the same text.
`python3 scripts/check-desktop-ui.py` passed. Windows, browser and final-source
Linux captures are **pending**; parity is unverified.

## Validation

### Run

- `git diff --check`: clean at every commit.
- `cargo fmt -p tbg-core -p tbg-server -p tbg-client --check`: exit 0.
- `python3 scripts/check-desktop-ui.py`: passed after each merge (source
  routing only).
- i18n catalog key/placeholder parity: 1549/1549 after P73, 1556/1556 after
  #43, no mismatch.
- Caddy validate + routing probe: see *Proxy and privacy*.
- `python3 scripts/ci-test-lanes.py`: passed (new test targets are covered by
  the package-level lanes).

First targeted batch, shared lock, `CARGO_BUILD_JOBS=2`, one test thread,
private target `/data/tbg-open-fronts-target`, at **`f1eb20f`** (P73 merge
and proxy/contract/schema-test commits; before the review fixes, #43 and the
build fixes):

| Command | Result |
|---------|--------|
| `cargo test -p tbg-core --test contract_protocol --test contract_protocol_server` | 16 + 2 passed |
| `cargo test -p tbg-server --lib lobby::player_challenges::tests` | 6 passed |
| `cargo test -p tbg-server --lib match_task::summary::tests` | 5 passed |
| `cargo test -p tbg-server --test player_challenges … --test ranked_stats_recovery` (11 targets) | 2/5/3/4/3/2/5/4/4/6/2 passed, 0 failed (incl. new `schema_upgrade` schema-11 case) |
| `cargo test -p tbg-client --bin tbg-client p73_` | **compile error** (fixture gating; fixed in `6f8f31f`) |
| `cargo check -p tbg-client --bin tbg-client --target wasm32-unknown-unknown` | **compile error** (same) |
| `cargo build -p tbg-client --bin tbg-client --features ui-acceptance` | exit 0 |

`tbg-core` is unchanged after `f1eb20f`, so its contract results stand.
The server results predate the review fixes; the client results identified
the build defect. None of them is evidence for the final revision.

Re-run of the four targets that failed in CI, at **`7281c34`** (clean tree,
same lock, jobs and private target):

| Command | Result |
|---------|--------|
| `cargo test -p tbg-core --test contract_ruleset_version` | 2 passed |
| `cargo test -p tbg-server --test leaderboard --test ranked` | 4 + 6 passed |
| `cargo test -p tbg-client --bin tbg-client` filtered to `ui::result::tests`, `p73_`, `ui::fortify::tests::`, `br_ag05_ag11_…` | 8 passed |

PR CI on `0b56bb4` ran the whole workspace (`clippy`, `wasm`, `balance`,
`desktop-ui` passed; `test` failed only in those four targets, so the
review-fix regressions passed there). The landing contract `quality`
failure is an open owner decision, recorded in the PR.

### Deferred

The earlier final batch at `6f8f31f` was
**not run: shared validation slot busy** (`flock -n` exit 75; another
front's champion preview capture held it). It was not retried in a loop,
bypassed or waited on. Run once, from the worktree:

```sh
cd /data/tbg-consolidate-open-fronts
export CARGO_TARGET_DIR=/data/tbg-open-fronts-target
export PKG_CONFIG_PATH=<main checkout>/.local/native-deps/root/usr/lib/x86_64-linux-gnu/pkgconfig
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" env CARGO_BUILD_JOBS=2 bash -u -c '
  cargo clippy --offline --locked -p tbg-server -p tbg-client -- -D warnings
  cargo test --offline --locked -p tbg-server --lib lobby::player_challenges::tests -- --test-threads=1
  cargo test --offline --locked -p tbg-server --lib match_task::summary::tests -- --test-threads=1
  cargo test --offline --locked -p tbg-server --test player_challenges --test player_match_history \
    --test history_privacy --test presence_global --test live_api --test contract_challenge \
    --test schema_upgrade --test ranked_admin --test ranked_recovery --test ranked_stats \
    --test ranked_stats_recovery -- --test-threads=1
  cargo test --offline --locked -p tbg-client --bin tbg-client -- --test-threads=1 \
    p73_ ui::fortify::tests:: br_ag05_ag11_picking_is_local_and_confirming_queues_one_intention
  cargo check --offline --locked -p tbg-client --bin tbg-client --target wasm32-unknown-unknown
  cargo build --offline --locked -p tbg-client --bin tbg-client --features ui-acceptance
'
```

The PR's own CI (`quality`, `clippy`, `test`, `balance`, `wasm`,
`desktop-ui`, required `check`) runs the workspace equivalents remotely.
Also deferred: #43's queued release `ui-acceptance` build, browser bundle and
browser screenshots, native final-source captures, Windows captures,
`scripts/test-desktop-ui.py` (Pillow absent locally; CI `desktop-ui` runs
it), the real two-client invitation/rematch/reconnect scenario and every
acceptance item listed in the
[P73 report](../planning/P73-EXECUTION-REPORT.md#blocked-and-deferred-validation)
and #43's phase reports.

## Rollout and rollback

Merge order: the native publication front tagged `v0.1.2-rc.1` on
`ebae364` (PR #42 head, protocol 31) and the landing now offers it. Merge
#42 first. Once this PR's protocol 32 reaches the server, those desktop
downloads become incompatible and disappear from `/downloads.json` until a
same-source `w32` native release is published.


Merging this PR changes the wire contract to `w32-…`. By design the
backend auto-deploy controller then refuses the candidate (`contract change
staged: coordinated client release required`), the web controller refuses
a `w32` browser candidate while the live server is `w31` (`browser contract
transition/drain requires coordinated release`), and `native_pull.py` stops
offering the `w31` desktop builds once the server is `w32`. Do not loosen
those guards. A coordinated release needs a backend candidate and the
matching web client together, then a native build of the same source before
downloads are offered again. Timers stay disabled
until the owner activates each family per `deploy/auto/README.md`.

Before any history activation (each a separate owner decision): deploy the
updated `Caddyfile.domains.template`, confirm every public-serving slot is
P73-aware, back up the DB through the normal procedure, then
`TBG_HISTORY_MIGRATE=1` once; after that, enable `COLLECT`, `API` and last
`PUBLIC`. Once the database reports 11, the auto-deploy policy fails closed
(`database change staged`) for manifests still declaring 10; teaching the
manifest about the opt-in schema is follow-up work, not a guard to disable.

Rollback: turn flags off; keep schema 11, preferences, revisions, journals
and the proxy barrier. A main binary opens a schema-11 file but ignores
privacy preferences, so it must not serve `/api/players/*`. Never drop
tables, purge journals or restore a backup over new player actions.
