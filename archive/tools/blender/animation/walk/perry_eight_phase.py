"""Perry/Winter 8-phase human gait — sagittal keyframes + Saunders determinants.

Reference: BIOMECHANICAL-WALK-SPEC §2; Winter (2009); Saunders et al. (1953).
Interpolation: cyclic Catmull-Rom (never linear between phases).
"""

from __future__ import annotations

import math
from dataclasses import dataclass

from tools.blender.animation.walk.curves import Keyframe, sample_cycle_curve

TAU = math.tau


@dataclass(frozen=True)
class GaitKeyframe:
    """Single phase boundary — angles in degrees, sagittal plane."""

    phase_percent: float
    hip_angle_deg: float
    knee_angle_deg: float
    ankle_angle_deg: float


@dataclass(frozen=True)
class GaitCycleConfig:
    cadence_steps_per_min: float = 115.0
    stride_length_m: float = 1.4
    com_vertical_amplitude_m: float = 0.045
    com_lateral_amplitude_m: float = 0.04
    pelvis_rotation_deg: float = 4.0
    pelvis_obliquity_deg: float = 4.5
    arm_swing_amplitude_deg: float = 35.0
    elbow_base_deg: float = 20.0
    angle_scale: float = 1.0


# Section 2 table — phase boundary samples (% → τ)
PERRY_PHASE_KEYFRAMES: tuple[GaitKeyframe, ...] = (
    GaitKeyframe(0, 20.0, 0.0, 0.0),
    GaitKeyframe(10, 20.0, 20.0, -8.0),
    GaitKeyframe(30, 0.0, 0.0, 5.0),
    GaitKeyframe(50, -20.0, 0.0, 10.0),
    GaitKeyframe(60, 0.0, 40.0, -18.0),
    GaitKeyframe(73, 15.0, 60.0, 0.0),
    GaitKeyframe(87, 25.0, 30.0, 0.0),
    GaitKeyframe(100, 20.0, 0.0, 0.0),
)

DEFAULT_GAIT_CONFIG = GaitCycleConfig()


def _phase_curves() -> tuple[tuple[Keyframe, ...], tuple[Keyframe, ...], tuple[Keyframe, ...]]:
    hip: list[Keyframe] = []
    knee: list[Keyframe] = []
    ankle: list[Keyframe] = []
    for row in PERRY_PHASE_KEYFRAMES:
        tau = row.phase_percent / 100.0
        hip.append((tau, row.hip_angle_deg))
        knee.append((tau, row.knee_angle_deg))
        ankle.append((tau, row.ankle_angle_deg))
    return tuple(hip), tuple(knee), tuple(ankle)


HIP_PHASE_DEG, KNEE_PHASE_DEG, ANKLE_PHASE_DEG = _phase_curves()


def leg_phase_tau(tau: float, side: str) -> float:
    """Right leg at τ; left leg offset 50% (§6 rule 2)."""
    if side == "R":
        return tau % 1.0
    return (tau + 0.5) % 1.0


def get_leg_angles(tau: float, side: str, config: GaitCycleConfig = DEFAULT_GAIT_CONFIG) -> tuple[float, float, float]:
    t_leg = leg_phase_tau(tau, side)
    scale = config.angle_scale
    hip = sample_cycle_curve(HIP_PHASE_DEG, t_leg) * scale
    knee = sample_cycle_curve(KNEE_PHASE_DEG, t_leg) * scale
    ankle = sample_cycle_curve(ANKLE_PHASE_DEG, t_leg) * scale
    return hip, knee, ankle


def com_vertical_offset(tau: float, config: GaitCycleConfig = DEFAULT_GAIT_CONFIG) -> float:
    """Two peaks per stride — double support at τ=0/0.5 (§4)."""
    return config.com_vertical_amplitude_m * 0.5 * (1.0 - math.cos(TAU * 2.0 * tau))


def com_lateral_offset(tau: float, config: GaitCycleConfig = DEFAULT_GAIT_CONFIG) -> float:
    return config.com_lateral_amplitude_m * 0.5 * math.sin(TAU * tau)


def pelvis_yaw_rad(tau: float, config: GaitCycleConfig = DEFAULT_GAIT_CONFIG) -> float:
    return math.radians(config.pelvis_rotation_deg) * math.sin(TAU * tau)


def pelvis_roll_rad(tau: float, config: GaitCycleConfig = DEFAULT_GAIT_CONFIG) -> float:
    return math.radians(config.pelvis_obliquity_deg) * math.sin(TAU * tau + math.pi / 2.0)


def shoulder_swing_rad(tau: float, side: str, config: GaitCycleConfig = DEFAULT_GAIT_CONFIG) -> float:
    phase = math.pi if side == "R" else 0.0
    amp = math.radians(config.arm_swing_amplitude_deg)
    return amp * math.sin(TAU * tau + phase)


def elbow_flex_rad(tau: float, config: GaitCycleConfig = DEFAULT_GAIT_CONFIG) -> float:
    base = math.radians(config.elbow_base_deg)
    swing = math.radians(8.0) * math.sin(TAU * tau + math.pi / 4.0)
    return base + swing
