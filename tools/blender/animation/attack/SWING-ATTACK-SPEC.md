# Heavy Melee Swing — Animation Spec

Two-handed overhead swing for `HumanoidHeavy` tank units. Timings align with `tank_melee_v1`.

## Timeline

| Phase | Time | Frames @ 30 fps |
|-------|------|-----------------|
| Windup | 0–400 ms | 0–12 |
| Hit | 400–920 ms | 12–28 |
| Recovery | 920–1270 ms | 28–38 |
| Impact event | 520 ms (`attackHitMs`) | ~16 |

## Kinetic chain (hit phase)

Segments fire with staggered delays — shared helpers in `chain.py`:

1. Leg drive (delay 0%, span 18%)
2. Torso (+2% / +16% span)
3. Shoulder (+4% / +14% span)
4. Elbow (+8% / +12% span)

Easing: `easeOut` windup/recovery, `easeInExpo` hit propulsion.

## Profiles

| Profile | Clip | Variant |
|---------|------|---------|
| `heavy_melee_swing_v1` | `Attack` | Primary guard coil |
| `heavy_melee_swing_v2` | `Attack2` | Alternate entry (`windup_alt_pose`, `phase_shift=0.12`) |

## Implementation

- Poses: `swing_poses.py`
- Sampler: `swing_sampler.py`
- Bake: `build_attack_clip.py HumanoidHeavy Attack` / `Attack2`
- Library: `content/characters/animations/clips/HumanoidHeavy/`

## Checklist

- Impact frame within `attackHitMs` ± 1 frame (contract test)
- Knees flex on downswing — no locked 180°
- Support arm counter-balances weapon arm
- Recovery completes before idle guard
- No spine yaw/roll on FK rig
