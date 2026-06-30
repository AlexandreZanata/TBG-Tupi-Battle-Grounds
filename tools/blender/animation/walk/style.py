"""Walk style — held-arm in-place gait (radians / meters)."""

from __future__ import annotations

from dataclasses import dataclass

from tools.blender.animation.walk.types import BoneEuler


@dataclass(frozen=True)
class WalkStyle:
    cadence_spm: float = 110.0
    hip_swing: float = 0.24
    knee_stance: float = 0.14
    knee_swing: float = 0.42
    foot_plant: float = 0.03
    foot_lift: float = 0.05
    pelvis_vertical_m: float = 0.016
    arm_swing: float = 0.22
    forearm_follow_ratio: float = 0.45
    # Legacy — unused by held_arm_gait
    angle_scale: float = 1.0
    torso_scale: float = 1.0
    neck_scale: float = 1.0
    head_scale: float = 1.0
    arm_swing_r_scale: float = 1.0
    right_arm_hold: BoneEuler = (0.0, 0.0, 0.0)
    spine1_pitch_scale: float = 0.5
    torso_pitch: float = 0.0


HUMANOID_LIGHT_STYLE = WalkStyle()
SCOUT_PILOT_STYLE = WalkStyle(
    hip_swing=0.20,
    knee_swing=0.38,
    pelvis_vertical_m=0.014,
    arm_swing=0.18,
)
