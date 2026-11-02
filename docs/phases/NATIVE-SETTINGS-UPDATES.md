# Native settings and automatic updates — 2026-11-02

Owner request: newest patch note first, oldest last; fix native key copy/download;
open Downloads and updates inside the app; discover and install updates without
website/manual uninstall. Reinstall Linux while preserving the complete installed
release and the authoritative competitive/ranked endpoint. The owner confirmed
possession of the existing recovery key for one-time in-app import; no key was
requested or sent in chat.

Base: previous session fix `c3b2f3a` / installed executable `94a0aa4`,
`0.1.4-colossus.3`. Branch `codex/native-settings-updates`; the corrected version
is `0.1.4-colossus.4`. No tbg-core/gameplay, tbg-server or proxy routing changes.
The required existing match contract is `w38-r15-cc459351581b7763`, and the
native default endpoint remains `wss://api.tupibg.com/ws`.

## Findings and implementation

The history renderer trusted feed order while newer entries were appended.
It now merges installed and online notes by ID and orders date/ruleset/SemVer
newest first. Existing entries are retained and an older online catalog cannot
hide an installed release's notes.

Settings read the retired plaintext AccountKey preference, although P68 login
uses opaque Access/Refresh and the server keeps only the AccountKey hash. A
logged-in token cannot reconstruct that old secret. Explicit masked key import
now validates a native key with the existing login endpoint using a temporary
family, revokes that proof without claiming a controller/socket, and writes only
the explicitly imported key to a separate account/origin/device OS-vault entry.
The current login/refresh/seat is unchanged. Export uses that protected copy;
wrong-account keys are refused and errors do not sign the player out. Clipboard
success is checked; Downloads uses a private create-new file and retains an
existing different backup. Browser backup is memory-only; no new key or token
is placed in preferences/localStorage, URLs, logs or process arguments.

The existing P66 launcher/helper already implements signed discovery, staging,
health acknowledgement and rollback, but the old installation linked directly
to the game. The settings control also opened GitHub instead of that UI.
The shared modal now remains available for checking/current/error states and
controls the launcher, with automatic idle-only download/restart and a persistent
opt-out toggle. Queuing Ranked preserves the play request and cancels/defer the
download. The bootstrap preserves both command aliases, desktop icon and class,
old release directories, preferences and vault data. Signed static publication
uses the existing /updates route, immutable targets/versioned metadata first,
and atomic timestamp last; it never restarts the backend or changes routing.

The public TUF anchor is in `deploy/native/trust/root.json`. Private role keys
are staged outside git and must remain owner-only and outside every published
folder. Initial channel/key custody activation is recorded below after review.

## Focused validation

The owner requests these corrections and reinstallation, continuing the same
native delivery task; the earlier explicit authorization to wait/retry the
shared validation slot is retained. No full suite is inferred. Compilation
uses the existing root cache, /tmp/tbg-validation-1000.lock and two jobs.

- `cargo test --offline --locked -p tbg-client --features ui-acceptance --bin
  tbg-client -- auth:: ui::account_backup ui::update ui::shell::patch_notes
  net::socket::tests --test-threads=1`: **28 passed**, 207 unrelated filtered.
  An earlier integrated selection passed 26 before the extra account-scope and
  Ranked-priority cases were added. The existing unused Layout::clear warning
  remains unrelated.
- `cargo test --offline --locked -p tbg-update --test updater -- u02_ u05_
  u06_ --test-threads=1`: **7 passed**, 3 unrelated filtered. Covers foreign
  roots, tampering/rollback rejection, journal interruption, health-gated
  activation, failed-candidate recovery and revoked fallback handling.
- Cached `cargo build --offline --locked -p tbg-client --features ui-acceptance
  --bin tbg-client` and `cargo build --offline --locked -p tbg-update --bins`:
  passed for the runtime journey.
- `node crates/tbg-client/tests/auth-session.test.cjs`: five existing regression
  cases passed. `python3 scripts/test-publish-native-channel.py`: two passed.
- Real-provider isolated native journey: `xvfb-run -a env -u WAYLAND_DISPLAY
  python3 scripts/check-native-auth.py --client <root-target>/debug/tbg-client
  --server /data/tbg-ab-target/debug/tbg-server --confirmation-outage
  --key-backup --report /tmp/tbg-native-settings-key-evidence.json`: passed.
  Existing manual legacy-key fallback, missing backup, wrong-key refusal,
  protected save/readback, isolated clipboard copy/paste, explicit private
  Downloads file content, current-family confirmation, auth-metadata secrecy,
  persistence recovery, restart/revocation and cleanup passed. The first run
  exposed a fixture assertion that incorrectly expected no key while the test
  had deliberately created a legacy key; the fixture was corrected and the
  missing-key branch is now tested explicitly too. Owner data/clipboard was not
  used. See the boolean evidence recorded below.
