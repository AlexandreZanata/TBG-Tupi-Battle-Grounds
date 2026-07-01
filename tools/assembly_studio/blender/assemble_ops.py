"""Blender bpy operations for modular character assembly."""

from __future__ import annotations

from pathlib import Path

import bpy

from assembly_studio.library.recipe import AssemblyRecipe
from assembly_studio.library.schema import load_json

REPO = Path(__file__).resolve().parents[3]


def assemble_in_blender(recipe: AssemblyRecipe) -> Path:
    _clear_scene()
    arm_obj, body_meshes = _import_base(recipe.base_body_path)
    _apply_body_material(body_meshes, load_json(recipe.material_path))
    for part in recipe.parts:
        _attach_part(arm_obj, part.mesh_path, part.socket, part.part_id)
    _apply_part_materials(load_json(recipe.material_path))
    recipe.export_path.parent.mkdir(parents=True, exist_ok=True)
    _export_glb(recipe.export_path, body_meshes, arm_obj)
    return recipe.export_path


def _clear_scene() -> None:
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete()
    for block in (bpy.data.meshes, bpy.data.armatures, bpy.data.materials):
        for item in list(block):
            if item.users == 0:
                block.remove(item)


def _import_base(body_path: Path) -> tuple[bpy.types.Object, list[bpy.types.Object]]:
    bpy.ops.import_scene.gltf(filepath=str(body_path))
    arms = [o for o in bpy.context.scene.objects if o.type == "ARMATURE"]
    meshes = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    if not arms or not meshes:
        raise RuntimeError(f"{body_path}: expected skinned mesh + armature")
    return arms[0], meshes


def _attach_part(arm_obj: bpy.types.Object, mesh_path: Path, socket: str, part_id: str) -> None:
    before = set(bpy.context.scene.objects)
    bpy.ops.import_scene.gltf(filepath=str(mesh_path))
    imported = [o for o in bpy.context.scene.objects if o not in before and o.type == "MESH"]
    if not imported:
        raise RuntimeError(f"{mesh_path}: no mesh imported")
    part_obj = imported[0]
    part_obj.name = part_id
    part_obj.parent = arm_obj
    part_obj.parent_type = "BONE"
    part_obj.parent_bone = socket


def _apply_body_material(meshes: list[bpy.types.Object], material_data: dict) -> None:
    factor = material_data.get("body", {}).get("baseColorFactor")
    if not factor:
        return
    mat = _ensure_material("TBG_Body", factor)
    for mesh_obj in meshes:
        if mesh_obj.data.materials:
            mesh_obj.data.materials[0] = mat
        else:
            mesh_obj.data.materials.append(mat)


def _apply_part_materials(material_data: dict) -> None:
    factor = material_data.get("parts", {}).get("baseColorFactor")
    if not factor:
        return
    mat = _ensure_material("TBG_Part", factor)
    for obj in bpy.context.scene.objects:
        if obj.type != "MESH" or obj.parent_type != "BONE":
            continue
        if obj.data.materials:
            obj.data.materials[0] = mat
        else:
            obj.data.materials.append(mat)


def _ensure_material(name: str, base_color: list[float]) -> bpy.types.Material:
    mat = bpy.data.materials.get(name) or bpy.data.materials.new(name)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    if bsdf:
        bsdf.inputs["Base Color"].default_value = tuple(base_color)
    return mat


def _export_glb(path: Path, meshes: list[bpy.types.Object], arm_obj: bpy.types.Object) -> None:
    bpy.ops.object.select_all(action="DESELECT")
    for obj in meshes:
        obj.select_set(True)
    arm_obj.select_set(True)
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.export_scene.gltf(
        filepath=str(path),
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_skins=True,
        export_animations=False,
        export_yup=True,
    )
