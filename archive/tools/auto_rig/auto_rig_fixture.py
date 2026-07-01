#!/usr/bin/env python3
"""Debug fixture: automatic weights on simple limb proxy in template scene."""

from __future__ import annotations

import json
import sys
from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("auto_rig_fixture requires Blender bpy") from exc

try:
    from mathutils import Vector
except ImportError as exc:
    raise SystemExit("auto_rig_fixture requires mathutils") from exc

_REPO = Path(__file__).resolve().parents[3]
_EXPERIMENTAL = Path(__file__).resolve().parent
_BLENDER_SCRIPTS = _REPO / "tools/blender/scripts"
for path in (_REPO, _EXPERIMENTAL, _BLENDER_SCRIPTS):
    if str(path) not in sys.path:
        sys.path.insert(0, str(path))

from auto_rig_ops import bind_mesh, export_skinned_glb, load_template_armature, weight_metrics  # noqa: E402
from tools.blender.package_path import resolve_repo_path  # noqa: E402


def _add_limb_proxy(name: str, location: tuple[float, float, float], scale: tuple[float, float, float]) -> None:
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=location)
    obj = bpy.context.object
    obj.name = name
    obj.scale = scale
    bpy.ops.object.transform_apply(scale=True)


def main() -> int:
    output = resolve_repo_path(".local/tmp/auto_rig/fixture_auto.glb")
    arm_obj = load_template_armature("HumanoidLight")

    _add_limb_proxy("FixtureTorso", (0.0, 1.0, 0.0), (0.35, 0.55, 0.22))
    _add_limb_proxy("FixtureHead", (0.0, 1.55, 0.0), (0.22, 0.22, 0.22))
    _add_limb_proxy("FixtureLegL", (-0.14, 0.22, 0.0), (0.16, 0.55, 0.16))
    _add_limb_proxy("FixtureLegR", (0.14, 0.22, 0.0), (0.16, 0.55, 0.16))
    _add_limb_proxy("FixtureArmL", (-0.42, 1.05, 0.0), (0.12, 0.38, 0.12))
    _add_limb_proxy("FixtureArmR", (0.42, 1.05, 0.0), (0.12, 0.38, 0.12))

    bpy.ops.object.select_all(action="DESELECT")
    for name in ("FixtureTorso", "FixtureHead", "FixtureLegL", "FixtureLegR", "FixtureArmL", "FixtureArmR"):
        bpy.data.objects[name].select_set(True)
    bpy.context.view_layer.objects.active = bpy.data.objects["FixtureTorso"]
    bpy.ops.object.join()
    mesh_obj = bpy.context.object
    mesh_obj.name = "FixtureBody"

    bind_mesh(mesh_obj, arm_obj, mode="proximity")
    metrics = weight_metrics(mesh_obj, arm_obj)
    export_skinned_glb(mesh_obj, arm_obj, output)
    report = {
        "vertices": len(mesh_obj.data.vertices),
        "groups": len(mesh_obj.vertex_groups),
        "dominant_share": metrics.dominant_share,
        "unweighted_vertices": metrics.unweighted_vertices,
        "weight_gate_pass": metrics.passes_gate,
    }
    print(json.dumps(report))
    return 0 if metrics.passes_gate else 1


if __name__ == "__main__":
    raise SystemExit(main())
