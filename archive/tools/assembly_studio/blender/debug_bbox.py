import sys
from pathlib import Path

import bpy
from mathutils import Vector

_SCRIPT = Path(__file__).resolve().parent
sys.path.insert(0, str(_SCRIPT))
from import_mixamo_ops import import_mixamo_fbx_incremental, retarget_mixamo_to_tbg
from mixamo_edit_ops import (
    _align_mesh_to_reference,
    _apply_transforms,
    _ground_object,
    _import_glb_mesh,
    _normalize_height,
    _normalize_rig,
)

ROOT = Path(__file__).resolve().parents[3]
custom = _import_glb_mesh(ROOT / "content/characters/imports/mixamo/meshes/ybot_skin_01.glb")
_normalize_height(custom)
_ground_object(custom)
mixamo_mesh, arm = import_mixamo_fbx_incremental(ROOT / "content/characters/imports/mixamo/raw/Y Bot.fbx")
retarget_mixamo_to_tbg(arm, mixamo_mesh)
_normalize_rig(mixamo_mesh, arm)
_ground_object(mixamo_mesh)
_apply_transforms(arm)
_apply_transforms(mixamo_mesh)
_align_mesh_to_reference(custom, mixamo_mesh, 1.0)


def bbox(obj):
    cs = [obj.matrix_world @ Vector(c) for c in obj.bound_box]
    xs, ys, zs = [p.x for p in cs], [p.y for p in cs], [p.z for p in cs]
    return (min(ys), max(ys), max(xs) - min(xs), max(ys) - min(ys), max(zs) - min(zs))


print("custom bbox", bbox(custom), "verts", len(custom.data.vertices))
print("mixamo bbox", bbox(mixamo_mesh))
print("arm bbox", bbox(arm))
