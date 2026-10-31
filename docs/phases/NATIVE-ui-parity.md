# Native UI parity checklist

The owner requires complete parity before the first Linux publication.
HTML remains available until the owner approves the Bevy screens.

Apply [VALIDATION.md](../VALIDATION.md): record work-front progress here,
consolidate the relevant checks once, and defer heavy/full-suite execution
until explicitly requested. This acceptance checklist does not authorize
each parallel front to repeat complete builds or browser campaigns.

- [ ] Account: create/login, bounded editable name/key, mask/reveal, copy,
  save acknowledgment, logout and server errors.
- [ ] Lobby: profile, ranked stats, feature cards, news, all play modes.
- [ ] Friends: room creation, five-character code entry, copied full link,
  challenge, cancellation and rematch.
- [ ] Settings: language, audio, fullscreen, vsync, quality and account.
- [ ] Arenas: all existing arena rules, descriptions and Practice selection.
- [ ] Strongholds: all four kinds, statistics, passives and branch details.
- [ ] Arcanas: three families, fifteen nodes, prerequisites, costs, affected
  Archetypes, ranks, defaults/reset and complete preset persistence.
- [ ] Live: online counts, accounts, public matches, watch/copy links,
  stale/error states and spectator exit/count.
- [ ] HUD: authoritative HP/max HP, Energy/regen, Shop portraits, inspect,
  Evolution, Fortification, War Council, two-step surrender.
- [ ] Queue/result: room/challenge copy, cancellation, outcome, Ranked
  change, rematch and replay/menu.
- [ ] Desktop/phone captures, English/Portuguese, 720p/1080p readability.
- [ ] Same screens in WASM; public `/ranking/` remains HTML (D-21).

## Owner review

Pending concrete rendered screens. No subjective sign-off is assumed.

## Implementation handoff — 2026-10-31

The Bevy implementations for the listed desktop surfaces are integrated in
`crates/tbg-client/src/ui/`. Persistent D-61 creator and origin links were
added to the shared renderer and existing HTML pages. The unchecked boxes
above are **acceptance**, not a claim that implementation is absent: final
visual review and the consolidated gameplay/browser checks remain pending.
Existing `.local/native-captures/` images are development evidence from before
the final attribution/button changes. No new gallery campaign or mobile matrix
was started after adoption of the minimal validation policy.
