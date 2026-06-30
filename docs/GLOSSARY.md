# Domain Glossary

> Ubiquitous language for TBG. Code, APIs, docs, and agents **MUST** use these terms exactly.  
> Do not invent synonyms in implementation.

---

## TBG

**Definition:** Official product name — **Tupi Battle Grounds**. Real-time auto lane strategy mobile game.  
**Not the same as:** "Auto Lane Strategy" (internal working title — use TBG in user-facing copy unless rebranded).  
**Code name:** `TBG`

---

## Match

**Definition:** A single live PvP session between two players from start to outcome (victory, defeat, or forfeit).  
**Not the same as:** Round (sub-segment within a match, if defined later).  
**Code name:** `Match`

---

## BattleLine

**Definition:** The single horizontal lane where units deploy and combat resolves. No lateral movement.  
**Not the same as:** Board (may include UI zones outside the combat lane).  
**Code name:** `BattleLine`

---

## Unit

**Definition:** A deployable character controlled by a player during a match. Has archetype, stats, faction, and energy cost.  
**Not the same as:** Cosmetic skin (visual only, no stat change).  
**Code name:** `Unit`

---

## Archetype

**Definition:** Mechanical role of a unit: Tank, DPS, Support, Ranged, or Control.  
**Enum values:** `Tank`, `DPS`, `Support`, `Ranged`, `Control`  
**Code name:** `Archetype`

---

## Faction

**Definition:** Thematic group defining unit visual language and synergy rules.  
**Enum values:** `Humans`, `Undead`, `Machines`, `Plants`  
**Code name:** `Faction`

---

## Energy

**Definition:** In-match resource spent to purchase and deploy units. Regenerates per server tick rules.  
**Not the same as:** Premium currency or soft currency (metagame, outside match).  
**Code name:** `Energy`

---

## Shop

**Definition:** Per-player rotating offer of units during a live match. Three options per rotation cycle; timer synchronized by authoritative server.  
**Not the same as:** Cosmetic store (metagame IAP).  
**Code name:** `Shop`

---

## Evolution

**Definition:** In-match upgrade path for a deployed unit along a branching choice tree (e.g. choice A vs B at tier 1, dependent tier 2).  
**Not the same as:** Account-level progression or cosmetic unlock.  
**Code name:** `Evolution`

---

## Synergy

**Definition:** Bonus effect when units of the same Faction (or defined combo) are active together.  
**Code name:** `Synergy`

---

## Slot

**Definition:** Space a unit occupies on the BattleLine — `1` or `2`. Larger units must be visually and mechanically consistent with slot count.  
**Enum values:** `One`, `Two`  
**Code name:** `SlotCount`

---

## PlayerIntention

**Definition:** Client-sent request for an action (purchase unit, evolve, reposition). Server validates before applying.  
**Not the same as:** Confirmed game state change.  
**Code name:** `PlayerIntention`

---

## AuthoritativeServer

**Definition:** Nakama match module that owns truth for combat, energy, shop timers, and match outcome.  
**Not the same as:** Unity client (prediction and display only).  
**Code name:** N/A — architectural concept

---

## Cosmetic

**Definition:** Purchasable visual item (skin, VFX, emote, board theme) with **no** gameplay stat effect.  
**Not the same as:** Unit unlock that changes balance.  
**Code name:** `Cosmetic`

---

## Inventory

**Definition:** Persistent collection of owned cosmetics bound to a player account.  
**Code name:** `Inventory`

---

## Commander

**Definition:** `[TBD]` — preferred player-facing address once brand voice is finalized. Placeholder for "the human playing the match."  
**Code name:** `Player` (technical)

---

## MVPUnitRoster

**Definition:** Initial launch set of **12 unique units** (balance and art target before scale).  
**Code name:** N/A

---

## CharacterDefinition

**Definition:** Parameter set describing a procedural unit — archetype, faction, proportions, colors, accessories, evolution stage. Persisted as JSON under `content/characters/definitions/`.  
**Not the same as:** ScriptableObject cache in Unity (derived import, not source of truth).  
**Code name:** `CharacterDefinition`

---

## ProceduralCharacterBuilder

**Definition:** Runtime component that assembles unit geometry from primitives according to a `CharacterDefinition`. No per-unit `.fbx` files.  
**Code name:** `ProceduralCharacterBuilder`

---

## CharacterStudio

**Definition:** Unity Editor tool (`CharacterStudioWindow`) that creates, edits, deletes unit JSON on disk and exports card images.  
**Not the same as:** In-game unit shop.  
**Code name:** `CharacterStudioWindow`

---

## VisualGrammar

**Definition:** Fixed set of shapes, proportion rules, and faction palettes used to generate all procedural units consistently.  
**Code name:** N/A

---

## _(Add terms below as domain grows)_

<!--
Agent: do not invent domain terms in code without an entry here.
Format: see agent-rules/11-documentation-and-glossary/ubiquitous-language.md
-->
