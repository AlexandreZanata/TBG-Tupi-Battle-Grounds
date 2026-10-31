# P67 — Commerce, account recovery and Zaflas fiscal foundation

> **Planning and documentary foundation — 2026-10-31.** Owner requested a
> separate initial module/folder structure, current official Asaas research,
> complete payment validation/antifraud flows, recovery email/replacement
> account key, legal/refund/fiscal preparation and opponent cosmetic controls.
> **No real integration yet**, including Sandbox, SMTP and invoice emission.
> This delivery creates substantive documentation folders, JSON contracts
> and synthetic examples; it does not change running game/account behavior.

Planning branch: `codex/p67-commerce-recovery-foundation-plan`.
Parent: P66 planning commit `406539707524f234247dc86bd4808cff54665619`.
Suggested offline-base execution branch: `codex/p67-commerce-recovery-base`.

## 1. Read first

- [AGENTS.md](../../AGENTS.md) and current `docs/VALIDATION.md` if present.
- [New module map](../modules/commerce/README.md), then its accounts,
  payments, provider, fiscal, presentation, contracts and validation files.
- GAME-DESIGN BR-C01–C04 and Proposed §20; DECISIONS D-17 and D-93–D-95;
  ARCHITECTURE §23. D-17's recovery reopen condition is met by paid ownership.
- Relevant P64 presentation, P65 staff/session privacy and P66 credential
  invalidation/generation contracts, **only where actually integrated**.
- [Offline-base execution prompt](../planning/P67-EXECUTION-PROMPT.md).

Only inspect the relevant source for the step being executed; no archive.
All code/comments/docs/commits remain English; pt-BR product text uses i18n.

## 2. Evidence and deliverables

Inspected main-derived native work front `a3ef8a8` and main code inherited
from `9008f117`. Planning parent includes later documentation, not those
phases' implementations. Current workspace has separate active integration
fronts; freeze their actual merged state before future code changes.

- `tbg-core/src/account.rs`: account name plus generated key, no email.
  `store/accounts.rs` hashes/verifies credentials; `schema.rs` has no
  recovery-email or token model on this baseline.
- D-17 explicitly reopens when lost keys/purchases make recovery necessary.
- BR-C01–C03 already require cosmetics-only, account Inventory and validated
  server receipts. Paid inventory/payment/fiscal adapters are not implemented
  by the audited source or by this branch.
- `ui/account_ui.rs`, `assets/lobby/account.js`, `ui/textform*`, `config.rs`
  and `lobby/account_done.rs` are future enrollment/login/recovery integration
  points; preserve stable AccountId and existing takeover/codec semantics.
- `audio.rs` supplies global mute; Settings does not yet establish distinct
  opponent-emote suppression or an opponent-skin resolver.
- No real Zaflas legal/fiscal/account metadata was provided. Treat these
  as explicit inputs, never guessed company data or a fabricated API.

Deliver now: the module folder tree, state/workflow contracts, dated official
Asaas dossier, accounting/legal launch checklist, recovery/antifraud threat
cases, initial machine-readable schemas/examples and staged implementation.
No Cargo manifest, code module, dependency, endpoint or DB migration is added
in this documentary work front. Code folder recommendations are not claimed
as implemented. This keeps the requested design/base separate from real sales.

## 3. Requirements and ordering

| ID | Required contract | Source / cases |
|---|---|---|
| B01 | Separate commerce module with no combat/Shop authority or provider coupling in core | Module map, C01/C17 |
| B02 | Email enrollment/verification and safe same-account replacement key before paid checkout | Accounts, C12–C16 |
| B03 | Server-owned quote/order, exact BRL amounts, one validated entitlement effect | Payments, C01–C08 |
| B04 | Asaas Pix-only mapping, authenticated durable inbox, reconciliation of unknown/repeated/out-of-order facts | Provider dossier, C03–C07/C10 |
| B05 | Traceable full/partial refunds/disputes and source-specific compensation | Payments, C08–C10 |
| B06 | Zaflas issuer policy and independent fiscal state/receipts/corrections | Fiscal, C11/C20 |
| B07 | Consumer/privacy/minor-protection documents and commercial launch gates | Fiscal, C17/C20 |
| B08 | Opponent emote mute and cosmetic hide preserve real combat cues/ownership | Presentation, C18/C19 |
| B09 | All external integrations off; useful offline contracts, fixtures and future transition gates | Contracts/validation, all cases |

Recovery/email ownership is the first prerequisite for monetization. Do not
enable an irreversible purchase flow for an account that still has “lose the
only key = lose all purchases” as its only identity policy.

## 4. Boundary contracts

### 4.1 Account identity and recovery

