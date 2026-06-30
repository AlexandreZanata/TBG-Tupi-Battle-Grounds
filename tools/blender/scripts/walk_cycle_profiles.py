"""In-place walk cycle pose tables — 4-pose gait (contact → down → pass → up).

References:
- Blender Actions cyclic range: https://docs.blender.org/manual/en/latest/animation/actions.html
- In-place walk (no root XZ): tools/mesh-pipeline/BUDGETS.md
"""

from __future__ import annotations

from typing import Final

WALK_FRAMES: Final[int] = 40
WALK_FPS: Final[int] = 30

# Euler XYZ offsets (radians) on TBG_* pose bones at key frames.
# Frame 0 == frame 40 for seamless loop (Blender cyclic action).
PoseTable = dict[int, dict[str, tuple[float, float, float]]]

# Shared light-humanoid base (agile gait).
_HUMANOID_LIGHT: PoseTable = {
    0: {
        "TBG_Root": (0.0, 0.0, 0.0),
        "TBG_Spine": (0.03, 0.0, 0.04),
        "TBG_Spine1": (0.05, 0.0, -0.03),
        "TBG_UpperLeg.R": (0.42, 0.0, 0.06),
        "TBG_LowerLeg.R": (-0.08, 0.0, 0.0),
        "TBG_Foot.R": (0.18, 0.0, 0.0),
        "TBG_UpperLeg.L": (-0.32, 0.0, -0.06),
        "TBG_LowerLeg.L": (0.22, 0.0, 0.0),
        "TBG_Foot.L": (-0.12, 0.0, 0.0),
        "TBG_UpperArm.R": (-0.28, 0.0, 0.12),
        "TBG_LowerArm.R": (-0.15, 0.0, 0.0),
        "TBG_UpperArm.L": (0.32, 0.0, -0.1),
        "TBG_LowerArm.L": (0.18, 0.0, 0.0),
    },
    10: {
        "TBG_Root": (0.0, 0.0, 0.0),
        "TBG_Spine": (0.06, 0.0, 0.0),
        "TBG_Spine1": (0.08, 0.0, 0.0),
        "TBG_UpperLeg.R": (0.12, 0.0, 0.04),
        "TBG_LowerLeg.R": (0.38, 0.0, 0.0),
        "TBG_Foot.R": (-0.05, 0.0, 0.0),
        "TBG_UpperLeg.L": (-0.08, 0.0, -0.04),
        "TBG_LowerLeg.L": (0.08, 0.0, 0.0),
        "TBG_Foot.L": (0.05, 0.0, 0.0),
        "TBG_UpperArm.R": (-0.08, 0.0, 0.08),
        "TBG_LowerArm.R": (-0.1, 0.0, 0.0),
        "TBG_UpperArm.L": (0.1, 0.0, -0.08),
        "TBG_LowerArm.L": (0.12, 0.0, 0.0),
    },
    20: {
        "TBG_Root": (0.0, 0.0, 0.0),
        "TBG_Spine": (0.03, 0.0, -0.04),
        "TBG_Spine1": (0.05, 0.0, 0.03),
        "TBG_UpperLeg.R": (-0.32, 0.0, -0.06),
        "TBG_LowerLeg.R": (0.22, 0.0, 0.0),
        "TBG_Foot.R": (-0.12, 0.0, 0.0),
        "TBG_UpperLeg.L": (0.42, 0.0, 0.06),
        "TBG_LowerLeg.L": (-0.08, 0.0, 0.0),
        "TBG_Foot.L": (0.18, 0.0, 0.0),
        "TBG_UpperArm.R": (0.32, 0.0, -0.1),
        "TBG_LowerArm.R": (0.18, 0.0, 0.0),
        "TBG_UpperArm.L": (-0.28, 0.0, 0.12),
        "TBG_LowerArm.L": (-0.15, 0.0, 0.0),
    },
    30: {
        "TBG_Root": (0.0, 0.0, 0.0),
        "TBG_Spine": (0.06, 0.0, 0.0),
        "TBG_Spine1": (0.08, 0.0, 0.0),
        "TBG_UpperLeg.R": (-0.08, 0.0, -0.04),
        "TBG_LowerLeg.R": (0.08, 0.0, 0.0),
        "TBG_Foot.R": (0.05, 0.0, 0.0),
        "TBG_UpperLeg.L": (0.12, 0.0, 0.04),
        "TBG_LowerLeg.L": (0.38, 0.0, 0.0),
        "TBG_Foot.L": (-0.05, 0.0, 0.0),
        "TBG_UpperArm.R": (0.1, 0.0, -0.08),
        "TBG_LowerArm.R": (0.12, 0.0, 0.0),
        "TBG_UpperArm.L": (-0.08, 0.0, 0.08),
        "TBG_LowerArm.L": (-0.1, 0.0, 0.0),
    },
}

# Scout Pilot — heavier silhouette, trident in right hand, wider stance.
_SCOUT_PILOT: PoseTable = {
    frame: {
        **poses,
        "TBG_Spine1": (
            poses.get("TBG_Spine1", (0.0, 0.0, 0.0))[0] + 0.06,
            poses.get("TBG_Spine1", (0.0, 0.0, 0.0))[1],
            poses.get("TBG_Spine1", (0.0, 0.0, 0.0))[2],
        ),
        "TBG_UpperLeg.R": (
            poses["TBG_UpperLeg.R"][0],
            poses["TBG_UpperLeg.R"][1],
            poses["TBG_UpperLeg.R"][2] + 0.05,
        ),
        "TBG_UpperLeg.L": (
            poses["TBG_UpperLeg.L"][0],
            poses["TBG_UpperLeg.L"][1],
            poses["TBG_UpperLeg.L"][2] - 0.05,
        ),
        "TBG_UpperArm.R": (-0.12, 0.15, 0.35),
        "TBG_LowerArm.R": (-0.45, 0.0, 0.1),
        "TBG_Hand.R": (0.0, 0.0, 0.0),
    }
    for frame, poses in _HUMANOID_LIGHT.items()
}

# Vertical bob on root (Y only — in-place lane walk).
_ROOT_BOB: dict[int, float] = {0: 0.0, 10: -0.018, 20: 0.0, 30: -0.018, 40: 0.0}

WALK_PROFILES: dict[str, PoseTable] = {
    "humanoid_light_v1": _HUMANOID_LIGHT,
    "scout_pilot_v1": _SCOUT_PILOT,
}

UNIT_WALK_PROFILE: dict[str, str] = {
    "demo_scout_01": "scout_pilot_v1",
}


def resolve_walk_profile(unit_id: str | None, body_class: str) -> str:
    if unit_id and unit_id in UNIT_WALK_PROFILE:
        return UNIT_WALK_PROFILE[unit_id]
    if body_class == "HumanoidLight":
        return "humanoid_light_v1"
    return "humanoid_light_v1"


def root_bob_y(frame: int) -> float:
    return _ROOT_BOB.get(frame, 0.0)
