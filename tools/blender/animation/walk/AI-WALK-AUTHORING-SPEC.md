# AI Walk Authoring Spec — Biomechanical Control-Point Format

> **Audience:** coding agents implementing procedural walk for TBG FK armatures (`TBG_*` bones) exported as GLB clips.  
> **Goal:** describe human walking in **physics-first, segment-keyframe language** that converts directly to sampler code — not artistic prose.

---

## 1. Why simple sine waves look wrong

Current `symmetric_gait.py` drives hips/knees with one cosine per leg. Real gait is **not** a single harmonic:

| Missing signal | Biomechanical effect |
|----------------|-------------------|
| Dual-lobe knee curve | Stance absorption (~5–10% cycle) + swing flexion peak (~73% cycle) |
| Ankle dorsiflex/plantar phases | Foot rollover and toe-off |
| Pelvis list + vertical bob (2× per cycle) | Weight transfer; lowest at double support |
| Thorax counter-rotation vs pelvis | COM stability; head stays visually forward |
| Shoulder vs elbow phase split | Arm swing is two-segment, not one angle |
| Stance/swing timing (~60/40) | Perry stance ends ~62%; swing is shorter |

**Agent rule:** NEVER drive all joints from `cos(2πτ)` with one amplitude. Use **per-segment keyframe tables** indexed by normalized gait cycle `τ`.

---

## 2. Authoritative sources (load before authoring)