At registration/login/settings let an authenticated user add a recovery email
with fresh key proof and a verification challenge. Guests remain possible;
legacy players need not submit tax IDs or email just to play. Only a verified
recovery address enables checkout. Pending/billing/provider email is not proof.

Recovery request is non-enumerating and bounded. A verified email receives
a temporary link; an explicit confirmation rotates the account key in the
same authoritative account, consumes the token atomically and revokes stale
sessions/epochs. New permanent key is displayed once through trusted HTTPS,
not mailed/stored as plaintext. Same Inventory/rating/purchases remain bound
to the same AccountId. Email-change and suspected compromise need independent
established proof, not merely confirmation of a newly added mailbox.

Preserve current game protocol/key bounds unless a separately documented
compatibility change is necessary. No Google/Discord provider is required
initially. Do not promise support recovery when both key and verified email
are unavailable. Paid receipts/CPF alone are not account ownership proof.

### 4.2 Money, external facts and fulfillment

Quote immutable catalog lines/terms/seller on the server. Use checked integer
centavos and exact provider decimal conversion. Never trust submitted prices,
client owner IDs or redirection screenshots. Persist intent and correlate
customer/payment/reference/environment before side effects.

Initial paid method is Asaas Pix. Normal grant requires authenticated,
reconciled RECEIVED evidence for the matching PIX charge and no blocking
refund/dispute/hold state. PAYMENT_CONFIRMED is not treated as equivalent by
this initial mapping. Price/value compares gross, not net after provider fees.
A first grant or restoration requires authenticated current-payment lookup
under the merchant credential; inability to validate leaves it pending.

Future provider callback authenticates `asaas-access-token`, records the
event durably and responds exactly HTTP 200, then processes asynchronously.
Dedup by merchant/environment/provider event ID and evidence digest. Own
commands reject unexpected mutable fields; external payloads tolerate new
irrelevant fields. Unknown event/state is safely recorded/quarantined rather
than granted or allowed to break the complete delivery queue.

Payment creation timeout is an unknown outcome requiring reconciliation;
`externalReference` is not a documented universal idempotency guarantee.
Local checkout expiry is not automatic provider cancellation. Late or excess
payments have explicit review/fulfillment/refund paths; no lost customer funds.

Commit paid facts and a unique per-line/source grant outbox; apply Inventory
idempotently with acknowledgement. Source-specific refunds/compensation cannot
remove other legitimate grants of the same cosmetic. Cross-store retries are
explicit; do not claim distributed atomic SQL or exactly-once network delivery.

### 4.3 Antifraud, private data and staff

Enforce object ownership, authentication, CSRF for browser commands, bounded
requests/queues, retry/concurrency limits and safe logging. Future callback
uses provider credentials instead of a browser session/CSRF bypass. Reject
wrong merchant/customer/method/amount, duplicate conflicting event evidence,
client payment assertions and replayed fulfillment.

Risk decisions are evidence-based/reviewable, with reason/expiry/appeal and
distinct finance powers. Do not equate shared IP, another payer's name,
verified email or a Pix balance hold with a definitive fraud verdict. MED
may lack a payment ID; do not revoke all paid inventory or ban gameplay.
P65 moderators/support cannot mark paid, reset keys or refund arbitrary Pix
destinations. No raw account/payment secret in public DTO/metric labels/logs.

Account store is authoritative for email/key changes; commerce worker/DB
owns orders, inbox/outbox and economic records. Keep their work out of match
ticks/results queues and fail closed for purchases when authority is unavailable.
Real external secrets and protected mail-outbox storage are future deployment
inputs. The offline base uses explicit test captures, not credentials in git.

### 4.4 Zaflas, consumer rights and channels

Zaflas is the intended company issuing documents. Confirm its actual legal/
merchant identity, jurisdiction, regime, product classification and issuance
system with the owner/accountant. Asaas has an NFS-e capability; its existence
does not classify the game's product or prescribe a tax code/rate.

Fiscal request/status is independent of payment/order/grant. Authorized fiscal
proof differs from a scheduled invoice. Refund can require separate fiscal
correction; failed cancellation is handled explicitly. Customer access to
private invoice data must be owner-scoped. No real document emission now.

Before commercial activation prepare approved terms/license, seller/contact,
privacy purposes/retention, offer/consent snapshot, delivery/refund/withdrawal
flow and age/parental requirements. CDC art. 49 and current electronic-commerce
and data-protection rules are review inputs, not a blanket digital-item
refund waiver. Do not publish final legal/tax claims from a sample template.

itch.io download sales remain the platform's configured channel. Its seller
payment modes have different merchant responsibilities; no Asaas duplicate
charge, fabricated receipt or automatic in-game grant is introduced here.

