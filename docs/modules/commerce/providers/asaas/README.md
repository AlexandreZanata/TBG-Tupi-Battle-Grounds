# Asaas v3 — official documentation dossier and future adapter contract

> Checked **2026-10-31**, official `docs.asaas.com` sources only for the
> provider. The requested “Assas” is interpreted as **Asaas**. This is not an
> integration: no credentials, API requests, Sandbox account or webhook was
> created. Recheck endpoints/contracts before any authorized live adapter.

## 1. Facts checked and corresponding TBG decisions

| Official behavior | TBG consequence | Primary source |
|---|---|---|
| Authenticated requests use `access_token`; identify the app with User-Agent; separate Sandbox and Production bases/keys | Backend-only secrets, explicit environment/account scope, redacted Config/logs, no client API key | [Authentication](https://docs.asaas.com/docs/autentica%25C3%25A7%25C3%25A3o-1) |
| Customer creation requires name and cpfCnpj for the documented domestic contract; duplicates are permitted | Separate restricted billing identity; reuse stored provider customer; don't demand CPF merely to play/recover; reconcile unknown creates | [Create customer](https://docs.asaas.com/reference/criar-novo-cliente) |
| Current Pix guide creates PIX through `POST /v3/lean/payments` and reads QR via `GET /v3/payments/{id}/pixQrCode` | Map the documented lean route or separately reviewed regular route; never concatenate `/v3/v3`; expose only owner's payment presentation | [Pix guide](https://docs.asaas.com/docs/cobrancas-via-pix) |
| QR response includes payload/image/expiration; QR validity is not necessarily the UI checkout countdown | Persist actual provider expiry; late payment is reconciled, not discarded on local timeout | [Pix guide](https://docs.asaas.com/docs/cobrancas-via-pix) |
| Documented Pix progression reaches PAYMENT_RECEIVED; CONFIRMED is a different settlement state used by other methods | Initial Pix fulfillment requires verified RECEIVED and PIX binding; no blanket “any paid-like event grants” rule | [Payment events](https://docs.asaas.com/docs/webhook-para-cobrancas) |
| `GET /v3/payments/{id}` supports targeted current-state lookup | Bounded validation/reconciliation of an authenticated merchant's known payment; not constant polling or arbitrary URL fetch | [Retrieve payment](https://docs.asaas.com/reference/recuperar-uma-unica-cobranca) |
| Unknown request outcome may already have produced a resource; externalReference is correlation | Durable local operation identity, concurrency guard and reconciliation before create retry; do not invent a universal Idempotency-Key guarantee | [Retries/idempotence](https://docs.asaas.com/docs/retries-e-idempot%C3%AAncia) |
| Webhooks authenticate configured authToken via `asaas-access-token`, not Bearer/Basic or the API key | Dedicated strong callback credential with constant-time validation, TLS and rotation; provider callbacks are not player-cookie endpoints | [Webhook receiving](https://docs.asaas.com/docs/receive-asaas-events-at-your-webhook-endpoint) |
| Notifications are at least once; same event ID survives redelivery | Unique inbox key `(provider, environment, merchant, event_id)`; no repeated grant/refund | [Webhook receiving](https://docs.asaas.com/docs/receive-asaas-events-at-your-webhook-endpoint) |
| Current FAQ recognizes **HTTP 200**, not arbitrary 2xx; delivery type affects ordering | Authenticate, validate envelope, durably persist, then 200; async rules and order-independent state projection. Do not use 202/204 as ack | [Webhook FAQ](https://docs.asaas.com/docs/webhooks-faq) |
| Repeated delivery failures can pause the queue; webhook storage is finite | Queue-lag/failure alerts, durable local inbox, recovery/reconciliation runbook independent of provider replay availability | [Webhook introduction](https://docs.asaas.com/docs/about-webhooks) |
| Pix refunds support full/multiple partial returns bounded by payment; fees may not be returned and balance may be insufficient | Refund ledger/reservations and seller reserve; failed/uncertain refund stays pending for support, never falsely completed | [Refund](https://docs.asaas.com/reference/estornar-cobranca) |
| Balance events distinguish PIX_INFRACTION/MED; paymentId may be null | Store financial hold separately, correlate only with evidence, human review for unassigned holds | [Balance holds](https://docs.asaas.com/docs/eventos-para-bloqueios-de-saldo) |
| NFS-e needs fiscal configuration/service mapping; authorization is asynchronous | Only consider Asaas fiscal adapter after Zaflas accounting approval; scheduling is not authorization | [Fiscal issuance](https://docs.asaas.com/docs/emitindo-notas-fiscais-de-servico) |
| INVOICE_AUTHORIZED confirms issuance; cancellation can fail independently | Fiscal state/proof separate from payment/refund; authorized document availability and correction handling | [Fiscal events](https://docs.asaas.com/docs/webhook-para-notas-fiscais) |

The current Pix guide also describes long QR validity after due date and a
special no-registered-key same-day path with announced future removal. Do
not build business expiry on either shortcut: use the actual returned
expiration and approved merchant setup. Avoid copying provider timeframes
or fee tables into game code without contract/config review.

## 2. Future ingress and verification flow

There is no HTTP callback route in this foundation. A later adapter must:

1. Accept only expected POST/content type/size at the configured final HTTPS
   endpoint. Verify `asaas-access-token` before storing financial payloads;
   configured authToken is independent of API credential. Apply source-IP
   controls only from current official infrastructure docs, not hardcoded guesses.
2. Parse required envelope fields safely and tolerate **unknown extra provider
   fields**; strict own-command DTO rules must not break provider evolution.
   Invalid auth/malformed transport is not acknowledged as a stored event.
3. Persist allowed bounded event/evidence and digest in durable inbox before
   returning HTTP 200. Duplicate event/same digest acknowledges without repeat.
   Duplicate ID/different digest is a security inconsistency: quarantine/alert,
   retain original evidence, never grant from the replacement.
4. A valid but unsupported event is recorded/acknowledged and ignored or
   quarantined by policy; never pause the entire provider queue because it
   is unrelated to the initial product. No lost success ack on slow Inventory.
5. Resolve merchant/environment/payment/customer/order from server bindings;
   confirm a first positive grant or entitlement restoration through a fixed
   authenticated current-payment lookup under the merchant API credential.
   If unavailable, keep validation pending; callback token alone must not
   authorize a fabricated success. Never follow URLs embedded in a webhook.
   Compare gross amounts
   exactly, not the net after fees, and treat optional fields accordingly.
6. Apply state facts under expected version. Received after Refund/Hold does
   not restore access without a reconciled business decision. Inbox event
   timestamp/arrival order is not a universal total-order guarantee.
7. Commit one grant/refund/fiscal command through an outbox and expose only
   the account owner's safe purchase status. Client redirect/return URL is
   presentation only, never proof that funds arrived.

No callback shared secret makes compromised merchant credentials harmless.
Least-privilege keys, rotation, audit, reconciliation and separation from
client/admin/public-news authority remain necessary. Do not claim webhook
HMAC signatures that the checked Asaas contract does not specify.

## 3. Offline fixtures and integration handoff

Use synthetic normalized observations for received/duplicate/refunded/held/
out-of-order/unknown cases, not real payer data or copied provider secrets.
An offline replay must show the same grant/refund ledger after duplication
and restart. These fixtures do not prove actual provider connectivity.

Later integration must explicitly approve: merchant/legal identity, account
activity eligibility, Pix setup, secrets/callback rotation, Sandbox account,
data retention/rate limits, refund funding, monitoring, authoritative payment
verification and a small authorized reconciliation exercise. This phase
does not create customers/charges in Sandbox, pay test Pix, register hooks,
configure withdrawals or enable production endpoints.
