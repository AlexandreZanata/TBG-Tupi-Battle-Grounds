# TBG Unity Project

Open `unity/` as a Unity 2022.3 LTS project.

## Character Library (Phase 02)

UPM package: `Packages/com.tbg.character-library/`

```csharp
var catalog = new TBG.CharacterLibrary.CharacterCatalog();
var instance = catalog.Spawn("tank_human_01", parentTransform);
```

- **Procedural** units use `ProceduralCharacterBuilder` from `TBG.CharacterSystem`.
- **Mesh3D** units use `Mesh3DCharacterFactory` (glTFast import, materials, animator).
- glTFast resolves via OpenUPM (`com.atteneder.gltfast`).

See [com.tbg.character-library/README.md](Packages/com.tbg.character-library/README.md).

## Character Studio

1. Open scene `Assets/TBG/CharacterSystem/Scenes/CharacterStudioTest.unity`
2. Menu **TBG → Character Studio**
3. Definitions read/write `content/characters/definitions/*.json` at repo root

## Headless pipeline (CI)

```bash
./scripts/verify.sh
python3 -m tools.character_studio export-all-cards
```

Card PNGs are also exportable from Unity via **Export Card** when `CardRenderer` is in scene.

Unity EditMode tests (`TBG.CharacterLibrary.Tests`) require the Unity Editor or batchmode CLI.
