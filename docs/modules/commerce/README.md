# Commerce foundation — module boundary and documentation map

> **P67, planning/documentary foundation only — 2026-10-31.** These are real
> versioned folders and contracts, not a running payment service. No Asaas
> SDK/client, API credential, webhook route, mail transport, invoice issuer,
> database migration or sale is activated by this branch.

Start with [P67](../../phases/P67-commerce-recovery-foundation.md).

| Folder | Responsibility |
|---|---|
| [accounts](accounts/README.md) | Recovery email enrollment, verification, key rotation and takeover protection |
| [payments](payments/README.md) | Catalog quotes, orders, payment/refund ledger, inventory grants and reconciliation |
| [providers/asaas](providers/asaas/README.md) | Official current-source findings and future Asaas mapping; no real adapter |
| [fiscal](fiscal/README.md) | Zaflas as seller/issuer, legal/privacy launch gates and fiscal lifecycle |
| [presentation](presentation/README.md) | Opponent emote mute and cosmetic visibility without changing combat |
| [validation](validation/README.md) | Synthetic business/security scenarios and acceptance |
| [contracts](contracts/README.md) | Machine-readable initial schemas and synthetic examples |

## Proposed code ownership, not directories silently claimed implemented

```text
crates/tbg-server/src/
  commerce/
    domain/          # money, quote/order/payment/refund/grant states
    application/     # idempotent business operations and outbox
    providers/asaas/ # sole future payment adapter, not a framework
    persistence/     # bounded commerce worker, ledger and migrations
    reconciliation/ # uncertain external operations and retries
    fiscal/          # issuer-neutral fiscal request and status projection
    http/            # authenticated private DTOs and future webhook ingress
  identity/recovery/ # uses authoritative account store, not a second account DB
crates/tbg-client/src/
  cosmetics/         # display resolver and local opponent preferences
```

Keep this inside existing crates initially. A separate Cargo crate/service
is unnecessary until a proven dependency/deployment boundary warrants it.
Do not add placeholder Rust modules, dead handlers or empty trait frameworks
just to reproduce this tree. Materialize code folders when their offline
contracts/consumers are implemented in the next execution front.

Commerce owns server business records, not combat. `tbg-core` gets no email,
HTTP, PII, fiscal, clock or monetary I/O. Core `Shop` purchases with `Energy`
remain different from cash purchases of cosmetic `Inventory` entitlements.
No cosmetic changes stats, targetability, timing, Faction, Evolution or Elo.

Identity owns credential rotation in the existing account authority. Inventory
references stable AccountId, not email/key/provider customer ID. Recovery
therefore preserves purchases and ranking instead of creating another account.
P65 staff roles do not automatically grant finance, recovery or invoice powers.

Recommended eventual commerce persistence: separate SQLite/worker for order/
provider/inbox/outbox/grant/fiscal records, with bounded admission. Account
verification/rotation stays in the authoritative account store. Document
cross-store outbox/retry behavior; cross-file SQL REFERENCES are not integrity.
If Inventory is stored elsewhere, use a unique grant command and durable ack
rather than pretend an atomic transaction spans two independent stores.

P67 delivers the design and offline foundation contract. Future sandbox and
production integration each require explicit authorization, reconciled official
contracts and legal/accounting decisions. Initial provider: **Asaas**, Pix
only. Zaflas is the owner's issuing company, not an invented second gateway.
