# Art Direction — Medieval Miniatures (TBG 2.0 visual suite)

> The visual style guide for phases P15–P31. Every visual phase reads the
> sections it names. Game rules stay in [GAME-DESIGN.md](GAME-DESIGN.md);
> technology in [ARCHITECTURE.md](ARCHITECTURE.md) §7; the reasons in
> [DECISIONS.md](DECISIONS.md) D-22–D-24. Status: **Proposed** until the
> owner signs off P15; values marked *start* are tuned by P15.

## 1. Pillars

1. **Readable first.** A player on a phone at arm's length tells side,
   unit, role, and health in under a second. Beauty never costs that.
2. **Medieval tabletop miniatures.** Hand-painted-looking figurines on a
   diorama battlefield between two castles: heroic proportions, oversized
   weapons and shields, warm light, clean shapes.
3. **One family.** Every unit, prop, card, and screen shares the same
   proportions, palette rules, detail density, and motion language.
4. **Light.** Same budgets on every device class; quality tiers remove
   cost, never information.

## 2. Look

- **Shapes:** low-poly with small bevels (rounded boxes, lathes, extrudes);
  smooth-shaded large forms, flat-shaded small details. No noisy detail
  under ~6 px on a phone screen.
- **Proportions (start):** about 4 heads tall; head = 0.26 of height;
  hands and feet 1.3× realistic; weapon or shield ≥ 0.6 of body height.
  Two-slot units (Bone Brute, Ogre Archer) are 1.5× scale with heavier
  build (shoulders 1.4× hips).
- **Color:** vertex colors from the palette only (no textures for units).
  Each material slot uses three values: base, shade (−18 % value), light
  (+12 % value) baked by height/normal. Saturation is reserved for faction
  accents and team colors.
- **Light:** one warm key light (`#FFE9C7`), sky/ground ambient
  (`#A9C8E8` / `#6B5A43`), soft fog. A rim term (fresnel, shader-free
  approximation via vertex light baking on silhouettes) keeps figures
  separated from the ground.
- **Team identity (readability, never optional):** tabard/plume/banner
  slots use the team color — ally `#3FA9F5`, enemy `#F2545B` — plus a
  ground ring (ally solid, enemy notched) so color-blind players read it
  by shape too.

## 3. Palette (start values; P15 locks them)

| Group | Base | Shade/second | Accent | Notes |
|-------|------|--------------|--------|-------|
| Humans — *The Crown* | steel `#9AA3AD` | oak `#7A5636` | gold `#D9B44A` | knights, yeomen; linen `#E6D8B8` |
| Undead — *The Barrow* | bone `#D8D2C0` | rusted iron `#6E4B3A` | soul-fire `#7CF29A` | tattered violet cloth `#4B3B5C` |
| Machines — *The Ironworks* | iron `#4A5058` | brass `#B5893B` | forge glow `#FF8A3D` | leather `#5C3B26`; ogre skin `#8C9A6B` |
| Plants — *The Wildwood* | bark `#6B4F2E` | moss `#4E9F3D` | bloom `#F28AB2` | amber sap `#F2B544` |
| Environment | grass `#7BB661` | road `#C9A46A` | water `#3FA7B5` | stone `#B9B2A3`, timber `#8A6A44`, sky top `#7FB8E6` → horizon `#E8D9B5` |
| UI | parchment `#EFE3C6` | ink `#2B2118` | gold `#C9A13B` | iron frame `#3A3F45`, danger `#C0392B`, ok `#3E8E41` |

## 4. Rig and motion language

- **Rig v2 candidate (P15):** 17 named body joints — Root, Hips, Spine, Chest, Head,
  UpperArmL/R, ForearmL/R, HandL/R, ThighL/R, ShinL/R, FootL/R — plus up
  to 3 **secondary** joints driven by springs (Cape/Quiver, Plume/Hood,
  Tabard/Banner). Props attach to sockets: `WeaponR`, `OffhandL`, `Back`.
- **Body plans:** `Humanoid` (one-slot units) and `Large` (two-slot:
  wider stance, slower cycle, heavier footfalls).
