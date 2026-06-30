"""Symmetric in-place gait — stance-biased phases, correct TBG L/R sagittal mapping."""

from __future__ import annotations

import math

from tools.blender.animation.walk.style import WalkStyle
from tools.blender.animation.walk.types import BoneEuler, GaitPose

TAU = 2.0 * math.pi
STANCE_END = 0.62


def sample_symmetric_pose(frame: int, cycle_frames: int, style: WalkStyle) -> GaitPose:
    tau = (frame % cycle_frames) / cycle_frames
    tau_opp = (tau + 0.5) % 1.0

    r_hip = style.hip_swing * math.cos(TAU * tau)
    l_hip = -style.hip_swing * math.cos(TAU * tau)

    r_knee, r_foot = _knee_foot(tau, r_hip, style)
    l_knee, l_foot = _knee_foot(tau_opp, -l_hip, style)
    l_foot = -l_foot

    pelvis_y = _pelvis_bob(tau, style)
    spine_x = style.torso_pitch * math.sin(TAU * 2.0 * tau)

    r_arm, r_fore = _arm(tau_opp, style, -1.0, style.arm_swing_r_scale, style.right_arm_hold)
    l_arm, l_fore = _arm(tau, style, 1.0, 1.0, (0.0, 0.0, 0.0))

    return {
        "TBG_Root": (0.0, pelvis_y, 0.0),
        "TBG_Spine": (spine_x, 0.0, 0.0),
        "TBG_Spine1": (spine_x * style.spine1_pitch_scale, 0.0, 0.0),
        "TBG_UpperLeg.R": (r_hip, 0.0, 0.0),
        "TBG_LowerLeg.R": (r_knee, 0.0, 0.0),
        "TBG_Foot.R": (r_foot, 0.0, 0.0),
        "TBG_UpperLeg.L": (l_hip, 0.0, 0.0),
        "TBG_LowerLeg.L": (l_knee, 0.0, 0.0),
        "TBG_Foot.L": (l_foot, 0.0, 0.0),
        "TBG_UpperArm.R": r_arm,
        "TBG_LowerArm.R": r_fore,
        "TBG_UpperArm.L": l_arm,
        "TBG_LowerArm.L": l_fore,
    }


def _knee_foot(tau: float, hip: float, style: WalkStyle) -> tuple[float, float]:
    knee = _knee_flex(tau, style)
    foot = _foot_angle(tau, hip, style)
    return knee, foot


def _knee_flex(tau: float, style: WalkStyle) -> float:
    if tau < 0.10:
        return style.knee_stance * (tau / 0.10)
    if tau < STANCE_END:
        fade = 1.0 - (tau - 0.10) / (STANCE_END - 0.10)
        return style.knee_stance * fade * 0.35
    swing = (tau - STANCE_END) / (1.0 - STANCE_END)
    return style.knee_stance * 0.15 + style.knee_swing * math.sin(math.pi * swing)


def _foot_angle(tau: float, hip: float, style: WalkStyle) -> float:
    if tau < STANCE_END:
        return style.foot_plant - hip * 0.12
    lift = (tau - STANCE_END) / (1.0 - STANCE_END)
    return style.foot_lift * math.sin(math.pi * lift)


def _pelvis_bob(tau: float, style: WalkStyle) -> float:
    return style.pelvis_vertical_m * 0.5 * (1.0 - math.cos(TAU * 2.0 * tau))


def _arm(
    leg_tau: float,
    style: WalkStyle,
    sign: float,
    scale: float,
    offset: BoneEuler,
) -> tuple[BoneEuler, BoneEuler]:
    swing = sign * style.arm_swing * scale * math.cos(TAU * leg_tau)
    follow = swing * style.forearm_follow_ratio
    upper = (-0.04 + swing + offset[0], offset[1], offset[2])
    fore = (-0.18 + follow + offset[0] * 0.2, offset[1] * 0.35, offset[2] * 0.2)
    return upper, fore
