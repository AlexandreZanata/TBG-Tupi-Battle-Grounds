# Zaflas issuance, consumer rights and private data

> Planning checklist, not final terms or a tax classification. **Zaflas is
> the owner's company that will issue documents**, not a payment API invented
> by this plan. Legal/accounting review is a launch gate for real sales.

## 1. Issuer and fiscal workflow

Owner/accountant supplies the actual legal name, CNPJ, address, municipal/state
registrations as applicable, municipality, tax regime, activity/service code,
product/license classification, customer-data requirements and chosen issuance
system. Verify the Asaas merchant account and seller/issuer responsibilities.
Never copy a sample service code, tax rate or municipal registration into
production. Company label alone does not establish fiscal eligibility.

Asaas documents NFS-e scheduling through `POST /v3/invoices`, municipal
configuration/service lookup and asynchronous authorization. That **does not
establish that NFS-e is the correct document for every TBG sale**. Zaflas's
accountant must decide whether that route fits the specific product and
jurisdiction, or whether a different authorized fiscal system is needed.
No Zaflas API endpoint or municipal credential is assumed to exist.
See [Asaas fiscal guide](https://docs.asaas.com/docs/emitindo-notas-fiscais-de-servico).

Define a fiscal request port independent of the payment adapter: order ID,
issuer snapshot/version, classified line values, allowed customer billing
data and correlation key. Initial port captures synthetic requests only.
States: Required/Pending → Submitted/Scheduled → Authorized/Error, plus
CancellationPending/Canceled/CancellationDenied. Persist issued number,
authorization evidence and permitted document references only after verified
issuance; invoice scheduling/HTTP success alone is not a valid fiscal document.

Do not conflate receipt of payment with tax authorization. Fulfillment follows
the approved commercial/legal policy while fiscal errors remain visible and
are escalated promptly; no repeated charge, second entitlement or silent
“invoice issued” status on retry. A refund may require fiscal cancellation
or a separate correction; applicable deadlines/rules are accountant inputs,
not assumed automatic consequences of PAYMENT_REFUNDED.

User invoice download/detail is authenticated and owner-scoped. PDFs/XML and
tax IDs never appear in public profiles, generic support logs or predictable
public URLs. Do not fetch arbitrary invoice URLs from user input/webhooks;
use allowlisted provider retrieval and controlled private document delivery.
Finance/fiscal operators have distinct access and safe audit metadata.

## 2. Consumer-facing documents to prepare before selling

Draft outlines, then obtain approval for:

- Seller identification/contact, product/license description, cosmetic-only
  effect, total price/currency, payment method, delivery timing and limitations.
- Terms/license version attached to the accepted order, available for saving;
  no retroactive price/rights changes to that snapshot.
- Refund/cancellation route in the same purchase environment, immediate
  request receipt, status tracking, support and escalation. P65 can supply
  the case interface, not authority to deny statutory rights.
- Withdrawal-right handling grounded in CDC art. 49, which provides a
  seven-day rule for qualifying distance transactions. Counsel determines
  its precise application/start and other applicable remedies for this
  offering. Do not assume “digital item used = no refund” or use provider
  fee retention as a blanket deduction from a legally due consumer refund.
- Clear effect of refund/dispute on the specific cosmetic grant; no automatic
  loss of unrelated purchases/account progress or forced waiver at checkout.

Primary legal sources: [CDC, official Senate text](https://legis.senado.gov.br/norma/549954/publicacao/34619932)
and [electronic-commerce decree](https://www.planalto.gov.br/ccivil_03/_ato2011-2014/2013/decreto/d7962.htm).
Direct Planalto fetches were blocked during research; its indexed official
text was available, with [Senacon guidance](https://www.gov.br/mj/pt-br/assuntos/seus-direitos/consumidor/notas-tecnicas/anexos/40-2013.pdf)
as official operational context. No bill/draft was treated as enacted law.

## 3. Privacy, email and younger players

Map purposes separately: recovery email, required billing/tax records, fraud
evidence, provider reconciliation and optional marketing. Identify legal
bases, retention/access, processor contracts, data-subject requests, breach
response and deletion limits for fiscal obligations. Collect CPF/customer
details only for an authorized purchase/fiscal purpose, not to play or simply
restore a key. Mask display and avoid PII in metrics/URLs/logs.

Keep recovery/email secrets and private billing data outside gameplay/public
DTOs. Document protection/key custody and consistent private backups. Deleting
an account does not erase legally retained fiscal records automatically;
minimize/pseudonymize links where lawful while preserving proven entitlements
and dispute obligations. Do not present blanket consent as the only basis
for every legally required record.

Use [LGPD, updated Chamber text](https://www2.camara.leg.br/legin/fed/lei/2018/lei-13709-14-agosto-2018-787077-normaatualizada-pl.html).
Age/purchase controls need current review: [ANPD ECA Digital guidance](https://www.gov.br/anpd/pt-br/assuntos/eca-digital/)
and [the October 2026 ANPD guide announcement](https://www.gov.br/anpd/pt-br/assuntos/noticias/anpd-publica-guia-que-esclarece-quem-deve-cumprir-e-quais-sao-as-responsabilidades-das-plataformas-na-protecao-de-criancas-e-adolescentes).
Do not assume an “18+” label alone removes applicability. P67 proposes fixed
cosmetics only, no paid randomized rewards. Age assurance/parental purchase
requirements and privacy design are counsel/owner gates before monetization,
not new third-party identity checks activated by this branch.

## 4. itch.io is a separate sales channel

Download-package sales on itch.io use its configured payment model, not
Asaas transactions. The [official seller documentation](https://itch.io/docs/creators/payments)
distinguishes Direct to you from Collected by itch.io/Payouts, with different
merchant responsibilities. Confirm the actual mode/accounting treatment;
do not assume itch.io is merchant of record in every configuration or issue
duplicate fiscal documents for a platform-handled transaction blindly.

Buying an itch.io download is not automatically proof of an in-game cosmetic
entitlement. Linking/redeeming a platform purchase needs a later explicit
receipt/claim contract, server verification and duplicate/refund policy.
No such integration or real itch.io sale is performed in P67.
