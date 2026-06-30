"""Scout one-hand trident guard — shared by idle walk hold and attack start pose."""

from __future__ import annotations

from tools.blender.animation.types import BoneEuler, BonePose

# Sagittal only on weapon arm — no Y/Z on shoulder (prevents right lean).
TRIDENT_GUARD_ARM: dict[str, BoneEuler] = {
    "TBG_Shoulder.R": (0.0, 0.0, 0.0),
    "TBG_UpperArm.R": (-0.38, 0.0, 0.14),
    "TBG_LowerArm.R": (0.92, 0.0, 0.04),
    "TBG_Hand.R": (0.06, 0.0, 0.0),
}


def merge_guard_arm(pose: BonePose) -> BonePose:
    out = dict(pose)
    out.update(TRIDENT_GUARD_ARM)
    return out
