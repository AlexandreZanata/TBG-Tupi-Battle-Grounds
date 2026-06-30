"""Attack profile registry — thrust, swing, and body-class clip mapping."""

from __future__ import annotations

from tools.blender.animation.attack.swing_sampler import SwingStyle
from tools.blender.animation.attack.thrust_sampler import ThrustStyle
from tools.blender.animation.attack.timing import AttackTiming

TANK_MELEE_ATTACK_HIT_MS = 520
RANGED_SHOT_ATTACK_HIT_MS = 450
TANK_MELEE_TIMING = AttackTiming(windup_ms=400, hit_ms=520, recovery_ms=350)
RANGED_SHOT_TIMING = AttackTiming(windup_ms=300, hit_ms=450, recovery_ms=300)

SCOUT_TRIDENT_STYLE = ThrustStyle(lunge_scale=1.0)
HEAVY_SWING_PRIMARY = SwingStyle(timing=TANK_MELEE_TIMING, variant="primary")
HEAVY_SWING_ALTERNATE = SwingStyle(
    timing=TANK_MELEE_TIMING,
    variant="alternate",
    phase_shift=0.12,
)

THRUST_PROFILES: dict[str, ThrustStyle] = {
    "scout_trident_thrust_v1": SCOUT_TRIDENT_STYLE,
}

SWING_PROFILES: dict[str, SwingStyle] = {
    "heavy_melee_swing_v1": HEAVY_SWING_PRIMARY,
    "heavy_melee_swing_v2": HEAVY_SWING_ALTERNATE,
}

UNIT_ATTACK_PROFILE: dict[str, str] = {"demo_scout_01": "scout_trident_thrust_v1"}

CLIP_ATTACK_PROFILE: dict[tuple[str, str], str] = {
    ("HumanoidHeavy", "Attack"): "heavy_melee_swing_v1",
    ("HumanoidHeavy", "Attack2"): "heavy_melee_swing_v2",
}


def resolve_attack_profile(
    unit_id: str | None,
    body_class: str | None = None,
    clip_name: str = "Attack",
) -> str | None:
    if unit_id and unit_id in UNIT_ATTACK_PROFILE:
        return UNIT_ATTACK_PROFILE[unit_id]
    if body_class:
        return CLIP_ATTACK_PROFILE.get((body_class, clip_name))
    return None


def get_thrust_style(profile_id: str) -> ThrustStyle:
    return THRUST_PROFILES[profile_id]


def get_swing_style(profile_id: str) -> SwingStyle:
    return SWING_PROFILES[profile_id]


def is_swing_profile(profile_id: str) -> bool:
    return profile_id in SWING_PROFILES


def behavior_attack_hit_ms(profile_id: str) -> int | None:
    if profile_id in SWING_PROFILES:
        return TANK_MELEE_ATTACK_HIT_MS
    if profile_id == "scout_trident_thrust_v1":
        return RANGED_SHOT_ATTACK_HIT_MS
    return None
