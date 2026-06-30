# Character and Unit Creation Guide

> How to design, balance, and produce unique units for TBG.  
> **Principle:** define mechanics first, art second.

---

## 0. Scope and constraints

A 16 GB VPS hosts the **game server** (logic, matches, database). It does **not** limit client graphics — rendering runs on the player's device.

The 2D vs 3D choice depends on:

- Production time and budget
- Target device tier (low-end vs flagship phones)
- Visual polish needed to compete in market

For TBG's single-lane, non-lateral-movement design, **2D with a strong consistent style** is the recommended starting point.

---

## 1. 2D vs 3D

### Option A — 2D (recommended for MVP)

**Advantages**

- Faster production (hours per unit vs weeks in 3D).
- Easier to hire 2D illustrators than mobile 3D generalists.
- Runs well on older phones.
- Matches single-lane gameplay — no free camera or complex locomotion.

**Disadvantages**

- Some audiences perceive 2D as less "premium" than polished 3D.

### Option B — 3D

**Advantages**

- AAA visual potential; dynamic lighting; shared rigs across units at scale.

**Disadvantages**

- Expensive pipeline: model, rig, animate, texture, mobile optimize per unit.
- No gameplay benefit for fixed-lane design.
- High risk of mediocre 3D vs distinctive 2D (Hearthstone, Clash Royale, Brotato).

### Recommendation

Start **2D with bold consistent art**. Migrate to 3D only if the product scales and budget justifies a full pipeline rebuild.

---

## 2. Designing unit abilities

Define **mechanical role** before any visual work.

### 2.1 Archetypes

Each unit must map clearly to one role so players read the board instantly:

| Archetype | Role | Example behavior |
|-----------|------|------------------|
| Tank | Absorb damage, hold the line | High HP, low damage, may occupy 2 slots |
| DPS | Eliminate threats quickly | Low HP, high damage |
| Support | Strengthen allies | Heal, attack buff, resurrection |
| Ranged | Attack from back line | Low HP, high range |
| Control | Disrupt enemies | Slow, stun, damage reduction |

### 2.2 The "why pick this unit?" test

For every unit at a given cost: **"In which situation would a player choose this over another unit at the same cost?"**

If there is no clear answer, the unit is redundant — fix the sheet before art.

### 2.3 Trade-off matrix

Every strong ability needs a visible weakness:

| Strength | Typical cost |
|----------|----------------|
| High damage | Low HP or high energy cost |
| Heal | Low or no damage |
| High range | Slow spawn or high cost |
| AoE damage | Low single-target damage |

### 2.4 Unit specification template

Maintain a master balance spreadsheet before art or code:

```
Name:
Archetype:
Energy cost:
Slots occupied: 1 or 2
HP:
Damage:
Range:
Passive (if any):
Evolution 1 (choice A):
Evolution 1 (choice B):
Evolution 2 (depends on prior choice):
Faction synergy:
```

### 2.5 One clear hook per unit

Avoid stacking five effects on one unit. Prefer **one strong readable trait**, with depth added through evolution branches (e.g. Archer → Sniper → Explosive), not upfront complexity.

---

## 3. Visual design (silhouette, color, scale)

### 3.1 Silhouette first

A unit must be identifiable **from black silhouette alone** — critical on small mobile screens.

**Test:** draw solid black shape only. If the archetype is not obvious, redesign before coloring.

### 3.2 Color with meaning

Assign palette by **archetype or faction**, not decoration:

| Archetype | Suggested palette |
|-----------|-------------------|
| Tank | Earthy / metallic |
| DPS | Red / orange |
| Support | Green / light blue |
| Control | Purple |

### 3.3 Scale communicates slots

Units occupying **2 slots** must look visibly larger/wider than 1-slot units.

### 3.4 Single art style

Pick one style for the entire game: cartoon, semi-realistic, pixel, flat vector, etc. **Never mix styles** across units.

### 3.5 Faction art direction

| Faction | Palette | Materials | Silhouette language |
|---------|---------|-----------|---------------------|
| Humans | `[TBD]` | Flesh, cloth | `[TBD]` |
| Undead | `[TBD]` | Bone, rot green | Organic, decayed |
| Machines | `[TBD]` | Metal, cold blue | Angular, rigid |
| Plants | `[TBD]` | Brown, living green | Organic curves |

Players should identify opponent faction at a glance.

---

## 4. Production tools (2D)

### 4.1 Illustration

| Tool | Use case |
|------|----------|
| Procreate (iPad) | Digital illustration |
| Krita (free) | PC/Mac/Linux illustration |
| Aseprite | Pixel art + sprite animation |
| Photoshop / Illustrator | If already in workflow |

### 4.2 Animation (idle, attack, death)

| Tool | Notes |
|------|-------|
| Spine 2D | Industry standard for mobile 2D (Clash Royale, Brawl Stars pattern) |
| DragonBones | Free skeletal 2D animation |

Skeletal 2D is far cheaper than frame-by-frame and integrates with Unity.

### 4.3 Generative AI

Use only for **concept exploration** — not final production art. Reasons: copyright risk, inconsistent style across units.

### 4.4 Reference boards

Maintain Pinterest/Milanote boards (Clash Royale, Brawl Stars, Hearthstone, Brotato) before production lock.

---

## 5. Recommended workflow

```
1. Archetype + balance sheet (spreadsheet)
        ↓
2. Validate numbers in spreadsheet (no art)
        ↓
3. Silhouette sketch — readability test
        ↓
4. Final 2D illustration
        ↓
5. Cut layers for Spine/DragonBones (if skeletal)
        ↓
6. Animations: idle, spawn, attack, death
        ↓
7. Unity prefab + stats from spreadsheet
        ↓
8. Playtest → adjust numbers → repeat
```

### MVP production rhythm

For **12 unique MVP units**, produce in batches of **3–4 per faction**. Validate archetype and readability before the next batch to avoid rework.

---

## 6. Decision summary

| Decision | Recommendation | Rationale |
|----------|----------------|-----------|
| Visual format | 2D | Speed, cost, fits lane gameplay, broad device support |
| Animation | Spine 2D / DragonBones | Mobile standard, good cost/benefit |
| Design order | Mechanics before art | Avoid redrawing units that fail role clarity |
| Identity | Silhouette + faction/archetype color | Fast competitive readability |
| Generative AI | Reference only | Style consistency and rights |

---

## 7. Next steps

- [ ] Build master balance spreadsheet for 12 MVP units
- [ ] Lock global art style (cartoon / flat / semi-realistic)
- [ ] Pilot one unit per faction through full pipeline to Unity
- [ ] Decide internal Spine production vs freelancer
- [ ] Link finalized faction palettes to [brand-system.md](brand-system.md)
