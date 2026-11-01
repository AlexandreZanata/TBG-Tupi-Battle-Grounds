# Initial documentary contracts

These versioned JSON Schemas and synthetic examples make the foundation
reviewable without implementing provider/email/fiscal integration. They are
not mounted HTTP DTOs, executable payment flows or actual Asaas payloads.

- [Order snapshot](order-snapshot.schema.json), [example](examples/order-snapshot.json).
- [Normalized synthetic observation](provider-observation.schema.json),
  [example](examples/provider-observation.json).
- [Recovery challenge metadata](recovery-challenge.schema.json),
  [example](examples/recovery-challenge.json).
- [Opponent preferences](opponent-preferences.schema.json),
  [example](examples/opponent-preferences.json).

Schema version 1 is local to these contracts, not a reserved DB migration
or game protocol version. Fixture SKU and 2,000-cent price are illustrative,
not approved products/prices. The all-zero digest is a deliberately nonsecret
fixture, never a recovery credential. `synthetic` observations cannot be
misrepresented as production or Sandbox receipt evidence.

Money uses safe integral minor units. Structural validation does not enforce
cross-field business invariants: line sums equal total, refunds never exceed
received gross, line IDs unique, no grant before verified receipt, challenge
expiry after issue, and no duplicate effect. The validation plan requires
those separately. Initial line bound 20 and quantity 1 are Proposed initial
nonconsumable-cosmetic limits, not new `Shop`/army limits.

No plaintext token, email, CPF, API key or permanent account key belongs in
these fixture snapshots. Future private schema/DTO design handles required
PII with explicit ownership/access and purpose. Provider external envelopes
allow extra fields under the adapter policy; these strict internal schemas
must not be applied blindly to raw provider Webhooks.
