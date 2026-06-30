"""One-hand trident lunge thrust — arm-led extension, subtle lunge, no torso yaw."""

from __future__ import annotations

from dataclasses import dataclass

from tools.blender.animation.attack.constants import (
    ATTACK_FPS,
    HIT_MS,
    RECOVERY_MS,
    WINDUP_MS,
    ms_to_frame,
)
from tools.blender.animation.attack.thrust_poses import (
    guard_pose,
    hit_peak_pose,
    recovery_end_pose,
    windup_end_pose,
    _lerp_pose,
)
from tools.blender.animation.easing import clamp01, ease_in_expo, ease_out
from tools.blender.animation.types import BoneEuler, BonePose

WEAPON_ARM = (
    "TBG_Shoulder.R",
    "TBG_UpperArm.R",
    "TBG_LowerArm.R",
    "TBG_Hand.R",
)
LEG_BONES = (
    "TBG_UpperLeg.L",
    "TBG_LowerLeg.L",
    "TBG_Foot.L",
    "TBG_UpperLeg.R",
    "TBG_LowerLeg.R",
    "TBG_Foot.R",
)
TORSO_BONES = ("TBG_Spine", "TBG_Spine1", "TBG_Neck", "TBG_Head")
SUPPORT_ARM = ("TBG_UpperArm.L", "TBG_LowerArm.L")


@dataclass(frozen=True)
class ThrustStyle:
    windup_ms: int = WINDUP_MS
    hit_ms: int = HIT_MS
    recovery_ms: int = RECOVERY_MS
    lunge_scale: float = 1.0


def sample_thrust_pose_at_ms(time_ms: float, style: ThrustStyle) -> BonePose:
    windup_end = style.windup_ms
    hit_end = windup_end + style.hit_ms
    total = windup_end + style.hit_ms + style.recovery_ms

    if time_ms <= 0.0:
        return _zero_torso_yaw_roll(guard_pose())
    if time_ms < windup_end:
        p = ease_out(time_ms / windup_end)
        return _zero_torso_yaw_roll(_lerp_pose(guard_pose(), windup_end_pose(), p))
    if time_ms < hit_end:
        p = (time_ms - windup_end) / style.hit_ms
        return _hit_pose(p, style)
    if time_ms < total:
        p = ease_out((time_ms - hit_end) / style.recovery_ms)
        return _zero_torso_yaw_roll(_lerp_pose(hit_peak_pose(), recovery_end_pose(), p))
    return _zero_torso_yaw_roll(guard_pose())


def sample_thrust_pose_at_frame(frame: int, style: ThrustStyle) -> BonePose:
    time_ms = frame / ATTACK_FPS * 1000.0
    return sample_thrust_pose_at_ms(time_ms, style)


def frame_count(style: ThrustStyle) -> int:
    total_ms = style.windup_ms + style.hit_ms + style.recovery_ms
    return ms_to_frame(total_ms)


def _hit_pose(progress: float, style: ThrustStyle) -> BonePose:
    start = windup_end_pose()
    end = hit_peak_pose()
    leg_p = ease_in_expo(clamp01(progress / 0.55)) * 0.65
    torso_p = ease_in_expo(clamp01((progress - 0.15) / 0.50))
    shoulder_p = ease_in_expo(clamp01((progress - 0.30) / 0.45))
    elbow_p = ease_in_expo(clamp01((progress - 0.42) / 0.38))
    hand_p = ease_in_expo(clamp01((progress - 0.55) / 0.30))

    out = dict(start)
    for key in LEG_BONES:
        out[key] = _lerp_euler(start.get(key), end.get(key), leg_p)
    for key in TORSO_BONES:
        out[key] = _lerp_euler(start.get(key), end.get(key), torso_p)
    out["TBG_Shoulder.R"] = _lerp_euler(start.get("TBG_Shoulder.R"), end.get("TBG_Shoulder.R"), shoulder_p)
    out["TBG_UpperArm.R"] = _lerp_euler(start.get("TBG_UpperArm.R"), end.get("TBG_UpperArm.R"), shoulder_p)
    out["TBG_LowerArm.R"] = _lerp_euler(start.get("TBG_LowerArm.R"), end.get("TBG_LowerArm.R"), elbow_p)
    out["TBG_Hand.R"] = _lerp_euler(start.get("TBG_Hand.R"), end.get("TBG_Hand.R"), hand_p)
    for key in SUPPORT_ARM:
        out[key] = _lerp_euler(start.get(key), end.get(key), torso_p * 0.5)

    root = _lerp_euler(start.get("TBG_Root"), end.get("TBG_Root"), leg_p)
    out["TBG_Root"] = (root[0], root[1], root[2] * style.lunge_scale)
    return _zero_torso_yaw_roll(out)


def _zero_torso_yaw_roll(pose: BonePose) -> BonePose:
    out = dict(pose)
    for key in ("TBG_Spine", "TBG_Spine1", "TBG_Neck", "TBG_Head"):
        if key in out:
            x, _, _ = out[key]
            out[key] = (x, 0.0, 0.0)
    for key in WEAPON_ARM:
        if key in out:
            x, _, z = out[key]
            out[key] = (x, 0.0, z)
    return out


def _lerp_euler(a: BoneEuler | None, b: BoneEuler | None, t: float) -> BoneEuler:
    av = a or (0.0, 0.0, 0.0)
    bv = b or (0.0, 0.0, 0.0)
    return tuple(av[i] + (bv[i] - av[i]) * t for i in range(3))
