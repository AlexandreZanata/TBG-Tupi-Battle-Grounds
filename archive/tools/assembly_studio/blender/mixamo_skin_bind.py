"""Weight transfer + armature bind for Mixamo Edit skin swap."""

from __future__ import annotations

import bpy
from mathutils import kdtree


def skin_custom_mesh(custom: bpy.types.Object, reference: bpy.types.Object, arm_obj: bpy.types.Object) -> None:
    try:
        _transfer_weights(reference, custom)
        if _has_skin_weights(custom):
            _bind_armature(custom, arm_obj)
            return
    except RuntimeError:
        pass
    _clear_armature_bind(custom)
    bpy.ops.object.select_all(action="DESELECT")
    custom.select_set(True)
    arm_obj.select_set(True)
    bpy.context.view_layer.objects.active = custom
    bpy.ops.object.parent_set(type="ARMATURE_AUTO")
    if not _has_skin_weights(custom):
        raise RuntimeError("skin bind failed — no vertex weights on custom mesh")


def _clear_armature_bind(mesh_obj: bpy.types.Object) -> None:
    mesh_obj.parent = None
    for mod in list(mesh_obj.modifiers):
        if mod.type == "ARMATURE":
            mesh_obj.modifiers.remove(mod)
    for group in list(mesh_obj.vertex_groups):
        mesh_obj.vertex_groups.remove(group)


def _has_skin_weights(mesh_obj: bpy.types.Object) -> bool:
    return bool(mesh_obj.vertex_groups) and any(vert.groups for vert in mesh_obj.data.vertices)


def _transfer_weights(source: bpy.types.Object, target: bpy.types.Object) -> None:
    _prepare_target_groups(source, target)
    if _try_data_transfer(source, target):
        return
    _transfer_weights_kdtree(source, target)
    if not _has_skin_weights(target):
        raise RuntimeError("weight transfer failed")


def _prepare_target_groups(source: bpy.types.Object, target: bpy.types.Object) -> None:
    for group in list(target.vertex_groups):
        target.vertex_groups.remove(group)
    for group in source.vertex_groups:
        target.vertex_groups.new(name=group.name)


def _try_data_transfer(source: bpy.types.Object, target: bpy.types.Object) -> bool:
    bpy.ops.object.select_all(action="DESELECT")
    target.select_set(True)
    source.select_set(True)
    bpy.context.view_layer.objects.active = target
    bpy.ops.object.data_transfer(
        use_object_transform=True,
        use_auto_transform=False,
        data_type="VGROUP_WEIGHTS",
        vert_mapping="NEAREST",
        layers_select_src="ALL",
        layers_select_dst="NAME",
        mix_mode="REPLACE",
    )
    return _has_skin_weights(target)


def _transfer_weights_kdtree(source: bpy.types.Object, target: bpy.types.Object) -> None:
    src_mesh = source.data
    mw_s = source.matrix_world
    tree = kdtree.KDTree(len(src_mesh.vertices))
    for i, vert in enumerate(src_mesh.vertices):
        tree.insert(mw_s @ vert.co, i)
    tree.balance()
    mw_t = target.matrix_world
    for vert in target.data.vertices:
        _, idx, _ = tree.find(mw_t @ vert.co)
        for group in src_mesh.vertices[idx].groups:
            target.vertex_groups[group.group].add([vert.index], group.weight, "REPLACE")


def _bind_armature(mesh_obj: bpy.types.Object, arm_obj: bpy.types.Object) -> None:
    for mod in list(mesh_obj.modifiers):
        if mod.type == "ARMATURE":
            mesh_obj.modifiers.remove(mod)
    mod = mesh_obj.modifiers.new("Armature", "ARMATURE")
    mod.object = arm_obj
    mesh_obj.parent = arm_obj
    mesh_obj.matrix_parent_inverse = arm_obj.matrix_world.inverted()
