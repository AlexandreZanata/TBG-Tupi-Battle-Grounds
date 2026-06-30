"""Keyframe poses for scout one-hand trident lunge thrust."""

from __future__ import annotations

from tools.blender.animation.shared.trident_guard import TRIDENT_GUARD_ARM
from tools.blender.animation.types import BonePose

# TBG_Root: (loc_x, loc_y, loc_z). Sagittal thrust — spine/neck Y and Z stay zero.


def guard_pose() -> BonePose:
    return {
        "TBG_Root": (0.0, 0.0, 0.0),
        "TBG_Spine": (0.0, 0.0, 0.0),
        "TBG_Spine1": (0.02, 0.0, 0.0),
        "TBG_Neck": (0.0, 0.0, 0.0),
        "TBG_Head": (0.0, 0.0, 0.0),
        "TBG_UpperLeg.L": (0.10, 0.0, 0.0),
        "TBG_LowerLeg.L": (0.06, 0.0, 0.0),
        "TBG_Foot.L": (0.02, 0.0, 0.0),
        "TBG_UpperLeg.R": (-0.06, 0.0, 0.0),
        "TBG_LowerLeg.R": (0.08, 0.0, 0.0),
        "TBG_Foot.R": (0.02, 0.0, 0.0),
        "TBG_UpperArm.L": (-0.10, 0.0, 0.0),
        "TBG_LowerArm.L": (0.30, 0.0, 0.0),
        **TRIDENT_GUARD_ARM,
    }


def windup_end_pose() -> BonePose:
    guard = guard_pose()
    return _merge(
        guard,
        {
            "TBG_Root": (0.0, 0.0, -0.02),
            "TBG_Spine": (-0.08, 0.0, 0.0),
            "TBG_Spine1": (-0.04, 0.0, 0.0),
            "TBG_Head": (0.04, 0.0, 0.0),
            "TBG_UpperLeg.R": (-0.14, 0.0, 0.0),
            "TBG_LowerLeg.R": (0.32, 0.0, 0.0),
            "TBG_Shoulder.R": (-0.06, 0.0, 0.0),
            "TBG_UpperArm.R": (-0.72, 0.0, 0.12),
            "TBG_LowerArm.R": (1.18, 0.0, 0.0),
            "TBG_Hand.R": (0.0, 0.0, 0.0),
        },
    )


def hit_peak_pose() -> BonePose:
    guard = guard_pose()
    return _merge(
        guard,
        {
            "TBG_Root": (0.0, -0.01, 0.05),
            "TBG_Spine": (0.14, 0.0, 0.0),
            "TBG_Spine1": (0.08, 0.0, 0.0),
            "TBG_Head": (0.02, 0.0, 0.0),
            "TBG_UpperLeg.L": (0.30, 0.0, 0.0),
            "TBG_LowerLeg.L": (0.48, 0.0, 0.0),
            "TBG_Foot.L": (-0.06, 0.0, 0.0),
            "TBG_UpperLeg.R": (-0.10, 0.0, 0.0),
            "TBG_LowerLeg.R": (0.16, 0.0, 0.0),
            "TBG_Foot.R": (-0.10, 0.0, 0.0),
            "TBG_Shoulder.R": (0.10, 0.0, 0.0),
            "TBG_UpperArm.R": (-0.02, 0.0, 0.10),
            "TBG_LowerArm.R": (0.18, 0.0, 0.0),
            "TBG_Hand.R": (-0.08, 0.0, 0.0),
            "TBG_UpperArm.L": (-0.22, 0.0, 0.0),
            "TBG_LowerArm.L": (0.40, 0.0, 0.0),
        },
    )


def recovery_end_pose() -> BonePose:
    return _lerp_pose(hit_peak_pose(), guard_pose(), 0.60)


def _merge(base: BonePose, overrides: BonePose) -> BonePose:
    out = dict(base)
    out.update(overrides)
    return out


def _lerp_pose(a: BonePose, b: BonePose, t: float) -> BonePose:
    keys = set(a) | set(b)
    out: BonePose = {}
    for key in keys:
        av = a.get(key, (0.0, 0.0, 0.0))
        bv = b.get(key, (0.0, 0.0, 0.0))
        out[key] = tuple(av[i] + (bv[i] - av[i]) * t for i in range(3))
    return out
