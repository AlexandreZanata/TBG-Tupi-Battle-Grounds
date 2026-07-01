"""Compare data_transfer vs kdtree after normalize fix."""
import sys
from pathlib import Path

import bpy

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
from mixamo_skin_bind import _transfer_weights

ROOT = Path(__file__).resolve().parents[3]


def weighted(obj):
    return sum(1 for v in obj.data.vertices if v.groups)


custom = _import_glb_mesh(ROOT / "content/characters/imports/mixamo/meshes/ybot_skin_01.glb")
_normalize_height(custom)
_apply_transforms(custom)
_ground_object(custom)
mixamo_mesh, arm = import_mixamo_fbx_incremental(ROOT / "content/characters/imports/mixamo/raw/Y Bot.fbx")
retarget_mixamo_to_tbg(arm, mixamo_mesh)
_normalize_rig(mixamo_mesh, arm)
print("mesh scale", mixamo_mesh.scale, "parent", mixamo_mesh.parent)
_ground_object(mixamo_mesh)
_align_mesh_to_reference(custom, mixamo_mesh, 1.0)

dec = custom.modifiers.new("Decimate", "DECIMATE")
dec.ratio = 20000 / len(custom.data.vertices)
bpy.context.view_layer.objects.active = custom
bpy.ops.object.modifier_apply(modifier=dec.name)

_transfer_weights(mixamo_mesh, custom)
print("data_transfer weighted", weighted(custom))
