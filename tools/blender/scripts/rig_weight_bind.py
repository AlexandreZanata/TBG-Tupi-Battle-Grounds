"""Bind mesh to TBG armature with proximity vs automatic weight fallback."""

from __future__ import annotations

try:
    import bpy
except ImportError as exc:
    raise SystemExit("rig_weight_bind requires Blender bpy") from exc

from rig_weight_paint import clear_mesh_skin, paint_proximity_weights
from rig_weight_eval import (
    apply_captured_weights,
    capture_vertex_weights,
    evaluate_bpy_weights,
    is_better_metrics,
    paint_automatic_weights,
)


def bind_mesh_with_weight_fallback(
    mesh_obj: bpy.types.Object,
    arm_obj: bpy.types.Object,
    *,
    widen_envelopes,
) -> str:
    """Return winning method name: ``proximity`` or ``automatic``."""
    widen_envelopes(arm_obj, mesh_obj)

    clear_mesh_skin(mesh_obj)
    paint_proximity_weights(mesh_obj, arm_obj)
    prox_metrics = evaluate_bpy_weights(mesh_obj, arm_obj)
    prox_weights = capture_vertex_weights(mesh_obj)

    clear_mesh_skin(mesh_obj)
    paint_automatic_weights(mesh_obj, arm_obj)
    auto_metrics = evaluate_bpy_weights(mesh_obj, arm_obj)
    auto_weights = capture_vertex_weights(mesh_obj)

    if is_better_metrics(auto_metrics, prox_metrics):
        winner = "automatic"
    else:
        winner = "proximity"
        clear_mesh_skin(mesh_obj)
        apply_captured_weights(mesh_obj, arm_obj, prox_weights)

    print(
        f"rig_weight_bind: winner={winner} "
        f"prox_share={prox_metrics.dominant_share:.2f} "
        f"auto_share={auto_metrics.dominant_share:.2f}"
    )
    mesh_obj.parent = arm_obj
    return winner