- **Principles every clip follows:** anticipation (≥ 25 % of an attack
  before the strike), strong silhouette at the key pose, follow-through
  and overlap (secondary joints lag 60–120 ms), arcs not straight lines,
  ease-in/out cubic curves, feet planted while in contact (stride length
  × cycle rate = the unit's simulation speed — no foot sliding), a random
  phase offset per unit so crowds never move in sync.
- **Simulation timing contract:** an attack starts on `AttackStart`; the
  simulation applies damage `WINDUP = 6` ticks (300 ms) later on `Hit`.
  Melee clips land the strike at 0.30 s; ranged clips release at 0.18 s so
  the projectile arrives near 0.30 s; casts release at 0.22 s. Hit
  reactions start on `Hit`, never earlier.

### Clip sets per Archetype ("class")

| Archetype | Idle | Move | Attack | Signature extras |
|-----------|------|------|--------|------------------|
| Tank | shield-ready breathing | heavy march | shield bash / overhead swing (alternate) | block flinch on hit |
| DPS | restless weight shifts | fast run | two-hit combo (alternate A/B) | dash-in step |
| Ranged | bow lowered, scanning | walk with bow | draw → anchor → release → follow-through | reload/nock |
| Control | staff glow, slow sway | measured walk | raise staff → channel → thrust | lingering curse glow |
| Support | gentle hover of hands | calm walk | lift staff → bloom → point at ally | petals on release |

Shared by all: `Spawn`, `Hit` (directional), `Death` (2 variants),
`Evolve`, `Victory`, `Defeat`. `Large` units use heavier timings (×1.25)
and add ground-impact on footfalls.

## 5. Units (identity, silhouette, motifs)

Code ids never change; display names are **Proposed** until P15 sign-off.

| Code id | Display name | Faction | Archetype | Silhouette hook | Evolution motifs (A / B) |
|---------|--------------|---------|-----------|-----------------|--------------------------|
| `Shieldbearer` | Shieldbearer | Humans | Tank | tall kite shield + flanged mace | A Bulwark→Fortress: tower shield, plate, crest / B Vanguard→Champion: spear, pauldrons, back banner |
| `Archer` | Archer | Humans | Ranged | hooded longbowman, longbow taller than him | A Longbow→Marksman: longer bow, feathered cap / B Volley→Arrow Rain: twin quivers, fire-arrow glow |
| `BoneBrute` | Bone Brute | Undead | Tank (2-slot) | hunched giant skeleton, cleaver, ribcage armor | A Ossified→Bone Wall: bone plates, shield of skulls / B Butcher→Reaper: twin cleavers, scythe |
| `Hexer` | Hexer | Undead | Control | hooded necromancer, skull-topped staff | A Withering→Doom: tall crooked staff, crown / B Plague→Pestilence: censer, green mist |
| `Sawbot` | Sawknight | Machines | DPS | clockwork knight with a saw-blade arm | A Overclock→Shredder: twin saws, exhaust vents / B Plating→Juggernaut: riveted plates, tower helm |
| `SiegeWalker` | Ogre Archer | Machines | Ranged (2-slot) | ogre with a geared great-bow (arbalest) on his back/arm | A Longshot→Artillery: longer limbs, double-limb arbalest / B Shrapnel→Concussion: barbed bolts, hammer-head bolts |
| `Bloomkeeper` | Bloomkeeper | Plants | Support | druid with a flowering staff, leaf cloak | A Blossom→Grove: blooming crown, petal aura / B Bramble→Thornwarden: thorn armor, briar whip |
| `Rootcaller` | Rootcaller | Plants | Control | small treant shaman, root-claw hands | A Deep Roots→Strangle: longer claws, vine coils / B Spores→Overgrowth: mushroom cap, spore sacs |

Evolution progression for every unit (**Proposed**, owner 2026-10-28,
D-31): an Evolution must read at match distance without zooming. Every tier
changes at least three things: (1) scale — tier 1 +8 %, tier 2 +16 % over
the base form; (2) a large-surface color or material change (cloak, hood,
armor plates — not only a trim); (3) a new silhouette piece. Tier 2 also
carries the branch's signature artifact with an accent glow (gold or team
color). Branch A reads defensive/precise, branch B offensive/area. Check:
on the 1280×800 lineup at match-camera distance a tester names base,
tier 1, and tier 2 of every unit correctly.

