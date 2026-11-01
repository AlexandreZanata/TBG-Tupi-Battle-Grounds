# Consolidation for main — 2026-11-01

**Status:** Draft candidate for a single PR to `main`; not accepted for merge.
Branch `codex/consolidate-linux-main`, built in an isolated worktree from the
latest PR heads fetched on 2026-11-01 (main `9008f117`). The owner merges and
watches CI; this front pushed the branch and opened one PR, nothing else.

Owner scope: consolidate every pertinent open front, resolve conflicts without
dropping features, optimize CI without reducing coverage, and prepare a Linux
prerelease; also generate a Windows executable and test it as far as Linux
allows. Policy: [AGENTS.md](../../AGENTS.md) and [VALIDATION.md](../VALIDATION.md)
(minimal fronts, targeted tests once at consolidation, no full suite).

## 1. Inventory and ancestry

Heads were re-read from GitHub, not from the request's snapshot. Ancestry was
computed with `git merge-base --is-ancestor` across all fifteen heads.
"Exclusive" counts commits not in `main` nor in any other PR head that is an
ancestor of it.

| PR | Head | Contains | Excl. | Exclusive content | State in its own report | Destination |
|---|---|---|---|---|---|---|
| #18 | `ab8e72a` | — | 5 | P57 client unit spacing (D-57) | owner look pending | via #29 |
| #20 | `3287410` | — | 57 | P59 unit shield, three Factions, Monsters (D-62–D-66) | balance W10 pending | via #29 |
| #21 | `0047830` | #20 | 16 | champion updates, crowd separation, Practice Shop preview, fortress emblem | protocol later superseded | via #29 |
| #22 | `1d27ae9` | #20 | 24 | all 21 character libraries, forms, abilities | numeric content Proposed | via #29 |
| #23 | `41b15d8` | — | 7 | five arenas toward approved references | owner look pending | via #29 |
| #24 | `c7af187` | — | 5 | P62 Step 0 and training kernel | continued in #29 | via #29 |
| #25 | `50e76ea` | — | 20 | P60/P61 durable ranked records v7, integrity shadow, stats v8 | enforcement inactive | via #29 |
| #26 | `3a95248` | #20 #21 | 67 | native Linux shared navigation, battle panels, loading | — | via #29 |
| #27 | `66eba9d` | — | 11 | P64 English/pt-BR completeness | — | via #29 |
| #28 | `89dd303` | — | 10 | P63 victory celebration, full lifetime cast | per-kind dances future | via #29 |
| #29 | `e26d4d4` | #18–#28 | 33 | Linux consolidation, native.2/native.3, CI efficiency | competitive acceptance open | via #31/#33 |
| #30 | `7ca4c36` | #18–#28 | 4* | P67 offline recovery/mute fixtures (test-only), commerce docs, owner validation policy | Rust batch never ran (lock busy) | merge `d084223` |
| #31 | `f706c91` | #18–#29 | 7 | competitive audit, illustrated patch notes, fail-closed CI aggregator, Shop test for 46 kinds | 18 audit rows fail (disclosed) | via #33 |
| #32 | `bd8c202` | #18–#29 | 11† | P68-A protected sign-in, protocol 30, auth schema 9, desktop UI contract, `v0.1.0-p68.1` | Windows pending | merge `f857811` |
| #33 | `e4cbd68` | #18–#29, #31 | 20 | P66 updater, journal/health, release slots, release schema 9, admission, protected resume | continuity blocked, O02 | branch base |

\* #30 forked from #29's earlier head `55510c5`; its 35 commits outside the
other heads are 31 inherited from that head plus 4 P67 commits.
† #32 forked from `014b0e0`; 6 of its 17 outside commits are #31 lineage
already present through #33, 11 are P68.

Integration order followed the real ancestry: start at #33 (it already
contains #18–#29 and #31), merge #32 (shared base `014b0e0`), then #30 (base
`55510c5`). Normal merge commits preserve all history; nothing was reapplied
or cherry-picked, and every PR head is an ancestor of the result.

## 2. Conflicts and decisions

