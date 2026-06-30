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
- `com.atteneder.gltfast` (OpenUPM) — mesh3d import (wired in Phase 02 §8)

## API

```csharp
var catalog = new CharacterCatalog();
var instance = catalog.Spawn("tank_human_01", parentTransform);
instance.PlayAction(CharacterAction.Attack);
catalog.Release(instance);
```

Definitions load from `content/characters/definitions/{id}.json` at repo root.

## Tests

Open the Unity project, then **Window → General → Test Runner → EditMode**:

| Suite | Coverage |
|-------|----------|
| `CharacterCatalogEditModeTests` | Procedural + mesh3d spawn |
| `CharacterSchemaValidationTests` | JSON contract validation |

Or batchmode:

```bash
unity -batchmode -projectPath unity -runTests -testPlatform EditMode \
  -assemblyNames TBG.CharacterLibrary.Tests -logFile -
```

Test fixtures: `content/characters/meshes/_test_fixture.glb`, `test_mesh3d_spawn.json`.
