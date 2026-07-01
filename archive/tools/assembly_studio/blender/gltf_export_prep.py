"""Prepare MB-Lab objects before Blender glTF export."""

from __future__ import annotations

from typing import Any

try:
    import bpy
except ImportError:
    bpy = None  # type: ignore[assignment]


def gltf_export_kwargs() -> dict[str, Any]:
    return {
        "export_format": "GLB",
        "use_selection": True,
        "export_apply": True,
        "export_skins": True,
        "export_animations": False,
        "export_yup": True,
        "export_tangents": True,
        "export_texcoords": True,
        "export_morph": True,
    }


def prepare_meshes_for_gltf(meshes: list[Any]) -> None:
    """Trim UV layers, bake tangents, root skinned meshes for portable glTF."""
    for obj in meshes:
        if obj.type != "MESH":
            continue
        _trim_uv_layers(obj.data)
        _detach_armature_parent(obj)
        if obj.data.uv_layers:
            obj.data.calc_tangents()


def _trim_uv_layers(mesh_data: Any, *, keep: int = 1) -> None:
    while len(mesh_data.uv_layers) > keep:
        mesh_data.uv_layers.remove(mesh_data.uv_layers[-1])


def _detach_armature_parent(mesh_obj: Any) -> None:
    parent = mesh_obj.parent
    if parent is None or parent.type != "ARMATURE":
        return
    matrix = mesh_obj.matrix_world.copy()
    mesh_obj.parent = None
    mesh_obj.matrix_world = matrix


def resize_images_pot() -> int:
    """Resize embedded images to power-of-two (clears IMAGE_NPOT validator info)."""
    resized = 0
    for image in bpy.data.images:
        if image.size[0] <= 0 or image.size[1] <= 0:
            continue
        w, h = int(image.size[0]), int(image.size[1])
        nw, nh = _next_pot(w), _next_pot(h)
        if nw == w and nh == h:
            continue
        image.scale(nw, nh)
        resized += 1
    return resized


def _next_pot(value: int) -> int:
    if value < 1:
        return 1
    pot = 1
    while pot < value:
        pot <<= 1
    return pot
