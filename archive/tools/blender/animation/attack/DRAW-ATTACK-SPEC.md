# Ranged Draw — Animation Spec (deferred)

Bow/crossbow draw-and-release for `HumanoidLight` ranged units. **No sampler yet** — no pilot unit with a dedicated draw clip in Phase 03b.

## Planned timeline (ranged_shot_v1)

| Phase | Time | Notes |
|-------|------|-------|
| Windup | 0–300 ms | Raise weapon, draw string |
| Hit | 300–750 ms | Release / projectile spawn |
| Recovery | 750–1050 ms | Return to guard |
| Impact event | 450 ms (`attackHitMs`) | Release frame |

## Planned implementation

- Sampler: `attack/draw_sampler.py` (not authored)
- Profile: `ranged_draw_v1` → future ranged pilot
- Reuse `chain.py` segment delays with arm-dominant ordering (shoulder → elbow → hand)

## Deferral rationale

`demo_scout_01` uses trident thrust (`scout_trident_thrust_v1`). Draw sampler ships when a ranged weapon mesh + `Attack` clip mapping exists.
