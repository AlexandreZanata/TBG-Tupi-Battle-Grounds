"""Walk style — symmetric sagittal gait (radians / meters)."""

from __future__ import annotations

from dataclasses import dataclass

from tools.blender.animation.walk.types import BoneEuler


@dataclass(frozen=True)
class WalkStyle:
    cadence_spm: float = 110.0
    hip_swing: float = 0.30
    knee_stance: float = 0.18
    knee_swing: float = 0.52
    foot_plant: float = 0.04
    foot_lift: float = 0.07
    pelvis_vertical_m: float = 0.022
    torso_pitch: float = 0.012
    arm_swing: float = 0.30
    forearm_follow_ratio: float = 0.5
    arm_swing_r_scale: float = 1.0
    right_arm_hold: BoneEuler = (0.0, 0.0, 0.0)
    spine1_pitch_scale: float = 0.5


HUMANOID_LIGHT_STYLE = WalkStyle()
SCOUT_PILOT_STYLE = WalkStyle(
    hip_swing=0.27,
    knee_swing=0.48,
    pelvis_vertical_m=0.018,
    torso_pitch=0.010,
    arm_swing=0.26,
    arm_swing_r_scale=0.15,
    right_arm_hold=(-0.10, 0.10, 0.18),
    spine1_pitch_scale=0.4,
)
