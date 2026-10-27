# TBG Glossary (Ubiquitous Language)

> Code, APIs, docs, and agents **must** use these terms exactly.
> Do not invent synonyms in implementation.
> Ported from `archive/docs/GLOSSARY.md`; Unity/Nakama-specific meanings
> generalized, retired tooling terms marked as legacy.

## TBG

Official product name — **Tupi Battle Grounds**. Real-time auto lane
strategy game, browser-first. Code name: `TBG`.

## Match

A single live session between two players from start to outcome
(victory, defeat, or forfeit). Code name: `Match`.

## BattleLine

The single horizontal lane where units deploy and combat resolves.
No lateral movement. Code name: `BattleLine`.

## Unit

A deployable character controlled by a player during a match. Has
archetype, stats, faction, and energy cost. Not a cosmetic skin.
Code name: `Unit`.

## Archetype

Mechanical role of a unit. Enum values: `Tank`, `DPS`, `Support`,
`Ranged`, `Control`. Code name: `Archetype`.

## Faction

Thematic group defining unit visual language and synergy rules.
Enum values: `Humans`, `Undead`, `Machines`, `Plants`.
Code name: `Faction`.

## Energy

In-match resource spent to purchase and deploy units. Regenerates per
server-owned rules. Not a premium or soft currency. Code name: `Energy`.

## Shop

Per-player rotating offer of units during a live match: three options per
rotation cycle, timers synchronized by the authoritative server.
Not a cosmetic store. Code name: `Shop`.

## Evolution

In-match upgrade path for a deployed unit along a branching choice tree
(choice at one tier conditions the next). Not account-level progression.
Code name: `Evolution`.

## Synergy

Bonus effect when units of the same `Faction` (or a defined combo) are
active together. Code name: `Synergy`.

## SlotCount

Space a unit occupies on the `BattleLine`: `1` or `2`. Larger units must
be visually and mechanically consistent with their slot count.
Enum values: `One`, `Two`. Code name: `SlotCount`.

## PlayerIntention

Client-sent request for an action (purchase unit, evolve, reposition).
The server validates it before applying. Not a confirmed state change.
Code name: `PlayerIntention`.

## AuthoritativeServer

The server process that owns truth for combat, energy, shop timers, and
match outcome. The client renders and predicts only.
(Legacy term named Nakama; generalized for the Rust server.)
Architectural concept, no code name.

## Cosmetic

Purchasable visual item (skin, VFX, emote, board theme) with **no**
gameplay stat effect. Code name: `Cosmetic`.

## Inventory

Persistent collection of owned cosmetics bound to a player account.
Code name: `Inventory`.

## Commander (TBD)

Preferred player-facing address for the human playing a match; placeholder
until brand voice is finalized. Technical code name: `Player`.

## MVPUnitRoster

Initial launch set of **12 unique units** (balance and art target before
scale). The 2.0 reboot proposes delivering the first **8** of these first
as a scope decision; the 12-unit target stands.

## Legacy tooling terms (retired — do not reuse for new concepts)

These names refer to the retired Unity/Python implementation only.
They are recorded here so future work does not collide with them.

| Legacy term | Meaning (historical) |
|-------------|----------------------|
| `CharacterDefinition` | JSON unit parameter set under legacy `content/characters/definitions/` |
| `CharacterCatalog` | C# entry point of legacy `TBG.CharacterLibrary` |
| `CharacterStudioWindow` / `CharacterStudio` | Legacy Unity editor + web authoring tool |
| `ProceduralCharacterBuilder` | Legacy runtime primitive-geometry assembler |
| `Mesh3DCharacterFactory` | Legacy `.glb` loader |
| `MaterialProfile` / `BehaviorProfile` / `CharacterTextures` / `CharacterAssetMode` / `VisualGrammar` / `CharacterInstance` | Legacy asset/loader contracts; 2.0 asset contracts will be defined under `tbg-assets` with new names where meanings differ |
