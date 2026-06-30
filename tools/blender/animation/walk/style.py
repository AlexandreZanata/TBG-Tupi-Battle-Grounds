"""Biomechanical walk style — cadence, Saunders determinants, unit overlays."""

from __future__ import annotations

from dataclasses import dataclass

from tools.blender.animation.walk.types import BoneEuler


@dataclass(frozen=True)
class WalkStyle:
    """Level-2 curve sampler parameters (see BIOMECHANICAL-WALK-SPEC.md)."""

    cadence_spm: float = 110.0
    angle_scale: float = 1.0
    pelvis_vertical_m: float = 0.042
    pelvis_yaw_deg: float = 4.0
    pelvis_roll_deg: float = 4.5
    arm_swing_deg: float = 22.0
    forearm_follow_ratio: float = 0.55
    arm_swing_r_scale: float = 1.0
    right_arm_hold: BoneEuler = (0.0, 0.0, 0.0)
    spine1_pitch_scale: float = 0.55


HUMANOID_LIGHT_STYLE = WalkStyle()
SCOUT_PILOT_STYLE = WalkStyle(
    angle_scale=0.92,
    pelvis_vertical_m=0.036,
    pelvis_yaw_deg=3.5,
    pelvis_roll_deg=4.0,
    arm_swing_deg=20.0,
    forearm_follow_ratio=0.5,
    arm_swing_r_scale=0.12,
    right_arm_hold=(-0.14, 0.12, 0.28),
    spine1_pitch_scale=0.45,
)
