# Trident Lunge Thrust — Animation Spec

One-handed trident thrust with fencing lunge. Mapped to `ranged_shot_v1` timings.

## Timeline

| Phase | Time | Frames @ 30 fps |
|-------|------|-----------------|
| Windup | 0–300 ms | 0–9 |
| Hit | 300–750 ms | 9–23 |
| Recovery | 750–1050 ms | 23–32 |
| Impact event | 450 ms (`attackHitMs`) | ~14 |

## Kinetic chain (hit phase)

Segments fire with staggered delays — never one shared curve:

1. Leg drive (0–50% of hit)
2. Pelvis/torso (+10% / +20% delay)
3. Shoulder (+35%)
4. Elbow (+50%)

Easing: `easeOut` windup/recovery, `easeInExpo` hit propulsion.

## Implementation

- Poses: `thrust_poses.py` (guard, windup, peak, recovery)
- Sampler: `thrust_sampler.py` (shared `chain.py`)
- Profile: `scout_trident_thrust_v1` → `demo_scout_01`
- Bake: one-shot `Attack` action via `retarget_clip_library.py`

## Checklist

- Knees/elbows never lock at 180°
- Back foot rises during leg drive (`TBG_Foot.R`)
- Front knee flexes on lunge landing
- Recovery before returning to idle
- No spine roll (yaw-only torso rotation on `TBG_Spine` Y)
