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

Pending delivery at this report revision: committed release build, signed
repository/bootstrap rehearsal, Linux captures, initial static channel
activation and owner installation. Actual Windows and real desktop-browser
observations remain pending consolidation; no parity or full-suite claim.
