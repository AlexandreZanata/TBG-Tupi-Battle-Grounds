"""Merge root forward translation into walk poses (no bpy)."""

from __future__ import annotations

from tools.blender.animation.root_motion.stride import DEFAULT_STRIDE_LENGTH_M, root_location_yz
from tools.blender.animation.walk.types import GaitPose

WALK_ROOT_CLIP = "Walk"
WALK_ROOT_MOTION_CLIP = "WalkRoot"


def clip_name_for_root_motion(root_motion: bool) -> str:
    return WALK_ROOT_MOTION_CLIP if root_motion else WALK_ROOT_CLIP


def merge_root_forward(
    pose: GaitPose,
    frame: int,
    cycle_frames: int,
    stride_length_m: float = DEFAULT_STRIDE_LENGTH_M,
) -> GaitPose:
    out = dict(pose)
    _x, pelvis_y, _z = out.get("TBG_Root", (0.0, 0.0, 0.0))
    y, z = root_location_yz(frame, cycle_frames, pelvis_y, stride_length_m)
    out["TBG_Root"] = (0.0, y, z)
    return out
