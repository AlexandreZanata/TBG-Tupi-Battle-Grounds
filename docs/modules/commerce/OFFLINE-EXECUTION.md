# P67 offline execution — recovery and opponent mute

## Step 0: frozen base and owner direction

Execution branch: `codex/p67-commerce-recovery-base`. Remote main was
`9008f117`; the owner's explicitly requested starting point is its descendant
`55510c5`, `codex/consolidated-linux-release`. This worktree was created from
main and fast-forwarded to that consolidation. Other worktrees are untouched.
The PR to main consequently depends on that consolidation; review the P67
increment against `55510c5` separately from the inherited stack.

Only P67 planning folders and Proposed sections BR-CO01–07/D-93–95/§23
are incorporated. Newer shared documents are preserved. The current owner
AGENTS/VALIDATION policy supersedes the older consolidated validation text.

Owner steering during execution: defer fiscal work and focus on email account
recovery and silencing a player. Commerce ledger/provider/fiscal implementation
(Steps 2–3, C01–C11/C20) is deferred, not an implicit approval of sales.

## Actual integration inventory

- AccountId is SQLite `accounts.id` (`i64`). The key encodes 80 random bits
  into 16 Crockford characters and retains the existing 40-byte protocol cap.
- Protocol version 29. Account login, takeover, match attach and reconnect are
  implemented. Account schema is v8; live migration/startup stays unchanged.
- P64 is integrated, using paired EN/pt-BR catalogs. No new active product
  labels or forms are introduced in this offline-only increment.
- P65 community HTTP sessions and P66 coexisting-generation auth authority are
  not integrated. They cannot be claimed as verified. Future activation must
  enforce committed auth epochs at each admission/command/reconnect boundary.
- No paid Inventory, chat, emote transport or optional skin consumer exists.
  Existing sound mute affects global game audio. StrongholdKind is mechanical.
  No enabled opponent-mute/skin switch is justified on this baseline.

## Decision dependencies

- O01 products/prices: deferred. No catalog or payable order introduced.
- O02 recovery: synthetic ASCII email policy only; trim outer whitespace,
  preserve local-part case/dots/plus tags, lowercase domain, reject Unicode/IDN.
  Proposed token lifetime 15 minutes/24 hours and 3 requests/hour apply only
  to fixtures. First enrollment needs fresh key proof; replacing an established
  address and suspected compromise fail closed pending a trusted-change policy.
- O03 fiscal/legal: explicitly postponed by owner; no service code/rate/type.
- O04 economic remedies: deferred; no automatic financial action.
- O05 opponent preferences: synthetic contract only, opt-in false defaults.
  Real UI persistence/consumer activation awaits real emotes/skins and owner
  approval. Combat audio/visual identity and own optional effects are retained.

## Activation boundary

All new recovery and presentation code is compiled only with `cfg(test)`.
There is no runtime feature flag, Config option, HTTP route, mail transport,
provider client or fiscal adapter. Test schema extensions never run from
`Store::open`, the server binary, or normal migration v8. Only isolated fixture
connections are accepted by the test harness. A future implementation needs
an explicit reviewed promotion of this code, real protected HTTP confirmation,
mail custody, runtime revocation delivery, P65/P66 integration and owner gates.

This is an offline foundation, not working email delivery or a playable mute
feature. The synthetic mail capture retains temporary tokens only in memory;
no permanent key is mailed, logged, or stored for later retrieval.

## Step 1: offline identity implementation

`store/recovery` installs fixture-only identity/challenge/admission/audit/
revocation tables on a disposable database already migrated by the existing
account schema. It uses the real account register/login implementation.
Enrollment proves the current key against the authenticated AccountId.
Verified email is private; pending/billing addresses cannot recover an account.
The capture holds only temporary proofs; token digests alone persist in SQL.

Protected confirmation fixtures validate POST, fixed synthetic origin and a
separate browser nonce. GET only previews. An immediate SQLite transaction
consumes all old challenges, replaces the existing key hash, advances the auth
epoch, appends audit and durable revocation intent. Profile and account-linked
purchase fixture rows stay untouched. Permanent keys are returned once only;
losing that response requires a fresh recovery. Mail/audit failures roll back.

Two stamped lobby fixtures exercise real Close/Detach/reconnect queues,
bounded delivery failure and replay without revoking fresh-epoch sessions.
The epoch stamps are a modeled future admission boundary, not an integrated
runtime solution. P65 community sessions and P66 generation admission need
their own integration tests after those implementations are integrated.

Minimum front checks: `cargo fmt -p tbg-server` and `git diff --check` passed.
Focused tests are queued once for Step 5; no tests were run per code step.

## Steps 2–3: deferred by scope steering

Money/order/refund/inbox/outbox/source-grant state machines, pure Asaas mapping
and fiscal capture are not implemented in this increment. Their contracts are
preserved, not presented as tested code. No provider documentation was used
to build an adapter and no API documentation Try It was invoked.

## Step 4: optional presentation contract

`tests/p67_presentation.rs` exercises the strict version-1 preference document,
synthetic emote text/audio/animation classification and reviewed-baseline
selection after authoritative identity. Both seats and canonical valid forms,
summon/carrier metadata and Stronghold kinds remain readable and immutable.
Without an approved baseline, the fixture retains existing readable equipment.

