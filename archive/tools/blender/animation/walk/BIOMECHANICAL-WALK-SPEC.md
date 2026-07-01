# Biomechanical Human Walk — Implementation Reference

> **Implementation:** Perry segment gait in `segment_gait.py` (Catmull-Rom on `biomech_curves.py`).  
> Legacy cosine gait: `symmetric_gait.py` (reference only).

## Implemented mapping (TBG FK rig)

| Spec channel | TBG bone | Axis |
|--------------|----------|------|
| Hip flex/ext | `TBG_UpperLeg.*` | X |
| Knee flex | `TBG_LowerLeg.*` | X |
| Ankle dorsi/plantar | `TBG_Foot.*` | X |
| Pelvis vertical (2× bob) | `TBG_Root` | location Y |
| Torso pitch | `TBG_Spine` / `TBG_Spine1` | X rotation only |
| Neck / head gaze stabilize | `TBG_Neck` / `TBG_Head` | X rotation only |
| Arm swing (contralateral) | `TBG_UpperArm.*` / `TBG_LowerArm.*` | X (+ unit hold offset) |

**Do not** apply pelvis yaw/roll on `TBG_Spine` for FK rigs — causes diagonal lean.

## Timing (§4)

- Default cadence: **110 spm** → **33 frames** @ 30 fps  
- `cycle_frames(cadence) = round(120 / cadence * fps)`  
- `velocity (m/s) = strideLength × cadence / 120` (use when adding root motion)

## Curve data

Joint angles vs % cycle: `biomech_curves.py` (`HIP_CURVE_DEG`, `KNEE_CURVE_DEG`, `ANKLE_CURVE_DEG`).  
Interpolation: cyclic Catmull-Rom in `curves.py`.

## Validation checklist (§10)

| Check | Automated test |
|-------|----------------|
| Dual knee lobe | `test_biomech.py::test_knee_curve_has_stance_and_swing_lobes` |
| Pelvis 2× bob | `test_biomech.py::test_pelvis_vertical_double_support_is_lowest` |
| Leg alternation | `test_contracts.py::test_gait_alternates_legs_at_contact_points` |
| Loop closure | `test_contracts.py::test_gait_loops_at_frame_zero_and_end` |
| FK legs X-only | `test_sampler.py::test_legs_rotate_on_x_axis_only` |
| Foot slide (in-place) | N/A — root XZ fixed; root motion is future work |

## Tuning

Edit `WalkStyle` in `style.py` (cadence, `angle_scale`, pelvis amplitudes, arm swing).  
Refine curve keyframes in `biomech_curves.py` against CMU mocap or studio preview.

## Not yet implemented (Level 3)

- Foot IK + raycast terrain
- COM pendulum / support polygon
- Runtime velocity scaling (cadence + stride together)