| Source | What to extract | URL / citation |
|--------|-----------------|----------------|
| **Perry & Burnfield** — *Gait Analysis: Normal and Pathological Function* (2nd ed., 2010) | 8 gait phases; hip/knee/ankle timing; trunk & arm chapters | [SLACK/Routledge](https://www.routledge.com/Gait-Analysis-Normal-and-Pathological-Function/Perry-Burnfield/p/book/9781556427664) |
| **AAPMR KnowledgeNow** — Biomechanics of Normal Gait | Stance/swing %; phase boundaries in plain language | [now.aapmr.org/biomechanics-normal-gait](https://now.aapmr.org/biomechanics-normal-gait/) |
| **Rancho Los Amigos OGA** | Systematic joint-by-joint observation through phases | Perry Pathokinesiology Lab method |
| **Stokes et al. (1989)** — pelvis/thorax rotation | Transverse counter-rotation; speed-invariant breakpoints | *Journal of Biomechanics* 22(1):43–50 |
| **Miyazaki et al. / trunk-pelvis coupling** | Pelvis leads thorax ~15% of cycle in transverse plane | *J NeuroEngineering Rehabil* 7:9 |
| **Roblox animation docs** | 4-pose game cycle (Contact / Low / Passing / High) | [create.roblox.com — walk cycle key poses](https://create.roblox.com/docs/en-us/tutorials/use-case-tutorials/animation/create-an-animation) |
| **Johansen Locomotion System** (optional runtime) | `foot-strike`, `foot-off`, `foot-land` keytimes; IK foot placement | [runevision.com thesis](https://runevision.com/thesis/rune_skovbo_johansen_thesis.pdf) |

**Clinical standard:** Perry/Rancho phase percentages. **Game production:** 4-pose cycle + Perry joint curves. **Runtime terrain:** Locomotion System keytimes + IK (Level 3 — not required for in-place bake).

---

## 3. Normalized time — the single clock

Define **gait cycle** `τ ∈ [0.0, 1.0)`:

- `τ = 0.0` → **right foot initial contact** (heel strike reference limb)
- `τ = 0.5` → opposite (left) foot initial contact
- `τ = 1.0` ≡ `τ = 0.0` (loop closure required)

Convert to frames:

```
cycle_frames = round(120 / cadence_spm * fps)
frame_at_τ     = floor(τ * cycle_frames) % cycle_frames
time_ms          = τ * (60_000 / cadence_spm)
```

Default TBG contract: **110 spm**, **30 fps** → **33 frames** (~1.09 s).

---

## 4. Body control points — segment hierarchy

Treat the body as **linked rigid segments**. Each segment exposes 1–3 rotation DOFs. Author motion as **angle vs τ** per DOF, not as mesh deformation.

### 4.1 Segment tree (proximal → distal)

```
ROOT (pelvis)           — vertical translation + optional yaw (travel only)
├── LUMBAR (TBG_Spine)  — pitch, yaw*, roll*
├── THORAX (TBG_Spine1) — pitch (split from lumbar for counter-motion)
│   ├── NECK (TBG_Neck) — pitch, yaw*, roll*
│   │   └── HEAD (TBG_Head) — pitch (gaze stabilization)
│   ├── CLAVICLE.L/R (TBG_Shoulder.*) — shrug/raise (usually minimal in walk)
│   │   ├── UPPER_ARM.*
│   │   │   └── LOWER_ARM.*
│   │   │       └── HAND.* (weapon hold offset — static or reduced swing)
│   └── (mirror R)
├── HIP.L → UPPER_LEG.L → LOWER_LEG.L → FOOT.L
└── HIP.R → UPPER_LEG.R → LOWER_LEG.R → FOOT.R
```

`*` **TBG FK rig constraint:** keep `TBG_Spine` / `TBG_Spine1` **yaw = 0, roll = 0** for in-place walk. Express pelvis transverse rotation as **small differential pitch split** between `Spine` and `Spine1`, or omit until root-yaw travel is added. Applying spine yaw/roll on FK rigs causes diagonal lean (verified in `test_symmetric_gait.py`).

### 4.2 TBG bone mapping table

| Control point ID | TBG bone | Primary axis | Angle sign convention |
|----------------|----------|--------------|------------------------|
| `root_vertical` | `TBG_Root` | location Y | +up |
| `pelvis_pitch` | `TBG_Spine` | rot X | +forward lean |
| `thorax_pitch` | `TBG_Spine1` | rot X | +forward lean (counter-phase vs pelvis) |
| `neck_pitch` | `TBG_Neck` | rot X | −counter-lean vs thorax (gaze level) |
| `head_pitch` | `TBG_Head` | rot X | fine gaze (±3°) |
| `hip_flex_R` | `TBG_UpperLeg.R` | rot X | +flexion forward |
| `hip_flex_L` | `TBG_UpperLeg.L` | rot X | **mirror:** `L = −R` at same τ (sagittal) |
| `knee_flex_R` | `TBG_LowerLeg.R` | rot X | +flexion |
| `knee_flex_L` | `TBG_LowerLeg.L` | rot X | same phase as L hip, not R |
| `ankle_R` | `TBG_Foot.R` | rot X | +dorsiflex / −plantarflex |
| `ankle_L` | `TBG_Foot.L` | rot X | mirror phase `τ+0.5` |
| `shoulder_R` | `TBG_UpperArm.R` | rot X primary | contralateral to R leg |
| `elbow_R` | `TBG_LowerArm.R` | rot X | follows shoulder with phase lag |
| `shoulder_L` | `TBG_UpperArm.L` | rot X | contralateral to L leg |
| `elbow_L` | `TBG_LowerArm.L` | rot X | follows shoulder |

**Leg phase offset:** right leg uses `τ`; left leg uses `τ_L = (τ + 0.5) mod 1.0`.

---

## 5. Gait phases — two compatible labels

Agents MAY use either label set; MUST keep **consistent τ boundaries**.

### 5.1 Perry stance / swing (reference limb = right)

| Phase | τ range | % cycle | Function |
|-------|---------|---------|----------|
| Initial contact | 0.00–0.02 | 0–2% | Heel touch; knee extends |
| Loading response | 0.02–0.10 | 2–10% | Weight accept; knee flexes |
| Midstance | 0.10–0.30 | 10–30% | Single limb; pelvis over foot |
| Terminal stance | 0.30–0.50 | 30–50% | Heel rise begins |
| Pre-swing | 0.50–0.62 | 50–62% | Toe-off prep; opposite foot contacts |
| Initial swing | 0.62–0.75 | 62–75% | Foot clears ground |
| Mid-swing | 0.75–0.87 | 75–87% | Tibia vertical |
| Terminal swing | 0.87–1.00 | 87–100% | Decelerate for contact |

**Stance:** `τ ∈ [0, 0.62)` — foot on ground. **Swing:** `τ ∈ [0.62, 1.0)`.

### 5.2 Game four-pose cycle (right leg)

| Pose | Approx τ | Maps to Perry |
|------|----------|---------------|
| **Contact** | 0.00 | Initial contact + loading start |
| **Low** | 0.10–0.15 | Midstance (COM lowest) |
| **Passing** | 0.50 | Pre-swing / leg passing |
| **High** | 0.75–0.87 | Mid-swing peak |

Agent workflow: define **4 poses as keyframes**, then **densify** with Perry % points from §6.

---

## 6. Agent data contract — keyframe tables

### 6.1 Required format

Each control point is a **monotonic-in-τ keyframe list**:

```yaml
control_point_id: hip_flex_R
unit: degrees          # convert to radians at bake: rad = deg * π/180
reference_frame: sagittal
reference_limb: right
keyframes:
  - { tau: 0.00, angle: 30.0 }   # flexed at contact
  - { tau: 0.10, angle: 30.0 }
  - { tau: 0.30, angle: 0.0 }
  - { tau: 0.50, angle: -20.0 }  # extension behind
  - { tau: 0.60, angle: 0.0 }
  - { tau: 0.73, angle: 12.0 }
  - { tau: 0.87, angle: 28.0 }
  - { tau: 1.00, angle: 30.0 }   # MUST equal 0.00 (loop closure)
interpolation: cyclic_catmull_rom   # implemented in walk/curves.py
```

**MUST** include `tau: 0.00` and `tau: 1.00` with **identical** angles for cyclic clips.

**MUST NOT** mirror copy R→L numerically; left leg evaluates same table at `τ_L = τ + 0.5` with sign flip on hip: `hip_flex_L = −sample(hip_flex_R, τ_L)`.

### 6.2 Starter curves (right leg, degrees) — already in repo

File: `tools/blender/animation/walk/biomech_curves.py`

| Control point | Keyframes |
|---------------|-----------|
| `hip_flex_R` | `HIP_CURVE_DEG` |
| `knee_flex_R` | `KNEE_CURVE_DEG` (dual lobe) |
| `ankle_R` | `ANKLE_CURVE_DEG` |

Agent task to improve naturalness: **wire `biomech_curves.py` into `sampler.py`** (replace cosine `_knee_flex` / `_foot_angle`), then add upper-body tables below.

### 6.3 Upper-body starter tables (authoritative ranges)

Values are **normal adult walking** — tune ±20% per character.

**Pelvis vertical (`root_vertical`, meters):**

```yaml
keyframes:
  - { tau: 0.00, angle: 0.000 }   # contact — neutral
  - { tau: 0.25, angle: 0.011 }   # midstance rise
  - { tau: 0.50, angle: 0.000 }   # double support — LOWEST
  - { tau: 0.75, angle: 0.011 }
  - { tau: 1.00, angle: 0.000 }
```

Physics: **2× frequency** vs step (bob twice per cycle). Minimum at both feet down (`τ=0`, `τ=0.5`).

**Thorax pitch (`thorax_pitch`, degrees) — counter pelvis:**

```yaml
keyframes:
  - { tau: 0.00, angle: -2.0 }
  - { tau: 0.25, angle:  2.0 }
  - { tau: 0.50, angle: -2.0 }
  - { tau: 0.75, angle:  2.0 }
  - { tau: 1.00, angle: -2.0 }
```

**Neck pitch (`neck_pitch`, degrees) — stabilize head:**

```yaml
keyframes:
  - { tau: 0.00, angle:  1.5 }   # opposite thorax
  - { tau: 0.25, angle: -1.5 }
  - { tau: 0.50, angle:  1.5 }
  - { tau: 0.75, angle: -1.5 }
  - { tau: 1.00, angle:  1.5 }
```

**Shoulder flex (`shoulder_R`, degrees) — contralateral to R leg:**

Phase = `τ_opp = τ + 0.5` (swings with L leg).

```yaml
reference_phase: tau_opp
keyframes:
  - { tau: 0.00, angle: -15.0 }   # arm back when R leg forward
  - { tau: 0.50, angle:  15.0 }
  - { tau: 1.00, angle: -15.0 }
```

**Elbow flex (`elbow_R`, degrees) — lags shoulder ~5% τ:**

```yaml
reference_phase: tau_opp
phase_delay: 0.05
keyframes:
  - { tau: 0.00, angle: 35.0 }
  - { tau: 0.50, angle: 20.0 }
  - { tau: 1.00, angle: 35.0 }
```

**Weapon arm override:** when `right_arm_hold` is set (Scout trident), scale shoulder swing by `arm_swing_r_scale` (~0.15) and add static offset tuple — do not remove contralateral phase entirely (causes "dead" torso).

---

## 7. Physics constraints — agent MUST enforce

| ID | Rule | Test |
|----|------|------|
| P1 | Knee never locks 180° under load | `knee_flex ≥ 3°` at contact and mid-stance |
| P2 | Ankle dorsiflexes before swing (~60% τ) | `ankle` minimum ≤ −15° before `τ=0.62` |
| P3 | Swing knee peak near `τ=0.73` | local max of `knee_flex` in swing |
| P4 | Pelvis lowest at double support | `root_vertical` min at `τ≈0` and `τ≈0.5` |
| P5 | Head net rotation < thorax rotation | `|head_pitch + neck_pitch + thorax_pitch| < 8°` |
| P6 | No spine yaw/roll on FK in-place rig | `spine.y == spine.z == 0` |
| P7 | Loop closure | `pose(τ=0) == pose(τ=1)` all channels |
| P8 | L/R hip antisymmetric at same τ | `hip_L(τ) ≈ −hip_R(τ)` |
| P9 | Stance ≥ 58% of cycle | stance_end ∈ [0.58, 0.65] |
| P10 | Foot on ground during stance | `foot_lift ≈ 0` for `τ < stance_end` |

---

## 8. Sampling pseudocode — agent implementation target

```python
def sample_walk_pose(tau: float, style: WalkStyle) -> BonePose:
  tau_l = (tau + 0.5) % 1.0

  # Lower body — Perry tables (degrees → radians)
  r_hip   = deg(sample_curve(HIP_CURVE_DEG, tau))
  l_hip   = -deg(sample_curve(HIP_CURVE_DEG, tau_l))
  r_knee  = deg(sample_curve(KNEE_CURVE_DEG, tau))
  l_knee  = deg(sample_curve(KNEE_CURVE_DEG, tau_l))
  r_ankle = deg(sample_curve(ANKLE_CURVE_DEG, tau))
  l_ankle = deg(sample_curve(ANKLE_CURVE_DEG, tau_l))

  # Pelvis / trunk — independent tables
  root_y = sample_curve(PELVIS_VERTICAL_M, tau)
  spine  = sample_curve(THORAX_PITCH_DEG, tau)
  spine1 = sample_curve(THORAX_PITCH_DEG, tau) * 0.6 + sample_curve(PELVIS_PITCH_DEG, tau) * 0.4
  neck   = sample_curve(NECK_PITCH_DEG, tau)
  head   = sample_curve(HEAD_PITCH_DEG, tau)

  # Arms — contralateral phase + delay
  r_shoulder = sample_curve(SHOULDER_CURVE_DEG, tau_l) * style.arm_swing_r_scale + hold_offset
  r_elbow    = sample_curve(ELBOW_CURVE_DEG, tau_l - 0.05)
  l_shoulder = sample_curve(SHOULDER_CURVE_DEG, tau)
  l_elbow    = sample_curve(ELBOW_CURVE_DEG, tau - 0.05)

  return assemble_tbg_pose(...)
```

`sample_curve` = cyclic Catmull-Rom (`walk/curves.py`). **NEVER** use linear interpolation across phase boundaries — use ≥5 keyframes per segment.

---

## 9. Interpolation and foot trajectories (Level 2 → 3)

| Level | Method | When |
|-------|--------|------|
| **L1** | Joint keyframes only | In-place bake (current) |
| **L2** | L1 + Perry tables + upper-body segments | **Target for natural walk** |
| **L3** | Foot target Bézier + two-bone IK | Locomotion on terrain; foot slide fix |

Foot swing path (sagittal plane): use **cubic Bézier**, not circular arc.

```
P0 = (hip_x, 0)           # toe-off
P1 = (hip_x + 0.1, h_max) # lift
P2 = (stride_x - 0.1, h_max)
P3 = (stride_x, 0)        # heel strike
```

Solve `ankle`/`knee`/`hip` from foot target via IK only when root translates. In-place: joint tables are sufficient.

Locomotion System keytimes (per leg, normalized): `foot_strike`, `foot_off`, `foot_land` — extract from baked clip heel Y for runtime retargeting.

---

## 10. How to prompt an AI agent (copy-paste template)

```
TASK: Implement natural in-place walk for TBG FK rig.

LOAD:
- tools/blender/animation/walk/AI-WALK-AUTHORING-SPEC.md
- tools/blender/animation/walk/biomech_curves.py
- tools/blender/animation/walk/curves.py

GIVEN:
- cadence_spm = 110, fps = 30
- reference limb = right, τ=0 at right heel strike
- stance_end = 0.62

WHEN sampling pose at τ:
- MUST sample HIP/KNEE/ANKLE from Perry keyframe tables via cyclic Catmull-Rom
- MUST set L_hip(τ) = -R_hip(τ + 0.5)
- MUST add PELVIS_VERTICAL (2× bob), THORAX_PITCH, NECK_PITCH, HEAD_PITCH tables
- MUST swing arms contralateral with elbow phase_delay = 0.05
- MUST keep TBG_Spine Y/Z rotation = 0

THEN:
- pose(τ=0) == pose(τ=1)
- pelvis lowest at τ=0 and τ=0.5
- knee swing peak near τ=0.73
- pytest tests/blender/walk/ passes

OUTPUT:
- Edit sampler.py (or new segment_gait.py) — no bpy in sampler
- Add upper-body curves to biomech_curves.py
- Add contract tests for new segments
```

---

## 11. Validation checklist (automated)

| Contract | Test file |
|----------|-----------|
| Perry knee dual lobe | `test_biomech.py` |
| Pelvis double-support low | `test_biomech.py` |
| Leg alternation at τ and τ+0.5 | `test_contracts.py` |
| Loop closure | `test_contracts.py` |
| No spine yaw/roll | `test_symmetric_gait.py` |
| L/R hip mirror | `test_symmetric_gait.py` |
| Neck/head counter-phase *(add)* | `test_segment_gait.py` |
| Stance foot not lifted *(add)* | `test_segment_gait.py` |

---

## 12. Repo migration path (recommended agent sequence)

1. **Create** `segment_gait.py` — sample all control points from `biomech_curves.py` + new upper-body tables.
2. **Delegate** `sampler.py` → `segment_gait.py` (keep `symmetric_gait.py` as fallback).
3. **Add** `NECK` / `HEAD` curves; verify in Animation Studio.
4. **Tune** `WalkStyle` amplitudes — do not change Perry τ boundaries without updating tests.
5. **Bake** → `python3 tools/blender/export_game_unit.py <unit_id>`.
6. **Level 3 (optional):** foot IK + root XZ from stride length.

---

## 13. References (full)

1. Perry J, Burnfield JM. *Gait Analysis: Normal and Pathological Function*. 2nd ed. SLACK; 2010.  
2. AAPM&R. Biomechanics of Normal Gait. [KnowledgeNow](https://now.aapmr.org/biomechanics-normal-gait/).  
3. Stokes VP, Andersson C, Forssberg H. Rotational and translational movement features of the pelvis and thorax during adult human locomotion. *J Biomech*. 1989;22(1):43-50.  
4. Fujii K et al. Kinematic aspects of trunk motion and gender effect in normal adults. *J NeuroEngineering Rehabil*. 2010;7:9.  
5. Johansen RS. *Automated Semi-Procedural Animation for Character Locomotion*. MSc thesis, ITU; 2010.  
6. Roblox Creator Docs. Create character animations — walk cycle key poses.  
7. TBG implementation: `symmetric_gait.py`, `biomech_curves.py`, `BIOMECHANICAL-WALK-SPEC.md`.
