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
