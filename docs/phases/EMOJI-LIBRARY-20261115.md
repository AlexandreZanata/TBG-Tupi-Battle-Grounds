# Browser emoji library refresh correction — 2026-11-15

## Cause and scope

The production colossus.10 browser displayed the generic P75 error when opening
the emoji library and after selecting a standard emoji. Reproduction used only
the synthetic QaWebA1115 account. Selecting Fire saved slots `[8,1,2]` in the
database, but the subsequent library refresh still displayed the error.

Same-origin browser GET requests normally omit `Origin` ([Fetch Standard,
Origin header](https://fetch.spec.whatwg.org/#origin-header)). The shared
account-read binding required that header unconditionally. PUT carries Origin,
so saving succeeded while the GET refresh was rejected. Native requests use a
separate envelope and were unaffected. No uploads, moderation switches or
account verification policy need enabling for the sixteen standard emojis.

## Correction

Only account GET requests may use the browser's same-origin Fetch Metadata and
the exact allowlisted Referer origin when Origin is absent. Require one Referer,
`Sec-Fetch-Site: same-origin`, a fetch mode of cors/same-origin, destination empty,
the existing JSON/custom-header envelope and a valid bearer binding. Duplicate,
missing and foreign-origin proofs fail closed. A supplied Origin is always
validated directly; this cannot replace a bad Origin with a good Referer.
PUT/POST/PATCH/DELETE and auth-cookie operations retain their strict Origin
requirements. Native cookie/Fetch Metadata rejection, slot fencing, expiry,
revocation, ownership, size limits and no-store responses remain enforced.

The same read binding serves the emoji library, account page and private match
history. No simulation, protocol, database migration, client binary, image,
animation or gameplay number changes. Existing colossus.10 clients retain R17
compatibility with the new backend generation.

## Targeted consolidation

Shared `/tmp/tbg-validation-$(id -u).lock`, nonblocking, two Cargo jobs, existing
`/data/tbg-open-fronts-target` cache; dev/test debug info disabled as in that cache.

- `cargo fmt -p tbg-server` and `git diff --check`: passed.
- `cargo test --locked -p tbg-server --lib browser_account_get_requires_exact_origin_proof_even_without_origin_header -- --test-threads=1`:
  one passed. Includes missing/duplicate proof headers, same-site/foreign origins,
  credential-bearing Referer, unsafe methods and native-channel rejection.
- `cargo test --locked -p tbg-server --test browser_emoji_library -- --test-threads=1`:
  one passed. Exercises browser-style initial GET, all three standard-slot saves,
  Origin-less refresh, missing bearer, bad Origin and Origin-less PUT rejection;
  rejected changes preserve the saved slots and reads stay no-store.

- `cargo test --locked -p tbg-server --test contract_p75_emoji br_ce05_only_approved_sheets_are_public_and_hashes_stay_stable -- --test-threads=1`:
  one passed, including native upload approval, ownership and slot selection.
- `scripts/check-desktop-ui.py`, `scripts/check-patch-notes.py --base origin/main`:
  passed; R17 / 3df52af730a4e914 is unchanged.

No full local suite was requested for this correction. Technical CI is pending. Production
browser reopen/save/reload verification follows deployment of a genuine main
backend artifact; do not treat the pre-fix screenshots as a successful check.
Linux and Windows UI observation remains pending; backend tests are functional
evidence, not visual correspondence acceptance. No motion cases change.
