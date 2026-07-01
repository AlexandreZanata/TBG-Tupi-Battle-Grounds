"""Blender ops: Mixamo FBX → TBG-named armature + skinned mesh."""

from __future__ import annotations

from pathlib import Path

import bpy

from mixamo_bone_map import load_mixamo_bone_map, tbg_bone_for_mixamo


def clear_scene() -> None:
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete()


def import_mixamo_fbx(fbx_path: Path) -> tuple[bpy.types.Object, bpy.types.Object]:
    clear_scene()
    return import_mixamo_fbx_incremental(fbx_path)


def import_mixamo_fbx_incremental(fbx_path: Path) -> tuple[bpy.types.Object, bpy.types.Object]:
    before = {obj.name for obj in bpy.context.scene.objects}
    bpy.ops.import_scene.fbx(filepath=str(fbx_path))
    meshes = [
        obj for obj in bpy.context.scene.objects if obj.type == "MESH" and obj.name not in before
    ]
    armatures = [
        obj for obj in bpy.context.scene.objects if obj.type == "ARMATURE" and obj.name not in before
    ]
    if not armatures:
        raise RuntimeError(f"no armature in Mixamo FBX: {fbx_path}")
    if not meshes:
        raise RuntimeError(f"no mesh in Mixamo FBX: {fbx_path}")
    return _join_meshes(meshes), armatures[0]


def retarget_mixamo_to_tbg(arm_obj: bpy.types.Object, mesh_obj: bpy.types.Object) -> dict[str, str]:
    bone_map = load_mixamo_bone_map()
    renamed: dict[str, str] = {}
    for bone in list(arm_obj.data.bones):
        tbg_name = tbg_bone_for_mixamo(bone_map, bone.name)
        if not tbg_name or tbg_name in renamed.values():
            continue
        old_name = bone.name
        bone.name = tbg_name
        renamed[old_name] = tbg_name
    _rename_vertex_groups(mesh_obj, bone_map)
    if bone_map.get("root_policy") == "inject_from_spine":
        _inject_tbg_root(arm_obj)
    return renamed


def _join_meshes(meshes: list[bpy.types.Object]) -> bpy.types.Object:
    if len(meshes) == 1:
        return meshes[0]
    bpy.ops.object.select_all(action="DESELECT")
    for obj in meshes:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    bpy.ops.object.join()
    return bpy.context.object


def _rename_vertex_groups(mesh_obj: bpy.types.Object, bone_map: dict) -> None:
    for group in list(mesh_obj.vertex_groups):
        tbg_name = tbg_bone_for_mixamo(bone_map, group.name)
        if tbg_name:
            group.name = tbg_name


def _inject_tbg_root(arm_obj: bpy.types.Object) -> None:
    if "TBG_Root" in arm_obj.data.bones:
        return
    spine = arm_obj.data.bones.get("TBG_Spine")
    if spine is None:
        raise RuntimeError("TBG_Spine missing — cannot inject TBG_Root")
    bpy.ops.object.select_all(action="DESELECT")
    arm_obj.select_set(True)
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.object.mode_set(mode="EDIT")
    edit_bones = arm_obj.data.edit_bones
    spine_edit = edit_bones.get("TBG_Spine")
    root = edit_bones.new("TBG_Root")
    root.head = spine_edit.head.copy()
    root.tail = spine_edit.tail.copy()
    root.roll = spine_edit.roll
    spine_edit.parent = root
    bpy.ops.object.mode_set(mode="OBJECT")


def export_skinned_glb(
    mesh_obj: bpy.types.Object,
    arm_obj: bpy.types.Object,
    output_path: Path,
    *,
    export_animations: bool = False,
) -> Path:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="DESELECT")
    mesh_obj.select_set(True)
    arm_obj.select_set(True)
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.export_scene.gltf(
        filepath=str(output_path),
        export_format="GLB",
        use_selection=True,
        export_apply=False,
        export_skins=True,
        export_animations=export_animations,
        export_yup=True,
    )
    return output_path
