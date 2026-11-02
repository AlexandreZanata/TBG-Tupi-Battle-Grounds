# Session persistence warning and recovery — 2026-11-02

Owner request: investigate the logged-in Linux temporary-session warning,
fix its cause, test and validate. The owner clarified that the notice
vanished after requesting a match. Source base is the actual installed
`0.1.4-colossus.3`, commit `6d96245bef217f3a11376e941e64fe882d125696`,
contract `w38-r15-cc459351581b7763`. Work branch:
`codex/auth-session-persistence-fix`. No gameplay, protocol or balance change.

## Investigation

- The running installation, desktop launcher and build metadata agreed.
- Secret Service was running; its Login collection was unlocked. The installed
  executable's `--auth-vault-probe` reported available, write/readback true and
  cleanup true. Only a fresh synthetic entry was created/deleted.
- Account metadata recorded remember=false. The matching vault entry existed
  and was unlocked. A read-only confirmation of that entry returned HTTP 200
  and the matching family, without logging any secret. Therefore the existing
  credential was usable even though the client retained a failure state.
- Native `save_grant` collapsed vault write failure, metadata failure and HTTP
  confirmation failure into secure=false and discarded the reason. It also
  persisted remember=false on confirmation failure, losing the player's intent.
- Native renewal fell back to the in-memory refresh with persist=false after
  a vault read failure, silently preventing later persistence recovery. A
  failed write could leave a consumed vault generation behind while memory
  held the current refresh. Retrying that stale generation risks BR-AS03 replay.
- The shared coordinator mislabeled transient HTTP renewal errors as storage
  failures. Its socket Active handler could clear a failure independently of
  secure persistence. The banner is additionally scoped to Menu and disappears
  when entering matchmaking; disappearance is not evidence of repair.
- Historical logs discarded the failing confirmation/read detail. The precise
  original transient transport/OS error cannot be recovered retrospectively.
  No claim is made that a permanent Linux vault failure was observed.

## Correction

Keep persistence failure reasons with signed-in outcomes; preserve Access and
profile while reporting transport and storage failures separately. Preserve
nonsecret remember intent and whether the current credential was saved,
backward-compatible with schema-1 metadata. Restore/retry a saved unconfirmed
family, retain the existing retry delay, allow explicit Retry, and persist a
renewal made from the current in-memory proof. Never overwrite another process's
newer family or reuse a vault generation known to be stale. Successful logout
and cancellation clear the new nonsecret markers. Confirm retries use the
existing bounded transport retry. Logs contain only stage and typed public
failure, never credential material. Secure vault and server confirmation remain
required before reporting successful persistence or completing legacy migration
(BR-AS03–05/09–11, D-97).

The shared visual contract records the affected account states. Windows and
real desktop-browser observations remain pending; shared code does not certify
parity.

## Validation

Level: focused consolidation authorized by the request to test and validate;
not a full suite. The first nonblocking Cargo attempt returned 75 (slot busy).
The owner explicitly authorized waiting/retrying the shared slot and completing
the focused tests, build and Linux installation in this chat. All compilation
continues to hold `/tmp/tbg-validation-1000.lock` with two Cargo jobs and the
existing root cache. Final commands/results and installation evidence follow.

Already run: `node crates/tbg-client/tests/auth-session.test.cjs` — five passed;
`python3 -m py_compile scripts/check-native-auth.py` — passed;
`cargo fmt -p tbg-client` and `git diff --check` — passed at this revision.

- Focused native compilation/tests: `cargo test --offline --locked -p
  tbg-client --features ui-acceptance --bin tbg-client -- auth::
  ui::auth_ui::tests net::socket::tests --test-threads=1` — **21 passed**, zero
  failures, 207 unrelated tests filtered out, 53.87 s with the reused cache.
  The existing `visuals/layout.rs::Layout::clear` dead-code warning remains
  unrelated to this change. This includes auth cancellation/epoch safety,
  confirmation recovery, transport/storage classification, queue renewal,
  reconnect and generation routing.
- `python3 scripts/check-desktop-ui.py` — passed the shared-routing source
  gate. It explicitly does not certify packaged visual equivalence.
- Cached native diagnostic client build: `cargo build --offline --locked -p
  tbg-client --features ui-acceptance --bin tbg-client` — passed, 19.44 s.
- Real native/cofre acceptance: `python3 scripts/check-native-auth.py --client
  <root-target>/debug/tbg-client --server /data/tbg-ab-target/debug/tbg-server
  --confirmation-outage --report /tmp/tbg-auth-session-confirmation-recovery.json`
  under the shared lock — passed. Two injected HTTP 503 confirmation replies
  were followed by same-family restoration and secure persistence. A deliberately
  stale consumed vault refresh was skipped in favor of the current memory proof.
  Process locking, nonsecret metadata, backup migration, backend restart,
  offline-pending logout, acknowledged revocation and synthetic vault cleanup
  all passed. See [boolean-only evidence](AUTH-SESSION-PERSISTENCE-EVIDENCE.json).
- Negative control: the installed `6d96245` release was run against the same
  synthetic confirmation-outage proxy and failed the recovery contract as
  expected. The corrected candidate passed that scenario. Both used separate
  synthetic account/data namespaces; no owner-account token was rotated.

Pending at this source commit: final committed release build, package/install
verification and installed Linux smoke test. Packaged Windows and real desktop
browser account/queue-return observations remain deferred to their respective
platform consolidation; no visual parity claim. No full suite, gameplay/balance
checks, push or public deployment is authorized or performed here.
