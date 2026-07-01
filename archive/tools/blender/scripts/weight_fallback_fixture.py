#!/usr/bin/env python3
"""Headless fixture: anatomical proximity beats automatic weights on limb proxy."""

from __future__ import annotations

import json
import sys
from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("weight_fallback_fixture requires Blender bpy") from exc

try:
    from mathutils import Vector
except ImportError as exc:
    raise SystemExit("weight_fallback_fixture requires mathutils") from exc

_SCRIPTS = Path(__file__).resolve().parent
if str(_SCRIPTS) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS))

from rig_weight_bind import bind_mesh_with_weight_fallback  # noqa: E402
from rig_weight_eval import evaluate_bpy_weights  # noqa: E402

TEMPLATE = _SCRIPTS.parent / "templates" / "HumanoidLight_template.blend"


def _widen_envelopes(arm_obj: bpy.types.Object, mesh_obj: bpy.types.Object) -> None:
    corners = [mesh_obj.matrix_world @ Vector(corner) for corner in mesh_obj.bound_box]
    ys = [corner.y for corner in corners]
    radius = max(max(ys) - min(ys), 0.01) * 0.12
    for bone in arm_obj.data.bones:
        bone.envelope_distance = radius


def _add_limb_proxy(name: str, location: tuple[float, float, float], scale: tuple[float, float, float]) -> None:
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=location)
    obj = bpy.context.object
    obj.name = name
    obj.scale = scale
    bpy.ops.object.transform_apply(scale=True)


def main() -> int:
    if not TEMPLATE.is_file():
        raise SystemExit(f"missing template: {TEMPLATE}")

    bpy.ops.wm.open_mainfile(filepath=str(TEMPLATE))
    template_body = bpy.data.objects.get("HumanoidLight_Body")
    if template_body is not None:
        bpy.data.objects.remove(template_body, do_unlink=True)

    _add_limb_proxy("FixtureTorso", (0.0, 1.0, 0.0), (0.35, 0.55, 0.22))
    _add_limb_proxy("FixtureHead", (0.0, 1.55, 0.0), (0.22, 0.22, 0.22))
    _add_limb_proxy("FixtureLegL", (-0.14, 0.22, 0.0), (0.16, 0.55, 0.16))
    _add_limb_proxy("FixtureLegR", (0.14, 0.22, 0.0), (0.16, 0.55, 0.16))
    _add_limb_proxy("FixtureFootL", (-0.14, 0.04, 0.05), (0.14, 0.08, 0.22))
    _add_limb_proxy("FixtureFootR", (0.14, 0.04, 0.05), (0.14, 0.08, 0.22))
    _add_limb_proxy("FixtureArmL", (-0.42, 1.05, 0.0), (0.12, 0.38, 0.12))
    _add_limb_proxy("FixtureArmR", (0.42, 1.05, 0.0), (0.12, 0.38, 0.12))

    bpy.ops.object.select_all(action="DESELECT")
    for name in (
        "FixtureTorso",
        "FixtureHead",
        "FixtureLegL",
        "FixtureLegR",
        "FixtureArmL",
        "FixtureArmR",
        "FixtureFootL",
        "FixtureFootR",
    ):
        bpy.data.objects[name].select_set(True)
    bpy.context.view_layer.objects.active = bpy.data.objects["FixtureTorso"]
    bpy.ops.object.join()
    mesh_obj = bpy.context.object
    mesh_obj.name = "FixtureBody"

    arm_obj = next(obj for obj in bpy.context.scene.objects if obj.type == "ARMATURE")
    winner = bind_mesh_with_weight_fallback(mesh_obj, arm_obj, widen_envelopes=_widen_envelopes)
    metrics = evaluate_bpy_weights(mesh_obj, arm_obj)
    report = {
        "winner": winner,
        "dominant_share": metrics.dominant_share,
        "skew_pass": metrics.dominant_share < 0.85 and metrics.unweighted_vertices == 0,
        "bones_missing": metrics.bones_missing,
    }
    print(json.dumps(report))
    ok = winner == "proximity" and report["skew_pass"] and not report["bones_missing"]
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
