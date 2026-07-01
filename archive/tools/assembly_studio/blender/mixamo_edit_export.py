"""Ground + unparent Mixamo Edit rig before GLB export."""

from __future__ import annotations

import bpy
from mathutils import Vector


def prepare_skinned_export(arm_obj: bpy.types.Object, mesh_obj: bpy.types.Object) -> None:
    pivot = mesh_obj.parent
    if pivot and pivot.type == "EMPTY" and pivot.name == "TBG_RigPivot":
        for obj in (arm_obj, mesh_obj):
            world = obj.matrix_world.copy()
            obj.parent = None
            obj.matrix_world = world
        bpy.data.objects.remove(pivot, do_unlink=True)
    corners: list[Vector] = []
    for obj in (mesh_obj, arm_obj):
        corners.extend(obj.matrix_world @ Vector(corner) for corner in obj.bound_box)
    if not corners:
        return
    dy = min(point.y for point in corners)
    for obj in (mesh_obj, arm_obj):
        obj.location.y -= dy
    bpy.context.view_layer.update()
