"""Map Perry 8-phase spec → TBG FK GaitPose (study / blocky walk)."""

from __future__ import annotations

import math

from tools.blender.animation.walk.perry_eight_phase import (
    DEFAULT_GAIT_CONFIG,
    GaitCycleConfig,
    com_lateral_offset,
    com_vertical_offset,
    elbow_flex_rad,
    get_leg_angles,
    pelvis_roll_rad,
    pelvis_yaw_rad,
    shoulder_swing_rad,
)
from tools.blender.animation.walk.style import WalkStyle
from tools.blender.animation.walk.types import BoneEuler, GaitPose

DEG = math.pi / 180.0


def sample_eight_phase_pose(
    frame: int,
    cycle_frames: int,
    style: WalkStyle,
    *,
    config: GaitCycleConfig | None = None,
) -> GaitPose:
    cfg = config or DEFAULT_GAIT_CONFIG
    tau = (frame % cycle_frames) / cycle_frames
    scale = style.angle_scale * cfg.angle_scale

    r_hip, r_knee, r_ankle = get_leg_angles(tau, "R", cfg)
    l_hip, l_knee, l_ankle = get_leg_angles(tau, "L", cfg)

    root_y = com_vertical_offset(tau, cfg) * (style.pelvis_vertical_m / cfg.com_vertical_amplitude_m)
    root_x = com_lateral_offset(tau, cfg) * (style.pelvis_vertical_m / cfg.com_vertical_amplitude_m)

    spine_roll = pelvis_roll_rad(tau, cfg) * style.torso_scale
    spine_yaw = pelvis_yaw_rad(tau, cfg) * style.torso_scale

    r_sh = shoulder_swing_rad(tau, "R", cfg) * (style.arm_swing / math.radians(cfg.arm_swing_amplitude_deg))
    l_sh = shoulder_swing_rad(tau, "L", cfg) * (style.arm_swing / math.radians(cfg.arm_swing_amplitude_deg))
    elbow = elbow_flex_rad(tau, cfg)

    return {
        "TBG_Root": (root_x, root_y, 0.0),
        "TBG_Spine": (0.0, spine_roll, spine_yaw),
        "TBG_Spine1": (0.0, spine_roll * style.spine1_pitch_scale, 0.0),
        "TBG_Neck": (0.0, -spine_roll * 0.35, 0.0),
        "TBG_Head": (0.0, -spine_roll * 0.2, 0.0),
        "TBG_UpperLeg.R": (r_hip * DEG * scale, 0.0, 0.0),
        "TBG_LowerLeg.R": (r_knee * DEG * scale, 0.0, 0.0),
        "TBG_Foot.R": (r_ankle * DEG * scale, 0.0, 0.0),
        "TBG_UpperLeg.L": (l_hip * DEG * scale, 0.0, 0.0),
        "TBG_LowerLeg.L": (l_knee * DEG * scale, 0.0, 0.0),
        "TBG_Foot.L": (-l_ankle * DEG * scale, 0.0, 0.0),
        "TBG_UpperArm.R": (-0.04 + r_sh, 0.0, 0.0),
        "TBG_LowerArm.R": (-0.18 + elbow * 0.85, 0.0, 0.0),
        "TBG_UpperArm.L": (-0.04 + l_sh, 0.0, 0.0),
        "TBG_LowerArm.L": (-0.18 + elbow * 0.85, 0.0, 0.0),
    }
