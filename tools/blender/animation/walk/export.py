"""Export Walk clip GLB for a body-class or unit-specific library path."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

from tools.blender.animation.walk.bake import bake_walk_action
from tools.blender.animation.walk.profiles import resolve_walk_profile
from tools.blender.package_path import REPO_ROOT

if TYPE_CHECKING:
    import bpy

TEMPLATES = REPO_ROOT / "tools" / "blender" / "templates"


def export_walk_clip(
    body_class: str,
    output_path: Path,
    *,
    unit_id: str | None = None,
    root_motion: bool = False,
) -> Path:
    import bpy

    template = TEMPLATES / f"{body_class}_template.blend"
    if not template.is_file():
        raise FileNotFoundError(f"missing template: {template} — run tools/blender/setup.sh")

    bpy.ops.wm.open_mainfile(filepath=str(template))
    arm_obj = next(obj for obj in bpy.context.scene.objects if obj.type == "ARMATURE")
    profile = resolve_walk_profile(unit_id, body_class)
    bake_walk_action(arm_obj, profile, root_motion=root_motion)

    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    output_path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="DESELECT")
    arm_obj.select_set(True)
    if meshes:
        meshes[0].select_set(True)
    bpy.context.view_layer.objects.active = arm_obj

    export_kwargs = {
        "filepath": str(output_path),
        "export_format": "GLB",
        "use_selection": True,
        "export_apply": False,
        "export_skins": True,
        "export_animations": True,
        "export_yup": True,
    }
    if hasattr(bpy.ops.export_scene.gltf, "export_animation_mode"):
        export_kwargs["export_animation_mode"] = "ACTIONS"
    bpy.ops.export_scene.gltf(**export_kwargs)
    return output_path


def default_output_path(body_class: str, unit_id: str | None = None, *, root_motion: bool = False) -> Path:
    clip = "WalkRoot" if root_motion else "Walk"
    if unit_id:
        return REPO_ROOT / "content/characters/animations/clips/units" / unit_id / f"{clip}.glb"
    return REPO_ROOT / "content/characters/animations/clips" / body_class / f"{clip}.glb"
