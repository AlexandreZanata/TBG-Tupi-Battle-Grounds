# Mesh pipeline — import high-poly characters for TBG

Use when you export characters from Blender/sculpt tools as static `.glb` files (embedded PBR, no rig).

## Quick import (web studio preview)

```bash
# Default: visual mode — Draco only, preserves geometry + texture resolution
./tools/mesh-pipeline/optimize_glb.sh /path/to/sculpt.glb content/characters/meshes/my_unit.glb

# Smaller / faster preview (decimated mesh + 1024 textures)
./tools/mesh-pipeline/optimize_glb.sh --mode preview /path/to/sculpt.glb content/characters/meshes/my_unit_lod.glb
```

Requires Node.js (`npx @gltf-transform/cli`).

| Mode | What it does | Typical size | Visual |
|------|----------------|--------------|--------|
| **visual** (default) | Draco compress only | ~37–44 MB from ~80 MB source | Same as source |
| **preview** | Simplify + resize textures + Draco | ~4 MB | Lower fidelity |

Web studio accepts meshes up to **150 MB** per asset.

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
- Game target: ≤ 8k tris per skinned mesh (TBG Export add-on warning).
