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

`docs/ui/DESKTOP-UI-CONTRACT.md` has a P73 change record and
`desktop-ui-contract.json` lists eight new screens. One shared Bevy
implementation serves Linux, Windows and web; the timestamp adapter differs
internally (Jiff system zone vs browser `Intl`) but renders the same text.
`python3 scripts/check-desktop-ui.py` passed. Windows, browser and final-source
Linux captures are **pending**; parity is unverified.

## Validation

### Run

- `git diff --check`: clean at every commit.
- `cargo fmt -p tbg-core -p tbg-server -p tbg-client --check`: exit 0.
- `python3 scripts/check-desktop-ui.py`: passed (source routing only).
- i18n catalog key/placeholder parity script: 1549/1549, no mismatch.
- Caddy validate + routing probe: see *Proxy and privacy*.

### Deferred

All compiling checks are deferred: **the shared validation slot was busy**
(`flock -n` exit 75; another front's champion preview capture held it).
Not retried in a loop, bypassed or waited on. Run once, in this order,
from the worktree with a private target (the main/consolidation targets
belong to other fronts; sharing them links stale workspace crates):

```sh
cd /data/tbg-consolidate-open-fronts
export CARGO_TARGET_DIR=/data/tbg-open-fronts-target
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" env CARGO_BUILD_JOBS=2 bash -eu -c '
  cargo test --offline -p tbg-core --test contract_protocol --test contract_protocol_server -- --test-threads=1
  cargo test --offline -p tbg-server --lib lobby::player_challenges::tests -- --test-threads=1
  cargo test --offline -p tbg-server --lib match_task::summary::tests -- --test-threads=1
  cargo test --offline -p tbg-server --test player_challenges --test player_match_history \
    --test history_privacy --test presence_global --test live_api --test contract_challenge \
    --test schema_upgrade --test ranked_admin --test ranked_recovery --test ranked_stats \
    --test ranked_stats_recovery -- --test-threads=1
  cargo test --offline -p tbg-client --bin tbg-client p73_ -- --test-threads=1
  cargo check --offline -p tbg-client --bin tbg-client --target wasm32-unknown-unknown
  cargo build --offline -p tbg-client --bin tbg-client --features ui-acceptance
'
```

Native client commands also need the ALSA pkg-config path documented in the
P73 report. Also deferred: browser bundle and browser screenshots, native
final-source captures, Windows captures, `scripts/test-desktop-ui.py`
(Pillow absent locally; CI `desktop-ui` runs it), the real two-client
invitation/rematch/reconnect scenario and every acceptance item listed in
the [P73 report](../planning/P73-EXECUTION-REPORT.md#blocked-and-deferred-validation).

## Rollout and rollback

Merging this PR changes the wire contract to `w32-…`. By design the
backend auto-deploy controller then refuses the candidate (`contract change
staged: coordinated client release required`), the web controller refuses
a `w32` browser candidate while the live server is `w31` (`browser contract
transition/drain requires coordinated release`), and `native_pull.py` stops
offering the `w31` desktop builds once the server is `w32`. Do not loosen those guards. A coordinated release needs: a
backend candidate and the matching web client together, then a native build
of the same source before downloads are offered again. Timers stay disabled
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
