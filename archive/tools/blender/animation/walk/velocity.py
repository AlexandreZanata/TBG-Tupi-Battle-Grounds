"""Runtime walk velocity scaling — cadence + stride from lane move speed."""

from __future__ import annotations

import math
from dataclasses import replace

from tools.blender.animation.root_motion.stride import walk_velocity_mps
from tools.blender.animation.walk.style import WalkStyle

MIN_SCALE = 0.25
MAX_SCALE = 2.0


def scale_walk_style_for_speed(
    style: WalkStyle,
    move_speed_mps: float,
    *,
    baseline_mps: float | None = None,
) -> WalkStyle:
    """Scale cadence and stride so walk velocity matches ``move_speed_mps``.

    ``velocity = stride × cadence / 120`` — both scale by ``√ratio`` for linear speed.
    Full animator blend trees deferred post-MVP; this is the authoring/runtime contract.
    """
    base = baseline_mps if baseline_mps is not None else walk_velocity_mps(
        style.stride_length_m,
        style.cadence_spm,
    )
    if base <= 0.0 or move_speed_mps <= 0.0:
        return style

    ratio = max(MIN_SCALE, min(MAX_SCALE, move_speed_mps / base))
    factor = math.sqrt(ratio)
    return replace(
        style,
        cadence_spm=style.cadence_spm * factor,
        stride_length_m=style.stride_length_m * factor,
    )


def scaled_walk_velocity_mps(style: WalkStyle) -> float:
    return walk_velocity_mps(style.stride_length_m, style.cadence_spm)
