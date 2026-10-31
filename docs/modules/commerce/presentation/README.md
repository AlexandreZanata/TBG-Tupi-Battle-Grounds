# Opponent emote and cosmetic controls

> Proposed presentation contract; no emote protocol or paid skin is added by
> this planning branch. Existing global audio mute is not an opponent-emote mute.

Provide distinct localized settings, usable before/within a match:

- **Mute opponent emotes:** suppress that opponent's cosmetic emote animation,
  text/sticker and associated emote audio. Keep own effects and game-critical
  attack/status/team/Stronghold cues. Do not mute all combat sound by accident.
- **Hide opponent cosmetics:** render the approved neutral baseline for the
  opponent's selected kind/form instead of their optional skin/ornament.
  Do not hide real enemies, projectiles, evolution/status information or
  mechanical StrongholdKind identity; it differs from StrongholdStyle.

These are local viewing preferences, not inventory or equipment mutations.
The owner still owns/equips the cosmetic and the server still validates it.
Purchased cosmetics confer no right to override another player's visibility
preferences. Make that limitation clear in product previews/terms before sale.

Preserve canonical silhouette, reach/aim, hit timing, colors/team readability
and collision/target identity for each kind/form, including P59 rigs and
Stronghold variants. Apply a rendering resolver after authoritative identity,
not a replacement UnitKind or a client change to simulation. Test both seats,
opponent summons, multiple forms and mid-match preference changes without
resetting the match or swapping models unsafely in a render frame.

Persist preferences through the existing browser/native preference abstraction
without adding billing/email fields to local settings. New keys must have
reviewed defaults and i18n/accessibility; proposed initial defaults are false
for both, subject to approved younger-player safety policy. Toggle emote mute
can act immediately; model replacement uses a safe visual transition while
keeping current game state/animation contacts. No opponent notification needed.

If emotes/skins do not exist on the chosen implementation base, deliver the
preference/resolver contract and synthetic presentation tests with a truthful
unavailable label. Do not ship an enabled switch that claims to suppress
events when no actual consumer reads it. Do not create emote/chat networking
or author a paid cosmetic catalog merely to finish this foundation.

Suggested future consumers: `audio.rs` emote-specific routing, visual figure/
skin selection, Settings, P62 preview and P63 celebration where eligible.
Own victory animation remains independent; do not hide a real combat action
or change a winner because an opponent muted cosmetic events.
