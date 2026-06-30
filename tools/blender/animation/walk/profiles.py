"""Walk profile registry — maps units and body classes to WalkStyle presets."""

from __future__ import annotations

from tools.blender.animation.walk.style import HUMANOID_LIGHT_STYLE, SCOUT_PILOT_STYLE, WalkStyle

STYLE_PROFILES: dict[str, WalkStyle] = {
    "humanoid_light_v1": HUMANOID_LIGHT_STYLE,
    "scout_pilot_v1": SCOUT_PILOT_STYLE,
}

UNIT_WALK_PROFILE: dict[str, str] = {"demo_scout_01": "scout_pilot_v1"}

GAIT_PROFILES = STYLE_PROFILES
WALK_PROFILES = STYLE_PROFILES


def resolve_walk_profile(unit_id: str | None, body_class: str) -> str:
    if unit_id and unit_id in UNIT_WALK_PROFILE:
        return UNIT_WALK_PROFILE[unit_id]
    return "humanoid_light_v1"


def get_walk_style(profile_id: str) -> WalkStyle:
    return STYLE_PROFILES[profile_id]