### 4.5 Player presentation controls

Separate opponent emote mute from global audio and paid-skin visibility from
mechanical kind/form identity. Local settings never alter server equipment,
owned Inventory, collisions or combat. Hide optional opponents' presentation
using a reviewed baseline, keeping tactical identity/team/status/attack cues.
Do not sell cosmetics under a promise that opponents cannot hide them.
If there is no actual emote/skin consumer yet, record deferred activation;
do not add a fake functional switch or invent a network feature to fill scope.

## 5. Staged execution after this documentary delivery

**Step 0 — freeze business/security decisions.** Refresh code/protocol/account
store and real integrated P60/P64/P65/P66 boundaries. Confirm O01–O05 and
inventory existing free/paid/unimplemented cosmetics and emote consumers.
Review dated provider sources again before any future adapter work.

**Step 1 — offline identity/recovery base.** Define account-store migrations
and transactional recovery/verification behavior in temporary DB fixtures;
in-memory mail capture, token lifecycle and auth-epoch invalidation tests.
No SMTP, real mailbox changes or production migration. Existing nonpaying
login remains compatible until an authorized activation scope is approved.

**Step 2 — offline commerce domain.** Materialize cohesive server modules for
money/catalog/order/refund/source grants and bounded inbox/outbox. Use fake
Inventory/provider evidence, checked state transitions and durable restart
fixtures. Payment external calls remain absent/disabled, not stubbed success.

**Step 3 — provider/fiscal contracts.** Implement only pure Asaas mapping/
validation over synthetic inputs and fiscal capture. Prove webhook ack/state
rules locally without exposing an HTTP hook or creating provider objects.
Define a future concrete Asaas adapter seam; do not add abstraction layers
for several hypothetical gateways or a fake Zaflas client.

**Step 4 — presentation base.** Add real local preferences/resolver consumers
where the integrated game has emote/skin support; otherwise keep only clear
contracts/fixtures until those exist. Verify all active kind/forms and
mechanical Strongholds; complete EN/pt-BR labels and accessible UI states.

**Step 5 — targeted consolidation.** Execute C01–C20 on isolated fixtures,
inspect secrets/ownership/session/result boundaries and update reports.
Prove flags/routes cannot accidentally reach network or grant paid inventory
without verified evidence. Record partial/deferred items honestly.

**Later integration gate — a different authorization.** Only after verified
recovery delivery, legal/accounting approval, provider merchant setup,
secret/mail custody and offline tests may a separately scoped Sandbox
integration begin. Production charges/webhooks/invoices follow their own
review and explicit authorization. Planning/publication is not that approval.

## 6. Validation policy

This planning delivery: resolve document links, parse/validate four schemas
and synthetic examples with an available validator, test representative
negative schema cases and cross-field example invariants, inspect scope and
run `git diff --check`. No Cargo, browser build, actual recovery/payment,
email, API call or fiscal test is needed to validate documentation.

Future code fronts follow AGENTS/current VALIDATION: format and at most one
cheap scoped cached check; queue focused tests. One reviewer executes the
deduplicated affected tests at consolidation using the shared nonblocking
lock and at most two jobs. Do not infer full-suite, release, Sandbox, live
VPS or production authorization from “execute the phase”. Exact future test
targets are recorded when created, not invented as already existing commands.

Acceptance: B01–B09 and C01–C20 mapped to real evidence, private identity
continuity, ledger invariants and tactical presentation checks. Failed/deferred
recovery/payment/security/legal gates block sales, not the offline design.
No global translation framework, marketplace, subscription wallet, card data
handling or multiple payment providers is justified for this initial scope.

## 7. Owner/accounting decisions and report

- **O01 products:** fixed cosmetic SKUs, pricing/refund allocations, preserve
  current free items; no power/paid random rewards. Schema fixture price is
  not a commercial decision.
- **O02 recovery:** verified-email enrollment/normalization, trusted email
  change/escalation, token limits and real mail provider/custody later.
- **O03 Zaflas/legal:** actual company/merchant data, fiscal classification/
  system, tax obligations, terms/refund/age/privacy/retention approval.
- **O04 economic risk:** late/duplicate payment, holds/refund reserves,
  reviewer permissions, thresholds and documented consumer escalation.
- **O05 presentation:** naming/defaults and approved baseline cosmetics,
  true consumers and younger-player safety controls.

Executor report includes integration SHA, implemented folders/contracts,
synthetic test commands/results, boundary invariants, decisions, known risks,
deferred work and explicit confirmation that external adapters are off.
No purchased item, financial operation or account recovery is executed here.
