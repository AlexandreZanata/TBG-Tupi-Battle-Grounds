# Business/security validation backlog

> Future offline execution cases, not provider test results. This planning
> branch validates documentary schemas/examples only and invokes no real API.

| ID | Controlled scenario | Acceptance |
|---|---|---|
| C01 | Guest/unverified account/tampered SKU, price, currency or recipient | No payable order or unauthorized ownership; server quote is authoritative |
| C02 | Exact decimal-to-cent conversion, fee net/gross, overflow and allocations | Exact checked money; line sums/partial refunds reconcile; no binary-float rounding |
| C03 | Same checkout retry, different body, concurrent create, unknown provider outcome | One local operation; conflict/reconciliation, not blind duplicate charge |
| C04 | Received event duplicate, altered duplicate digest, restart before/after ack | Durable inbox first, correct 200 policy, one business effect and audit |
| C05 | Forged/missing token, wrong merchant/environment/customer/payment/method/amount | Reject or quarantine; no cosmetic from redirects, screenshots, cash or unrelated payments |
| C06 | Refund/hold before stale Received; unknown event/extra provider fields | Facts preserved, safe reconciliation; no restored grant from stale event or queue break |
| C07 | Local expiry then actual payment, multiple paid attempts, changed catalog | Preserve original sale evidence; late/excess-payment policy, no lost funds/double grant |
| C08 | Crash after paid commit before Inventory ack; grant replay and compensation | Same source grant applied once; other legitimate ownership sources remain |
| C09 | Parallel partial refunds, in-progress/failure/timeout, insufficient seller balance | No over-refund or false completion; retry/review and consumer escalation recorded |
| C10 | MED account-wide hold with null payment ID and later unblock | Financial review, no arbitrary account-wide cosmetic revocation or game ban |
| C11 | Fiscal Scheduled/Error/Authorized; refund with denied invoice cancellation | Independent correct fiscal projection; no fake invoice, extra charge or duplicate fulfillment |
| C12 | Recovery enrollment without key proof/unverified address, another account or billing-email substitution | No recovery takeover; verified stable AccountId association only |
| C13 | Unknown/existing recovery requests, mail flood, expired/replayed/wrong-purpose token | Generic bounded response, no enumeration/lockout, no mutation until valid explicit confirmation |
| C14 | Mail scanner GET, concurrent confirm, lost completion response | GET inert; one key-hash rotation; fresh recovery required if one-time display lost |
| C15 | Key rotation/email change during active game/community/multi-generation sessions | All stale credentials invalidated through authority; same AccountId/purchases; no result rewrite |
| C16 | Recent hostile email change, lost old mailbox, support screenshot/manual override | No new-factor-only recovery or unsupported staff escalation |
| C17 | Source/DB/queue/audit/mail failures, secret canaries, private-object ID changes | Fail closed, bounded work, redacted logs, no public email/CPF/invoice/session leakage |
| C18 | Mute opponent emote/text/audio versus own and real combat effects | Only intended optional opponent presentation suppressed; preferences persist |
| C19 | Hide enemy cosmetics for both seats, every kind/form and StrongholdKind | Approved baseline readability and identical simulation/targeting/timing; equipment ownership unchanged |
| C20 | itch.io receipt claim, unavailable fiscal classification, terms/refund-policy mismatch | No fabricated cross-channel entitlement or live checkout before launch gates |

Use test-only captured mail, normalized provider observations and in-memory
fiscal requests. No test may need a real email, CPF, account, API key, Pix,
invoice, provider tunnel or live endpoint. Validate all active content forms
and future emote consumers against actual integration status; don't enable
a no-op preference and call it functional.

When code is later implemented, focused targets should cover money/order,
inbox/outbox/grant/refund transitions, authoritative recovery transaction,
session invalidation and client cosmetic resolution. Name actual targets
then, queue tests on individual fronts and run the deduplicated targeted
union once at consolidation under the shared nonblocking host lock with
at most two compile jobs. Full suite, Sandbox exercise and production are
separate explicit scopes. Failed/deferred recovery/payment/privacy checks
block real sales, not the publication of this design.

Planning checks: validate local links, schema syntax, synthetic examples,
negative schema cases and cross-field fixture invariants; inspect diffs with
`git diff --check`. No Cargo, Trunk, gameplay or provider tests are authorized
by this documentary foundation alone.

## Planning delivery evidence — 2026-10-31

Passed: all 10 new Markdown files have balanced fences and 19 valid local
links; 25 external source citations inventoried. Draft 2020-12 validation
checked four schemas, four positive synthetic examples and eight negative
cases (fractional/negative amount, currency, quantity, extra client field,
real environment, invalid digest and nonboolean preference). Fixture line
sums/refund bounds/unique IDs and recovery timestamp ordering passed.
C01–C20 are present as future acceptance, not executed business behavior.

`git diff --check` passed; source/script/deploy/workflow diff against
origin/main is empty. Final staged documentation is checked before
publication. No provider Try It, real/Sandbox request, email, invoice,
financial/account mutation, migration, Cargo/Trunk or gameplay test was run.
