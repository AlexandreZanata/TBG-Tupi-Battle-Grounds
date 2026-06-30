"""Attack profile registry."""

from __future__ import annotations

from tools.blender.animation.attack.thrust_sampler import ThrustStyle

SCOUT_TRIDENT_STYLE = ThrustStyle(lunge_scale=1.0)

STYLE_PROFILES: dict[str, ThrustStyle] = {
    "scout_trident_thrust_v1": SCOUT_TRIDENT_STYLE,
}

UNIT_ATTACK_PROFILE: dict[str, str] = {"demo_scout_01": "scout_trident_thrust_v1"}


def resolve_attack_profile(unit_id: str | None) -> str | None:
    if unit_id and unit_id in UNIT_ATTACK_PROFILE:
        return UNIT_ATTACK_PROFILE[unit_id]
    return None


def get_thrust_style(profile_id: str) -> ThrustStyle:
    return STYLE_PROFILES[profile_id]
