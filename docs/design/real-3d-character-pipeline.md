# Real 3D Character Pipeline

> Production path for authored meshes, PBR textures, animation, and behaviors.  
> **Related:** [ADR-003](../adr/ADR-003-real-3d-character-pipeline.md), [procedural-character-generator.md](procedural-character-generator.md), [character-creation-guide.md](character-creation-guide.md)

Phase 01 (procedural primitives) stays for fast iteration. This document defines **Phase 02+** for real 3D units and the **TBG.CharacterLibrary** C# package.

---

## 1. Goals

| Goal | Success metric |
|------|----------------|
| Real 3D units per faction | 1 pilot unit per faction in `.glb` + PBR textures |
| Ownable toolchain | Blender/Krita/Material Maker configs in repo |
| C# library | `TBG.CharacterLibrary` — load, materialize, animate, behave |
| Same JSON contract | Extended with `assetMode` and asset paths |
| Mobile-safe | ≤8k tris MVP target per unit; 512–1024 texture sets |

---

## 2. Open-source toolchain (modifiable)

### 2.1 Blender — model, rig, animate

| Use | Details |
|-----|---------|
| Modeling | Low-poly stylized meshes per [brand-system.md](brand-system.md) |
| Rigging | **Shared rig** per body class: `HumanoidLight`, `HumanoidHeavy`, `Mechanical`, `Organic` |
| Animation | Idle, spawn, attack, death — exported in glTF |
| Customization | **TBG Blender add-on** in `tools/blender/tbg_export/` (Python, GPL — authoring only) |

Export preset: glTF 2.0 binary (`.glb`), Y-up, apply transforms, include animations.

### 2.2 Krita — texture painting

| Map | File |
|-----|------|
| Albedo (base color) | `{id}_albedo.png` |
| Optional hand details | Painted on UV0 |

Templates: `tools/krita/templates/` (PSD/KRA with UV overlay guides).

### 2.3 Material Maker — procedural PBR maps

Fork-friendly node graphs for faction surfaces:

| Graph | Output |
|-------|--------|
| `humans_fabric.mm` | albedo + roughness variation |
| `undead_decay.mm` | albedo + emissive masks |
| `machines_metal.mm` | metallic + normal detail |
| `plants_organic.mm` | albedo + subsurface tint |

Exported maps merged with Krita albedo where needed.

### 2.4 Optional reference OSS (not runtime dependencies)

