# GLB character animation — open-source references

Survey of projects that load **skinned GLB/GLTF** and apply locomotion / IK, mapped to TBG patterns.

## Repositories

| Project | Stack | Technique | TBG mapping |
|---------|-------|-----------|-------------|
| [ossos](https://github.com/sketchpunklabs/ossos) | TS, engine-agnostic | GLTF load → IK rigs (FABRIK, Limb, SwingTwist); bone springs | Target for D1 full-body IK; web spike uses same two-bone math |
| [three.js CCDIK](https://threejs.org/docs/#examples/en/animations/CCDIKSolver) | three.js | MMD-style IK targets on skinned mesh | Heavier setup; we use custom `twoBoneIk.js` |
| [procedural-ik-crawler](https://github.com/sneha-belkhale/procedural-ik-crawler) | three.js | Raycast foot placement + IK per step | Same layer as `footIkDebug.js` |
| [threejs-procedural-spider](https://github.com/majidmanzarpour/threejs-procedural-spider) | three.js | Analytic IK, no clips | Inspiration for code-first samplers (`walk/sampler.py`) |
| [AI4AnimationPy](https://github.com/facebookresearch/ai4animationpy) | Python | BVH/GLB import, FK, FABRIK | Retarget + future ML (D3) |
| [glTF-Transform](https://github.com/donmccurdy/glTF-Transform) | JS | Inspect/optimize GLB animation tracks | CI + preview debugging |

## Common pattern (FK + IK layer)

```text
1. Load GLB → SkinnedMesh + AnimationClips
2. AnimationMixer on armature root (NOT scale wrapper)
3. Play Walk/Attack clips (FK)
4. Optional LateUpdate: foot raycast + two-bone IK
5. Optional post-mixer: quaternion damping for LINEAR glTF tracks
```

TBG implements steps 1–3 in `viewer3d.js`, 4 in `footIkDebug.js`, 5 in `motionSmoothing.js`.

## Why preview felt stiff (2026-06 diagnosis)

| Cause | Fix |
|-------|-----|
| Scout used cosine `held_arm_gait` (frozen torso) | Segment gait + `merge_guard_arm` |
| Hard clip cut (`stopAllAction`) | `clipPlayback.js` crossfade 220 ms |
| glTF LINEAR quaternion keys | `motionSmoothing.js` post-mixer slerp |
| Default unit was tank, not scout thrust pilot | Studio defaults to `demo_scout_01` |

## Rebake after sampler change

```bash
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_walk_clip.py -- HumanoidLight
python3 tools/blender/export_game_unit.py demo_scout_01
```

## Related

- [MOVEMENT-MODULE.md](../../tools/blender/animation/MOVEMENT-MODULE.md)
- [ADR-005](../adr/ADR-005-deferred-movement-features.md)
