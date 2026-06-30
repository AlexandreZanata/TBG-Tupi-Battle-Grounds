#!/usr/bin/env python3
"""Headless fixture: weight fallback picks lower dominance skew."""

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


def main() -> int:
    if not TEMPLATE.is_file():
        raise SystemExit(f"missing template: {TEMPLATE}")

    bpy.ops.wm.open_mainfile(filepath=str(TEMPLATE))
    template_body = bpy.data.objects.get("HumanoidLight_Body")
    if template_body is not None:
        bpy.data.objects.remove(template_body, do_unlink=True)

    bpy.ops.mesh.primitive_cylinder_add(radius=0.28, depth=1.65, location=(0.0, 0.82, 0.0))
    mesh_obj = bpy.context.object
    mesh_obj.name = "FixtureBody"
    arm_obj = next(obj for obj in bpy.context.scene.objects if obj.type == "ARMATURE")

    winner = bind_mesh_with_weight_fallback(mesh_obj, arm_obj, widen_envelopes=_widen_envelopes)
    metrics = evaluate_bpy_weights(mesh_obj, arm_obj)
    report = {
        "winner": winner,
        "dominant_share": metrics.dominant_share,
        "skew_pass": metrics.dominant_share < 0.85 and metrics.unweighted_vertices == 0,
    }
    print(json.dumps(report))
    ok = winner == "proximity" and report["skew_pass"]
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
