"""Tunable walk style parameters and preset profiles."""

from __future__ import annotations

from dataclasses import dataclass

from tools.blender.animation.walk.types import BoneEuler


@dataclass(frozen=True)
class WalkStyle:
    """Amplitudes in radians; hip_bob in meters."""

    stride: float = 0.40
    knee_swing: float = 0.48
    knee_support: float = 0.15
    foot_lift: float = 0.09
    hip_bob: float = 0.014
    spine_pitch: float = 0.028
    spine_roll: float = 0.016
    arm_swing: float = 0.34
    forearm_follow: float = 0.18
    arm_swing_r_scale: float = 1.0
    right_arm_hold: BoneEuler = (0.0, 0.0, 0.0)
    spine1_pitch_scale: float = 0.55


HUMANOID_LIGHT_STYLE = WalkStyle()
SCOUT_PILOT_STYLE = WalkStyle(
    stride=0.36,
    knee_swing=0.44,
    knee_support=0.13,
    foot_lift=0.08,
    hip_bob=0.011,
    spine_pitch=0.022,
    spine_roll=0.012,
    arm_swing=0.30,
    forearm_follow=0.16,
    arm_swing_r_scale=0.12,
    right_arm_hold=(-0.14, 0.12, 0.28),
    spine1_pitch_scale=0.45,
)
