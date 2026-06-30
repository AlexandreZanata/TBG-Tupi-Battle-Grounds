"""Backward-compatible shim — prefer tools.blender.animation.walk.profiles."""

from tools.blender.animation.walk.profiles import (  # noqa: F401
    GAIT_PROFILES,
    STYLE_PROFILES,
    UNIT_WALK_PROFILE,
    WALK_PROFILES,
    get_walk_style,
    resolve_walk_profile,
)
from tools.blender.animation.walk.api import (  # noqa: F401
    WALK_FRAMES,
    WALK_FPS,
    root_bob_y,
    sample_pose_at_frame,
)