## 6. Strongholds

Three **free cosmetic** styles, identical in gameplay (BR-U05, BR-C04):
**Stone Keep** (round stone tower, crenellations, banner), **Timber Fort**
(palisade ring + wooden watchtower), **Gatehouse** (twin towers and a
portcullis arch). Each has: team banners, a defender on top that animates
the Stronghold's attack (crossbow bolt), four damage states (100–75 %,
75–50 %, 50–25 %, < 25 %) with smoke and debris, a hit shake, and a
collapse sequence at defeat. Budget ≤ 4 000 triangles per style.

## 7. UI kit

- **Theme:** parchment panels in iron/wood frames, banner headers, gold
  accents; ink text. Icons are flat two-tone SVGs rendered to one PNG
  atlas at build time.
- **Fonts (OFL):** a display face for titles only (e.g. Cinzel) and a
  highly legible body face (e.g. Alegreya Sans); Latin subset; ≤ 120 KB
  total. Body text never uses the display face.
- **Card (one component everywhere):** 3:4; cost gem top-left (Energy
  icon + number), faction crest top-right, unit portrait (runtime render
  of the base form, cached), name banner, archetype icon + slot pips at the
  bottom. States: affordable, unaffordable (desaturated, cost in red),
  pending (pulsing outline), disabled, selected. Press feedback: 0.95 scale
  + click sound + haptic (`navigator.vibrate(10)` where available).
- **Accessibility:** body text ≥ 16 px equivalent on a 390 px-wide phone;
  contrast ≥ 4.5:1; touch targets ≥ 48 px; team read by shape and color;
  settings for reduced motion, sound, and quality tier.

## 8. Character refinement procedure (P22–P29 follow it exactly)

1. **Brief:** copy the unit's §5 row into the phase report; list the five
   forms and what each adds.
2. **Silhouette check:** `?gallery=<unit>&silhouette` renders black
   silhouettes of all 8 units' base forms side by side; the new unit must
   be identifiable at 64 px height. Screenshot in the report.
3. **Model:** base form on rig v2 using only palette slots; then the four
   evolution forms per §5. Triangles per form are logged from the gallery
   and reported; there is no fixed maximum (D-30) — the FPS check in step 7
   decides.
4. **Animation:** the Archetype clip set (§4) tuned to this body, plus the
   signature extras; timing table (clip, duration, key frames, strike/
   release time) in the report; strike/release times obey §4's contract.
5. **Effects and sound hooks:** projectile/impact/signature VFX from the
   P21 kit; sound cue names.
6. **Consistency:** a lineup screenshot with the reference character
   (Archer) and every finished unit: same scale logic, palette rules,
   detail density, outline/rim, motion energy.
7. **Performance:** 30 copies of this unit fighting in `?stress=30` on the
   Low tier on a mid-range phone ≥ 30 FPS; draw calls recorded for diagnostics (no fixed maximum; D-28).
8. **Validation:** desktop 1920×1080 and phone 390×844 screenshots — turn-
   around, five forms, a contact sheet of every clip (frames at 0, 25, 50,
   75 %), and an in-match shot. Saved ≤ 300 KB each to
   `docs/media/units/<unit>/`. The owner approves in the report.

## 9. Budgets (all visual phases)

| Item | Budget |
|------|--------|
| Triangles: unit forms, Stronghold, arena | Measured and reported; no fixed maximum (D-30) |
| Draw calls, busy match (30 units) | Measured; no fixed maximum (D-28) |
| WASM (brotli) / other first-load assets | ≤ 6 MB hard ceiling / ≤ 1.5 MB |
| Frame rate | 60 FPS desktop; ≥ 30 FPS mid-range phone on Low with 30 units |
| Time to menu | ≤ 5 s desktop; ≤ 10 s mid-range phone on 4G |

