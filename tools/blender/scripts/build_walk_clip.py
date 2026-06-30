#!/usr/bin/env python3
"""Export Walk clip GLB for a body-class library (Blender bpy)."""

from __future__ import annotations

import sys
from pathlib import Path

_SCRIPTS_DIR = Path(__file__).resolve().parent
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))

try:
    import bpy
except ImportError as exc:
    raise SystemExit("build_walk_clip requires Blender bpy") from exc

from walk_cycle_builder import bake_walk_action  # noqa: E402
from walk_cycle_profiles import resolve_walk_profile  # noqa: E402

REPO_ROOT = Path(__file__).resolve().parents[3]
TEMPLATES = REPO_ROOT / "tools" / "blender" / "templates"


def export_walk_clip(
    body_class: str,
    output_path: Path,
    *,
    unit_id: str | None = None,
) -> Path:
    template = TEMPLATES / f"{body_class}_template.blend"
    if not template.is_file():
        raise FileNotFoundError(f"missing template: {template} — run tools/blender/setup.sh")

    bpy.ops.wm.open_mainfile(filepath=str(template))
    arm_obj = next(obj for obj in bpy.context.scene.objects if obj.type == "ARMATURE")
    profile = resolve_walk_profile(unit_id, body_class)
    bake_walk_action(arm_obj, profile, "Walk")

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


def _cli(argv: list[str]) -> int:
    if not argv:
        raise SystemExit("usage: build_walk_clip.py <bodyClass> [unit_id]")
    body_class = argv[0]
    unit_id = argv[1] if len(argv) > 1 else None
    if unit_id:
        out = REPO_ROOT / "content/characters/animations/clips/units" / unit_id / "Walk.glb"
    else:
        out = REPO_ROOT / "content/characters/animations/clips" / body_class / "Walk.glb"
    export_walk_clip(body_class, out, unit_id=unit_id)
    print(f"Exported walk clip: {out}")
    return 0


if __name__ == "__main__":
    args = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else sys.argv[1:]
    raise SystemExit(_cli(args))
