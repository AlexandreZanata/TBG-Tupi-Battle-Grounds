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

## API

```csharp
var catalog = new CharacterCatalog();
var instance = catalog.Spawn("tank_human_01", parentTransform);
instance.PlayAction(CharacterAction.Attack);
catalog.Release(instance);
```

Definitions load from `content/characters/definitions/{id}.json` at repo root.

## Material profiles

`MaterialProfileLoader` reads `content/characters/materials/{id}.json` and builds `TBG/CharacterPBR` materials.

Shader: `Shaders/CharacterPBR.shader` — albedo (`_MainTex`), normal (`_BumpMap`), ORM packed (`_ORMMap`, R=AO G=roughness B=metallic).

## Behavior profiles

`BehaviorProfileLoader` reads `content/characters/behaviors/{id}.json`. `BehaviorPlayer` maps `CharacterAction` → animator triggers/states from the profile (defaults when `behaviorProfile` is omitted on a definition).

Shared animator controllers: `content/characters/animations/{BodyClass}.controller` (regenerate via `tools/unity/generate_animator_controllers.py`).

## Mesh3D factory

`Mesh3DCharacterFactory.Load(glbPath, definition, parent)` imports `.glb` via glTFast, applies `MaterialProfile` + unit textures, attaches `Animator`, and scales from definition proportions (`heightScale`, `bodyWidthScale`).

## Tests

Open the Unity project, then **Window → General → Test Runner → EditMode**:

| Suite | Coverage |
|-------|----------|
| `CharacterCatalogEditModeTests` | Procedural + mesh3d spawn |
| `CharacterSchemaValidationTests` | JSON contract validation |
| `MaterialProfileLoaderTests` | Material profile JSON + `TBG/CharacterPBR` |
| `BehaviorProfileLoaderTests` | Behavior profiles + `BehaviorPlayer` action mapping |
| `Mesh3DCharacterFactoryTests` | glTF import, scale, animator attachment |

Or batchmode:

```bash
unity -batchmode -projectPath unity -runTests -testPlatform EditMode \
  -assemblyNames TBG.CharacterLibrary.Tests -logFile -
```

Test fixtures: `content/characters/meshes/_test_fixture.glb`, `test_mesh3d_spawn.json`.
