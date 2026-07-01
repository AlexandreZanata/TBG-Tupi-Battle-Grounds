"""Walk style — segment gait presets (Perry curves + tunable amplitudes)."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

from tools.blender.animation.walk.types import BoneEuler

GaitSampler = Literal["segment", "held_arm", "eight_phase"]


@dataclass(frozen=True)
class WalkStyle:
    cadence_spm: float = 110.0
    angle_scale: float = 1.0
    pelvis_vertical_m: float = 0.040
    arm_swing: float = 0.26
    forearm_follow_ratio: float = 0.45
    torso_scale: float = 1.0
    neck_scale: float = 1.0
    head_scale: float = 1.0
    arm_swing_r_scale: float = 1.0
    right_arm_hold: BoneEuler = (0.0, 0.0, 0.0)
    spine1_pitch_scale: float = 0.68
    gait_sampler: GaitSampler = "segment"
    stride_length_m: float = 1.4
    # held_arm_gait only
    hip_swing: float = 0.24
    knee_stance: float = 0.14
    knee_swing: float = 0.42
    foot_plant: float = 0.03
    foot_lift: float = 0.05


HUMANOID_LIGHT_STYLE = WalkStyle(
    cadence_spm=115.0,
    angle_scale=0.90,
    pelvis_vertical_m=0.038,
    arm_swing=0.24,
)

HUMANOID_HEAVY_STYLE = WalkStyle(
    cadence_spm=95.0,
    angle_scale=1.05,
    pelvis_vertical_m=0.044,
    arm_swing=0.18,
    arm_swing_r_scale=0.85,
    spine1_pitch_scale=0.62,
)

SCOUT_PILOT_STYLE = WalkStyle(
    gait_sampler="segment",
    cadence_spm=110.0,
    angle_scale=0.88,
    pelvis_vertical_m=0.038,
    arm_swing=0.20,
    arm_swing_r_scale=0.0,
    spine1_pitch_scale=0.65,
)

BLOCKY_STUDY_STYLE = WalkStyle(
    gait_sampler="eight_phase",
    cadence_spm=115.0,
    angle_scale=1.0,
    pelvis_vertical_m=0.045,
    arm_swing=0.35,
    torso_scale=0.85,
    spine1_pitch_scale=0.7,
)
