"""Perry/Winter gait curves — % cycle → joint angle (degrees, sagittal plane).

Right-leg phase τ=0 is initial contact (heel strike). Left leg uses τ+0.5.
Source: BIOMECHANICAL-WALK-SPEC.md §2.
Calibrated: CMU subjects 35/39 via retarget pipeline (blend 0.25).
"""

from __future__ import annotations

from tools.blender.animation.walk.curves import Keyframe

# Hip: +flexion / -extension
HIP_CURVE_DEG: tuple[Keyframe, ...] = (
    (0.00, 30.0),
    (0.02, 29.9),
    (0.05, 29.6),
    (0.10, 29.3),
    (0.30, 5.8),
    (0.50, -12.8),
    (0.55, -11.4),
    (0.60, -2.7),
    (0.73, 6.5),
    (0.87, 20.4),
    (1.00, 26.8),
)

# Knee: +flexion — dual lobe (stance absorption + swing peak)
KNEE_CURVE_DEG: tuple[Keyframe, ...] = (
    (0.00, 0.0),
    (0.05, 14.0),
    (0.10, 16.3),
    (0.30, 6.5),
    (0.50, 2.2),
    (0.55, 2.0),
    (0.60, 33.1),
    (0.73, 45.8),
    (0.87, 23.5),
    (1.00, -2.8),
)

# Ankle: +dorsiflexion / -plantarflexion
ANKLE_CURVE_DEG: tuple[Keyframe, ...] = (
    (0.00, 0.0),
    (0.08, -6.0),
    (0.30, 5.3),
    (0.50, 8.8),
    (0.55, 8.9),
    (0.60, -13.8),
    (0.73, 0.3),
    (0.87, 0.1),
    (1.00, -0.5),
)

# Pelvis vertical bob — normalized 0..1; multiply by WalkStyle.pelvis_vertical_m
PELVIS_VERTICAL_NORM: tuple[Keyframe, ...] = (
    (0.00, 0.0),
    (0.25, 1.0),
    (0.50, 0.0),
    (0.75, 1.0),
    (1.00, 0.0),
)

# Thorax / neck / head pitch (degrees) — counter-phase for gaze stability
THORAX_PITCH_DEG: tuple[Keyframe, ...] = (
    (0.00, -2.0),
    (0.25, 2.0),
    (0.50, -2.0),
    (0.75, 2.0),
    (1.00, -2.0),
)

NECK_PITCH_DEG: tuple[Keyframe, ...] = (
    (0.00, 1.5),
    (0.25, -1.5),
    (0.50, 1.5),
    (0.75, -1.5),
    (1.00, 1.5),
)

HEAD_PITCH_DEG: tuple[Keyframe, ...] = (
    (0.00, 0.5),
    (0.25, -0.5),
    (0.50, 0.5),
    (0.75, -0.5),
    (1.00, 0.5),
)

# Shoulder / elbow (degrees) — sample at contralateral leg phase
SHOULDER_CURVE_DEG: tuple[Keyframe, ...] = (
    (0.00, -15.0),
    (0.50, 15.0),
    (1.00, -15.0),
)

ELBOW_CURVE_DEG: tuple[Keyframe, ...] = (
    (0.00, 35.0),
    (0.50, 20.0),
    (1.00, 35.0),
)

ARM_SWING_REFERENCE_DEG: float = 15.0
ELBOW_REFERENCE_DEG: float = 35.0