## 10. P15 candidate implementation notes (not owner-approved)

Palette values, proportions marked *start*, and display names remain
**Proposed**: owner sign-off is pending. No value is locked by the executor.

- The exact enum supplied by P15 contains 17 body names despite its "18"
  label. The candidate implements those names; the reviewer/owner must
  resolve that documentation discrepancy before treating the rig as locked.
- Humanoid anatomy is approximately 1.05 units tall with a 0.273 head
  diameter (26%); the Archer retains the client's existing 1.2× lane
  prominence. Large blockouts use 1.5× scale; old adapted bodies retain
  their previous proportions until refinement.
- Human skin retains the existing `#E8B98A` palette constant as a
  **Proposed** neutral skin slot. No simulation value changes.
- Secondary defaults are stiffness 60, damping 8, fixed 120 Hz sub-steps,
  ±35° per-axis rotation clamps. Only used secondary attachments are driven.
- Archer base has 1,116 triangles including the shared strings and arrow.
  Idle lasts 1.60 s; Walk cadence follows content speed and stride;
  AttackRanged lasts 0.50 s: draw 0–0.12, anchor 0.12–0.18,
  Release 0.18, follow-through 0.18–0.30, recovery 0.30–0.50.
- The other seven units are v2 gray blockouts for silhouette review; their
  existing in-match forms, and the four Archer evolutions, remain adapted.
