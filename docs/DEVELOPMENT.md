# Development guide

The authoritative rules and numbered contracts are in [GAME-DESIGN](GAME-DESIGN.md).
Read [AGENTS](../AGENTS.md) and [VALIDATION](VALIDATION.md) before validation. Use
one isolated worktree per front; preserve other worktrees and running builds.
Code, comments, commits and engineering documentation are English. Product text
uses the existing English/pt-BR catalogs. Mobile remains outside this consolidation.

## Shared clients and service boundaries

`tbg-core` owns deterministic combat, Shop, Energy, Evolution and the wire contract.
`tbg-server` owns connections, authentication, outcomes, ranked settlement and SQLite.
The browser and native Linux/Windows clients use the same Bevy desktop presentation
(`platform::desktop_ui`); do not add a second HTML implementation of its screens.
[AGENTS non-negotiable 8](../AGENTS.md#non-negotiables) makes Linux the canonical
reference for every in-game visual change. Windows and desktop web must match it
exactly, including every screen, loading/transition state, champion/form, animation,
VFX and visible behavior. Every visual change updates the
[correspondence contract](ui/DESKTOP-UI-CONTRACT.md), affected scenarios/motion and
shared presentation together. Shared code or a successful build is not parity
acceptance; record per-target evidence and deferred checks under VALIDATION.

The landing and public admin are independent static sites under `web/`.
`web/admin/app` is a local preview with fictional fixtures, not an authenticated
production console. Only the script-free `web/admin/public` placeholder may ship.
[The web guide](../web/README.md) explains manifests, i18n and preview commands.
Never expose operator status, privileged actions or mock credentials as real admin.

## Local work

Use the compiler pinned by `rust-toolchain.toml`; install the WASM target through
rustup. Native development additionally needs the Linux window/audio dependencies
listed in [native packaging](../deploy/native/README.txt). Keep credentials in the
real OS vault; `Remember me` must never fall back silently to a plaintext file.

Small checks, selected for the changed area:

```bash
cargo fmt -p tbg-client -- --check
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo check --locked -p tbg-server --bin tbg-server
python3 web/scripts/check.py
python3 scripts/check-patch-notes.py --base origin/main
git diff --check
```

Do not run all of these by habit. Exit 75 means defer; do not retry, raise jobs,
clean caches or select a fresh target directory. Targeted tests run once at
consolidation; the full suite and expensive simulations require an explicit request.

To run a previously built server with an isolated development database:

```bash
TBG_ADDR=127.0.0.1:8080 TBG_DB=/tmp/tbg-development.sqlite3 \
  TBG_AUTH_ORIGINS=http://localhost:8080,http://127.0.0.1:8080 \
  TBG_STATIC_DIR=crates/tbg-client/dist target/debug/tbg-server
```

Use a free port; never overwrite a running front's database or terminate its server.
For a browser change, build the WASM target under the same lock. A plain local
Trunk build serves `/`; deployment builds require the immutable public URL described
in [RELEASES](RELEASES.md). `scripts/install-web-build-tools.sh` verifies official
pinned tool archives before installing into its CI directory.

## Compatibility and migrations

Never copy a protocol number from an old phase. Read the actual core contract and
all consumers. `cargo run --locked -p tbg-core --example match-contract` prints the
canonical ID (compile under the lock). This integrated line is protocol 31, ruleset
6 / `097385fee9a10b0f`, SQLite schema 10. Previous protocol-30 clients are incompatible.

Schema 10 recognizes both prior schema-9 table sets and verifies their shape before
completing migration. A matching integer alone is not migration evidence. Review
`schema_upgrade`, `auth_websocket`, `revocation_feed` and `match_continuity` integration
tests when touching these contracts. Keep stable account identity, credential
rotation, seat ownership, result receipts and the single ranked writer intact.

Server changes reach players by promotion into the idle release slot while the other
slot keeps running its matches, so both generations must read and write the same
database and speak the same contract: schema changes are additive (expand first), and
a wire/gameplay contract change is a separate coordinated release the automation refuses.
Anything that lives in one process's memory (queues, rooms, invitations, rematch offers,
spectator lists) does not follow a promotion; `lobby_waiting` and the drain refusal in
`lobby/matchmaking.rs` exist for that, so extend them with any new waiting state.
`python3 scripts/rehearse-slot-release.py --help` rehearses a promotion locally with real
processes and bots; run it when touching `release.rs`, the lobby or `deploy/auto/slots.py`.

Email registration/recovery and provider integrations remain P67 offline contracts
and fixtures. Live account creation still uses a name and generated key; merging
documents does not implement SMTP, deliver email or activate commerce. Document any
such future integration and obtain the required owner/provider inputs separately.

Gameplay edits require their Proposed canon entries, source fingerprint and bilingual
patch notes; balance values additionally require D-33 evidence. Use the existing
[patch process](patches/README.md), never edit a contract test to hide a failure.

## Desktop versions

A pull request that changes shipped desktop inputs (client/core/update crates outside
tests, `Cargo.*`, `.gitattributes`, `rust-toolchain.toml`, licenses, packaging scripts,
`deploy/native/**`, `docs/patches/**`) must bump `crates/tbg-client/Cargo.toml`
(and `Cargo.lock`) to an unpublished SemVer and title `deploy/native/RELEASE-NOTES.md`
with it. `python3 scripts/check-native-version.py --base origin/main` runs the same
check locally; CI fails early otherwise. Its merge publishes that version
automatically; see [RELEASES](RELEASES.md).

## Review and handoff

Record the source revision, changed scope, actual commands/results, known failures
and exact deferred checks. A native check is not browser evidence. Upload only
pertinent source changes; no builds, databases, credentials, reference downloads or
other fronts. Main's required CI remains fail-closed. Normal merging preserves the
verified deployed source ancestry; a squash/rebase needs explicit floor reconciliation.
See [RELEASES](RELEASES.md) before publishing and [SCALING](SCALING.md) before capacity changes.
