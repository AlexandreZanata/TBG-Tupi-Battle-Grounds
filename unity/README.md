# Unity client — Phase 05

`Packages/com.tbg.character-library` restored from archive with **assembly export** support.

## Assembly recipe (§5)

`CharacterDefinition` JSON may include optional `assemblyRecipe`:

```json
{
  "id": "assembled_guard_01",
  "assetMode": "mesh3d",
  "animationSet": "humanoid_heavy_combat_v1",
  "assemblyRecipe": {
    "unitId": "test_male_head_sword",
    "baseId": "male",
    "parts": ["head_round_01", "weapon_sword_01"],
    "materialId": "default"
  }
}
```

`Mesh3DCharacterFactory` resolves mesh path to `content/characters/library/exports/{unitId}.glb`.

## Tests

```bash
# Generate export GLB first (from repo root)
./scripts/seed_library.sh
PYTHONPATH=tools python3 -m assembly_studio.library.assemble_gltf  # or POST /api/assembly/export

# Unity EditMode (requires Unity 2022+ project wired to repo root)
# See archive/unity for full project; package tests in:
# unity/Packages/com.tbg.character-library/Tests/AssemblyExportSpawnTests.cs
```

`AssemblyExportSpawnTests.Create_AssembledExport_HasAnimatorAndWalkClip` ignores when export GLB is missing.

## Related

- [modular-character-assembler.md](../docs/design/modular-character-assembler.md)
- [assembly_studio README](../tools/assembly_studio/README.md)
