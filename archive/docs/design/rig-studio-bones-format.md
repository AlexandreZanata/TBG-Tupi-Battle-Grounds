# Rig Studio — `bones.json` format

> **Purpose:** portable TBG skeleton placement from the browser Rig Studio.  
> **Versioned archive:** `content/characters/rig-sessions/{unitId}/` (git-tracked).  
> **Live session:** `.local/rig-studio/sessions/{unitId}/` (gitignored).

---

## Workflow

```text
Showcase mesh
  → Rig Studio prep (Blender headless) → session.blend + preview.glb
  → Browser: place 21 joints → Save bone positions → bones.json
  → Archive (optional, git) → content/characters/rig-sessions/{unitId}/
  → Bind & export → game GLB + retargeted clips
  → Walk test → bake_session_walk_bpy / Motion Arena
```

| Step | UI / command | Artifact |
|------|----------------|----------|
| 1 Prep | **Load skeleton** | `.local/.../session.blend`, `preview.glb` |
| 2 Edit | Drag numbered joints, double-click lock | — |
| 3 Save | **Save bone positions** | `.local/.../bones.json` |
| 4 Archive | `archive_rig_session.py --id {unitId}` | `content/.../rig-sessions/{unitId}/` |
| 5 Bind | **Bind & export game mesh** | `content/characters/meshes/game/{unitId}.glb` |
| 6 Walk | `run_session_walk_test` or Motion Arena | `content/.../clips/units/{unitId}/Walk.glb` |

---

## JSON schema (version 1)

```json
{
  "version": 1,
  "armature": "HumanoidHeavy_Armature",
  "bones": {
    "TBG_Root": {
      "head": [0.0, 0.0, 0.0],
      "tail": [0.0, 0.1, 0.0],
      "parent": null
    },
    "TBG_Shoulder.L": {
      "head": [-0.22, 1.26, 0.0],
      "tail": [-0.22, 1.36, 0.0],
      "parent": "TBG_Spine1"
    }
  }
}
```

| Field | Type | Rule |
|-------|------|------|
| `version` | `1` | Required |
| `armature` | string \| null | Blender armature object name (optional) |
| `bones` | object | Map of **Blender-canonical** `TBG_*` bone names |
| `bones.*.head` | `[x, y, z]` | Bone head in **armature local space** (meters, Y-up) |
| `bones.*.tail` | `[x, y, z]` | Bone tail in armature local space |
| `bones.*.parent` | string \| null | Parent bone name or `null` for root |

### Canonical bone set (21 joints)

Placement order in Rig Studio UI (numbers 1–21):

1. `TBG_Root` → `TBG_Spine` → `TBG_Spine1` → `TBG_Neck` → `TBG_Head`  
2. Left arm: `TBG_Shoulder.L` … `TBG_Hand.L`  
3. Right arm: `TBG_Shoulder.R` … `TBG_Hand.R`  
4. Left leg: `TBG_UpperLeg.L` … `TBG_Toe.L`  
5. Right leg: `TBG_UpperLeg.R` … `TBG_Toe.R`

Full names: [RIG-SPEC.md](../../tools/blender/RIG-SPEC.md).

---

## glTF name normalization

The browser preview GLB may export side bones **without** the dot:

| Blender (canonical) | glTF preview node |
|---------------------|-------------------|
| `TBG_Shoulder.L` | `TBG_ShoulderL` |
| `TBG_UpperArm.R` | `TBG_UpperArmR` |

**Always store and archive Blender names** (with `.L` / `.R`). Normalization runs on:

- `PUT /api/rig/sessions/{id}/bones` (server save)
- `archive_rig_session.py`
- `apply_bones_payload()` in Blender

Implementation: `tools/experimental/auto_rig/rig_bone_names.py`, `rig-studio/rigBoneNames.js`.

---

## Archive layout (do not lose skeleton)

```text
content/characters/rig-sessions/
  demo_guard_01/
    bones.json      # canonical 21-bone placement (commit this)
    manifest.json   # unitId, bodyClass, archivedAt, boneCount
```

`session.blend` stays under `.local/` (large, regenerable from prep + `bones.json`).

### Archive after every manual rig pass

```bash
python3 tools/experimental/auto_rig/archive_rig_session.py --id demo_guard_01
git add content/characters/rig-sessions/demo_guard_01/
```

### Restore into a live session

```bash
cp content/characters/rig-sessions/demo_guard_01/bones.json \
   .local/rig-studio/sessions/demo_guard_01/bones.json
# Then: Rig Studio → reload unit, or Blender sync subcommand
```

---

## Walk simulation from archived skeleton

Procedural **Perry eight-phase / segment gait** baked on **your** `bones.json` rest pose, then embedded in the game GLB.

### One command (CLI)

```bash
python3 tools/experimental/auto_rig/promote_rig_session_walk.py --id demo_guard_01
```

Steps inside:
1. Apply archived `bones.json` to `session.blend` armature
2. Bake `Walk` + `WalkRoot` (`humanoid_heavy_v1` for HumanoidHeavy)
3. Write `content/characters/animations/clips/units/{unitId}/Walk.glb`
4. Re-embed all combat clips into `content/characters/meshes/game/{unitId}.glb`

### Rig Studio UI

After **Bind & export** → step 5 **Bake walk from skeleton** (API job `bake_walk`).

### Preview

`/motion-arena.html#{unitId}` — **▶ Walk preview** or WASD.

Gait spec: [AI-WALK-AUTHORING-SPEC.md](../../tools/blender/animation/walk/AI-WALK-AUTHORING-SPEC.md).

---

## Related

| Doc | Topic |
|-----|--------|
| [rig-exception-workflow.md](rig-exception-workflow.md) | When manual rig is required |
| [biped-game-ready-pipeline.md](biped-game-ready-pipeline.md) | Production export |
| [AI-WALK-AUTHORING-SPEC.md](../../tools/blender/animation/walk/AI-WALK-AUTHORING-SPEC.md) | Gait biomechanics |
| [tools/experimental/auto_rig/README.md](../../tools/experimental/auto_rig/README.md) | CLI index |

**Updated:** 2026-06-30