`git merge` reported 23 conflicted files for #32 and 5 for #30. Each was
resolved by reading both sides, their consumers and tests; no global
ours/theirs. Decision record: [D-99](../DECISIONS.md).

| Area | Both sides | Resolution |
|---|---|---|
| Vault module | P66 had copied P68's `platform/credentials` (only the probe name differed) | one module; the P66 probe flag remains an alias |
| Auth vs resume | P66: resume record written by its own thread without P68's process lock; P68: one coordinator under a per-origin lock | P68 coordinator is the only vault owner; P66 record (now with family binding) is restored in its first restore with the 5 s visible wait and written as coalesced coordinator operations |
| Connect path | P66 admission/contract URL; P68 authenticated hello | both required in one `connect`; both regression tests kept |
| Auth routing | P66 slots each have their own process; P68 Access index is per process | clients send `X-TBG-Slot`; Caddy routes `/api/auth/*` by it; server refuses another slot's request; one forced renewal per refusal |
| Ownership | P66 claims cross-slot ownership on legacy WS login; P68 attaches accounts after HTTP auth | authenticated attach also claims ownership |
| Schema | both allocated v9 (release directory vs auth families) | v10 verifies both sets by shape, creates the missing one, fails closed on partial/foreign shapes |
| Protocol | P66 = 29 (no wire change); P68 = 30 | 30 retained after inspection: identical wire, unchanged gameplay; contract `w30-r2-3d56156049e626c5` |
| Registration tx | P66 immediate write lock; P68 shared issuance helper | helper kept inside an immediate transaction |
| Updates | P66 socket guard | plus `auth::idle_for_update` (restore, renewal, sign-out, any vault work) |
| Store worker | P66 extracted `serve`; P68 added `Auth` arm | arm moved into `serve` |
| Docs | P66 §19/§22/D-91/D-92, P67 §20/§23/D-93–D-95, P68 §21/§24/D-96/D-97 | all kept; consolidation adds D-99, GAME-DESIGN note, ARCHITECTURE §26 |
| Packaging | messages/staging creation differed | P66 refusal semantics kept; managed packages now ship the P68 license notices; build-info carries the contract |

Fixes discovered during integration (not conflicts): #33's `online` test still
used the removed `Shared::draining`; #30's fixtures had never compiled
(rusqlite 0.41 has no `usize` decoding; P68 changed `ConnHandle`/`hello`).
Both adapted without weakening an assertion.

## 3. Traceability to the final tree

| Feature | Origin | Final location | Preserved limits |
|---|---|---|---|
| Units take space; crowd separation | #18, #21 | `tbg-client/src/visuals/layout.rs`, `crowd.rs` | owner look |
| 46 champions, Factions, Monsters, forms, abilities | #20, #22 | `tbg-core/src/{content,ruleset,shop}.rs` and per-kind modules; client art kits | numeric content Proposed (D-33/W10) |
| Champion updates, Practice Shop preview, emblem | #21 | Shop UI and `assets/branding/` (emblem = app icon, Tupi wordmark in header) | preview stays Practice-only |
| Five arenas with training in each | #23, #29 | `tbg-client/src/arena/`, `ui/shell/arenas.rs` | owner look |
| Troops/training laboratory | #24, #29 | `tbg-core/src/training/`, `tbg-client/src/training/` | difficulty ordering unmeasured |
| Durable ranked records, integrity, stats | #25 | `tbg-server/src/ranked/`, `store/{ranked,stats}.rs` (v7/v8) | enforcement inactive |
| Native Linux UI: header, login, ranking, Settings, skeletons, HUD, Shop, reserve, Fortification, inspection, Augments | #26, #29 | `tbg-client/src/ui/{native_battle,native_loading,native_ranking,shell}/` | Linux is the visual reference |
| English/pt-BR | #27 | `assets/i18n/{en,pt-BR}.json` (1,445 keys each, same set) | — |
| Victory celebration | #28 | `tbg-client/src/celebration/`, `tbg-core/src/celebration.rs` | per-kind dances future |
| Competitive audit, illustrated patch notes | #31 | `tbg-balance`, `docs/patches/`, `ui/shell/patch_notes.rs`, `tbg-server/src/patch_notes.rs` | 18 audit rows fail, no balance change |
| Fail-closed CI aggregator, Shop test for N kinds | #31 (`f706c91`) | `scripts/ci-required-checks.py`, `tbg-core/tests/rules.rs` | 0.625/N floor (D-48 amendment) |
| Offline recovery and opponent mute | #30 | `tbg-server/src/store/recovery/` (`#[cfg(test)]`), `docs/modules/commerce/` | no SMTP/payment/fiscal/mute runtime |
| Protected sign-in, families, cookies, vaults | #32 | `tbg-client/src/auth/`, `tbg-server/src/auth/`, `store/auth_sessions.rs`, `assets/auth/` | Windows vault/visual gates |
| Desktop Linux/Windows UI contract | #32 | `docs/ui/`, `scripts/{check,test}-desktop-ui.py`, `tbg_desktop` cfg | shared code is not Windows proof |
| Updater, journal, health, receipts, floor, notes, Later/badge | #33 | `crates/tbg-update/`, `tbg-client/src/ui/update/` | O02 signing, no bundled root |
| Release slots, admission, ownership, drain | #33 | `tbg-server/src/release/`, `store/release*.rs`, `net/admission.rs`, `deploy/` | two-slot rollout inactive |
| Protected resume to original generation | #33 + D-99 | `platform/resume.rs` (format) + `auth/native.rs` (vault owner) | public continuity unproven |
| Combined schema and auth routing | consolidation | `store/schema.rs` v10, `auth/http.rs` slot check, `Caddyfile.template` | cross-slot revocation pending |

