# Movement module — walk, attack, root motion, retarget

Public index for the TBG **biomechanical movement module** (Phase 03b). Authoring lives under `tools/blender/animation/`; optional runtime hooks in Unity and Animation Studio.

## Packages

| Package | Purpose | Spec |
|---------|---------|------|
| `walk/` | Segment gait (default), held-arm scout profile | [walk/BIOMECHANICAL-WALK-SPEC.md](walk/BIOMECHANICAL-WALK-SPEC.md) |
| `attack/` | Thrust + heavy swing samplers | [attack/SWING-ATTACK-SPEC.md](attack/SWING-ATTACK-SPEC.md) |
| `root_motion/` | `WalkRoot` forward +Z on `TBG_Root` | [root_motion/README.md](root_motion/README.md) |
| `retarget/` | CMU BVH → `TBG_*` curve calibration | [retarget/README.md](retarget/README.md) |

## Layer rule

| Layer | bpy | Tests |
|-------|-----|-------|
| `sampler.py`, `api.py`, `profiles.py`, `velocity.py` | **Never** | `tests/blender/` |
| `bake.py`, `export.py` | Yes | Blender CLI smoke |
| `scripts/build_*_clip.py` | Thin CLI only | — |

## Walk profiles

| Profile | Body / unit | Sampler |
|---------|-------------|---------|
| `humanoid_light_v1` | HumanoidLight | `segment_gait` |
| `humanoid_heavy_v1` | HumanoidHeavy | `segment_gait` |
| `scout_pilot_v1` | `demo_scout_01` | `held_arm_gait` |

**Velocity scaling (runtime contract):** `walk/velocity.py` — `scale_walk_style_for_speed(style, move_speed_mps)` scales cadence and stride so `velocity = stride × cadence / 120`. Full animator blend trees deferred post-MVP.

## Attack profiles

| Profile | Clip | Behavior |
|---------|------|----------|
| `scout_trident_thrust_v1` | `Attack` | `ranged_shot_v1` |
| `heavy_melee_swing_v1` | `Attack` | `tank_melee_v1` |
| `heavy_melee_swing_v2` | `Attack2` | `tank_melee_v1` (phase shift) |

Impact frame within `behavior.timings.attackHitMs` ± 1 frame — see `tests/blender/attack/`.

## Root motion convention

- **In-place:** `Walk` — `TBG_Root.z = 0`
- **Root motion:** `WalkRoot` — `TBG_Root.z = stride_length_m × τ`
- **Velocity:** `stride × cadence / 120` m/s

Bake: `build_walk_clip.py --root-motion HumanoidHeavy`

## Foot IK evaluation (§5.1)

| Option | Verdict | Notes |
|--------|---------|-------|
| Unity Animation Rigging | Deferred | Extra package; overkill for lane MVP |
| Lightweight two-bone IK | **Chosen** | `LocomotionFootIk.cs` — raycast + `TwoBoneIk.Solve` |

Gated by `behavior.locomotion.footIk` (default `false`). Walk state only (`IsWalking` bool or `Walk` state).

## Web preview IK (§5.2)

Animation Studio toggle **Foot IK debug** — `footIkDebug.js` + `twoBoneIk.js`. Debug overlay only; not shipped in game build.

## CLI quick reference

```bash
pytest tests/blender/walk/ tests/blender/attack/ tests/blender/root_motion/ -q

./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_walk_clip.py -- HumanoidHeavy
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_walk_clip.py -- --root-motion HumanoidHeavy
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_attack_clip.py -- HumanoidHeavy Attack2
```

## Runtime (Unity)

| Feature | Flag | Component |
|---------|------|-----------|
| Root motion walk | `locomotion.useRootMotionWalk` | `BehaviorPlayer.SyncWalkRootMotion` |
| Foot IK | `locomotion.footIk` | `LocomotionFootIk` |

## Related

- Phase tasks: `.local/phases/03b-biomechanical-movement-module/TASKS.md`
- Agent skill: `.cursor/skills/tbg-walk-animation/SKILL.md`
- Clip budgets: `tools/mesh-pipeline/BUDGETS.md`
