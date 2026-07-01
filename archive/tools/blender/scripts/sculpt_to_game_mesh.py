#!/usr/bin/env python3
"""Decimate showcase GLB to mobile tri budget; rebake normals from high-poly (Blender bpy).

Fallback when host-side gltf-transform simplify was not run.
"""

from __future__ import annotations

import sys
from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("sculpt_to_game_mesh requires Blender bpy") from exc

_SCRIPTS_DIR = Path(__file__).resolve().parent
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))

from bake_game_maps import rebake_normal_map  # noqa: E402

TRI_TARGET = 8000
TEXTURE_SIZE = 1024
MAX_DECIMATE_PASSES = 8
MERGE_DISTANCE = 0.0005


def sculpt_to_game_mesh(
    showcase_path: Path,
    output_path: Path,
    tri_target: int = TRI_TARGET,
    texture_size: int = TEXTURE_SIZE,
) -> Path:
    _clear_scene()
    bpy.ops.import_scene.gltf(filepath=str(showcase_path))
    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    if not meshes:
        raise RuntimeError("showcase import produced no meshes")

    mesh_obj = _join_and_clean(meshes)
    high_ref = _duplicate_high_reference(mesh_obj)
    _decimate_to_budget([mesh_obj], tri_target)
    _shade_smooth(mesh_obj)
    rebake_normal_map(high_ref, mesh_obj, texture_size)
    bpy.data.objects.remove(high_ref, do_unlink=True)
    _resize_embedded_textures(texture_size)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="DESELECT")
    mesh_obj.select_set(True)
    bpy.context.view_layer.objects.active = mesh_obj
    bpy.ops.export_scene.gltf(
        filepath=str(output_path),
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_animations=False,
        export_yup=True,
    )
    return output_path


def _join_and_clean(meshes: list[bpy.types.Object]) -> bpy.types.Object:
    bpy.ops.object.select_all(action="DESELECT")
    for obj in meshes:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    if len(meshes) > 1:
        bpy.ops.object.join()
    mesh_obj = bpy.context.object
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.mesh.remove_doubles(threshold=MERGE_DISTANCE)
    bpy.ops.mesh.delete_loose(use_verts=True, use_edges=True, use_faces=True)
    bpy.ops.mesh.normals_make_consistent(inside=False)
    bpy.ops.object.mode_set(mode="OBJECT")
    return mesh_obj


def _duplicate_high_reference(mesh_obj: bpy.types.Object) -> bpy.types.Object:
    high_ref = mesh_obj.copy()
    high_ref.data = mesh_obj.data.copy()
    bpy.context.collection.objects.link(high_ref)
    return high_ref


def _shade_smooth(mesh_obj: bpy.types.Object) -> None:
    bpy.ops.object.select_all(action="DESELECT")
    mesh_obj.select_set(True)
    bpy.context.view_layer.objects.active = mesh_obj
    bpy.ops.object.shade_smooth()


def _decimate_to_budget(meshes: list[bpy.types.Object], tri_target: int) -> None:
    for _ in range(MAX_DECIMATE_PASSES):
        current = _face_count(meshes)
        if current <= tri_target:
            break
        ratio = min(0.75, tri_target / max(1, current))
        for obj in meshes:
            bpy.context.view_layer.objects.active = obj
            modifier = obj.modifiers.new(name="TBGDecimate", type="DECIMATE")
            modifier.ratio = ratio
            bpy.ops.object.modifier_apply(modifier=modifier.name)


def _resize_embedded_textures(texture_size: int) -> None:
    for image in bpy.data.images:
        if image.size[0] <= texture_size and image.size[1] <= texture_size:
            continue
        image.scale(texture_size, texture_size)


def _clear_scene() -> None:
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete()


def _face_count(meshes: list[bpy.types.Object]) -> int:
    """Count mesh faces (Blender DECIMATE ratio is face-based, not corner-sum)."""
    total = 0
    for obj in meshes:
        total += len(obj.data.polygons)
    return total


def _cli(argv: list[str] | None = None) -> int:
    import sys

    args = argv if argv is not None else sys.argv
    if "--" in args:
        args = args[args.index("--") + 1 :]
    if len(args) < 3:
        raise SystemExit("usage: sculpt_to_game_mesh.py <input.glb> <output.glb> <tri_target> [texture_size]")
    tex = int(args[3]) if len(args) > 3 else TEXTURE_SIZE
    sculpt_to_game_mesh(Path(args[0]), Path(args[1]), int(args[2]), tex)
    return 0


if __name__ == "__main__":
    import sys

    raise SystemExit(_cli())
