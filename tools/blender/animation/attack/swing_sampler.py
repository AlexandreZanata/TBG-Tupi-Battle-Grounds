"""Heavy melee overhead swing — tank_melee_v1 timings."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

from tools.blender.animation.attack.chain import (
    ChainSegment,
    apply_bone_group,
    hit_segment_progress,
    lerp_pose,
    phase_progress,
    zero_torso_yaw_roll,
)
from tools.blender.animation.attack.constants import ATTACK_FPS
from tools.blender.animation.attack.swing_poses import (
    WEAPON_ARM,
    guard_pose,
    hit_peak_pose,
    recovery_end_pose,
    windup_alt_pose,
    windup_end_pose,
)
from tools.blender.animation.attack.timing import AttackTiming, frame_count, ms_to_frame
from tools.blender.animation.types import BonePose

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

# Short spans — visual peak near tank_melee_v1 attackHitMs (120 ms into hit phase).
SEG_LEGS = ChainSegment(0.0, 0.18)
SEG_TORSO = ChainSegment(0.02, 0.16)
SEG_SHOULDER = ChainSegment(0.04, 0.14)
SEG_ELBOW = ChainSegment(0.08, 0.12)

SwingVariant = Literal["primary", "alternate"]


@dataclass(frozen=True)
class SwingStyle:
    timing: AttackTiming
    variant: SwingVariant = "primary"
    phase_shift: float = 0.0
    power_scale: float = 1.0


def sample_swing_pose_at_ms(time_ms: float, style: SwingStyle) -> BonePose:
    timing = style.timing
    windup_end = timing.windup_ms
    hit_end = windup_end + timing.hit_ms
    total = timing.total_ms
    guard = guard_pose()

    if time_ms <= 0.0:
        return zero_torso_yaw_roll(guard, WEAPON_ARM)
    if time_ms < windup_end:
        start = windup_alt_pose() if style.variant == "alternate" else guard
        end = windup_end_pose()
        p = phase_progress(time_ms, timing.windup_ms)
        return zero_torso_yaw_roll(lerp_pose(start, end, p), WEAPON_ARM)
    if time_ms < hit_end:
        p = (time_ms - windup_end) / timing.hit_ms
        return _hit_pose(p, style)
    if time_ms < total:
        p = phase_progress(time_ms - hit_end, timing.recovery_ms)
        return zero_torso_yaw_roll(lerp_pose(hit_peak_pose(), recovery_end_pose(), p), WEAPON_ARM)
    return zero_torso_yaw_roll(guard, WEAPON_ARM)


def sample_swing_pose_at_frame(frame: int, style: SwingStyle) -> BonePose:
    return sample_swing_pose_at_ms(frame / ATTACK_FPS * 1000.0, style)


def swing_frame_count(style: SwingStyle) -> int:
    return frame_count(style.timing)


def _hit_pose(progress: float, style: SwingStyle) -> BonePose:
    start = windup_end_pose()
    end = hit_peak_pose()
    shift = style.phase_shift
    leg_p = hit_segment_progress(progress, SEG_LEGS, shift) * style.power_scale
    torso_p = hit_segment_progress(progress, SEG_TORSO, shift)
    shoulder_p = hit_segment_progress(progress, SEG_SHOULDER, shift)
    elbow_p = hit_segment_progress(progress, SEG_ELBOW, shift)

    out = dict(start)
    apply_bone_group(out, start, end, LEG_BONES, leg_p)
    apply_bone_group(out, start, end, TORSO_BONES, torso_p)
    apply_bone_group(out, start, end, ("TBG_Shoulder.R", "TBG_UpperArm.R"), shoulder_p)
    apply_bone_group(out, start, end, ("TBG_LowerArm.R",), elbow_p)
    apply_bone_group(out, start, end, ("TBG_Hand.R",), elbow_p)
    apply_bone_group(out, start, end, SUPPORT_ARM, torso_p * 0.45)

    root = start.get("TBG_Root", (0.0, 0.0, 0.0))
    end_root = end.get("TBG_Root", root)
    out["TBG_Root"] = tuple(root[i] + (end_root[i] - root[i]) * leg_p for i in range(3))
    return zero_torso_yaw_roll(out, WEAPON_ARM)