## 4. Visual contract and portrait audit

The Linux desktop stays the visual reference. P68's contract now routes the
refined presentation through the `tbg_desktop` target selector for Linux and
Windows; the source gate (`scripts/check-desktop-ui.py`) passes and is a
required CI lane. Shared code is **not** evidence of identical Windows
rendering: W07/W08 matched packaged captures on real Windows remain open.
Browser presentation was not redesigned; its shared flows were kept.

Shop portraits, audited read-only against the integrated models: the Shop
draws only base kinds from the 46-kind pool (`CANON.pool = FULL_POOL`), so no
Evolution form is ever offered. All 46 have distinct 400x400 renders of their
final models (`assets/battle/portraits/`, refreshed in `cd0944f`, Longaxe in
`6fb0fb7`); every kit's last model commit precedes that refresh and later art
commits changed only tests and one clip timing. **No capture was missing or
incompatible, so none was regenerated.** Fixes made: the browser cache
revision (`PORTRAIT_REV`) was bumped because Longaxe changed after it was set,
and the Node coverage test's parser now accepts the rustfmt-wrapped
`is_summon` (it had been failing outside CI; assertions unchanged).

Open, not changed here: the Longaxe crop cuts the axe head (owner look);
evolved forms reuse the base portrait in inspection and the never-sold Crate
summon has none; `docs/phases/PORTRAITS-shop-cards.md` still describes the
25-kind era. The unmerged P70 champion-polish front (not an open PR, edited
during this audit) rewrites seven models (Archer, Shieldbearer, Longaxe, Solar
Sentinel, Old Finn, Pyre Warden, Trident); their portraits must be recaptured
with `scripts/native-portraits.py` if P70 lands.

Wordmark and emblem: the Tupi wordmark remains in the shared header and the
emblem (`assets/branding/tbg-icon-512.png`, `tbg.ico`) remains the application
icon used by the AppImage; ranking-specific art under `assets/ranking/` is
unchanged. The local-only docs commit `a3ef8a8` on `phase/native-linux-windows`
(not part of any PR) was not included.

Known documentation debt carried forward: DECISIONS contains two records each
for D-87, D-88 and D-89 from earlier parallel fronts; they stay append-only and
are distinguished by title.

## 5. Protocol, schema and compatibility

