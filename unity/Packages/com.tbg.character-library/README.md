# TBG Character Library (UPM)

Local package at `Packages/com.tbg.character-library/`.

## Assemblies

| Assembly | Role |
|----------|------|
| `TBG.CharacterLibrary.Runtime` | `CharacterCatalog`, loaders, behavior player |
| `TBG.CharacterLibrary.Editor` | Import validation scaffold (Phase 02+) |
| `TBG.CharacterLibrary.Tests` | EditMode contract tests |

## Dependencies

- `TBG.CharacterSystem.Runtime` — `ProceduralCharacterBuilder` (procedural mode)
- `com.atteneder.gltfast` (OpenUPM) — mesh3d glTF import via `Mesh3DCharacterFactory`

## CharacterAction (Phase 03)

| Action | Loop | Typical use |
|--------|------|-------------|
| `Idle` | Yes | Default ambient state |
| `Walk` | Yes | Lane march (`EnterWalk` / `ExitWalk`) |
| `Attack` | No | Primary attack; index 0 in `PlayAttack(int)` |
| `Ultimate` | No | One-shot hero ability |
| `Hit` | No | Damage react (Phase 04+) |
| `Cast` | No | Support / control archetypes |
| `Spawn` | No | Unit enter battlefield |
| `Death` | No | Unit removed |

## API

```csharp
var catalog = new CharacterCatalog();
var instance = catalog.Spawn("demo_sentinel_01", parentTransform);

// Basic actions
instance.PlayAction(CharacterAction.Attack);
instance.PlayAction(CharacterAction.Walk);
instance.PlayAction(CharacterAction.Ultimate);

// BehaviorPlayer v2 (lane + combat)
var behavior = instance.Behavior as BehaviorPlayer;
behavior.EnterWalk();           // loop Walk until ExitWalk
behavior.ExitWalk();            // return to Idle
behavior.PlayAttack(0);         // explicit attack index
behavior.PlayAttackRepeat();    // respects combat.attackCycle (repeat | rotate)
behavior.PlayUltimate();        // one-shot Ultimate
behavior.OnClipComplete += action => { /* gameplay sync — Phase 04 */ };

catalog.Release(instance);
```

Definitions load from `content/characters/definitions/{id}.json` at repo root.

### Dual-tier definition fields (Phase 03)

| Field | Purpose |
|-------|---------|
| `mesh` | **Game** runtime GLB — skinned, clipped, mobile budget |
| `meshShowcase` | High-poly preview GLB — web carousel / studio only |
| `animationSet` | Id of manifest in `content/characters/animation_sets/` |
| `gameLod` | Optional `{ triTarget, textureSize }` overrides for export |

Unity always loads `mesh` (game tier). Web studio loads `meshShowcase` when present.

## Animation sets

Manifests map gameplay action keys to clip names shared per body class.

Example (`humanoid_heavy_combat_v1.json`):

```json
{
  "id": "humanoid_heavy_combat_v1",
  "bodyClass": "HumanoidHeavy",
  "clipSource": "library",
  "clips": {
    "idle": "Idle",
    "walk": "Walk",
    "attack": "Attack",
    "attack2": "Attack2",
    "ultimate": "Ultimate",
    "death": "Death",
    "spawn": "Spawn"
  }
}
```

| Component | Role |
|-----------|------|
| `AnimationSetLoader` | Load manifest; validate required fields; register body class |
| `BodyClassClipRegistry` | Track one animation set id per body class; resolve shared clip directory |
| Game GLB | Embeds retargeted clips for glTFast import (MVP) |

Library clip files: `content/characters/animations/clips/{bodyClass}/{ClipName}.glb`

Schema example: `content/characters/animation_sets/_schema.example.json`

## Material profiles

`MaterialProfileLoader` reads `content/characters/materials/{id}.json` and builds `TBG/CharacterPBR` materials.

Shader: `Shaders/CharacterPBR.shader` — albedo (`_MainTex`), normal (`_BumpMap`), ORM packed (`_ORMMap`, R=AO G=roughness B=metallic).

## Behavior profiles

`BehaviorProfileLoader` reads `content/characters/behaviors/{id}.json`.

Phase 03 extensions:

| Field | Purpose |
|-------|---------|
| `actions.walk`, `actions.ultimate`, `actions.attack2` | Animator state/trigger names |
| `triggers` | Per-action trigger overrides |
| `timings.ultimateWindupMs`, `timings.ultimateDurationMs` | Ultimate presentation sync |
| `combat.attackCycle` | `"repeat"` (same attack) or `"rotate"` (alternate Attack / Attack2) |
| `locomotion.useRootMotionWalk` | Optional `WalkRoot` clip sync (Phase 04 lane) |
| `locomotion.footIk` | Optional foot plant IK during Walk (default `false`) |

`BehaviorPlayer` maps `CharacterAction` → animator triggers/states from the profile (defaults when `behaviorProfile` is omitted).

Shared animator controllers: `content/characters/animations/{BodyClass}.controller`  
Regenerate v2 (Walk bool layer + action triggers): `python3 tools/unity/generate_animator_controllers_v2.py`

## Mesh3D factory

`Mesh3DCharacterFactory.Create(definition, parent)`:

1. Loads **game** `mesh` path (not showcase)
2. Resolves `animationSet` via `AnimationSetLoader`
3. Imports `.glb` via glTFast (embedded clips)
4. Applies `MaterialProfile` + unit textures
5. Attaches `CharacterLodGroup` (LOD0 game mesh, LOD1 from `meshes/game/lod1/`)
6. Attaches `Animator` + `BehaviorPlayer`
7. Scales from definition proportions (`heightScale`, `bodyWidthScale`)

## Runtime LOD

`CharacterLodGroup.Attach(root, unitId)` — LOD0 uses game mesh; LOD1 loads static simplified mesh from `content/characters/meshes/game/lod1/{id}.glb` when present.

## Tests

Open the Unity project, then **Window → General → Test Runner → EditMode**:

| Suite | Coverage |
|-------|----------|
| `CharacterCatalogEditModeTests` | Procedural + mesh3d spawn |
| `CharacterSchemaValidationTests` | JSON contract validation |
| `MaterialProfileLoaderTests` | Material profile JSON + `TBG/CharacterPBR` |
| `BehaviorProfileLoaderTests` | Behavior profiles + `BehaviorPlayer` action mapping |
| `Mesh3DCharacterFactoryTests` | glTF import, scale, animator attachment |
| `PilotRosterEditModeTests` | Mesh3d pilot spawn; sentinel game mesh Walk/Ultimate + `PlayAttackRepeat` |
| `AnimationSetLoaderTests` | Animation set manifest clip resolution |
| `BodyClassClipRegistryTests` | Shared clip directory per body class |

Or batchmode:

```bash
unity -batchmode -projectPath unity -runTests -testPlatform EditMode \
  -assemblyNames TBG.CharacterLibrary.Tests -logFile -
```

Test fixtures: `content/characters/meshes/_test_fixture.glb`, `test_mesh3d_spawn.json`, `test_procedural_spawn.json`.

## Related

- Mesh export pipeline: [tools/mesh-pipeline/README.md](../../../tools/mesh-pipeline/README.md)
- ADR: [docs/adr/ADR-004-dual-tier-mesh-animation.md](../../../docs/adr/ADR-004-dual-tier-mesh-animation.md)
- Mobile budgets: [tools/mesh-pipeline/BUDGETS.md](../../../tools/mesh-pipeline/BUDGETS.md)