- Gallery evidence and the unanswered owner decisions are in the
  [P15 phase report](phases/P15-art-direction-and-rig.md#phase-report).

## 11. Visual standard: Unit proportions, faction morphology, and evolution (A-1)

This section formalizes the visual contract (D5) governing unit models on the rig-v2 anatomy, ensuring high readability at the match camera across all archetypes, factions, and tiers.

### 11.1 Class proportion table (Rig-v2 Humanoid ≈ 1.05 tall, head 0.26 of height; Large plan ×1.5)

| Archetype (Class) | Shoulder : Hip Ratio | Limb Thickness Factor | Weapon / Prop Scale | Stance & Posture Characteristics | Readability Priority at Match Camera |
|---|---|---|---|---|---|
| **Tank** | 1.40 | ×1.25 | Shield or 2H weapon ≥ 0.70 of body height | Wide stance, grounded feet, vertical heavy posture | Bulky silhouette, immovable mass, prominent defensive shield |
| **DPS** | 1.15 | ×1.00 | Asymmetric oversized weapon ≥ 0.65 of body height | Forward lean, aggressive crouch, dynamic tension | Silhouette asymmetry, sharp weapon arc, kinetic energy |
| **Ranged** | 0.95 | ×0.85 | Weapon (bow/arbalest) ≥ 1.05 of body height (taller than body) | Slim upright stance, off-hand free/nocked, drawn tension | Tall vertical weapon profile, clear off-hand silhouette |
| **Support** | 1.00 | ×0.90 | Staff or relic with glowing top orb/bloom (≥ 0.60 height) | Rounded contours, upright calm posture, open hands | Soft organic curves, luminous head/staff accent, light tones |
| **Control** | 0.90 | ×0.85 | Tall crooked staff / symbol with lingering glow accent | Tall head silhouette (hood, crown, horns), slender body | High vertical reach, imposing headgear, occult focal point |

*Large body plan:* Units with two slots (`BoneBrute`, `SiegeWalker`) scale uniformly by ×1.50 over humanoid baseline (height ≈ 1.575), shoulder:hip ratio 1.40 minimum, with wide heavy gait and ground impact shockwaves.

### 11.2 Faction morphology and material palette rules

Colors strictly follow the palette defined in §3. Mechanical and organic constraints are non-negotiable:

| Faction | Primary Material (Base) | Secondary Material (Shade) | Accent Material (Glow / Energy) | Structural & Joint Morphology Rules |
|---|---|---|---|---|
| **Humans** (*The Crown*) | Steel `#9AA3AD` | Oak wood `#7A5636` | Gold `#D9B44A` (plus linen `#E6D8B8`) | Classic medieval armor, plate and mail, articulated elbow/knee cops, heroic humanoid arcs. |
| **Undead** (*The Barrow*) | Bone `#D8D2C0` | Rusted iron `#6E4B3A` | Soul-fire `#7CF29A` (plus tattered violet `#4B3B5C`) | Exposed ribcages, jagged bone protrusions, asymmetrical rotting joints, jerky hunched postures. |
| **Machines** (*The Ironworks*) | Iron `#4A5058` | Brass `#B5893B` | Forge glow `#FF8A3D` (plus leather `#5C3B26`) | **Rigid mechanical pivots and pistons only.** No organic flexing or human limb arcs; boxy plate assemblies, visible rivets and exhausts. |
| **Plants** (*The Wildwood*) | Bark `#6B4F2E` | Moss green `#4E9F3D` | Bloom pink `#F28AB2` (plus amber sap `#F2B544`) | Curving organic vine boughs, root tendrils, mushroom caps, natural foliage volumes; zero sharp industrial corners. |

### 11.3 Evolution visual progression rules

To ensure tier progression reads instantly without zooming (D-31), every evolution form must satisfy four mandatory criteria:
1. **Geometric Scale Growth:**
   - Tier 0 (Base): 1.00× scale.
   - Tier 1 (Branches A1, B1): exactly **+8 % scale** (1.08×).
   - Tier 2 (Branches A2, B2): exactly **+16 % scale** (1.16×).
2. **Surface Material / Color Shift:**
   - At least one major structural surface (cloak, tabard, pauldrons, carapace, or plating) must undergo a prominent material/color change (not merely an edge highlight or trim line).
3. **Silhouette Piece Addition:**
   - Each tier adds at least one distinct new silhouette-breaking attachment: crest, horn, banner, cape, pauldrons, spore sac, or auxiliary weapon piece.
4. **Branch Identity & Signature Artifact:**
   - **Branch A (Defensive / Precision):** Reinforced armor pieces, tower shields, extended weapon lengths, balanced symmetric silhouettes.
   - **Branch B (Offensive / Area / Havoc):** Aggressive asymmetry, dual weapons, heavy warheads, spiked appendages, or auxiliary quivers/censers.
   - **Tier 2 Signature Artifact:** Must carry the branch's capstone artifact displaying a subtle emissive glow (gold `#D9B44A` or faction/team accent).

### 11.4 Detail density and rendering rules

- **Structural Readability:** Large, clean geometric shapes only. No micro-detail readable below ≈ 6 px from match camera distance.
- **Shading Treatment:** Smooth-shaded large organic/curved surfaces; crisp flat-shaded small mechanical details, blades, and bevels.
- **Single Shared Material:** All figures share `FigureLibrary.material` across the workspace. Zero per-unit custom shaders or dynamic pipeline allocations.
- **HUD & Status Integrity:** Ring bases, rank pips, and vitality bars use standard scale formulas and remain visually anchored regardless of evolution tier.

### 11.5 "Tupi Look" validation checklist

Before signing off on any troop, strongpoint, or arena asset, verify the 6-point Tupi aesthetic checklist:
- [ ] **Palette Fidelity:** Vertex colors drawn exclusively from §3 palette; no arbitrary uncalibrated RGB hexes.
- [ ] **Proportions & Scale:** Matches class proportion table (§11.1); head is 26% of body height; weapon ≥ 60% of body; evolution scaling strictly respects 1.00 / 1.08 / 1.16.
- [ ] **Warm Light Harmony:** Renders coherently under key light `#FFE9C7`, dual ambient sky/ground (`#A9C8E8` / `#6B5A43`), and soft arena fog.
- [ ] **Joint Pivots & Morphology:** Biological arcs for Humans/Undead/Plants; strict mechanical hinges/pivots with zero organic flexion for Machines.
- [ ] **Ground Plane / Floor Height:** Feet contact floor firmly at $Y = 0.0$; no clipping into the arena turf and no floating during walk/attack clips.
- [ ] **Rim Separation:** Silhouette edges separate distinctly against both green grass (`#7BB661`) and sandy road (`#C9A46A`) backgrounds under match camera zoom.

