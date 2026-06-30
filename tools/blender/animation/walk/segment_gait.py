"""Perry keyframe gait — legs, pelvis bob, trunk, neck, contralateral arms."""

from __future__ import annotations

import math

from tools.blender.animation.walk.biomech_curves import (
    ANKLE_CURVE_DEG,
    ARM_SWING_REFERENCE_DEG,
    ELBOW_CURVE_DEG,
    ELBOW_REFERENCE_DEG,
    HEAD_PITCH_DEG,
    HIP_CURVE_DEG,
    KNEE_CURVE_DEG,
    NECK_PITCH_DEG,
    PELVIS_VERTICAL_NORM,
    SHOULDER_CURVE_DEG,
    THORAX_PITCH_DEG,
)
from tools.blender.animation.walk.curves import sample_cycle_curve
from tools.blender.animation.walk.style import WalkStyle
from tools.blender.animation.walk.types import BoneEuler, GaitPose

DEG = math.pi / 180.0
ELBOW_PHASE_LAG = 0.05


def sample_segment_pose(frame: int, cycle_frames: int, style: WalkStyle) -> GaitPose:
    tau = (frame % cycle_frames) / cycle_frames
    tau_l = (tau + 0.5) % 1.0
    scale = style.angle_scale

    r_hip, l_hip = _legs(HIP_CURVE_DEG, tau, tau_l, scale)
    r_knee, l_knee = _legs(KNEE_CURVE_DEG, tau, tau_l, scale)
    r_foot, l_foot = _legs(ANKLE_CURVE_DEG, tau, tau_l, scale)
    l_foot = -l_foot

    root_y = sample_cycle_curve(PELVIS_VERTICAL_NORM, tau) * style.pelvis_vertical_m
    spine_x, spine1_x, neck_x, head_x = _trunk(tau, style)

    r_arm, r_fore = _arm(tau_l, style, style.arm_swing_r_scale, style.right_arm_hold)
    l_arm, l_fore = _arm(tau, style, 1.0, (0.0, 0.0, 0.0))

    return {
        "TBG_Root": (0.0, root_y, 0.0),
        "TBG_Spine": (spine_x, 0.0, 0.0),
        "TBG_Spine1": (spine1_x, 0.0, 0.0),
        "TBG_Neck": (neck_x, 0.0, 0.0),
        "TBG_Head": (head_x, 0.0, 0.0),
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


def _legs(
    curve: tuple,
    tau: float,
    tau_l: float,
    scale: float,
) -> tuple[float, float]:
    return (
        sample_cycle_curve(curve, tau) * DEG * scale,
        sample_cycle_curve(curve, tau_l) * DEG * scale,
    )


def _trunk(tau: float, style: WalkStyle) -> tuple[float, float, float, float]:
    thorax = sample_cycle_curve(THORAX_PITCH_DEG, tau) * DEG * style.torso_scale
    spine_x = thorax
    spine1_x = thorax * style.spine1_pitch_scale
    neck_x = sample_cycle_curve(NECK_PITCH_DEG, tau) * DEG * style.neck_scale
    head_x = sample_cycle_curve(HEAD_PITCH_DEG, tau) * DEG * style.head_scale
    return spine_x, spine1_x, neck_x, head_x


def _arm(
    leg_tau: float,
    style: WalkStyle,
    scale: float,
    offset: BoneEuler,
) -> tuple[BoneEuler, BoneEuler]:
    elbow_tau = (leg_tau - ELBOW_PHASE_LAG) % 1.0
    shoulder_ref = ARM_SWING_REFERENCE_DEG * DEG
    elbow_ref = ELBOW_REFERENCE_DEG * DEG
    swing = sample_cycle_curve(SHOULDER_CURVE_DEG, leg_tau) * DEG
    swing *= style.arm_swing / shoulder_ref * scale
    elbow = sample_cycle_curve(ELBOW_CURVE_DEG, elbow_tau) * DEG
    elbow *= style.arm_swing / elbow_ref * scale
    follow = elbow * style.forearm_follow_ratio
    upper = (-0.04 + swing + offset[0], offset[1], offset[2])
    fore = (-0.18 + follow + offset[0] * 0.2, offset[1] * 0.35, offset[2] * 0.2)
    return upper, fore
