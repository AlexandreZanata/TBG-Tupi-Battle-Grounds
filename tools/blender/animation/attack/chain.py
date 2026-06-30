"""Kinetic chain — staggered segment delays during hit phase."""

from __future__ import annotations

from dataclasses import dataclass

from tools.blender.animation.easing import clamp01, ease_in_expo, ease_out
from tools.blender.animation.types import BoneEuler, BonePose

TORSO_YAW_ROLL_KEYS = ("TBG_Spine", "TBG_Spine1", "TBG_Neck", "TBG_Head")


@dataclass(frozen=True)
class ChainSegment:
    delay: float
    span: float


def hit_segment_progress(hit_progress: float, segment: ChainSegment, phase_shift: float = 0.0) -> float:
    start = segment.delay + phase_shift
    if hit_progress < start:
        return 0.0
    return ease_in_expo(clamp01((hit_progress - start) / segment.span))


def phase_progress(elapsed_ms: float, phase_ms: int, *, ease: str = "out") -> float:
    if phase_ms <= 0:
        return 1.0
    raw = clamp01(elapsed_ms / phase_ms)
    return ease_out(raw) if ease == "out" else ease_in_expo(raw)


def lerp_euler(a: BoneEuler | None, b: BoneEuler | None, t: float) -> BoneEuler:
    av = a or (0.0, 0.0, 0.0)
    bv = b or (0.0, 0.0, 0.0)
    return tuple(av[i] + (bv[i] - av[i]) * t for i in range(3))


def lerp_pose(a: BonePose, b: BonePose, t: float) -> BonePose:
    keys = set(a) | set(b)
    return {key: lerp_euler(a.get(key), b.get(key), t) for key in keys}


def apply_bone_group(
    out: BonePose,
    start: BonePose,
    end: BonePose,
    keys: tuple[str, ...],
    progress: float,
) -> None:
    for key in keys:
        out[key] = lerp_euler(start.get(key), end.get(key), progress)


def zero_torso_yaw_roll(pose: BonePose, weapon_arm: tuple[str, ...] = ()) -> BonePose:
    out = dict(pose)
    for key in TORSO_YAW_ROLL_KEYS:
        if key in out:
            x, _, _ = out[key]
            out[key] = (x, 0.0, 0.0)
    for key in weapon_arm:
        if key in out:
            x, _, z = out[key]
            out[key] = (x, 0.0, z)
    return out
