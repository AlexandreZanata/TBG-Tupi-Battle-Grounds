# prep_ai_mesh — Input Contract (Phase 04b §1.1)

> Implementation: `prep_ai_mesh.py` (§1.2).  
> Related: [mesh-pipeline README](../../mesh-pipeline/README.md), [RIG-SPEC.md](../RIG-SPEC.md)

---

## Purpose

Normalize **raw AI or sculpt exports** before they enter the TBG showcase tier. Game-ready rigging assumes a clean, human-scale, Y-up biped mesh.

---

## Accepted inputs

| Format | Notes |
|--------|-------|
| **GLB** | Preferred — embedded PBR textures preserved |
| **OBJ** | Accepted — materials may be lost; repack in Blender export |

### Scene requirements

- **Up axis:** Y-up (glTF default). Z-up sources MUST be rotated before export or via `--z-up` (§1.2).
- **Topology:** Single skinned-ready mesh or multiple parts that can be **joined** without losing silhouette.
- **Scale:** Arbitrary on input — script normalizes to target height (below).
- **Facing:** Character should face **+Z** in Blender/glTF rest pose when possible.

### Rejected / flagged inputs

- No mesh geometry
- Only armature / empty nodes
- Non-manifold internal shells that survive cleanup (flag for manual review)
- Non-biped silhouettes — route via `rigException: true` ([EXCEPTION-WORKFLOW](../../../.local/phases/04b-biped-game-ready-pipeline/EXCEPTION-WORKFLOW.md))

---

## Output contract

| Property | Target |
|----------|--------|
| **Height** | 1.75–1.85 m (world units) after normalize |
| **Feet** | Lowest mesh point at **Y = 0** |
| **Origin XZ** | Mesh center on X=0, Z=0 |
| **Facing** | +Z forward (glTF / Three.js / Unity import convention) |
| **Format** | GLB, Y-up, single merged mesh object |
| **Cleanup** | No loose verts/edges; consistent outward normals |

Tolerance band for automated tests: height **1.6–2.0 m** (allows stylized heroes before fine-tune).

---

## Processing steps (§1.2)

1. Import GLB/OBJ
2. Join mesh objects
3. `remove_doubles` (merge by distance)
4. `delete_loose`
5. `normals_make_consistent` (outside)
6. Optional `--voxel-remesh` when triangle count **> 500k** (typical AI output)
7. Uniform scale to target height (default 1.8 m)
8. Ground align (feet Y=0) + center XZ
9. Export GLB

---

## CLI

```bash
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/prep_ai_mesh.py -- \
  /path/in.glb /path/out.glb [--height 1.8] [--voxel-remesh] [--z-up] [--report-json]
```

## Integration

- `import_showcase_sculpts.py --prep` — run before `optimize_glb.sh --mode visual`
- Showcase path: `content/characters/meshes/showcase/{id}.glb`

```bash
python3 tools/mesh-pipeline/import_showcase_sculpts.py \
  --source-dir /path/to/downloads --prep --only demo_titan_01 --force
```

---

**Updated:** 2026-06-30
