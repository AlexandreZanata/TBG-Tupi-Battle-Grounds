"""One-hand trident lunge thrust — uses shared kinetic chain."""

from __future__ import annotations

from dataclasses import dataclass

from tools.blender.animation.attack.chain import (
    ChainSegment,
    apply_bone_group,
    hit_segment_progress,
    lerp_pose,
    phase_progress,
    zero_torso_yaw_roll,
)
from tools.blender.animation.attack.constants import ATTACK_FPS, HIT_MS, RECOVERY_MS, WINDUP_MS
from tools.blender.animation.attack.thrust_poses import (
    TRIDENT_GUARD_ARM,
    guard_pose,
    hit_peak_pose,
    recovery_end_pose,
    windup_end_pose,
)
from tools.blender.animation.attack.timing import ms_to_frame
from tools.blender.animation.types import BonePose

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

SEG_LEGS = ChainSegment(0.05, 0.22)
SEG_TORSO = ChainSegment(0.08, 0.20)
SEG_SHOULDER = ChainSegment(0.11, 0.18)
SEG_ELBOW = ChainSegment(0.15, 0.16)
SEG_HAND = ChainSegment(0.19, 0.14)


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
    guard = guard_pose()

    if time_ms <= 0.0:
        return zero_torso_yaw_roll(guard, WEAPON_ARM)
    if time_ms < windup_end:
        p = phase_progress(time_ms, style.windup_ms)
        return zero_torso_yaw_roll(lerp_pose(guard, windup_end_pose(), p), WEAPON_ARM)
    if time_ms < hit_end:
        p = (time_ms - windup_end) / style.hit_ms
        return _hit_pose(p, style)
    if time_ms < total:
        p = phase_progress(time_ms - hit_end, style.recovery_ms)
        return zero_torso_yaw_roll(lerp_pose(hit_peak_pose(), recovery_end_pose(), p), WEAPON_ARM)
    return zero_torso_yaw_roll(guard, WEAPON_ARM)


def sample_thrust_pose_at_frame(frame: int, style: ThrustStyle) -> BonePose:
    return sample_thrust_pose_at_ms(frame / ATTACK_FPS * 1000.0, style)


def frame_count(style: ThrustStyle) -> int:
    total_ms = style.windup_ms + style.hit_ms + style.recovery_ms
    return ms_to_frame(total_ms)


def _hit_pose(progress: float, style: ThrustStyle) -> BonePose:
    start = windup_end_pose()
    end = hit_peak_pose()
    out = dict(start)
    apply_bone_group(out, start, end, LEG_BONES, hit_segment_progress(progress, SEG_LEGS) * 0.65)
    apply_bone_group(out, start, end, TORSO_BONES, hit_segment_progress(progress, SEG_TORSO))
    apply_bone_group(out, start, end, ("TBG_Shoulder.R", "TBG_UpperArm.R"), hit_segment_progress(progress, SEG_SHOULDER))
    apply_bone_group(out, start, end, ("TBG_LowerArm.R",), hit_segment_progress(progress, SEG_ELBOW))
    apply_bone_group(out, start, end, ("TBG_Hand.R",), hit_segment_progress(progress, SEG_HAND))
    apply_bone_group(out, start, end, SUPPORT_ARM, hit_segment_progress(progress, SEG_TORSO) * 0.5)

    root = start.get("TBG_Root", (0.0, 0.0, 0.0))
    end_root = end.get("TBG_Root", root)
    leg_p = hit_segment_progress(progress, SEG_LEGS) * 0.65
    rz = root[2] + (end_root[2] - root[2]) * leg_p * style.lunge_scale
    out["TBG_Root"] = (root[0], root[1], rz)
    return zero_torso_yaw_roll(out, WEAPON_ARM)
