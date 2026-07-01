"""Merge base body GLB + socket-attached parts without Blender (CI fallback)."""

from __future__ import annotations

import copy
from pathlib import Path

from assembly_studio.library.glb_inspect import load_glb
from assembly_studio.library.glb_write import fix_body_display_orientation, write_glb
from assembly_studio.library.recipe import AssemblyRecipe
from assembly_studio.library.attachments import PartTransform, parse_attachments, transform_node
from assembly_studio.library.schema import load_json


def assemble_gltf(recipe: AssemblyRecipe) -> Path:
    """Attach static part meshes to socket bones; preserve base skin only."""
    attachments = recipe.attachments
    base_gltf, base_bin = load_glb(recipe.base_body_path)
    if base_bin is None:
        raise ValueError(f"{recipe.base_body_path}: missing BIN chunk")
    gltf = copy.deepcopy(base_gltf)
    bin_chunk = bytearray(base_bin)
    fix_body_display_orientation(gltf, bytes(bin_chunk))
    bone_count = _skin_joint_count(gltf)

    material_data = load_json(recipe.material_path)
    _apply_body_material(gltf, material_data)

    for part in recipe.parts:
        if not part.mesh_path.is_file():
            raise FileNotFoundError(part.mesh_path)
        part_gltf, part_bin = load_glb(part.mesh_path)
        if part_bin is None:
            raise ValueError(f"{part.mesh_path}: missing BIN chunk")
        gltf, bin_chunk = _attach_part(
            gltf,
            bin_chunk,
            part_gltf,
            part_bin,
            part.socket,
            part.part_id,
            attachments.get(part.part_id),
        )

    if _skin_joint_count(gltf) != bone_count:
        raise RuntimeError("assembly changed bone count — parts must not add joints")

    recipe.export_path.parent.mkdir(parents=True, exist_ok=True)
    write_glb(recipe.export_path, gltf, bytes(bin_chunk))
    return recipe.export_path


def _skin_joint_count(gltf: dict) -> int:
    skins = gltf.get("skins", [])
    if not skins:
        return 0
    return max(len(skin.get("joints", [])) for skin in skins)


def _node_index(gltf: dict, name: str) -> int:
    for idx, node in enumerate(gltf.get("nodes", [])):
        if node.get("name") == name:
            return idx
    raise ValueError(f"socket bone not found: {name}")


def _attach_part(
    gltf: dict,
    bin_chunk: bytearray,
    part_gltf: dict,
    part_bin: bytes,
    socket: str,
    part_id: str,
    transform: PartTransform | None = None,
) -> tuple[dict, bytearray]:
    offset = len(bin_chunk)
    bin_chunk.extend(part_bin)

    base_accessors = len(gltf.get("accessors", []))
    base_views = len(gltf.get("bufferViews", []))
    base_meshes = len(gltf.get("meshes", []))

    part_accessors = copy.deepcopy(part_gltf.get("accessors", []))
    part_views = copy.deepcopy(part_gltf.get("bufferViews", []))
    part_meshes = copy.deepcopy(part_gltf.get("meshes", []))

    for view in part_views:
        view["byteOffset"] = view.get("byteOffset", 0) + offset
    for acc in part_accessors:
        if "bufferView" in acc:
            acc["bufferView"] += base_views

    for mesh in part_meshes:
        for prim in mesh.get("primitives", []):
            for key, acc_idx in list(prim.get("attributes", {}).items()):
                prim["attributes"][key] = acc_idx + base_accessors
            if "indices" in prim:
                prim["indices"] += base_accessors

    gltf.setdefault("accessors", []).extend(part_accessors)
    gltf.setdefault("bufferViews", []).extend(part_views)
    gltf.setdefault("meshes", []).extend(part_meshes)

    part_node_idx = len(gltf.get("nodes", []))
    part_node: dict = {"name": part_id, "mesh": base_meshes}
    transform_node(part_node, transform)
    gltf.setdefault("nodes", []).append(part_node)
    socket_idx = _node_index(gltf, socket)
    gltf["nodes"][socket_idx].setdefault("children", []).append(part_node_idx)
    gltf["buffers"][0]["byteLength"] = len(bin_chunk)
    return gltf, bin_chunk


def _apply_body_material(gltf: dict, material_data: dict) -> None:
    factor = material_data.get("body", {}).get("baseColorFactor")
    if not factor or not gltf.get("materials"):
        return
    gltf["materials"][0].setdefault("pbrMetallicRoughness", {})["baseColorFactor"] = factor
