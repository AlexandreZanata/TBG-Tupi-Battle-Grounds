# Procedural Character Generator

> Technical specification: geometry and color defined **100% in code**.  
> **Related:** [character-creation-guide.md](character-creation-guide.md), [brand-system.md](brand-system.md), [GLOSSARY.md](../GLOSSARY.md)

## Purpose

Remove dependency on manual 3D modeling and traditional illustration for unit creation. Every unit (body, proportions, color, accessories) is defined by **parameters**; geometry is **generated at build or runtime** inside Unity.

2D art is optional and final-stage only: render the procedural 3D model from a fixed camera angle for card images — no illustrator required.

---

## 1. Core principle

```
Unit = f(Parameters)
```

Instead of modeling each unit manually, define a **visual grammar**: fixed shapes, proportion rules, and color palette. Each new unit is a combination of values within that grammar — not a new `.fbx` file.

Creating a unit becomes **editing a configuration file** (JSON or ScriptableObject), not opening Blender or Photoshop.

---

## 2. Visual grammar

Decide the visual alphabet before coding. Sections marked `[TBD]` must be locked in Phase 01 (see `.local/phases/01-procedural-character-studio/`).

### 2.1 Shape vocabulary (primitives)

| Part | Primitive (locked Phase 01) |
|------|----------------------------|
| Head | **Cube** |
| Torso | **Cube** (width/height scaled) |
| Limbs | **Capsule** |
| Feet / base | **Cube** (flattened) |
| Accessories | **Shield**, **Weapon**, **Helmet**, **ArmorBlock** (procedural cubes) |

**Recommendation:** use the **smallest primitive set** (cube, cylinder, sphere, capsule). Intentional constraint drives consistency (Minecraft model).

### 2.2 Proportion rules by archetype

Proportion communicates role — expressed as **numbers**, not hand-drawn silhouettes:

| Archetype | Height | Torso width | Head scale | Limb length |
|-----------|--------|-------------|------------|-------------|
| Tank | 1.2× | 1.8× | 0.8× | 0.7× (short, stocky) |
| DPS | 1.0× | 0.8× | 0.9× | 1.1× (agile) |
| Support | 0.9× | 0.7× | 1.2× (large head = friendly) | 0.8× |
| Ranged | 1.1× | 0.6× | 0.8× | 1.3× (long limbs) |
| Control | 1.0× | 0.7× | 1.1× | 0.9× |

Example values — tune via studio playtests.

### 2.3 Faction color palette

Programmable HEX/RGB values — align with [brand-system.md](brand-system.md):

| Faction | Primary | Secondary | Procedural material |
|---------|---------|-----------|---------------------|
| Humans | `#8B7355` | `#C4A882` | smooth / fabric noise |
| Undead | `#4A5D23` | `#7D8B55` | decay noise + edge fade |
| Machines | `#4A6FA5` | `#B0C4DE` | metallic Fresnel + circuit pattern |
| Plants | `#2D5A27` | `#6B8E23` | organic green noise + pulse emissive |

### 2.4 Evolution visual modifiers

Mechanical evolution must have visual expression via parameters — no new art per stage:

| Stage | Visual change |
|-------|----------------|
| Evolution 0 (base) | Default proportions and palette |
| Evolution 1 | Scale ×1.1; add **ArmorBlock** accessory |
| Evolution 2 | Scale ×1.2; saturate colors ×1.25; emissive glow on faction shader |

---

## 3. Technical architecture (Unity / C#)

### 3.1 Character definition (data)

Each unit is a configuration file, not a modeled scene:

```csharp
[CreateAssetMenu(fileName = "NewCharacter", menuName = "TBG/CharacterDefinition")]
public class CharacterDefinition : ScriptableObject
{
    public string characterName;
    public Archetype archetype;
    public Faction faction;

    [Header("Proportions")]
    public float heightScale;
    public float bodyWidthScale;
    public float headScale;
    public float limbLengthScale;

    [Header("Color")]
    public Color primaryColor;
    public Color secondaryColor;

    [Header("Procedural accessories")]
    public AccessoryType[] accessories;

    [Header("Evolution")]
    public EvolutionStage currentStage;
}
```

**On-disk source of truth (versioned):** `content/characters/definitions/{id}.json`  
Studio imports JSON → ScriptableObject; exports back on save.

### 3.2 Procedural mesh builder

Single central script assembles geometry from primitives:

```csharp
public class ProceduralCharacterBuilder : MonoBehaviour
{
    public GameObject BuildCharacter(CharacterDefinition def)
    {
        GameObject root = new GameObject(def.characterName);

        var head = CreatePrimitive(PrimitiveType.Cube, def.headScale);
        var torso = CreatePrimitive(PrimitiveType.Cube, def.bodyWidthScale);
        var limbs = BuildLimbs(def.limbLengthScale);

        ApplyMaterial(root, def.primaryColor, def.secondaryColor, def.faction);
        AttachAccessories(root, def.accessories);
        PositionParts(root, head, torso, limbs, def);

        return root;
    }
}
```