| Project | Use as reference |
|---------|------------------|
| [Godot](https://github.com/godotengine/godot) | Scene/resource composition patterns |
| [glTF Sample Assets](https://github.com/KhronosGroup/glTF-Sample-Assets) | Validation models |
| [Mixamo](https://www.mixamo.com/) | **Dev-only** retarget testing — replace with own anims for ship |

---

## 3. Repository layout (versioned)

```text
content/characters/
  definitions/           # JSON — source of truth (extended schema)
  meshes/                # .glb per unit or shared mesh + material variants
  textures/              # PBR sets per unit or per faction
  materials/             # Material profile JSON (shader params)
  behaviors/             # Behavior profile JSON (combat presentation)
  animations/            # Shared .glb clips or Unity .controller
  cards/                 # Rendered cards (unchanged)

tools/
  blender/tbg_export/    # Custom Blender add-on
  blender/RIG-SPEC.md    # Rig naming + body classes
  material-maker/        # .mm project files + export scripts
  krita/templates/

unity/Packages/
  com.tbg.character-library/   # C# library (UPM local package)
    Runtime/
    Editor/
    Tests/
```

Git LFS: enable for `*.glb`, `*.png` > 1MB when repo grows.

---

## 4. TBG.CharacterLibrary (C#)

### 4.1 Package boundaries

| Assembly | Responsibility |
|----------|----------------|
| `TBG.CharacterLibrary.Runtime` | Load definitions, spawn units, apply materials, play behaviors |
| `TBG.CharacterLibrary.Editor` | Import validation, asset post-processors, preview |
| `TBG.CharacterLibrary.Tests` | Contract tests (JSON schema, spawn smoke) |

Lives at: `unity/Packages/com.tbg.character-library/`

### 4.2 Core types

```csharp
public enum CharacterAssetMode { Procedural, Mesh3D }

public sealed class CharacterCatalog
{
    public CharacterInstance Spawn(string id, Transform parent);
    public void Release(CharacterInstance instance);
}

public sealed class CharacterInstance
{
    public string Id { get; }
    public GameObject Root { get; }
    public ICharacterBehavior Behavior { get; }
    public void PlayAction(CharacterAction action);
}

public interface ICharacterBehavior
{
    void EnterIdle();
    void PlaySpawn();
    void PlayAttack();
    void PlayDeath();
}
```

### 4.3 Loading strategy

```
definitions/{id}.json
    → CharacterDefinitionLoader (existing + extended fields)
    → if assetMode == Procedural → ProceduralCharacterBuilder (Phase 01)
    → if assetMode == Mesh3D     → Mesh3DCharacterFactory
          → load .glb via glTFast
          → apply MaterialProfile (URP Lit or custom TBG shader)
          → attach Animator + BehaviorProfile
```

### 4.4 Material profiles

JSON-driven shader parameters — faction-wide reuse:

```json
{
  "id": "humans_pbr_v1",
  "shader": "TBG/CharacterPBR",
  "properties": {
    "_Metallic": 0.1,
    "_Smoothness": 0.45
  },
  "textureSlots": ["albedo", "normal", "orm"]
}
```

### 4.5 Behavior profiles

Separate **presentation** from **server combat logic** (Nakama owns truth).

```json
{
  "id": "tank_melee_v1",
  "animatorController": "TankHumanoid",
  "actions": {
    "idle": "Idle",
    "spawn": "Spawn",
    "attack": "AttackHeavy",
    "death": "Death"
  },
  "timings": {
    "attackWindupMs": 400,
    "attackHitMs": 520
  }
}
```

`BehaviorPlayer` maps `CharacterAction` → Animator triggers. Gameplay damage timing syncs to server tick later (Phase 03).

---

## 5. End-to-end workflow

```
1. Design sheet (archetype, faction, mechanics) — character-creation-guide.md
        ↓
2. Blender: model on shared rig → UV unwrap → export .glb (tools/blender)
        ↓
3. Krita + Material Maker: PBR textures → content/characters/textures/
        ↓
4. Author material + behavior JSON profiles
        ↓
5. Extend definitions/{id}.json (assetMode: mesh3d, paths)
        ↓
6. Unity: CharacterCatalog.Spawn(id) — library validates and loads
        ↓
7. Character Studio: preview mesh3d unit, export card (render 3D)
        ↓
8. Playtest scene: behaviors + silhouette check at mobile FOV
```

---

## 6. Technical standards (mobile)

| Constraint | MVP target |
|------------|------------|
| Triangles per unit | ≤ 8,000 |
| Texture size | 1024² (hero), 512² (minions) |
| Bones | ≤ 64 per skinned mesh |
| Materials per unit | ≤ 2 |
| Animation clips | 4 minimum (idle, spawn, attack, death) |
| Format | glTF 2.0 `.glb` |

---

## 7. Phase breakdown (implementation)

| Sub-phase | Deliverable |
|-----------|-------------|
| **02.1** | ADR-003 + this doc + glossary terms |
| **02.2** | Extended JSON schema + validation (Python + C#) |
| **02.3** | `com.tbg.character-library` package scaffold + tests |
| **02.4** | Blender add-on v0 (export preset + rig validation) |
| **02.5** | Material Maker graphs + texture folder convention |
| **02.6** | `Mesh3DCharacterFactory` + glTFast import |
| **02.7** | `MaterialProfile` + URP/TBG shader |
| **02.8** | `BehaviorProfile` + Animator integration |
| **02.9** | 1 pilot `.glb` unit per faction + studio preview |
| **02.10** | Card export from real 3D render |

Agent checklist: `.local/phases/02-real-3d-character-library/TASKS.md`

---

## 8. Coexistence with procedural (Phase 01)

| `assetMode` | When to use |
|-------------|-------------|
| `procedural` | Blockout, balance tests, placeholder cards |
| `mesh3d` | Production units, marketing assets, final cards |

Studio UI: filter/list by `assetMode`. Same CRUD paths under `content/characters/definitions/`.

---

## 9. Risks

| Risk | Mitigation |
|------|------------|
| Art scope explosion | Shared rig + material profiles; 4 pilots before scaling to 12 |
| Large git repo | LFS + texture atlases per faction later |
| Blender GPL confusion | Add-on is authoring-only; game ships glTF + textures only |
| Animation rework | Retarget to shared rig; one attack set per archetype class |

---

## 10. Next action

Start **Phase 02.2**: extend `_schema.example.json` and validation for `assetMode`, mesh, textures, profiles — then scaffold `com.tbg.character-library`.
