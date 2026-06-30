#!/usr/bin/env python3
"""Embed animation-set library clips into a skinned unit GLB (Blender bpy)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

_TOOLS_DIR = Path(__file__).resolve().parents[2]
if str(_TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(_TOOLS_DIR))

from animation_sets.validation import AnimationSet, clip_library_path, validate_animation_set

try:
    import bpy
except ImportError as exc:
    raise SystemExit("retarget_clip_library requires Blender bpy") from exc


def retarget_clip_library(
    skinned_mesh_path: Path,
    animation_set_id: str,
    output_path: Path,
    repo_root: Path,
) -> Path:
    set_path = repo_root / "content/characters/animation_sets" / f"{animation_set_id}.json"
    defn = AnimationSet.from_dict(json.loads(set_path.read_text(encoding="utf-8")))
    validate_animation_set(defn, repo_root)

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(skinned_mesh_path))

    clip_paths = [
        clip_library_path(repo_root, defn.bodyClass, clip_name)
        for clip_name in dict.fromkeys(defn.clips.values())
    ]
    for clip_path in clip_paths:
        bpy.ops.import_scene.gltf(filepath=str(clip_path))

    output_path.parent.mkdir(parents=True, exist_ok=True)
    export_objects = [
        obj for obj in bpy.context.scene.objects if obj.type in {"MESH", "ARMATURE"}
    ]
    bpy.ops.object.select_all(action="DESELECT")
    for obj in export_objects:
        obj.select_set(True)
    if export_objects:
        bpy.context.view_layer.objects.active = export_objects[0]

    bpy.ops.export_scene.gltf(
        filepath=str(output_path),
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_animations=True,
        export_yup=True,
    )
    return output_path