These are isolated presentation fixtures. No actual renderer, audio path,
settings store or UI consumer exists for opponent emotes/skins on this base.
Persistence round-trip is a JSON contract test only. There are no new product
strings, no translation fallback and no enabled but ineffective toggle.

## Step 5: targeted consolidation

Validation level: targeted consolidation of the P67 increment above `55510c5`.
Implemented commits before consolidation: `307facb`, `e736d62`, `e9f6db7`;
the consolidation diff also covers base-form presentation and safer secret
assertions. Rust 1.99.0 is installed. No full suite was requested or run.

### Commands actually run

- `cargo fmt -p tbg-server` and `cargo fmt -p tbg-server -- --check`: pass.
- `git diff --check`: pass on the consolidated increment.
- Python with the already installed `jsonschema.Draft202012Validator`:
  all 4 schema definitions and 4 examples validate; 8 extra/missing-field
  negative cases reject. No packages installed.
- Python `sqlite3` parsed the actual `schema.rs` v1–v8 SQL constants and the
  exact recovery fixture DDL on an in-memory DB: pass, no FK violations.
  This checks SQL/schema compatibility, not Rust transaction execution.
- Relative-link inspection: 11 P67 module/phase/prompt documents, zero
  missing local targets. Source review confirmed no new route, Config flag,
  live migration, dependency, protocol, match-result handler or network call.
- The following Rust batch returned **75 before Cargo started**. It acquired
  no compilation slot. No Rust compile/test result exists. Neither command
  below ran; there was no retry, wait, extra target directory or lock bypass.

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 \
  CARGO_TARGET_DIR=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/target \
  bash -eu -o pipefail -c '
    cargo test --offline --locked -p tbg-server --lib p67_ -- --test-threads=1
    cargo test --offline --locked -p tbg-server --test p67_presentation -- --test-threads=1
  '
```

The attempt used the existing repository target cache because the new isolated
worktree had none. The shared host lock was unchanged. The invocation teed
output to `/tmp/tbg-p67-{recovery,presentation}-tests.log`, but Cargo never
started, so no test logs were created. These exact two commands remain queued
for the next authorized consolidation; unrelated tests/builds remain outside
scope. Formatting and structural checks do not establish Rust compilation.

### Case-by-case acceptance status

- C01–C11: payment, ledger, provider, fiscal fixtures are **deferred by owner
  scope steering**, not implemented or run. The original contracts remain.
- C12: fresh key/account binding, pending-email exclusion and normalization
  fixtures written; **execution deferred: shared validation slot busy**.
- C13: generic bounded dispatch, expiry, wrong purpose and replay fixtures
  written; **execution deferred**. HTTP timing equality remains untested.
- C14: inert preview, method/origin/browser-nonce rejection, parallel confirms
  and lost-response/restart fixtures written; **execution deferred**.
- C15: same account/rating/synthetic source ownership, committed epoch and
  two independently stamped real-lobby queue fixtures written; **execution
  deferred**. Production community/coexisting-generation integration is absent.
- C16: established-address replacement, billing-email substitution and
  suspected-compromise rejection fixtures written; **execution deferred**.
  There is no staff override or trusted-email-change implementation.
- C17: mail/audit/authority failures, bounded queue reservation, generic
  errors and digest-only secret storage fixtures written; **execution
  deferred**. Real mail secret custody/public HTTP DTO review is a later gate.
- C18: optional-opponent-only filtering and local JSON round-trip fixture
  written; **execution deferred**. Real persistence/UI/emotes do not exist.
- C19: base plus four forms of all 51 canonical kinds, both seats, summon/
  carried metadata and all five Stronghold kinds in immutable-view fixtures;
  **execution deferred**. Renderer/animation readability is not verified.
- C20: cross-channel claims and fiscal/legal approval remain **deferred**.
  No receipt claim endpoint, fiscal classification or live checkout exists.

### Remaining mandatory gates and delivery status

Draft delivery: Rust fixtures have not compiled or run. Future activation
must supply trusted runtime key/email/epoch authority; revoke stale sockets,
resume credentials and delayed login replies on every actual generation;
preserve frozen match/account ownership; integrate real P65/P66 boundaries;
review protected confirmation and one-time display; approve O02 email/change/
retention policy and protected real mail delivery. No current runtime recovery
claim is made. A newly verified or billing mailbox never bypasses established
proof, and loss of both key and trusted mailbox has no support shortcut.

Mute activation separately needs a real optional-emote consumer, reviewed
baselines, persisted browser/native preferences, paired i18n/accessibility and
desktop visual/audio evidence preserving all combat cues. No client behavior
or user labels changed in this increment.

Fiscal/payment/accounting/product decisions are still owner gates. The P67
increment contains no real/Sandbox Asaas client, credentials, checkout,
webhook registration/ingress, SMTP, invoice, production migration, installation
or deploy action. `cfg(test)` makes the new recovery implementation unreachable
from the ordinary server binary; there is no configuration switch to activate
it. Publishing this Draft PR does not authorize or enable any integration.
