"""In-place walk — legs + pelvis only; weapon arm locked; no torso yaw."""

from __future__ import annotations

import math

from tools.blender.animation.shared.trident_guard import TRIDENT_GUARD_ARM
from tools.blender.animation.walk.style import WalkStyle
from tools.blender.animation.walk.types import BoneEuler, GaitPose

TAU = 2.0 * math.pi
STANCE_END = 0.62


def sample_held_arm_pose(frame: int, cycle_frames: int, style: WalkStyle) -> GaitPose:
    tau = (frame % cycle_frames) / cycle_frames
    tau_opp = (tau + 0.5) % 1.0

    r_hip = style.hip_swing * math.cos(TAU * tau)
    l_hip = -style.hip_swing * math.cos(TAU * tau)

    r_knee, r_foot = _knee_foot(tau, style)
    l_knee, l_foot = _knee_foot(tau_opp, style)
    l_foot = -l_foot

    pelvis_y = style.pelvis_vertical_m * 0.5 * (1.0 - math.cos(TAU * 2.0 * tau))
    l_arm, l_fore = _left_arm_swing(tau, style)

    pose: GaitPose = {
        "TBG_Root": (0.0, pelvis_y, 0.0),
        "TBG_Spine": (0.0, 0.0, 0.0),
        "TBG_Spine1": (0.0, 0.0, 0.0),
        "TBG_Neck": (0.0, 0.0, 0.0),
        "TBG_Head": (0.0, 0.0, 0.0),
        "TBG_UpperLeg.R": (r_hip, 0.0, 0.0),
        "TBG_LowerLeg.R": (r_knee, 0.0, 0.0),
        "TBG_Foot.R": (r_foot, 0.0, 0.0),
        "TBG_UpperLeg.L": (l_hip, 0.0, 0.0),
        "TBG_LowerLeg.L": (l_knee, 0.0, 0.0),
        "TBG_Foot.L": (l_foot, 0.0, 0.0),
        "TBG_UpperArm.L": l_arm,
        "TBG_LowerArm.L": l_fore,
    }
    pose.update(TRIDENT_GUARD_ARM)
    return pose


def _knee_foot(tau: float, style: WalkStyle) -> tuple[float, float]:
    knee = _knee_flex(tau, style)
    foot = _foot_angle(tau, style)
    return knee, foot


def _knee_flex(tau: float, style: WalkStyle) -> float:
    if tau < 0.10:
        return style.knee_stance * (tau / 0.10)
    if tau < STANCE_END:
        fade = 1.0 - (tau - 0.10) / (STANCE_END - 0.10)
        return style.knee_stance * fade * 0.35
    swing = (tau - STANCE_END) / (1.0 - STANCE_END)
    return style.knee_stance * 0.15 + style.knee_swing * math.sin(math.pi * swing)


def _foot_angle(tau: float, style: WalkStyle) -> float:
    if tau < STANCE_END:
        return style.foot_plant
    lift = (tau - STANCE_END) / (1.0 - STANCE_END)
    return style.foot_lift * math.sin(math.pi * lift)


def _left_arm_swing(tau: float, style: WalkStyle) -> tuple[BoneEuler, BoneEuler]:
    swing = style.arm_swing * math.cos(TAU * tau)
    follow = swing * style.forearm_follow_ratio
    upper = (-0.06 + swing, 0.0, 0.0)
    fore = (-0.20 + follow, 0.0, 0.0)
    return upper, fore
