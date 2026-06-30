#!/usr/bin/env python3
"""Attach TBG body-class armature to a low-poly mesh (Blender bpy)."""

from __future__ import annotations

from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("attach_tbg_rig requires Blender bpy") from exc

TEMPLATE_DIR = Path(__file__).resolve().parents[1] / "templates"


def attach_tbg_rig(low_mesh_path: Path, body_class: str, output_path: Path) -> Path:
    template = TEMPLATE_DIR / f"{body_class}_template.blend"
    if not template.is_file():
        raise FileNotFoundError(
            f"missing rig template: {template} — run tools/blender/setup.sh"
        )

    bpy.ops.wm.open_mainfile(filepath=str(template))
    template_body = f"{body_class}_Body"
    template_mesh = bpy.data.objects.get(template_body)
    if template_mesh is not None:
        bpy.data.objects.remove(template_mesh, do_unlink=True)

    bpy.ops.import_scene.gltf(filepath=str(low_mesh_path))

    imported = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    armatures = [obj for obj in bpy.context.scene.objects if obj.type == "ARMATURE"]
    if not imported or not armatures:
        raise RuntimeError("expected imported mesh and template armature")

    mesh_obj = imported[0]
    arm_obj = armatures[0]
    mesh_obj.parent = arm_obj
    bpy.context.view_layer.objects.active = mesh_obj
    bpy.ops.object.parent_set(type="ARMATURE_AUTO")

    output_path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="DESELECT")
    mesh_obj.select_set(True)
    arm_obj.select_set(True)
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.export_scene.gltf(
        filepath=str(output_path),
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_animations=False,
        export_yup=True,
    )
    return output_path
