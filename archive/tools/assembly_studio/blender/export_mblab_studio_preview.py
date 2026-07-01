"""Bake decimated studio preview GLB with morph targets for browser sliders."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

try:
    import bpy
except ImportError as exc:
    raise SystemExit("export_mblab_studio_preview requires Blender bpy") from exc

REPO = Path(__file__).resolve().parents[3]
LIBRARY = REPO / "content" / "characters" / "library"
SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))

if str(REPO / "tools") not in sys.path:
    sys.path.insert(0, str(REPO / "tools"))

from export_base_body import export_glb_objects  # noqa: E402
from gltf_export_prep import gltf_export_kwargs  # noqa: E402
from mblab_enable import ensure_mblab_addon  # noqa: E402
from mblab_morph_bake import bake_body_morph_targets  # noqa: E402

CONFIG_PATH = LIBRARY / "mblab_studio_morphs.json"
PREVIEW_DECIMATE_RATIO = 0.04


def _load_config() -> dict[str, Any]:
    return json.loads(CONFIG_PATH.read_text(encoding="utf-8"))


def _bake_sliders() -> list[dict[str, Any]]:
    cfg = _load_config()
    return [s for s in cfg.get("sliders", []) if not s.get("browserOnly")]


def _humanoid_mesh() -> bpy.types.Object:
    mesh = next((o for o in bpy.context.scene.objects if o.type == "MESH" and o.get("manuellab_id")), None)
    if mesh is None:
        mesh = next((o for o in bpy.context.scene.objects if o.type == "MESH"), None)
    if mesh is None:
        raise RuntimeError("no mesh for studio preview bake")
    return mesh


def _decimate_for_preview(mesh_obj: bpy.types.Object) -> None:
    mod = mesh_obj.modifiers.new("TBG_PreviewDecimate", "DECIMATE")
    mod.ratio = PREVIEW_DECIMATE_RATIO
    bpy.context.view_layer.objects.active = mesh_obj
    mesh_obj.select_set(True)
    bpy.ops.object.modifier_apply(modifier=mod.name)


def bake_studio_preview_mesh(mesh_obj: bpy.types.Object) -> list[str]:
    """Bake tbg_* morph targets on mesh_obj (duplicate — do not run on library body)."""
    ensure_mblab_addon()
    sliders = _bake_sliders()
    created = bake_body_morph_targets(mesh_obj, sliders)
    _decimate_for_preview(mesh_obj)
    return created


def export_studio_preview(base_id: str) -> dict[str, Any]:
    source = _humanoid_mesh()
    bpy.ops.object.select_all(action="DESELECT")
    source.select_set(True)
    bpy.context.view_layer.objects.active = source
    bpy.ops.object.duplicate()
    preview = bpy.context.active_object
    preview.name = "tbg_studio_preview"
    try:
        targets = bake_studio_preview_mesh(preview)
        arms = [o for o in bpy.context.scene.objects if o.type == "ARMATURE"]
        out = LIBRARY / "bases" / base_id / "preview.glb"
        export_glb_objects(out, [preview] + arms)
    finally:
        bpy.data.objects.remove(preview, do_unlink=True)
    manifest = {
        "baseId": base_id,
        "previewUrl": f"/api/assembly/bases/{base_id}/preview",
        "morphTargets": targets,
        "decimateRatio": PREVIEW_DECIMATE_RATIO,
        "exportMorph": gltf_export_kwargs().get("export_morph", True),
    }
    path = LIBRARY / "bases" / base_id / "preview_manifest.json"
    path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return manifest


def main(argv: list[str] | None = None) -> int:
    import argparse

    args_list = argv if argv is not None else sys.argv
    if "--" in args_list:
        args_list = args_list[args_list.index("--") + 1 :]
    parser = argparse.ArgumentParser(description="Bake studio preview.glb with morph targets")
    parser.add_argument("--base-id", required=True, choices=["male", "female", "small", "large"])
    args = parser.parse_args(args_list)
    print(json.dumps(export_studio_preview(args.base_id)))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
