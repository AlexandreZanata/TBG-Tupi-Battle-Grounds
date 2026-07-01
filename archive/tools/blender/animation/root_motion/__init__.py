"""Root motion package — forward walk on TBG_Root."""

from tools.blender.animation.root_motion.bake import (
    WALK_ROOT_CLIP,
    WALK_ROOT_MOTION_CLIP,
    clip_name_for_root_motion,
    merge_root_forward,
)
from tools.blender.animation.root_motion.contacts import (
    STANCE_END_TAU,
    heel_strike_tau,
    stance_phase_frames,
    toe_off_tau,
)
from tools.blender.animation.root_motion.stride import (
    DEFAULT_STRIDE_LENGTH_M,
    root_forward_z,
    walk_velocity_mps,
)

__all__ = [
    "DEFAULT_STRIDE_LENGTH_M",
    "STANCE_END_TAU",
    "WALK_ROOT_CLIP",
    "WALK_ROOT_MOTION_CLIP",
    "clip_name_for_root_motion",
    "heel_strike_tau",
    "merge_root_forward",
    "root_forward_z",
    "stance_phase_frames",
    "toe_off_tau",
    "walk_velocity_mps",
]
