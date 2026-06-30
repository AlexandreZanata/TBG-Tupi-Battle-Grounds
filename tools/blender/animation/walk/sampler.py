"""Level-2 biomechanical walk sampler — Catmull-Rom joint curves + Saunders pelvis."""

from __future__ import annotations

import math

from tools.blender.animation.walk.biomech_curves import ANKLE_CURVE_DEG, HIP_CURVE_DEG, KNEE_CURVE_DEG
from tools.blender.animation.walk.constants import WALK_FPS, cycle_frames
from tools.blender.animation.walk.curves import sample_cycle_curve
from tools.blender.animation.walk.style import WalkStyle
from tools.blender.animation.walk.types import BoneEuler, GaitPose

TAU = 2.0 * math.pi
DEG = math.pi / 180.0


def sample_walk_pose(frame: int, cycle_frame_count: int, style: WalkStyle) -> GaitPose:
    tau = (frame % cycle_frame_count) / cycle_frame_count
    tau_left = (tau + 0.5) % 1.0

    r_hip, r_knee, r_ankle = _leg_angles(tau, style)
    l_hip, l_knee, l_ankle = _leg_angles(tau_left, style)

    pelvis_y = _pelvis_vertical(tau, style)
    spine_x, spine_y, spine_z = _pelvis_determinants(tau, style)

    r_arm, r_fore = _arm_chain(tau_left, style, -1.0, style.arm_swing_r_scale, style.right_arm_hold)
    l_arm, l_fore = _arm_chain(tau, style, 1.0, 1.0, (0.0, 0.0, 0.0))

    return {
        "TBG_Root": (0.0, pelvis_y, 0.0),
        "TBG_Spine": (spine_x, spine_y, spine_z),
        "TBG_Spine1": (spine_x * style.spine1_pitch_scale, spine_y * 0.4, spine_z * 0.35),
        "TBG_UpperLeg.R": (r_hip, 0.0, 0.0),
        "TBG_LowerLeg.R": (r_knee, 0.0, 0.0),
        "TBG_Foot.R": (r_ankle, 0.0, 0.0),
        "TBG_UpperLeg.L": (l_hip, 0.0, 0.0),
        "TBG_LowerLeg.L": (l_knee, 0.0, 0.0),
        "TBG_Foot.L": (l_ankle, 0.0, 0.0),
        "TBG_UpperArm.R": r_arm,
        "TBG_LowerArm.R": r_fore,
        "TBG_UpperArm.L": l_arm,
        "TBG_LowerArm.L": l_fore,
    }


def cycle_frames_for_style(style: WalkStyle) -> int:
    return cycle_frames(style.cadence_spm)


def walk_cycle_duration_s(style: WalkStyle) -> float:
    return cycle_frames_for_style(style) / WALK_FPS


def _leg_angles(tau: float, style: WalkStyle) -> tuple[float, float, float]:
    scale = style.angle_scale * DEG
    hip = sample_cycle_curve(HIP_CURVE_DEG, tau) * scale
    knee = sample_cycle_curve(KNEE_CURVE_DEG, tau) * scale
    ankle = sample_cycle_curve(ANKLE_CURVE_DEG, tau) * scale
    return hip, knee, ankle


def _pelvis_vertical(tau: float, style: WalkStyle) -> float:
    # Double-frequency bob; lowest at double-support (τ≈0, 0.5)
    return style.pelvis_vertical_m * 0.5 * (1.0 - math.cos(TAU * 2.0 * tau))


def _pelvis_determinants(tau: float, style: WalkStyle) -> tuple[float, float, float]:
    pitch = 0.012 * math.sin(TAU * 2.0 * tau + 0.2)
    yaw = style.pelvis_yaw_deg * DEG * math.sin(TAU * tau)
    roll = style.pelvis_roll_deg * DEG * math.sin(TAU * tau + math.pi / 2.0)
    return pitch, yaw, roll


def _arm_chain(
    leg_tau: float,
    style: WalkStyle,
    sign: float,
    scale: float,
    offset: BoneEuler,
) -> tuple[BoneEuler, BoneEuler]:
    swing = sign * style.arm_swing_deg * DEG * scale * math.sin(TAU * leg_tau + math.pi)
    follow = swing * style.forearm_follow_ratio
    upper = (-0.05 + swing + offset[0], offset[1], 0.08 * sign + offset[2])
    fore = (-0.20 + follow + offset[0] * 0.25, offset[1] * 0.4, offset[2] * 0.25)
    return upper, fore
