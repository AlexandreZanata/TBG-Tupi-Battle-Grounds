"""Parametric in-place walk sampler — pure Python, no bpy."""

from __future__ import annotations

import math

from tools.blender.animation.walk.style import WalkStyle
from tools.blender.animation.walk.types import BoneEuler, GaitPose

TAU = 2.0 * math.pi


def sample_walk_pose(frame: int, cycle_frames: int, style: WalkStyle) -> GaitPose:
    phase = TAU * (frame % cycle_frames) / cycle_frames
    bob = 2.0 * phase
    r_upper, r_knee, r_foot = _leg_chain(phase, style)
    l_upper, l_knee, l_foot = _leg_chain(phase + math.pi, style)
    pelvis_y = style.hip_bob * math.sin(bob)
    spine_x = style.spine_pitch * math.sin(bob + math.pi * 0.2)
    spine_z = style.spine_roll * math.sin(phase)
    r_arm, r_fore = _arm_chain(phase, style, -1.0, style.arm_swing_r_scale, style.right_arm_hold)
    l_arm, l_fore = _arm_chain(phase, style, 1.0, 1.0, (0.0, 0.0, 0.0))
    return {
        "TBG_Root": (0.0, pelvis_y, 0.0),
        "TBG_Spine": (spine_x, 0.0, spine_z),
        "TBG_Spine1": (spine_x * style.spine1_pitch_scale, 0.0, spine_z * 0.35),
        "TBG_UpperLeg.R": (r_upper, 0.0, 0.0),
        "TBG_LowerLeg.R": (r_knee, 0.0, 0.0),
        "TBG_Foot.R": (r_foot, 0.0, 0.0),
        "TBG_UpperLeg.L": (l_upper, 0.0, 0.0),
        "TBG_LowerLeg.L": (l_knee, 0.0, 0.0),
        "TBG_Foot.L": (l_foot, 0.0, 0.0),
        "TBG_UpperArm.R": r_arm,
        "TBG_LowerArm.R": r_fore,
        "TBG_UpperArm.L": l_arm,
        "TBG_LowerArm.L": l_fore,
    }


def _leg_chain(phase: float, style: WalkStyle) -> tuple[float, float, float]:
    cos_p = math.cos(phase)
    sin_p = math.sin(phase)
    upper = style.stride * cos_p
    lift = max(0.0, sin_p)
    plant = max(0.0, -sin_p)
    knee = style.knee_support + style.knee_swing * lift + 0.07 * plant
    foot = style.foot_lift * (lift * 0.85 - plant * 0.3)
    return upper, knee, foot


def _arm_chain(
    phase: float,
    style: WalkStyle,
    sign: float,
    scale: float,
    offset: BoneEuler,
) -> tuple[BoneEuler, BoneEuler]:
    swing = sign * style.arm_swing * scale * math.cos(phase)
    follow = sign * style.forearm_follow * scale * math.cos(phase) * 0.6
    upper = (-0.06 + swing + offset[0], offset[1], 0.1 * sign + offset[2])
    fore = (-0.22 + follow + offset[0] * 0.25, offset[1] * 0.4, offset[2] * 0.25)
    return upper, fore
