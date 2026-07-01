#!/usr/bin/env python3
"""Bake MB-Lab-adaptive hair mesh (morph targets) for library/parts/hair/."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

try:
    import bpy
except ImportError as exc:
    raise SystemExit("bake_mblab_hair requires Blender bpy") from exc

REPO = Path(__file__).resolve().parents[3]
LIBRARY = REPO / "content" / "characters" / "library"
SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))

from export_base_body import export_glb_objects  # noqa: E402
from mblab_enable import ensure_mblab_addon  # noqa: E402
from mblab_morph import apply_morphs  # noqa: E402
from mblab_morph_bake import bake_hair_morph_targets, fit_hair_to_head, neutral_morphs  # noqa: E402

CONFIG_PATH = LIBRARY / "mblab_studio_morphs.json"
HAIR_ID = "hair_mblab_01"
HAIR_DIR = LIBRARY / "parts" / "hair"


def _load_sliders() -> list[dict[str, Any]]:
    cfg = json.loads(CONFIG_PATH.read_text(encoding="utf-8"))
    return [s for s in cfg.get("sliders", []) if not s.get("browserOnly")]


def _body_mesh() -> bpy.types.Object:
    mesh = next((o for o in bpy.context.scene.objects if o.type == "MESH" and o.get("manuellab_id")), None)
    if mesh is None:
        mesh = next((o for o in bpy.context.scene.objects if o.type == "MESH"), None)
    if mesh is None:
        raise RuntimeError("no MB-Lab body mesh in scene")
    return mesh


def _hair_material() -> bpy.types.Material:
    mat = bpy.data.materials.get("TBG_Hair") or bpy.data.materials.new("TBG_Hair")
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    links = mat.node_tree.links
    nodes.clear()
    out = nodes.new("ShaderNodeOutputMaterial")
    bsdf = nodes.new("ShaderNodeBsdfPrincipled")
    bsdf.inputs["Base Color"].default_value = (0.12, 0.08, 0.05, 1.0)
    bsdf.inputs["Roughness"].default_value = 0.72
    noise = nodes.new("ShaderNodeTexNoise")
    noise.inputs["Scale"].default_value = 48.0
    noise.inputs["Detail"].default_value = 6.0
    bump = nodes.new("ShaderNodeBump")
    bump.inputs["Strength"].default_value = 0.18
    links.new(noise.outputs["Fac"], bump.inputs["Height"])
    links.new(bump.outputs["Normal"], bsdf.inputs["Normal"])
    links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])
    return mat


def _build_hair_volume() -> bpy.types.Object:
    bpy.ops.mesh.primitive_uv_sphere_add(segments=32, ring_count=16, radius=0.09, location=(0.0, 0.0, 0.07))
    cap = bpy.context.active_object
    cap.name = HAIR_ID
    cap.scale = (1.4, 1.2, 0.95)
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    solid = cap.modifiers.new("Solidify", "SOLIDIFY")
    solid.thickness = 0.016
    bpy.context.view_layer.objects.active = cap
    bpy.ops.object.modifier_apply(modifier=solid.name)
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=(0.0, -0.05, -0.03))
    back = bpy.context.active_object
    back.scale = (0.2, 0.08, 0.32)
    bpy.ops.object.transform_apply(scale=True)
    bpy.ops.object.select_all(action="DESELECT")
    cap.select_set(True)
    back.select_set(True)
    bpy.context.view_layer.objects.active = cap
    bpy.ops.object.join()
    hair = bpy.context.active_object
    hair.name = HAIR_ID
    hair.data.materials.append(_hair_material())
    bpy.ops.object.shade_smooth()
    return hair


def bake_hair_part() -> dict[str, Any]:
    ensure_mblab_addon()
    sliders = _load_sliders()
    body = _body_mesh()
    apply_morphs(neutral_morphs(sliders))
    hair = _build_hair_volume()
    fit_hair_to_head(hair, body)
    targets = bake_hair_morph_targets(hair, body, sliders)
    bpy.ops.object.select_all(action="DESELECT")
    hair.select_set(True)
    bpy.context.view_layer.objects.active = hair
    bpy.ops.object.origin_set(type="ORIGIN_GEOMETRY", center="BOUNDS")
    out = HAIR_DIR / f"{HAIR_ID}.glb"
    export_glb_objects(out, [hair])
    manifest = {
        "id": HAIR_ID,
        "category": "hair",
        "baseCompat": list(("male", "female", "small", "large")),
        "socket": "TBG_Head",
        "mesh": f"{HAIR_ID}.glb",
        "thumbnail": f"{HAIR_ID}.webp",
        "morphTargets": targets,
        "source": "mblab_shrinkwrap_bake",
    }
    HAIR_DIR.mkdir(parents=True, exist_ok=True)
    (HAIR_DIR / f"{HAIR_ID}.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    _write_thumb(HAIR_DIR / f"{HAIR_ID}.webp")
    return {"hair": str(out), "morphTargets": len(targets), "manifest": str(HAIR_DIR / f"{HAIR_ID}.json")}


def _write_thumb(path: Path) -> None:
    try:
        from PIL import Image
    except ImportError:
        return
    Image.new("RGB", (128, 128), (45, 32, 22)).save(path, format="WEBP")


def main(argv: list[str] | None = None) -> int:
    print(json.dumps(bake_hair_part()))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