- `cargo fmt -p tbg-client -p tbg-update`, `python3 -m py_compile` for the
  changed scripts, shared desktop routing gate and `git diff --check`: passed.

Actual Windows and real desktop-browser observations remain pending
consolidation; no parity or full-suite claim. Initial static channel activation
and owner installation await the explicit O02 decision described below.

## Pre-activation rehearsal

The committed `8f78e5a` release built successfully (client 3m 48s, launcher/tool
18.64s) and was packaged/signed locally as `linux-20261102-8f78e5a`, sequence 1.
An isolated managed bootstrap verified all 394 package files and the unchanged
competitive contract. Three actual Linux captures were inspected: latest notes
first despite the older backend feed, the internal current/check/automatic
modal, and the masked one-time key-import form with the same logged-in test
account. No owner shortcut/account was used in that rehearsal.

Review additionally identified a same-frame Ranked-vs-automatic-restart race
once a download was fully staged. Online play intents now win before automatic
Now/Restart actions are consumed, and a new regression keeps Ranked queued and
the update staged for a later idle point. The three affected BR-UP02 cases
passed after this correction (29 distinct focused client cases in total).
The final source is `edab60c2ce8ab53a814612cd8b67a8c36bc1f7b0`; its Linux
release build passed in 4m 12s and its signed package is
`linux-20261102-edab60c`, sequence 2. Sequence 1 remains an isolated rehearsal
baseline, not the owner's installation.

The actual release launcher then automatically discovered, downloaded,
verified, staged, restarted and health-acknowledged sequence 2 from sequence 1
against a local signed repository. The `previous` pointer retained sequence 1,
the preference sentinel survived, and pending post-update notes named the
running build. The resulting Linux capture shows the in-app modal on the final
release. This journey used a loopback backend and temporary data; owner account
and shortcuts were not touched. See `NATIVE-SETTINGS-AUTO-UPDATE-EVIDENCE.json`.

The native auth/key journey was repeated with the final packaged release
executable, using the same isolated real OS-vault/clipboard/Downloads scenario;
all assertions passed. See `NATIVE-SETTINGS-RELEASE-KEY-EVIDENCE.json`. Inspection
also confirmed all 374 assets in the previous installed release are retained;
only the two translation catalogs differ intentionally.

## O02 activation decision

The phase explicitly requires owner approval of signing thresholds, custody,
rotation and protected publication before real activation. The prepared Linux
proposal uses one Ed25519 key per TUF role, threshold 1, with private keys and a
private local backup outside git and outside the public repository. The current
publisher loads the four role keys locally; this is owner-controlled manual
signing, not an offline-root or protected-CI claim. No private key goes to the
VPS. Public trust-anchor SHA-256:
`db40abac2b2b5fbf3f19cc8fb37cbc1bed274f56223a3e4bd20771346df2d141`.

Activation uses only the existing static `/updates/` route. Immutable targets
and versioned metadata are uploaded before the atomic timestamp. An existing
different trust root is refused; changes of signing authority require reviewed
TUF root rotation. Old targets and installed releases remain available.
Metadata expiry requires owner renewal even when no executable changes.
Windows signing/budget and the remaining broader P66 milestone acceptance are
pending; this focused Linux delivery does not claim completion of that phase.

The explicit O02 question was submitted after the final release and automatic
upgrade rehearsal were ready. Until answered, real publication, private-key
custody activation and owner shortcut migration remain pending.

## Pull request preparation

The owner subsequently explicitly authorized pushing this task and opening a
GitHub pull request; this overrides the default no-push rule for this delivery.
It does not approve O02 or deploy the signed executable channel. The task branch
retains the session fix, settings work and all recorded evidence. Unrelated
uncommitted files in the owner's primary checkout are excluded.

Current `origin/main` was merged without rewriting history. Its only conflict
was appended desktop-contract documentation; both sets of observations were
retained. The PR preparation also completed the new catalog entry's required
ruleset fingerprint, concrete bilingual changes, evidence/document links and
candidate status, and updated native release notes to `.4`. No game or auth
implementation changed after the recorded `edab60c` runtime evidence. The
packaged rehearsal predates this additional release-note presentation content.

`python3 scripts/check-native-version.py --base origin/main`,
`python3 scripts/check-patch-notes.py --base origin/main` (nine notes, unchanged
R15 fingerprint), `python3 scripts/check-desktop-ui.py` and `git diff --check`
passed on the PR preparation. The version lookup needed normal network access;
its first sandboxed attempt could not resolve GitHub. The patch-note check
first rejected the incomplete new metadata, which was corrected before push.
Only the public trust anchor is tracked; no private signing keys or release
archives are included in the PR.
