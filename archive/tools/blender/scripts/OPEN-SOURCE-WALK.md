# Open-source walk pipeline (Linux)

Natural biped walk without Adobe/Mixamo. **100% Linux**: Blender, Python 3, CMU mocap (GitHub), gltf-transform.

## 3-layer strategy

| Layer | Controls | TBG stack |
|-------|----------|-----------|
| **1. Skin weights** | Which bone moves each vertex | Anatomical `rig_weight_paint.py` + coverage gate (25/25) |
| **2. Animation** | How bones move in Walk | CMU BVH → calibrate `biomech_curves.py` → Perry procedural bake |
| **3. Runtime preview** | Must not break skinning | No parent scale; discrete Idle/Walk; FK upright off during Walk |

```
Showcase GLB → prep → game mesh → attach TBG rig (anatomical weights)
                                        ↓
              CMU walk BVH (.local/reference/mocap/) — optional recalibration
                                        ↓
              Procedural Walk clip (humanoid_heavy_v1 / body-class profile)
                                        ↓
              export_game_unit.py → embed Walk in game GLB
                                        ↓
              Motion Arena: scale=1, playClip(Walk), arena moves root
```

## Step 0 — Download CMU reference clips

```bash
python3 tools/blender/scripts/download_reference_mocap.py
```

Output: `.local/reference/mocap/{35_01,35_04,39_01}.bvh` (gitignored).

Source: [CMU Graphics Lab Motion Capture](http://mocap.cs.cmu.edu/) via [una-dinosauria/cmu-mocap](https://github.com/una-dinosauria/cmu-mocap).

## Step 1 — Weights (Layer 1)

```bash
python3 tools/blender/export_game_unit.py demo_guard_01
```

Uses TBG anatomical weight paint on attach. Gate: `weights_skewed_to_one_bone()` in batch export.

## Step 2 — Walk animation (Layer 2)

Default: **procedural Perry gait** baked at export. Curves are pre-calibrated from CMU subjects 35/39 (`biomech_curves.py`).

Optional recalibration from fresh CMU BVH:

```bash
python3 tools/blender/scripts/retarget_mocap_walk.py --calibrate \
  .local/reference/mocap/35_01.bvh .local/reference/mocap/35_04.bvh

# Writes keyframes into biomech_curves.py (review diff before commit):
python3 tools/blender/scripts/retarget_mocap_walk.py --write-curves \
  .local/reference/mocap/35_01.bvh .local/reference/mocap/35_04.bvh
```

Re-bake body-class library clip (optional):

```bash
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_walk_clip.py -- HumanoidHeavy
```

Re-export unit (embeds procedural Walk):

```bash
python3 tools/blender/export_game_unit.py demo_guard_01
```

### Optional: CMU mocap Walk override (dev preview)

Retarget one CMU walk cycle onto a unit skeleton (replaces procedural bake for that unit):

```bash
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/import_cmu_walk_bvh.py -- \
  --unit-id demo_guard_01 \
  --cmu-bvh .local/reference/mocap/35_01.bvh \
  --report-json

python3 tools/blender/export_game_unit.py demo_guard_01
```

Output: `content/characters/animations/clips/units/{unit_id}/Walk.glb` (used when present at export).

## Step 3 — Preview (Layer 3)

- Animation Studio → Guard Pilot → Walk
- Motion Arena → WASD (`/motion-arena.html?unit=demo_guard_01`)
- Hard refresh if cached; Foot IK off by default

## Toolchain (Linux)

| Tool | Role |
|------|------|
| Blender (`tools/blender/blender_cmd.sh`) | Rig, weights, clip bake |
| Python 3 | CMU curve extract, export orchestration |
| gltf-transform | Mesh simplify, Draco, textures |
| CMU BVH (GitHub raw) | Mocap reference — no account |

**Not used:** Adobe Mixamo, FBX from proprietary services, Windows-only tools.

**Updated:** 2026-06-30