**No per-unit `.fbx` files.** Assembly uses Unity primitives (`Cube`, `Capsule`, `Cylinder`, `Sphere`) or custom `Mesh` from vertices/triangles in code.

### 3.3 Shared rig and animation

All units share block structure (head, torso, two arms, two legs):

- One **generic rig** (light humanoid skeleton) shared by all units.
- Animations (idle, attack, death) authored **once** on the generic rig.
- New units inherit rig + animations automatically.

Animation is **per movement archetype**, not per unit (e.g. all Tanks share heavy attack; all DPS share light attack).

### 3.4 Procedural faction shaders

Faction appearance via Shader Graph parameters — not hand-painted textures:

| Faction | Shader behavior |
|---------|-----------------|
| Humans | Smooth surface, subtle tone noise |
| Undead | Stain noise + slight edge transparency |
| Machines | Metallic Fresnel + procedural circuit lines |
| Plants | Organic green noise + animated emissive pulse |

---

## 4. End-to-end pipeline

```
1. Define archetype + faction (balance spreadsheet)
        ↓
2. Create CharacterDefinition (JSON or ScriptableObject in Character Studio)
        ↓
3. ProceduralCharacterBuilder assembles geometry
        ↓
4. Unit inherits generic rig + shared animations
        ↓
5. Faction shader applies appearance
        ↓
6. (Optional) CardRenderer → fixed-angle RenderTexture → card sprite
        ↓
7. Unit ready for match integration
```

### 4.1 Automatic card generation (2D from 3D)

Fixed camera → `RenderTexture` → standard frame template → export sprite. Editor button generates card for any unit instantly.

Output path: `content/characters/cards/{id}.png`

---

## 5. Benefits (solo dev)

| Previous problem | Procedural solution |
|------------------|---------------------|
| Manual 3D modeling is slow/expensive | Geometry is parameterized — no manual modeling |
| Per-unit animation cost | Shared rig and animation sets |
| Card art needs illustrator | Auto-render from 3D model |
| Visual inconsistency | Same grammar for all units |
| Slow unit addition | New unit = new definition file |

---

## 6. Limitations and risks

| Risk | Mitigation |
|------|------------|
| Visual expressiveness ceiling | Embrace constraint as identity (voxel/low-poly); avoid dragon-vs-human extremes in one grammar |
| Initial engineering cost | Build studio in Phase 01; payback on unit 4+ |
| Procedural shader limits | Sufficient for stylized look; not for photorealism |

---

## 7. Repository layout

Versioned paths (committed to git):

```text
content/characters/
  definitions/          # JSON source of truth — studio CRUD
    tank_human_01.json
    dps_machine_01.json
  cards/                # Rendered card PNGs (generated)
  previews/             # Optional viewport screenshots

unity/Assets/TBG/CharacterSystem/
  Scripts/
    CharacterDefinition.cs
    ProceduralCharacterBuilder.cs
    AccessoryAttacher.cs
    CardRenderer.cs
    Editor/
      CharacterStudioWindow.cs    # Create / edit / delete definitions on disk
      CharacterDefinitionImporter.cs
  ScriptableObjects/Characters/   # Imported .asset from JSON (generated, may gitignore or commit)
  Shaders/
    Faction_Humans.shadergraph
    Faction_Undead.shadergraph
    Faction_Machines.shadergraph
    Faction_Plants.shadergraph
  Rigs/
    GenericRig_Light.fbx          # Single shared skeleton
  Animations/
    Generic_Idle.anim
    Generic_Attack_Light.anim
    Generic_Attack_Heavy.anim
    Generic_Death.anim
```

**Studio rule:** Character Studio **must** read/write `content/characters/definitions/*.json`. Deleting a unit removes its JSON and associated generated assets under `cards/` and `previews/`.

---

## 8. Character Studio (editor tool)

The **Character Studio** is a Unity Editor window that:

| Action | Behavior |
|--------|----------|
| **Create** | New JSON in `content/characters/definitions/`, import to ScriptableObject, build preview |
| **Edit** | Load JSON, modify parameters, rebuild mesh, re-export JSON |
| **Delete** | Remove JSON + card + preview files; unregister from catalog |
| **Test** | Spawn unit in test scene; cycle archetype/faction shaders; silhouette check |
| **Export card** | Run `CardRenderer` → save PNG to `content/characters/cards/` |

Phase 01 procedural pipeline remains for blockout. Production path: [real-3d-character-pipeline.md](real-3d-character-pipeline.md).

---

## 9. Next steps

- [ ] Lock visual grammar (section 2) in studio prototype
- [ ] Scaffold Unity project + `content/characters/` layout
- [ ] Implement `ProceduralCharacterBuilder` for Tank archetype first
- [ ] Build faction Shader Graphs (4 factions)
- [ ] Implement `CharacterStudioWindow` with disk CRUD
- [ ] Validate 2–3 units same archetype, different parameters — silhouette readability
- [ ] Implement `CardRenderer`
- [ ] Scale to 12 MVP units after pipeline validation
