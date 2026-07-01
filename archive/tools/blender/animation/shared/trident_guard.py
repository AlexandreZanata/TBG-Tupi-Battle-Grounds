"""Scout one-hand trident guard — shared by idle walk hold and attack start pose."""

from __future__ import annotations

from tools.blender.animation.types import BoneEuler, BonePose

# Sagittal only on weapon arm — no Y on shoulder (prevents right lean).
TRIDENT_GUARD_ARM: dict[str, BoneEuler] = {
    "TBG_Shoulder.R": (0.0, 0.0, 0.0),
    "TBG_UpperArm.R": (-0.38, 0.0, 0.14),
    "TBG_LowerArm.R": (0.92, 0.0, 0.04),
    "TBG_Hand.R": (0.06, 0.0, 0.0),
}

_WEAPON_KEYS = tuple(TRIDENT_GUARD_ARM.keys())
_COUNTERSPINE_PITCH = -0.028


def merge_guard_arm(pose: BonePose) -> BonePose:
    from tools.blender.animation.attack.chain import zero_torso_yaw_roll

    out = dict(pose)
    out.update(TRIDENT_GUARD_ARM)
    sx, _, sz = out.get("TBG_Spine", (0.0, 0.0, 0.0))
    s1x = out.get("TBG_Spine1", (0.0, 0.0, 0.0))[0]
    out["TBG_Spine"] = (sx + _COUNTERSPINE_PITCH, 0.0, sz)
    out["TBG_Spine1"] = (s1x + _COUNTERSPINE_PITCH * 0.5, 0.0, 0.0)
    return zero_torso_yaw_roll(out, _WEAPON_KEYS)
