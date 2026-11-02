# Main authentication and P71 integration — 2026-11-01

## Scope and authority

The owner requests merging current GitHub `main` into the character/arena branch
and deploying the complete backend to their reset VPS. The owner subsequently
said they will supply VPS settings later. Finish integration independently;
remote installation waits for those settings and trusted host identity.

Base: P71 `9da3ff9920c21ac260d6beef54d1200179a023ea`.
Incoming main: `027fd61` (PR #34, consolidated P66/P67/P68).
Branch: `phase/backend-main-integration`; normal merge, no rewritten history.

## Reconciliation

- Keep all P71 character changes, independent Tick, eight Arenas, signed Arena
  Energy offsets, procedural scenery, thumbnails, crowd bounds and Evolution UI.
- Keep main's D-97 access/refresh families, secure platform storage, same-socket
  renewal, authenticated resume, schema 10, P66 admission/update/release slots and
  D-99 single native credential coordinator. No auth lifetime changes.
- Both heads allocated protocol 30 differently. Use protocol 31; preserve all
  existing message tags and Arena IDs. Keep both protocol regression additions.
  GAME-DESIGN and DECISIONS record the combined epoch in this same merge.
- Main adds no watched gameplay edits over this branch: ruleset 6 and its
  append-only source registry remain unchanged, subject to the targeted check.
- Capture conflict keeps P71 opt-in tools under main's shared desktop routing.
  Action conflict keeps persistent Evolution focus and main's desktop handling.
- Fresh-host preparation sets the deployed auth origin for both service modes,
  installs sudo on minimal Debian and excludes local-only evidence from transfer.
  Deployment builds use two jobs and each host's compilation lock.
- The native auth harness reads the client build identity instead of hardcoding
  protocol 30. Added a real HTTP/WS scenario renewing Access and reconnecting to
  the same seat, result and server-assigned Arena.

## Validation (targeted consolidation)

All commands ran on the resolved merge tree, using the existing shared cache,
`CARGO_BUILD_JOBS=2`, `/tmp/tbg-validation-1000.lock` and one test thread. The first
nonblocking attempt returned 75. The owner's standing “Aguardar neste e nos
próximos ajustes” exception was used to wait without interrupting another build.
Sources in this worktree had mtimes refreshed while holding the lock to avoid
reusing another worktree's stale workspace artifacts; no cache was deleted or
fresh target directory created.

Commands actually run (common prefix: `cargo ... --offline --locked`):

- `test -p tbg-core --test arenas --test contract_protocol --test
  contract_protocol_server --test contract_ruleset_version --test
  contract_release_identity -- --test-threads=1`: **32 passed**.
- `test -p tbg-server --test auth_http --test auth_sessions --test auth_websocket
  --test arenas --test schema_upgrade --test release_slots -- --test-threads=1`:
  **25 passed**, including the new refresh/reconnect/Arena scenario. The first
  sandboxed attempt failed opening loopback sockets (`Operation not permitted`);
  the corrected host execution passed. This was an execution restriction, not a
  dismissed test failure.
- `test -p tbg-client --bin tbg-client auth:: -- --test-threads=1`: **8 passed**.
- Same client command with `net::`: **13 passed**; with `arena`: **11 passed**.
- `test -p tbg-server --test patch_notes -- --test-threads=1`: **1 passed**.
- `check -p tbg-client --target wasm32-unknown-unknown`: **passed**, 5 dead-code
  warnings in platform-only auth and opt-in gallery helpers; no WASM errors.
- `cargo fmt -p tbg-core -p tbg-client -p tbg-server`; `git diff --check`;
  `bash -n deploy/deploy.sh deploy/provision.sh`: passed.
- `python3 scripts/check-desktop-ui.py`: passed after routing P71 capture helpers
  through `tbg_desktop`; this is source routing, not Windows visual acceptance.
- `python3 scripts/check-patch-notes.py` and `python3 scripts/test-patch-notes.py`:
  passed (5 public entries; 5 Python tests). The initial source gate correctly
  exposed the missing revision-6 note, which was added with Proposed limits.
- `node --input-type=module --check < assets/auth/session.js` and `ui.js`, plus
  `node --check assets/battle/hud.js` (paths under `crates/tbg-client`): passed.
  Plain `node --check` had initially treated the auth modules as CommonJS;
  explicit module parsing corrected the invocation without source changes.
- Source identity calculation and locale JSON parity: `w31-r6-097385fee9a10b0f`,
  **1,451 equal locale keys**. No watched gameplay source changed in this merge.

Total scoped Rust scenarios: **90 passed**, without a workspace suite. Native
installation and backend executable identity are recorded below when available.
No full suite, balance gate, load test or Windows acceptance was requested.
Existing balance and visual approval limits in the source reports remain open.

## VPS status

Strict SSH host verification refused the post-reset host before authentication:
old known-host identity differs from the new ED25519 fingerprint. No known-host
entry was removed and no remote mutation occurred. The owner will supply settings
later, including trusted host identity and domain. Debian 13 provisioning, public
TLS, browser cookie flow and live backend health therefore remain pending.

### Merge whitespace inspection

The staged merge's whole-tree whitespace check reports CRLF/trailing spaces in
verbatim third-party license copies already present on main. They are preserved
byte-for-byte (`git diff origin/main -- deploy/native/licenses` is empty).
`git diff --cached --check -- . ':(exclude)deploy/native/licenses/**'` passes for
all source, documentation and integration edits. No license text was reformatted.

## Delivery and pending native packaging

Integrated merge commit: `f35d9c8c09ae51e5e8a337287cadbf3de26f9fe2`, pushed to
`origin/phase/backend-main-integration`. Both `027fd61` (main) and `9da3ff9`
(P71) are ancestors. The execution tree was clean after that commit. Main itself
was not modified remotely. No tags, releases or signed artifacts were published.

> blocked: Native executable generation was terminated twice with exit 143
> (SIGTERM), without a Rust compiler error. First command: `cargo build --offline
> --locked -p tbg-client -p tbg-server --bin tbg-client --bin tbg-server --bin
> tbg-release-admin` under the shared lock. A separate-package retry under that
> same lock also received 143 while waiting for the compilation slot. A host
> inspection showed the unrelated Android front owning the lock; it was preserved
> and was not interrupted. The sender of the termination signal is unknown.

Per AGENTS' repeated-error rule, stop this packaging step after the second
termination. No frozen candidate or new Linux installation was created. Keep the
previous native installation (Shatterback source `ce0e406`, protocol 30), desktop
launcher, running windows and player data untouched. The old client is incompatible
with the integrated protocol-31 backend and must be updated before testing it.
Native test compilation passed; that does not establish an installed executable.

Exact remaining commands, using the existing target cache, two jobs and the host
lock: `cargo build --offline --locked -p tbg-client --bin tbg-client`, then
`cargo build --offline --locked -p tbg-server --bin tbg-server --bin
tbg-release-admin`. Stamp `TBG_SOURCE_COMMIT` with the integrated runtime commit,
verify the frozen client's `--build-info` identifies protocol 31 and contract
`w31-r6-097385fee9a10b0f`, then install that exact client plus assets/notices in a
new version directory and atomically update the local launcher.

The prepared synthetic executable HTTP smoke check and one isolated Clockwork
Practice capture were **not run**, since they require that frozen candidate.
Actual Debian 13 release compilation/provisioning, TLS, browser cookie acceptance,
public health and a live remote reconnect await the owner's VPS settings. None
of those deferred steps is reported as passing. No release or broad load build
was run locally.