| Identity | Value | Evidence |
|---|---|---|
| Wire protocol | **30** (P68's, retained) | P66 changed no wire type; `contract_protocol` 14/14 and `contract_protocol_server` 2/2 pass unchanged |
| Ruleset | 2 / `3d56156049e626c5` | `contract_ruleset_version` 2/2; patch-note gate |
| Match contract | `w30-r2-3d56156049e626c5` | `contract_release_identity` 2/2 (computed, never typed) |
| Server schema | **10** | `schema_upgrade` 4/4 (P66 v9, P68 v9, v8, partial/foreign refusal) |
| Native resume record | vault schema 2 (family binding) | `platform::resume` unit tests; P66 schema-1 synthetic records ignored |
| Client version | `0.1.1-rc.1` | above `0.1.0-p68.1` and `0.1.0-native.3` in SemVer order |

Compatibility: protocol-29 builds (production server, native.2/native.3,
P66 previews) get the readable VersionMismatch from this server and this
client gets an update notice from them; Practice and the laboratory stay
offline-capable. `v0.1.0-p68.1` speaks the same wire and gameplay, so by code
inspection (not exercised) it stays compatible with a server built from this
tree: it sends no P66 contract parameter and is admitted on the legacy `/ws`
path. Old
binaries open a v10 database (their own migration chain sees no missing
version); a v10 database is never downgraded. Bots (`tbg-server` botclient)
use the native HTTP routes; the browser bundle uses the cookie coordinator and
now sends the slot header. The balance lab does not use the wire.

## 6. Validation actually run (consolidation level)

Policy: [VALIDATION.md](../VALIDATION.md) — targeted tests once on the
integrated tree, shared nonblocking lock `/tmp/tbg-validation-1000.lock`,
`CARGO_BUILD_JOBS=2`, `--test-threads=1`. No full workspace suite was
requested or run.

**Shared-cache hazard found.** The first client test build in the shared
`target/` linked a `tbg-core` built from another checkout: Cargo's metadata
hash ignores the checkout path for workspace members and freshness is decided
by source mtimes, so worktrees sharing one target directory can reuse each
other's artifacts. All results below come from a dedicated target directory
(`/data/tbg-consolidate-target`, cold build). Earlier fronts that relied on the
shared cache should treat cross-worktree results with care.

Commands (binaries from one `cargo test --offline --locked -p tbg-core
-p tbg-server -p tbg-client -p tbg-update --no-run` build, each run from its
package directory with `--test-threads=1`):

| Target | Result |
|---|---|
| tbg-client unit tests (auth, net, resume, update guard, UI) | 162 passed, 2 ignored |
| tbg-server lib unit tests (lobby incl. P67 recovery, match task, auth slot guard, release, store) | 58 passed |
| tbg-core `contract_protocol`, `contract_protocol_server`, `contract_release_identity`, `contract_ruleset_version`, `rules` (BR-S04 Shop for 46 kinds) | 14, 2, 2, 2, 14 passed |
| tbg-server `accounts`, `arcanas`, `auth_http`, `auth_sessions`, `auth_websocket`, `contract`, `contract_challenge`, `contract_registration` | 12, 3, 6, 3, 4, 3, 2, 2 passed |
| tbg-server `leaderboard`, `live_api`, `online` (incl. P66 drain), `spectate`, `arenas`, `lab_equivalence` | 4, 3, 32, 6, 2, 1 (+1 ignored) passed |
| tbg-server `p67_presentation`, `patch_notes`, `ranked`, `ranked_admin`, `ranked_integrity`, `ranked_recovery`, `ranked_stats`, `ranked_stats_recovery` | 2, 1, 6, 4, 7, 4, 6, 2 passed |
| tbg-server `schema_upgrade` (new) | 4 passed |
| tbg-server `release_slots` | first run **2 failed** (U13/U15 used legacy WS sign-in refused by protocol 30); after adaptation 5 passed, 1 ignored (external probe) |
| tbg-update `updater` | 10 passed |
| Node: `auth-session.test.cjs`, `portraits.test.js` | 3, 2 passed (portraits failed before the parser fix) |
| Python: desktop UI source gate and comparator tests, CI required-check tests (9), lane-split tests (13), patch-note tests and gate | passed |
| Toy workspace with the exact split `cargo test` flags | lanes rc=0; negative case rc=101 as required |
| `cargo fmt --check` (client, server), `git diff --check` | clean |
