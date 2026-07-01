# Blocky walk study (Minecraft-style)

Case-study character: **one cube per bone**, rigid weights — ideal for reading hip/knee/ankle walk phases in Motion Arena.

## Build (Linux + Blender)

```bash
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_blocky_walk_study.py -- \
  --unit-id demo_blocky_walk_01 --report-json

./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_blocky_walk_study.py -- \
  --unit-id demo_blocky_walk_01 --showcase-only
```

Output:

| File | Content |
|------|---------|
| `content/characters/meshes/game/demo_blocky_walk_01.glb` | Skinned cubes + Idle/Walk |
| `content/characters/meshes/showcase/demo_blocky_walk_01.glb` | T-pose reference |

## Preview

- Motion Arena: `/motion-arena.html` — loads **Blocky Walk Study** first
- Animation Studio: select `demo_blocky_walk_01`, play Walk

Walk clip: **Perry 8-phase** biomechanical cycle (`humanoid_blocky_v1`) — see `perry_eight_phase.py`.

## Design

- 15 cubes (head, torso, pelvis, legs×3, arms×3 per side)
- Rigid 1-bone weights — limbs rotate as solid blocks (Minecraft readability)
- ~1.75 m tall, feet at Y=0

**Updated:** 2026-06-30
