# P66 — Native updater and match-preserving release infrastructure

> **Planning only — 2026-10-31.** Owner request: professional Linux updates
> without uninstalling, a Windows path, automatic in-game startup prompts,
> durable post-update notes, and independent client/backend releases that
> preserve running matches. No code, key generation, installation, signing,
> deployment or capacity experiment is executed by this document.
> New product policies/limits are **Proposed** in GAME-DESIGN §19.
> Planning review (2026-10-31) added requirements traceability (§2.1),
> prompt cadence and the notes content contract (§3.1, §3.4), contract
> transitions and computed contract identity (§6.4–§6.5), independently
> shippable milestones (§8) and a risk register (§10).

Planning branch: `codex/p66-native-updates-release-safety-plan`.
Execution branch: `codex/p66-native-updates-release-safety`.
Parent planning commit: `b2a12c65ce88a4b14f99ba92500641a4ce5eb2f7`.

## 1. Read first and evidence

Read [AGENTS.md](../../AGENTS.md), current `docs/VALIDATION.md`, then
[release contracts and test matrix](../planning/P66-RELEASE-CONTRACTS.md),
[executor prompt](../planning/P66-EXECUTION-PROMPT.md), GAME-DESIGN §19,
ARCHITECTURE §22 and DECISIONS D-91/D-92. Read P60's durable-result contract
if integrated, P64 i18n and the current native phase/install documentation.
Do not read or change `archive/`.

Audit baselines: main-derived native work front `a3ef8a8` and native desktop
snapshot `3a952485b01b78baa070cda82c465905a552ab7a`. This planning tree inherits
main `9008f117` code and the later planning documents, not those native code
changes. Refresh all conclusions on the real integration SHA.

- `scripts/install-linux.sh`: per-user versioned installation and symlinks;
  the desktop entry targets a version-specific binary directly. Native
  preview adds commit suffixes. A future stable launcher must migrate both
  command shortcuts and desktop entries, not just update a symlink.
- `scripts/package-native.py`, `package-appimage.sh`, `deploy/native/*`:
  portable archive/AppImage/build provenance and SHA-256 exist. A checksum
  downloaded beside the package is not a trusted publisher signature.
- `.github/workflows/native-release.yml` on the native branch builds Linux;
  Windows packaging is not proof of Windows installation/update acceptance.
  It downloads a continuous linuxdeploy artifact: pin trusted tooling and
  action revisions/digests in the release pipeline before granting signing.
- `tbg-server/src/version.rs`: `/api/version` exposes a compile-time latest
  string/current protocol and release-page links, not a signed update feed.
- Client `ui/shell/data.rs`, `platform/http.rs`, `net/socket.rs`: startup
  discovery/notice and download navigation exist. The native HTTP helper
  buffers a JSON response; do not reuse it unchanged for large archives.
- Native `platform/storage.rs` keeps Scope::Session in memory; `config.rs`
  uses that scope for reconnect tokens. A native crash/restart currently
  lacks the durable resume context needed by the requested update workflow.
- `tbg-server/src/main.rs`: SIGTERM drain stops waiting after 300 seconds;
  `deploy/tbg-server.service` has TimeoutStopSec=330. Neither preserves an
  unbounded match. There is no game-rule time limit to justify that cutoff.
- `match_task.rs`: running-match metric is decremented before asynchronous
  ranked settlement completes. Zero running matches alone is not safe exit.
- `deploy/deploy.sh`: rebuilds/copies assets with deletion and restarts one
  server. It is not a match-preserving rollout. Caddy currently routes one
  upstream and config reload can close WebSockets; see primary references.
  This is the repository template, not a verified live proxy inventory; an
  Nginx/other deployed ingress needs its own proven connection-drain behavior.
- Protocol equality exists, but release version, protocol, content/rules,
  assets, account/session ownership and DB compatibility are different axes.

## 2. Required outcome and realistic service contract

Linux first: startup checks asynchronously, shows an update modal at a safe
idle point, downloads/verifies/stages the selected release, then lets the
user update and relaunch without uninstalling or losing data. A durable
receipt shows the installed release's notes after successful activation,
including after closing/reopening. Windows implements the same contract with
platform-specific file/process behavior, after native Windows acceptance.

Routine releases must not restart a playing client, migrate a live match's
simulation, swap its rules/assets, force-close its socket, or terminate its
server to meet a deployment timer. Client-only and backend-only releases
are supported with an explicit compatibility/admission contract.

This guarantee covers **controlled planned release operations**, not arbitrary
host/network failure, power loss, malicious clients or urgent incident
containment. Existing in-memory matches cannot survive a dead worker merely
because a load balancer retries a socket. Match checkpoint/replay failover is
a later recovery project; no claim of seamless crash migration is made here.

