# Character content (versioned)

Unit definitions and generated assets for TBG. **Source of truth:** `definitions/*.json`.

The **Character Studio** (Unity Editor + web UI) creates, edits, and deletes files here. Do not hand-edit JSON without matching the schema in `_schema.example.json` (procedural) or `_schema.mesh3d.example.json` (mesh3d).

## Layout

| Folder | Purpose | Committed |
|--------|---------|-----------|
| `definitions/` | Unit parameters (JSON) — studio CRUD | Yes |
| `cards/` | Card images rendered from procedural 3D | Yes |
| `previews/` | Optional viewport / test captures | Yes |
| `meshes/showcase/` | High-poly preview sculpts (web carousel / studio) | Yes — 8 pilot `demo_*.glb` |
| `meshes/game/` | Mobile runtime game meshes (Phase 03) | Yes — pilot `demo_sentinel_01.glb` |
| `textures/` | PBR map sets per unit (Phase 02) | Yes — pilot `*_*.png` sets |
| `materials/` | Material profile JSON (Phase 02) | Yes — `*_pbr_v1.json` per faction |
| `behaviors/` | Behavior profile JSON (Phase 02) | Yes — `*_v1.json` per archetype |
| `animations/` | Shared animator controllers (Phase 02) | Yes — one `.controller` per body class |
| `animation_sets/` | Clip library manifests (Phase 03) | Yes — e.g. `humanoid_heavy_combat_v1.json` |

## Definition schema (Phase 02 extension)

All units share Phase 01 mechanics fields. Phase 02 adds:

| Field | Required | Values / notes |
|-------|----------|----------------|
| `assetMode` | No (default `procedural`) | `procedural` \| `mesh3d` |
| `mesh` | Yes when `mesh3d` | Game runtime path to `.glb` (≤ 8k tris) |
| `meshShowcase` | No | High-poly preview `.glb` for web carousel / studio |
| `animationSet` | No | Clip library id (e.g. `humanoid_heavy_combat_v1`) |
| `gameLod` | No | `{ "triTarget": 8000, "textureSize": 1024 }` export overrides |
| `textures` | No | `albedo`, `normal`, `orm`; optional `emissive` |
| `materialProfile` | No | Profile id (e.g. `humans_pbr_v1`) — see `materials/` |
| `behaviorProfile` | No | Profile id (e.g. `tank_melee_v1`) — see `behaviors/` |
| `animatorController` | No | Path to Unity `.controller` — see `animations/` |

## Material profiles (Phase 02 §6)

| Profile id | Faction | Shader |
|------------|---------|--------|
| `humans_pbr_v1` | Humans | `TBG/CharacterPBR` |
| `undead_pbr_v1` | Undead | `TBG/CharacterPBR` |
| `machines_pbr_v1` | Machines | `TBG/CharacterPBR` |
| `plants_pbr_v1` | Plants | `TBG/CharacterPBR` |

Texture naming: `{unit_id}_albedo.png`, `{unit_id}_normal.png`, `{unit_id}_orm.png` (optional `{unit_id}_emissive.png`).

Export placeholders: `python3 tools/material-maker/export_textures.py --all-pilots`

## Behavior profiles (Phase 02 §7)

| Profile id | Archetype | Body class |
|------------|-----------|------------|
| `tank_melee_v1` | Tank | HumanoidHeavy |
| `dps_melee_v1` | DPS | Organic |
| `support_cast_v1` | Support | Mechanical |
| `ranged_shot_v1` | Ranged | HumanoidLight |
| `control_cast_v1` | Control | HumanoidLight |

Regenerate shared animator controllers: `python3 tools/unity/generate_animator_controllers.py`

## Pilot roster (Phase 02 §10)

Manifest: `pilot_roster.json` — four mesh3d pilots (one per faction).

| id | Body class | Behavior profile |
|----|------------|------------------|
| `tank_human_01` | HumanoidHeavy | `tank_melee_v1` |
| `dps_undead_01` | Organic | `dps_melee_v1` |
| `support_machine_01` | Mechanical | `support_cast_v1` |
| `ranged_plant_01` | HumanoidLight | `ranged_shot_v1` |

Provision placeholder meshes (copies `_test_fixture.glb` until authored assets land):

```bash
python3 tools/material-maker/export_textures.py --all-pilots
python3 tools/unity/provision_pilot_meshes.py
```

Examples: `_schema.example.json`, `_schema.mesh3d.example.json`.

Test-only definitions: `test_procedural_spawn.json`, `test_mesh3d_spawn.json`, `test_mesh3d_missing_glb.json`.  
Mesh fixture: `meshes/_test_fixture.glb` (Khronos Box sample for Unity spawn tests).

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
