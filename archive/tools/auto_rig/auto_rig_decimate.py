"""Decimate dense AI meshes so Blender automatic weights can run."""

from __future__ import annotations

try:
    import bpy
except ImportError as exc:
    raise SystemExit("auto_rig_decimate requires Blender bpy") from exc

DEFAULT_MAX_FACES = 100_000


def maybe_decimate_for_rig(mesh_obj: bpy.types.Object, *, max_faces: int = DEFAULT_MAX_FACES) -> bool:
    faces = len(mesh_obj.data.polygons)
    if faces <= max_faces:
        return False
    ratio = max_faces / faces
    modifier = mesh_obj.modifiers.new(name="TBGAutoRigDecimate", type="DECIMATE")
    modifier.ratio = ratio
    bpy.context.view_layer.objects.active = mesh_obj
    bpy.ops.object.modifier_apply(modifier=modifier.name)
    return True
