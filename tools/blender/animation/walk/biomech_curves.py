"""Perry/Winter gait curves — % cycle → joint angle (degrees, sagittal plane).

Right-leg phase τ=0 is initial contact (heel strike). Left leg uses τ+0.5.
Source: BIOMECHANICAL-WALK-SPEC.md §2.
"""

from __future__ import annotations

from tools.blender.animation.walk.curves import Keyframe

# Hip: +flexion / -extension
HIP_CURVE_DEG: tuple[Keyframe, ...] = (
    (0.00, 30.0),
    (0.02, 30.0),
    (0.10, 30.0),
    (0.30, 0.0),
    (0.50, -20.0),
    (0.55, -12.0),
    (0.60, 0.0),
    (0.73, 12.0),
    (0.87, 28.0),
    (1.00, 30.0),
)

# Knee: +flexion — dual lobe (stance absorption + swing peak)
KNEE_CURVE_DEG: tuple[Keyframe, ...] = (
    (0.00, 0.0),
    (0.05, 17.0),
    (0.10, 18.0),
    (0.30, 3.0),
    (0.50, 0.0),
    (0.60, 40.0),
    (0.73, 58.0),
    (0.87, 30.0),
    (1.00, 0.0),
)

# Ankle: +dorsiflexion / -plantarflexion
ANKLE_CURVE_DEG: tuple[Keyframe, ...] = (
    (0.00, 0.0),
    (0.08, -8.0),
    (0.30, 6.0),
    (0.50, 9.0),
    (0.60, -20.0),
    (0.73, -2.0),
    (0.87, 0.0),
    (1.00, 0.0),
)
