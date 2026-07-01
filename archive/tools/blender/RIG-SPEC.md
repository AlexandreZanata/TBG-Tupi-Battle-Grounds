# TBG Blender Rig Specification

> Authoring-only (GPL Blender). Game ships glTF + textures only.  
> **Related:** [real-3d-character-pipeline.md](../../docs/design/real-3d-character-pipeline.md)

---

## Body classes

Shared rigs — one skeleton template per class. Do not author per-unit unique skeletons for MVP.

| Body class | Use case | Example pilots |
|------------|----------|----------------|
| `HumanoidLight` | Agile humanoids, ranged | `ranged_plant_01` (organic light) |
| `HumanoidHeavy` | Tanks, bruisers | `tank_human_01` |
| `Mechanical` | Machines faction, rigid joints | `support_machine_01` |
| `Organic` | Undead / Plants non-human silhouettes | `dps_undead_01`, `ranged_plant_01` |

Template file: `templates/HumanoidHeavy_template.blend` (generate via `scripts/build_rig_template.py`).

---

## Bone naming convention

Prefix all deformation bones with `TBG_`. Use `.L` / `.R` suffix for paired limbs.

| Bone | Parent | Role |
|------|--------|------|
| `TBG_Root` | — | World root / ground contact |
| `TBG_Spine` | `TBG_Root` | Pelvis / lower torso |
| `TBG_Spine1` | `TBG_Spine` | Upper torso |
| `TBG_Neck` | `TBG_Spine1` | Neck |
| `TBG_Head` | `TBG_Neck` | Head |
| `TBG_Shoulder.L` | `TBG_Spine1` | Left clavicle |
| `TBG_UpperArm.L` | `TBG_Shoulder.L` | Left upper arm |
| `TBG_LowerArm.L` | `TBG_UpperArm.L` | Left forearm |
| `TBG_Hand.L` | `TBG_LowerArm.L` | Left hand |
| `TBG_Shoulder.R` | `TBG_Spine1` | Right clavicle (mirror `.L`) |
| `TBG_UpperLeg.L` | `TBG_Spine` | Left thigh |
| `TBG_LowerLeg.L` | `TBG_UpperLeg.L` | Left shin |
| `TBG_Foot.L` | `TBG_LowerLeg.L` | Left foot |
| `TBG_UpperLeg.R` | `TBG_Spine` | Right leg (mirror `.L`) |

Mechanical rigs may replace limb chains with `TBG_Armature_Mech_*` segments but **must** keep `TBG_Root` and `TBG_Head` for retargeting.

Organic rigs may add `TBG_Tail` or `TBG_Wing.L` / `TBG_Wing.R` as optional children of `TBG_Spine1`.

Optional toe bones `TBG_Toe.L` / `TBG_Toe.R` (children of `TBG_Foot.*`) enable toe-off without spine lean.

---

## Animation clips (minimum)

Export embedded in `.glb` with these exact names:

| Clip | glTF animation name |
|------|---------------------|
| Idle | `Idle` |
| Spawn | `Spawn` |
| Attack | `Attack` |
| Death | `Death` |

---

## Export rules

- glTF 2.0 binary (`.glb`), Y-up, +Z forward
- Apply modifiers on export
- Tri count ≤ 8,000 per skinned mesh (warning if exceeded)
- UV0 required on all export meshes
- Armature required in export collection

Use the **TBG Export** add-on (`tbg_export/`) or CLI (`export_unit.py`).

---

## Generate rig template

```bash
blender --background --python tools/blender/scripts/build_rig_template.py
```

Output: `tools/blender/templates/HumanoidHeavy_template.blend`
