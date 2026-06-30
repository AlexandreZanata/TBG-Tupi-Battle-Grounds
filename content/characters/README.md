# Character content (versioned)

Unit definitions and generated assets for TBG. **Source of truth:** `definitions/*.json`.

The **Character Studio** (Unity Editor + web UI) creates, edits, and deletes files here. Do not hand-edit JSON without matching the schema in `_schema.example.json` (procedural) or `_schema.mesh3d.example.json` (mesh3d).

## Layout

| Folder | Purpose | Committed |
|--------|---------|-----------|
| `definitions/` | Unit parameters (JSON) — studio CRUD | Yes |
| `cards/` | Card images rendered from procedural 3D | Yes |
| `previews/` | Optional viewport / test captures | Yes |
| `meshes/` | Authored `.glb` units (Phase 02) | Yes |
| `textures/` | PBR map sets per unit (Phase 02) | Yes |
| `materials/` | Material profile JSON (Phase 02) | Yes |
| `behaviors/` | Behavior profile JSON (Phase 02) | Yes |
| `animations/` | Shared animator controllers (Phase 02) | Yes |

## Definition schema (Phase 02 extension)

All units share Phase 01 mechanics fields. Phase 02 adds:

| Field | Required | Values / notes |
|-------|----------|----------------|
| `assetMode` | No (default `procedural`) | `procedural` \| `mesh3d` |
| `mesh` | Yes when `mesh3d` | Path to `.glb` |
| `textures` | No | `albedo`, `normal`, `orm`; optional `emissive` |
| `materialProfile` | No | Profile id (e.g. `humans_pbr_v1`) |
| `behaviorProfile` | No | Profile id (e.g. `tank_melee_v1`) |
| `animatorController` | No | Path to Unity `.controller` |

Examples: `_schema.example.json`, `_schema.mesh3d.example.json`.

## Naming convention

```
{archetype}_{faction}_{nn}.json
```

Examples: `tank_human_01.json`, `dps_undead_01.json`

## Tools

```bash
pip install -r tools/requirements.txt
python3 -m tools.character_studio list
python3 -m tools.character_studio export-all-cards
./scripts/verify.sh
```

## Related docs

- [procedural-character-generator.md](../docs/design/procedural-character-generator.md) — Phase 01 blockout
- [real-3d-character-pipeline.md](../docs/design/real-3d-character-pipeline.md) — Phase 02 production 3D
- [character-creation-guide.md](../docs/design/character-creation-guide.md)
- Agent tasks: `.local/phases/` (gitignored)