“Professional” means signed artifacts, recoverable activation, pinned
compatibility, reliable results, observable drain and measured capacity.
Do not introduce Kubernetes, service mesh, Kafka, global Redis or multi-region
operation just to serve the initial game. Prepare boundaries that can scale.

### 2.1 Requirements and traceability

The owner's requirements (R) and the quality attributes (Q) below are the
acceptance contract. Each row names where the design answers it and which
cases of the [release contracts](../planning/P66-RELEASE-CONTRACTS.md#4-focused-verification-matrix)
prove it. A row without passing evidence keeps its milestone open.

| ID | Requirement | Design | Rule | Evidence |
|---|---|---|---|---|
| R1 | Opening the Linux game shows an in-game update modal automatically when a newer eligible build exists | §3.1 | BR-UP01, BR-UP08 | U01, U09, U20 |
| R2 | Update in place: no uninstall/reinstall, no sudo, user data kept | §5.1–§5.2 | BR-UP01, BR-UP03 | U05–U07 |
| R3 | After updating, and again after a close or crash before acknowledgement, a "what's new" modal shows the installed release's notes | §3.2, §3.4 | BR-UP04, BR-UP10 | U10, U21 |
| R4 | Windows gets the same experience once verified on real Windows | §5.3 | BR-UP01 | U08 |
| R5 | Client-only and backend-only releases ship independently | §6.1, §6.5 | BR-UP05 | U12, U19 |
| R6 | No planned client or backend release disconnects, re-rules or slows a running online match | §3.1, §6.2–§6.3, §7 | BR-UP02, BR-UP05, BR-UP06 | U09, U11, U13–U16, U18 |
| R7 | Public matchmaking never splits silently across incompatible builds | §6.4 | BR-UP09 | U22 |
| R8 | The infrastructure can grow toward thousands of players in measured steps | §7 | BR-UP07 | U18 |
| Q1 | Integrity: only publisher-signed releases execute; the version API cannot grant trust | §4 | BR-UP03 | U02–U04 |
| Q2 | Recoverability: any interruption lands on a complete known-good build | §5.1 | BR-UP03 | U05, U06 |
| Q3 | Performance: no update network/disk work in the render loop or during online play | §3.1 | BR-UP02 | U01, U09 |
| Q4 | Privacy: no secrets in URLs, arguments or logs; aggregate-only update telemetry | §3.3, §7 | — | U11, U20 |
| Q5 | Operability: observable rollout, drain and update health; rollback without killing matches | §6.2, §7 | BR-UP06 | U14, U15, U17, U20 |
| Q6 | EN/pt-BR, accessibility and D5 glyphs on every updater and notes surface | §3.2, §3.4 | BR-UP04, BR-UP10 | U10, U21 |

## 3. Player flow and updater lifecycle

### 3.1 Startup and in-game modal

Boot the existing usable client immediately; fetch bounded signed update
metadata in a background worker outside Bevy's render/simulation loop.
States: Checking → Current/Available → Downloading → Verifying → Staged →
WaitingForIdle → Applying → LaunchingCandidate → Healthy/Recovered/Failed.
Persist transaction states, not merely an “update available” boolean.

When a newer eligible build exists, display localized version, download size,
notes and Update now / Later. Manual checking remains in Settings. Detect
offline, insufficient disk, unsupported install mode, invalid metadata and
retryable download failure with actionable messages. Failure to check updates
does not invalidate a compatible installed game. No raw English errors/HTML.

Do not open a blocking modal over an active match, reconnect attempt,
spectating or time-sensitive challenge/rematch. Queue notifications until idle.
Downloads pause by default during online play to protect bandwidth/I/O;
resume after idle. Verification/extraction also waits until idle. Updates do
not automatically cancel a queue: offer an explicit Leave queue and update.

Before Apply acquire a local update/launch lock, disable new match actions,
and confirm no pending join/seat/reconnect transaction. This is one shared
client transition, preventing the click-Update/click-Play race. Exit the
client cooperatively, preserving settings and any valid resume context.
Never kill a game process to free locked files. Installing new files is not
permission to replace a currently playing client's runtime or asset folder.

A required newer build blocks **new** incompatible admission with explanation,
not existing match completion. Release policy must define exceptional security
revocation separately (O03); do not hide forced disconnects as normal updates.

Prompt cadence (Proposed, BR-UP08): the startup modal appears at most once
per launch for a given build. **Later** leaves a persistent non-modal badge
in the lobby and Settings; idle rechecks may refresh the badge or show a
short toast, never a second modal in the same session. A *required* update
(a new public match contract, §6.4, or a raised security floor) blocks only
new online admissions: Practice vs Bot, Settings, installed notes and other
offline surfaces keep working on the installed build, and every online entry
point shows the localized reason next to the Update action. Whether eligible
updates download automatically while idle in the lobby is an owner default
(O05); verification, extraction and activation always wait for idle.

### 3.2 Update applied and “what changed”

Receipt fields include transaction ID, old/new build IDs, platform/install
root, verified target identity, activation state and health acknowledgement.
The candidate reports Healthy only after its owned assets, preferences and
safe initial UI are ready; successful spawn/PID or `/health` is insufficient.
No server login, new account or online match is needed to prove boot health.

On healthy start, queue the exact running build's localized release notes.
The modal remains pending until user acknowledgement is durably recorded.
Crash/close before acknowledgement shows it again on the next safe launch;
this is at-least-once presentation, not an impossible exactly-once screen.
Never show “updated successfully” when rollback returned to the old build.

Notes are a signed/bundled target linked to the binary release, available
offline. P65 news may link to them later but is not an update trust source
or mandatory availability dependency. Show skipped-release highlights in a
bounded history, with full installed-release notes accessible in Settings.
Do not display whatever a mutable latest URL says if another release has
arrived since this installation. Preserve locale and reduced-motion settings.
The notes themselves follow the content contract in §3.4.

### 3.3 Resume after crash or voluntary close

Persist a minimal protected resume record: generation endpoint, compatibility
tuple, match/session identifiers, reconnect credential and owning installed
build. No token in URL query, command-line args, public logs or telemetry.
Use OS credential storage when available; keep nonsecret metadata separate.
An unavailable secure store disables durable secret persistence explicitly
instead of silently copying account keys into another plain preferences file.

On relaunch, resolve a still-active match against its original generation
and compatible retained build **before** installing/showing modal updates.
An expired/finished match clears the resume record safely; result receipts
and pending notes remain available. A staged update cannot erase the old
client needed for reconnect. Exercise account takeover/second-device behavior
with the server's single-session policy and generation ownership checks.

### 3.4 Release notes content contract

Notes are authored in the repository next to the release (path fixed at
Step 0, for example `release-notes/<release_sequence>.json`) and reviewed in
the same PR as the change they describe. The pipeline turns them into the
signed notes target; nobody edits notes on a server.

- One structured file per release: `title`, UTC date and ordered sections
  (`highlights`, `gameplay`, `fixes`, `known_issues`) of short plain-text
  items. No HTML, Markdown, images, scripts or remote resources; the client
  renders text nodes or Bevy `Text` only. Links, if any, are limited to
  allowlisted project HTTPS URLs and open in the system browser on click.
- English and pt-BR are both mandatory with matching section/item counts,
  and every value passes the P41 D5 charset check used for the catalogs (the
  native UI font has no other glyphs). A missing locale, an oversize file or
  an unknown field fails the release build before signing, like catalog parity.
- When the computed ruleset/content identity (§6.5) differs from the previous
  stable release, a non-empty `gameplay` section is mandatory, so balance and
  rule changes are never silent to players.
- Skipped releases contribute their `highlights` to the bounded history; the
  full notes of the installed build stay available in Settings.

## 4. Artifact trust and release metadata

Use an established TUF client/repository implementation rather than inventing
signed JSON cryptography. Recommended Rust candidate: `awslabs/tough` and
`tuftool`; validate current maintenance/MSRV/transport/dependencies on the
actual workspace and pin compatible versions at Step 0. No Tauri/Electron
migration is justified for a Bevy game. Isolate updater I/O in its own small
native crate/helper; never add it to tbg-core or the authoritative tick loop.

Bundle trusted root metadata in the bootstrap installation. Verify the
role chain, target lengths/digests, expiration, monotonic metadata versions,
rollback/freeze protections and root rotations through the library. Persist
trusted metadata safely. Clock anomalies yield a clear failure, not disabling
expiry. HTTPS alone and an unsigned `.sha256` are insufficient publisher trust.

Separate signing from the game server, P65 news publishing and general CI
PR jobs. Offline root keys with an owner-approved threshold/backup ceremony;
protected release signing for validated targets; no signing credentials in
git, game binaries, job logs or ordinary build artifacts. Build from the
reviewed commit, create immutable artifacts/provenance, then sign and promote
metadata atomically after all targets/notes are available. Use channel
permissions, protected environments and least-privilege release jobs.

Harden the build that feeds signing: pin actions and tools by commit or
digest, build with `--locked`, gate on dependency advisories and licenses
(`cargo deny`/`cargo audit`), embed the dependency list (`cargo auditable`)
and publish an SBOM plus build provenance (for example SLSA/GitHub artifact
attestations) as additional signed targets. Aim for reproducible builds:
rebuild each release in a second clean job and compare digests; Step 0
measures whether bit-for-bit equality is achievable and records the causes
of any difference instead of silently ignoring them. These are pipeline
tools, not runtime dependencies of the game.

Release descriptor is itself a signed target. It names build ID/sequence,
human version, channel, OS/architecture/package, OS/runtime minima, archive
length/hash, content/asset IDs, compatibility tuple, updater ABI, state schema,
localized notes target, rollout policy and security floor. Never use SemVer
ordering alone or permit one tag/version to silently change artifact bytes.
`/api/version` may advertise availability for legacy UI but cannot authorize
execution or supply a new trust root.

Start with stable and explicit opt-in preview channels. Rollout assignment
uses an installation-local nonsecret ID or deterministic bucket, not account
keys/IP addresses; no updater analytics system is required. Integrity and
compatibility must pass regardless of rollout bucket. Do not follow arbitrary
redirects or path/host supplied by a custom game server; allowlisted HTTPS
delivery plus trusted target verification controls downloads.
Serve metadata/immutable targets through a static repository/cache/CDN path,
initially compatible with existing release hosting, rather than a GitHub REST
request or a game-server archive stream per player launch. Bound startup
conditional fetches and jitter subsequent checks/retries; signing/expiration
checks still apply to cached data. Artifact egress must not consume the
match server's bandwidth budget when thousands of installations check in.

Download size: start with one full compressed archive per release (simple
and always valid). Measure what actually changes between real releases;
when most bytes are unchanged assets, move to per-file content-addressed
targets listed by the signed descriptor, reusing unchanged files from the
verified current release after hash checks and downloading only the rest.
The full archive stays the fallback and the repair path. Binary diff patches
are deferred: they need the exact previous build and add a second failure
mode for little gain over file-level reuse.

Authorized operational rollback uses a newer metadata version pointing at
a known retained artifact, respecting the signed security floor. Local boot
failure can revert to a previously verified installed build only if it is
not revoked; network replay of old metadata is never accepted as rollback.
Keep trusted anti-rollback metadata when falling back to an older game build.

## 5. Installation architecture and platform policy

### 5.1 Stable launcher, side-by-side releases and crash recovery

Install layout: stable launcher, immutable release directories keyed by
build/platform, transaction journal, current/previous selection and staging.
Preferences/keys/replays/logs remain in the existing user-data location,
outside installed release directories. A signed upgrade cannot request an
arbitrary install/data path, run a shell command or erase user data.
Local preference/journal schemas have explicit reader/writer compatibility.
Use additive changes or recoverable snapshots before migration; a candidate
cannot irreversibly upgrade data that the retained fallback cannot read.
Never include account secrets in diagnostic backups or release artifacts.

Recommended bootstrap/helper split: a small stable launcher resolves the
selected release; a versioned updater/helper does bounded network/staging/
activation, supervised by the launcher. Helper/bootstrap ABI is explicit.
Bootstrap replacement is a separate reviewed transaction, not an assumption
that an executable can overwrite itself on every OS. Keep a manual signed
repair path if the bootstrap is too old for a new target.
Launcher/game/helper IPC is per-user and transaction-bound, using protected
local sockets/named pipes with OS peer/permission checks and a fresh challenge.
Health acknowledgements bind the expected build/transaction, not an arbitrary
PID or unauthenticated localhost port. No account/session secret in arguments.

Stream to a private staging directory with size/disk/time bounds. Validate
archive extraction against traversal, absolute paths, symlink/hardlink escapes,
special files, duplicate/case-colliding paths, reserved Windows names and
decompression bombs. No postinstall scripts from metadata or network content.
Keep executable plus all required assets/licenses/provenance in one verified
release; never mix old binary with newly copied assets.

Write journal and files durably, verify destination identity, then activate
using a same-filesystem atomic selection operation appropriate to the OS.
Do not assume atomic rename alone proves power-loss durability; specify
file/directory flushes and recoverable ordering. On any restart reconcile
journal/current/previous/staging and either finish a validated commit or
return to last known good. Partial extraction is never executable.

Serialize multiple launches/updaters with an OS lock. Treat local state as
untrusted input; validate ownership, permissions and root boundaries, never
run elevated or follow attacker-chosen links. Same-user code execution is
outside the updater's security boundary, not a reason to trust unsigned files.
Keep current plus previous verified version; cleanup excludes executing,
resume-pinned, recovery and security-required files.

### 5.2 Linux first

Primary supported mode: per-user managed archive under the existing native
install convention. Migrate recognized legacy versioned installs by adopting
provenance only after verification; never delete unknown directories or
overwrite an unrelated launcher. The previously unsigned installation needs
one trusted bootstrap adoption/install step; it cannot authenticate itself
retroactively. Future updates need no uninstall/reinstall or sudo.

Desktop Exec and command shortcuts target the stable launcher. Preserve
XDG/user-data overrides and icon/launcher identity. Validate paths with spaces,
non-ASCII home paths, read-only destination, cross-filesystem staging and
launch outside the repository. Never invoke curl-pipe-shell as the updater.

AppImage is a separate install mode: detect its actual persistent artifact,
not its temporary mounted path. Do not promise in-place managed updates for
every AppImage/package-manager installation. For unsupported mode, offer a
verified migration or explicit package-manager update path without uninstall.
Do not secretly take ownership of system/Flatpak/Steam-managed packages.

### 5.3 Windows subsequent front

Use the same manifest/trust/journal/notes contract with a per-user managed
release directory and native launcher/helper. The helper waits for the old
process to exit, installs alongside it, then changes selection/shortcut
targets. No overwrite of a loaded EXE/DLL, forced termination, reboot or
administrator requirement for a normal per-user update.

Choose Authenticode signing/certificate custody before public Windows release;
TUF target trust and OS code signing solve different problems. Do not promise
that signing eliminates SmartScreen warnings. MSIX/App Installer is a viable
later alternate channel if its identity/certificate/distribution requirements
fit; do not run custom self-update inside a package-manager-owned MSIX install.
Windows certificate spending and actual Windows devices remain owner inputs.

Test on actual supported Windows, including Unicode paths, antivirus file
locks and shortcut behavior. Cross-compilation/ZIP generation on Linux does
not verify Windows updates. Keep this front explicitly pending until tested;
Linux delivery need not wait for Windows visual/installation acceptance.

## 6. Compatibility and deployment without planned match interruption

### 6.1 Immutable match contract

Separate `client_build`, `server_build`, `wire_epoch`, `ruleset_fingerprint`,
`content_manifest`, `feature_set`, `asset_manifest`, DB reader/writer schema
and release channel. An admission record chooses one exact compatible
generation and freezes its rules/content/features for the whole match.
Every replacement worker build receives a new generation ID even when its
match contract is unchanged; compatible generations may coexist. API/UI
compatibility is not permission to hot-replace a process running a match.
Linux, Windows and browser can cross-play only when that contract is supported.
Never advertise a protocol range merely because version numbers are adjacent;
different wire support needs actual codec/behavior fixtures.
Artifact identity and compatibility are distinct: different cosmetic asset
hashes/client build IDs may share a tested presentation compatibility ID.
Do not reject a safe UI-only patch simply because its package hash differs;
critical silhouettes/attack cues need explicit content/presentation validation.

Client-only UI/performance patches with the same proven match contract may
join the current server. Backend-only/API changes may publish independently
when existing reader/writer/wire contracts remain compatible. New combat rules,
wire shape or feature semantics create a new generation. Do not hot-load
balance tables, assets or a new ruleset into an existing match.

New-admission compatibility checks return structured UpdateRequired or
TemporarilyUnavailable before consuming a seat; minimum-version policy does
not mutate a playing user's connection. Existing invite/room/rematch tokens
are generation-bound and never pair different generations silently.

### 6.2 Initial safe operation and recommended two-slot rollout

**Immediate fallback:** stage a new backend, stop new admissions, let current
matches finish, flush/ack result jobs and reconnect/result delivery, then
replace. This preserves matches but pauses new matchmaking and can wait a
long time. Label it honestly; it is not continuous-admission deployment.

**Recommended rolling foundation:** two backend slots on separate localhost
ports, immutable versioned executables/config, with predeclared stable Caddy
routes `/ws/slot-a` and `/ws/slot-b` plus a small admission/release directory.
No proxy reload during normal app promotion; bootstrap proxy changes happen
before activation with no active sockets. Do not rely on a finite
stream_close_delay to preserve an unbounded match. Gateway/proxy changes
themselves require their own drain procedure, not a claim that app-slot
rollout solves every infrastructure restart.

The admission directory supplies candidate generation endpoints/capabilities;
server validates account/guest identity and owns the final admission decision.
Add a durable shared ownership record keyed by account (or guest session),
generation and epoch, plus match/result state, in the existing authoritative
account/result DB or through one equivalent reviewed coordinator. Do not
spawn two independent lobbies with only process-local single-session rules.
The shared claim must be atomic, scoped and fenced against stale writers.

An account still playing on A cannot create a second match on B. A reconnect
on A is routed to A; signing in through B discovers the owner generation and
returns a bounded authenticated resume decision rather than taking over
locally and losing the old seat. Do not expose account/session details in
public discovery or put reconnect bearer secrets in URL query strings.

Give each match a durable unique result key and idempotent finalization;
reuse P60 if implemented, otherwise implement the minimal necessary contract
here without claiming all P60 delivered. Retain owner/fencing state until
result commit and terminal delivery/outbox policy complete. A heartbeat TTL
alone cannot reassign an active match/account during a partition. On owner
uncertainty fail closed for new admission while allowing the live worker to
continue. A crashed in-memory owner needs the explicit recovery policy;
never fabricate a resumed match on a different simulation.

Routine rollout:
1. Build/sign/verify artifacts outside the live host; expand DB compatibly.
2. Start B as Warming, not ready for matchmaking; verify dependencies,
   contract, assets, schema and capacity on the same constrained host.
3. For a **same-contract** release, allow a measured canary of new
   compatible admissions on B. A release with a new match contract follows
   §6.4 instead, because a percentage canary would split the public queue.
   Existing A sockets, players and matchmaking partitions keep their contract.
4. Promote B and mark A Draining: no new ordinary queues/rooms on A, but
   existing reconnects/active match interactions continue. Decide invitation/
   rematch policy explicitly, and show a localized reason for denied new play.
5. Await A: active matches zero **and** result jobs/outbox/delivery settled,
   reconnect/result windows handled, no remaining protected ownership leases.
   Do not use matches_running alone. Remove A only after verified retirement.
6. Retain immutable assets/builds/compatible DB readers through rollback and
   reconnect policy. Do not overwrite A to publish C while A is still busy.

There is no forced five-minute drain cutoff. Remove the 300/330-second
termination assumption from the planned release lifecycle; process-manager
stop must occur only after the application retirement gate, not kill it
because a control command timed out. A stuck drain alerts the operator and
defers the next release. Urgent security containment is an explicit audited
exception, not a normal deploy success. P66 changes no maximum match duration.

If B fails canary, stop its new admissions; keep **both** A and B's already
running matches pinned until completion. Roll back the admission pointer or
publish a corrected generation, not destructive executable replacement.
Two occupied slots mean defer another incompatible release; extra slots
are a measured cost/operations decision. Same-host two-slot rollout is not
host high availability and needs memory/CPU headroom before activation.

### 6.3 Database, public APIs and browser bundles

Use expand → migrate/backfill → verify → switch → contract. New and draining
versions must both read/write the expanded schema correctly. No destructive
contract migration until old writers/readers and rollback window retire.
Backfills are bounded, throttled and outside simulation; failed migration
prevents readiness. Restoring an old DB backup to roll back app code would
lose newer results and is not a standard rollback strategy.

Retain immutable browser bundle/asset directories per build. Update the
entry pointer for new loads only; keep old WASM/JS/assets addressable for
open tabs and reconnection. No deploy `rsync --delete` over their live tree,
forced page refresh, service-worker takeover or mutable URL reuse. Classify
unhashed JS/JSON/CSS/fonts too; current broad JS caching is not sufficient.

Public read APIs can roll independently under additive DTO contracts and
bounded versioning. P65 admin/news changes cannot restart match workers or
change release signing policy. Schema/update metadata/public news each have
their own authority. No account secrets or private route registry in the
update/CDN feed. Readiness differs from liveness; a Draining process is alive
for existing matches but unavailable for new admissions.

### 6.4 Contract transitions and matchmaking

Two releases with different match contracts cannot share a match, so a
percentage canary of a new contract splits the public queue. With a small
population that lengthens waits for both halves and pairs players by rating
less precisely. Proposed policy (BR-UP09):

- Public matchmaking (Casual, Ranked) admits exactly one **active** contract.
  Same-contract client/backend builds mix freely and may canary by percentage.
- A new contract is first exercised on the opt-in preview channel with its
  own queue. Its stable client build is then published ahead of the switch
  (lead time Proposed 24 h, O03) so idle clients stage it in advance.
- At the scheduled switch, generation B becomes the active public contract in
  one step; A stops new public admissions and drains its running matches.
  Clients still on the old contract get UpdateRequired at queue or room time,
  with the update usually already staged.
- Private rooms, challenges, rematches and spectating are contract-scoped:
  pairing or watching across contracts is refused with a localized reason,
  and the P47 live list shows only matches the viewer's build can watch.
  Rooms created on A before the switch follow the O03 grace policy.
- Batch rule and balance changes into scheduled contract releases; most
  releases should be same-contract and need no switch at all.

### 6.5 Computed contract identity

Compatibility must not depend on someone remembering to bump a number. The
repository already freezes wire bytes (`tbg-core/tests/contract_protocol.rs`)
and simulation outcomes (`contract_determinism.rs` golden hashes); P66 binds
the contract identities to them:

- `wire_epoch` records a digest of the frozen message corpus. A test fails
  when the corpus digest changes without a new epoch (and the matching
  `PROTOCOL_VERSION` handling), and when the epoch changes without new bytes.
- `ruleset_fingerprint` and `content_manifest` are computed at build time
  from a canonical encoding of the ruleset and content tables (encoding chosen
  at Step 0) plus the determinism goldens; they are never typed by hand. A
  changed fingerprint makes the release a new contract (§6.4) and requires a
  `gameplay` notes section (§3.4).
- A compatibility claim between two epochs needs evidence: keep a recorded
  client message corpus per supported epoch and replay it against the new
  server's decoder and admission in tests (consumer-driven contract tests).
  No release marks two contracts compatible without that corpus passing.
- Durable match/result records store the admitted contract identity, so P60
  audits and any future replay viewer know which rules produced a result.

## 7. Capacity path and observability

Stage A: managed Linux updater, safe single-host drain and proven two-slot
compatibility/admission foundation. Stage B: production Windows acceptance
and validated two-slot canary. Stage C, when measured saturation justifies
it: multiple match workers and shared admission/result authority across
hosts; a stable routing layer and durable store selected from measurements.
SQLite ownership coordination is a single-host design, not a globally shared
filesystem database for thousands of players.

Do not claim any fixed player capacity today. Measure online **players,
concurrent matches, spectators and idle connections** separately. Report CPU,
RSS, per-tick p95/p99/late ticks, WS view delay, RTT and writer queue/lock
latency under peak plus two-slot overlap. Reserve overlap capacity before
canary; protect existing matches by refusing/defering new admission rather
than letting deployment exhaust the host. Large capacity runs need explicit
owner authorization; use bounded representative fixtures first.

Metrics: release/generation state, compatible admissions/refusals, active
ownership, drain age, pending results, update stages/failures/rollbacks,
manifest expiry/signature failures and notes receipts. Never label metrics
by account/session/token, installation ID or arbitrary build download URL.
Aggregate by bounded release/channel/platform. Local logs redact secrets;
release metrics must not create a surveillance or unbounded-cardinality system.

Update health without an analytics system: the client's next admission
handshake carries its signed `build_id` and a bounded last-update outcome
(`none`, `updated`, `recovered`, `failed:<stage>`), added additively at the
next planned wire epoch. The server only increments aggregate counters by
(build, outcome); nothing is stored per player or installation. A new client
build's rollout halts automatically when failed/recovered outcomes reach the
Proposed threshold (2% of at least 100 reports), pending owner review.

Report these indicators per release: planned-release disconnects of running
matches (target 0), lost or duplicated results (target 0), update success and
recovery rates, time from availability to staged and to applied, admissions
by build, drain age and admission refusals by reason. Targets for the
non-zero indicators come from measurements (O04), not from this plan.

On a shared host, run slots and maintenance work in separate systemd units or
slices: live match workers get the higher `CPUWeight`/`IOWeight`, while
warming, backfills, migrations and artifact preparation run at a lower weight
so they cannot take tick time from running matches.

## 8. Delivery milestones and execution steps

Ship in four milestones, each with its own branch, PR and exit gate, so the
Linux updater the owner needs first never waits for backend coexistence or
Windows hardware. M1 and M2 touch different crates and can run in parallel;
M3 needs M2; M4 needs M1's contracts and real Windows devices. Step 0 runs at
the start of every milestone against its real integration SHA.

| Milestone | Steps | Exit gate | Depends on |
|---|---|---|---|
| **M1 — Linux self-update and notes** (`codex/p66-m1-linux-updater`) | 1, 2, 3 and the part of 7 needed to build, sign with test keys and publish a static TUF repository plus notes | Isolated install: one trusted bootstrap adoption, then in-app N → N+1 → N+2, each showing its notes after reopen; journal killpoints recover; U01–U07, U09–U11, U19 (identity computation), U20, U21 pass. No game-server change. Real activation stays off until O02 | — |
| **M2 — Backend release safety** (`codex/p66-m2-backend-release-safety`) | 5 and §6.5 | Isolated scenario: a planned backend restart never ends or reroutes a running match and every result lands exactly once; U12, U14, U15, U16 (database part), U17, U19 pass | — |
| **M3 — Two-slot rollout and contract switch** (`codex/p66-m3-two-slot-rollout`) | 6 and §6.4 | U13, U15, U16, U18, U22 evidence on isolated listeners; owner approval before any proxy bootstrap | M2 |
| **M4 — Windows adapter** (`codex/p66-m4-windows`) | 4 and the Authenticode decision | U08 plus the U01–U07/U10 subset on real supported Windows; missing hardware keeps M4 pending | M1 |

Release activation (rest of Step 7) is per platform: protected signing,
metadata promotion and rotation/repair runbooks go live only after O02 and
the platform's milestone exit.

**Step 0 — freeze decisions/inventory.** Record integrated SHA, install modes,
paths, launcher/storage behavior, schemas, proxy version/config and result
delivery. Resolve O01–O04 or keep dependent activation disabled. Select/pin
maintained TUF/transport/OS-credential libraries with Cargo reason comments.

**Step 1 — compatibility/release schemas and trusted fixtures.** Implement
versioned descriptors, channel/build identity, TUF verification tests, durable
journal/resume/notes models and old-client bootstrap policy. No production keys.

**Step 2 — Linux managed updater.** Stream/stage/verify/extract, atomic switch,
health/rollback, stable desktop/CLI entry and legacy adoption. Tests use an
isolated temporary installation; preserve the owner's existing installed game.

**Step 3 — game UI and notes.** Integrate async startup modal, Later/manual
check, idle application guard, bandwidth pause and crash/reopen receipts.
Complete English/pt-BR/error/accessibility paths and protected resume storage.

**Step 4 — Windows adapter.** Implement same state machine with native file
locks/paths/shortcut behavior. Record Linux and Windows acceptance separately;
do not claim Windows done from Linux CI or a ZIP.

**Step 5 — backend durable ownership/drain.** Add generation/ownership/result
contracts, truthful readiness/retirement and remove finite forced-drain
assumptions. Test single-host fallback and old/new codec/schema compatibility.

**Step 6 — two-slot rollout and browser assets.** Prepare fixed proxy routes,
new-admission directory, canary/promotion/rollback commands and retained
immutable bundles. Prove reconnect/single-session/cross-device behavior and
that all result tasks finish before retirement. No live proxy/server action.

**Step 7 — release pipeline and consolidation.** Protected artifact signing,
metadata promotion and repair/rotation docs; targeted test union and
isolated Linux/Windows/dual-generation scenario evidence. Keep new release
activation disabled until owner security/UX/operations approval. Open/update
PR to main when implementing under the companion prompt's authorization;
do not merge, install, sign a real release or deploy automatically.

## 9. Verification and decisions

Execute U01–U22 in the companion on synthetic roots and isolated listeners.
Planned test targets: updater `update_trust`, `update_transaction`,
`update_receipts`; server `release_admission`, `release_drain`,
`release_compatibility`; client focused `native_update_flow`. Exact crate/
target names are finalized at Step 0; they do not exist in this plan.

Follow minimal work-front policy; one reviewer runs the deduplicated affected
tests once at consolidation under the shared nonblocking host lock and at
most two Cargo jobs. Do not wait/bypass a busy lock, force cold builds or
run broad workspace suites by habit. Release/packaging, named platform
installation tests and capacity experiments require explicit scope and an
isolated destination. The owner authorizes real installation/deployment
separately. A failed/deferred security or live-match isolation gate is not
release-ready just because normal compilation passed.

- **O01 install modes:** recommend managed per-user Linux first, Windows
  per-user adapter next; decide supported OS/architecture and AppImage adoption.
- **O02 trust/signing:** approve root threshold/custody/rotation, protected
  signer/channel process and Windows signing budget before real publication.
- **O03 availability:** approve new-admission version floor, emergency
  containment policy, pending-room/rematch behavior and maintenance messaging,
  plus the contract-switch cadence, client lead time (Proposed 24 h) and
  pre-switch room grace of §6.4. Routine updates never force a running match
  to end.
- **O04 capacity/retention:** measure overlap headroom, set disk/download/
  metadata/time limits, rollout cohorts and retained-build/result windows.
  A single-host bottleneck blocks promotion; don't invent player capacity.
- **O05 player-facing defaults:** automatic idle download on or off by
  default, prompt cadence (BR-UP08) and notes history length.

Executor report: baseline and actual artifact hashes, compatibility matrix,
install/OS coverage, test commands/results and deferred gates, journal fault
injection, keyless test trust setup, drain/result/reconnect evidence, capacity
measurements, owner decisions and PR URL. No vulnerability-free/zero-failure
guarantee or thousands-player certification is implied by planning.

## 10. Risk register

| Risk | Impact | Mitigation | Gate |
|---|---|---|---|
| Signing key compromise ships a malicious build to every player | Critical | TUF role separation, offline threshold root, short timestamp expiry, rotation runbook | O02, U02 |
| Root key loss leaves no way to rotate keys | High | Backup ceremony with k-of-n custody; manual signed repair installer | O02 |
| A bad build bricks installs | High | Health-gated activation, retained previous build, journal recovery | U05, U06 |
| Bootstrap too old for a new target | Medium | `minimum_bootstrap` in the descriptor and a signed repair path | §5.1 |
| Someone changes rules or wire without a new contract | High | Identities computed from frozen bytes, content and goldens | §6.5, U19 |
| Public queue split across contracts | Medium | One active public contract and a scheduled switch | §6.4, U22 |
| Long matches hold a drain and delay releases | Medium | Defer promotion; never kill; schedule contract switches | §6.2, U15 |
| Slot overlap starves running matches of CPU/memory | High | Measured headroom, unit weights, refuse new admissions first | §7, O04, U18 |
| Database migration fails mid-rollout | High | Expand/contract, readiness gate, no restore-based rollback | §6.3, U16 |
| Retained builds exhaust disk | Low–Medium | Free-space preflight, quota, cleanup that skips pinned builds | O04, U03 |
| Static repository/CDN outage | Low | The installed game keeps working; jittered retries; mirror later | §4 |
| Windows antivirus/SmartScreen friction | Medium | Authenticode, reputation, real-device tests, honest docs | O02, U08 |
