"""Keyframe poses for heavy melee overhead swing."""

from __future__ import annotations

from tools.blender.animation.attack.chain import lerp_pose
from tools.blender.animation.types import BonePose

WEAPON_ARM = (
    "TBG_Shoulder.R",
    "TBG_UpperArm.R",
    "TBG_LowerArm.R",
    "TBG_Hand.R",
)


def guard_pose() -> BonePose:
    return {
        "TBG_Root": (0.0, 0.0, 0.0),
        "TBG_Spine": (0.02, 0.0, 0.0),
        "TBG_Spine1": (0.04, 0.0, 0.0),
        "TBG_Neck": (0.0, 0.0, 0.0),
        "TBG_Head": (0.0, 0.0, 0.0),
        "TBG_UpperLeg.L": (0.14, 0.0, 0.0),
        "TBG_LowerLeg.L": (0.10, 0.0, 0.0),
        "TBG_Foot.L": (0.02, 0.0, 0.0),
        "TBG_UpperLeg.R": (-0.08, 0.0, 0.0),
        "TBG_LowerLeg.R": (0.12, 0.0, 0.0),
        "TBG_Foot.R": (0.02, 0.0, 0.0),
        "TBG_UpperArm.L": (-0.12, 0.0, 0.0),
        "TBG_LowerArm.L": (0.35, 0.0, 0.0),
        "TBG_Shoulder.R": (0.0, 0.0, 0.0),
        "TBG_UpperArm.R": (-0.42, 0.0, 0.08),
        "TBG_LowerArm.R": (0.55, 0.0, 0.0),
        "TBG_Hand.R": (0.0, 0.0, 0.0),
    }


def windup_end_pose() -> BonePose:
    base = guard_pose()
    overrides: BonePose = {
        "TBG_Root": (0.0, 0.0, -0.03),
        "TBG_Spine": (-0.12, 0.0, 0.0),
        "TBG_Spine1": (-0.08, 0.0, 0.0),
        "TBG_UpperLeg.R": (-0.18, 0.0, 0.0),
        "TBG_LowerLeg.R": (0.28, 0.0, 0.0),
        "TBG_Shoulder.R": (-0.14, 0.0, 0.0),
        "TBG_UpperArm.R": (-1.05, 0.0, 0.10),
        "TBG_LowerArm.R": (0.95, 0.0, 0.0),
    }
    return {**base, **overrides}


def windup_alt_pose() -> BonePose:
    """Attack2 entry — phase-shifted coil (left weight)."""
    return lerp_pose(guard_pose(), windup_end_pose(), 0.55)


def hit_peak_pose() -> BonePose:
    base = guard_pose()
    overrides: BonePose = {
        "TBG_Root": (0.0, -0.01, 0.04),
        "TBG_Spine": (0.18, 0.0, 0.0),
        "TBG_Spine1": (0.10, 0.0, 0.0),
        "TBG_UpperLeg.L": (0.32, 0.0, 0.0),
        "TBG_LowerLeg.L": (0.42, 0.0, 0.0),
        "TBG_Foot.L": (-0.04, 0.0, 0.0),
        "TBG_UpperLeg.R": (-0.06, 0.0, 0.0),
        "TBG_LowerLeg.R": (0.20, 0.0, 0.0),
        "TBG_Foot.R": (-0.08, 0.0, 0.0),
        "TBG_Shoulder.R": (0.12, 0.0, 0.0),
        "TBG_UpperArm.R": (0.38, 0.0, 0.06),
        "TBG_LowerArm.R": (0.22, 0.0, 0.0),
        "TBG_Hand.R": (-0.06, 0.0, 0.0),
        "TBG_UpperArm.L": (-0.28, 0.0, 0.0),
        "TBG_LowerArm.L": (0.45, 0.0, 0.0),
    }
    return {**base, **overrides}


def recovery_end_pose() -> BonePose:
    return lerp_pose(hit_peak_pose(), guard_pose(), 0.65)
