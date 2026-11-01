# P67 — Offline-base executor prompt

```text
Implement the OFFLINE foundation of P67 — commerce, recovery and fiscal
contracts for Tupi Battle Grounds, following the owner's explicit exclusion
of real integration. This is not authorization to integrate Asaas live.

Repository: https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds
Planning branch: codex/p67-commerce-recovery-foundation-plan
Read current AGENTS/VALIDATION, docs/phases/P67-commerce-recovery-foundation.md
and docs/modules/commerce/README.md plus all linked module contracts.

Use current main in an isolated worktree on codex/p67-commerce-recovery-base.
Refresh actual account/protocol/Inventory and P64/P65/P66 integration, preserve
other fronts and incorporate only relevant planning docs without overwriting
newer work. Follow P67 Steps 0-5, keeping all external integration disabled.

Materialize cohesive code only where tests/consumers exercise the offline
contract: account verification/recovery transaction with captured fake mail,
money/order/refund/inbox/outbox/grant state machines, pure Asaas event mapping,
fiscal request capture and cosmetic-preference resolution where supported.
No orphan placeholder modules, multi-provider framework or Bevy server code.

Preserve stable AccountId/purchases on recovery. Verified email is required
before paid checkout; rotate key via one-time email proof and protected
confirmation, never send permanent keys in email. Revoke stale sessions,
including existing community/game generations. Do not infer recovery rights
from billing email/CPF/receipt screenshots or newly hijacked mailbox settings.

Server owns price/currency/recipient. Use exact minor-unit arithmetic,
durable intent/inbox/outbox and unique source grants. Initial method is
Asaas Pix; verified RECEIVED is the normal grant gate, not client redirects,
CONFIRMED from other methods, cash receipts or screenshots. Reconcile unknown
creates and late/excess payments; externalReference is not a magic uniqueness
guarantee. Keep holds/refunds/fiscal state separate and compensate only the
disputed source. Honor HTTP 200-after-persistence webhook design in fixtures.

Treat Zaflas as the owner's issuing company, not an invented API. Do not
choose fiscal service codes, tax rates or a document type without accounting
approval. Keep terms/refund/privacy/age decisions explicit before live sales.
Opponent mute/hide are presentation preferences preserving all combat cues
and server ownership, not permission to remove real gameplay information.

Run C01-C20 as scoped synthetic tests during targeted consolidation. Follow
the shared nonblocking build lock/max two jobs and minimum per-front checks;
no automatic full suite or broad unrelated builds. Code/comments/docs/commits
English; user labels EN/pt-BR through existing i18n; discuss in Portuguese.

No real/Sandbox Asaas calls, API keys, checkout, external webhook registration,
SMTP/domain changes, real email, invoices, production DB migration, paid
resources, installation or VPS/deploy action is authorized. Recheck official
docs as needed, but do not use documentation Try It or create external objects.

Resolve O01-O05 at their dependency points and continue independent work;
do not silently approve product/legal/accounting choices. Record actual tests,
deferred gates and explicitly show no external integration can activate.

For delivery, normal push and PR to main are authorized by this execution
instruction, overriding general AGENTS no-push only for pertinent code/tests/
docs. No force-push, merge, deploy or real integration. Use Draft if mandatory
offline/security/product scope remains blocked. PR describes same-account
recovery, ledger/trust boundaries, synthetic evidence, limitations and later
integration gates. Finish with PR URL and actual validation status.
```
