# Mesh pipeline — import high-poly characters for TBG

Use when you export characters from Blender/sculpt tools as static `.glb` files (embedded PBR, no rig).

## Quick import (web studio preview)

```bash
# Default: visual mode — Draco only, preserves geometry + texture resolution
./tools/mesh-pipeline/optimize_glb.sh /path/to/sculpt.glb content/characters/meshes/my_unit.glb

# Smaller / faster preview (decimated mesh + 1024 textures)
./tools/mesh-pipeline/optimize_glb.sh --mode preview /path/to/sculpt.glb content/characters/meshes/my_unit_lod.glb

# Mobile game mesh (≤ 8k tris, 1024² textures, Draco)
./tools/mesh-pipeline/optimize_glb.sh --mode game --tri 8000 --tex 1024 \
  /path/to/showcase.glb content/characters/meshes/game/my_unit.glb
```

Requires Node.js (`npx @gltf-transform/cli`).

| Mode | What it does | Typical size | Visual |
|------|----------------|--------------|--------|
| **visual** (default) | Draco compress only | ~37–44 MB from ~80 MB source | Same as source |
| **preview** | Simplify + resize textures + Draco | ~4 MB | Lower fidelity |
| **game** | Simplify to tri budget + resize + Draco | ≤ 2 MB target | Mobile runtime |

Validate game meshes: `python3 tools/mesh-pipeline/validate_game_glb.py <path.glb>`  
Export one pilot: `python3 tools/blender/export_game_unit.py demo_sentinel_01` (requires `./tools/blender/setup.sh` once)  
Batch export all pilots: `./tools/mesh-pipeline/batch_seed_to_game.sh`  
Post-process: ORM texture pack + LOD1 static mesh in `meshes/game/lod1/` (automatic on export)  
Budget reference: [BUDGETS.md](./BUDGETS.md)

Web studio accepts meshes up to **150 MB** per asset.

## Batch import showcase sculpts

Manifest: `content/characters/fixed_test_imports.json` (15 fixed test characters).

```bash
python3 tools/mesh-pipeline/bootstrap_fixed_test_characters.py  # new ids + roster order
python3 tools/mesh-pipeline/import_showcase_sculpts.py --source-dir /path/to/downloads --force
python3 -m character_studio export-all-cards   # refresh cards (PYTHONPATH=tools)
```

Import reads `fixed_test_imports.json` by default. Use `--only demo_scout_01` for a subset.

## Batch export game meshes (Phase 03 §8)

```bash
./tools/blender/setup.sh                    # HumanoidHeavy + HumanoidLight rig templates
./tools/mesh-pipeline/batch_seed_to_game.sh # all pilot_roster.json ids
./scripts/verify.sh                         # validates every meshes/game/*.glb
```

Showcase carousel continues to use `meshShowcase` tier; game meshes land in `meshes/game/` with LOD1 in `meshes/game/lod1/`.

Manual Blender weight/silhouette fixes: re-export individual ids after editing showcase sculpt.

## Studio demo units

| id | Source sculpt |
|----|----------------|
| `demo_scout_01` | Scout ranged pilot |
| `demo_striker_01` | DPS striker |
| `demo_guard_01` | Tank guard |

Open `./scripts/start-studio.sh` → select unit → **3D** tab. First load of visual-mode meshes can take 10–30s (high poly).

## Add a unit

1. Run `optimize_glb.sh` (visual mode).
2. Create `content/characters/definitions/{id}.json` with `assetMode: "mesh3d"`.
3. Omit `textures` when the GLB already has embedded PBR.
4. Tint with `primaryColor` / `secondaryColor` in the studio.

## Animations

Static sculpts use procedural Idle / Attack / Death in the browser. For skeletal clips, skin to a Blender `TBG_*` rig per `tools/blender/RIG-SPEC.md` and re-export.

## Limits

- Web preview: **150 MB** per asset.
- Game target: see [BUDGETS.md](./BUDGETS.md) — ≤ 8k tris, ≤ 64 bones, ≤ 2 MB target GLB.
