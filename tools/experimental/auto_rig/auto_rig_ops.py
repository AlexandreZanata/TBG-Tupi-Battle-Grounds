"""Experimental auto-rig: template armature + Blender automatic weights only."""

from __future__ import annotations

import sys
from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("auto_rig_ops requires Blender bpy") from exc

_REPO = Path(__file__).resolve().parents[3]
_BLENDER_SCRIPTS = _REPO / "tools/blender/scripts"
if str(_BLENDER_SCRIPTS) not in sys.path:
    sys.path.insert(0, str(_BLENDER_SCRIPTS))

from auto_rig_align import align_armature_to_mesh, widen_bone_envelopes  # noqa: E402
from rig_weight_bind import bind_mesh_with_weight_fallback  # noqa: E402
from rig_weight_eval import evaluate_bpy_weights, paint_automatic_weights  # noqa: E402
from rig_weight_paint import clear_mesh_skin, paint_proximity_weights  # noqa: E402

BindMode = str
BIND_AUTOMATIC = "automatic"
BIND_PROXIMITY = "proximity"
BIND_FALLBACK = "fallback"

TEMPLATE_DIR = _REPO / "tools/blender/templates"


def load_template_armature(body_class: str) -> bpy.types.Object:
    template = TEMPLATE_DIR / f"{body_class}_template.blend"
    if not template.is_file():
        raise FileNotFoundError(f"missing rig template: {template} — run tools/blender/setup.sh")

    bpy.ops.wm.open_mainfile(filepath=str(template))
    proxy = bpy.data.objects.get(f"{body_class}_Body")
    if proxy is not None:
        bpy.data.objects.remove(proxy, do_unlink=True)

    armatures = [obj for obj in bpy.context.scene.objects if obj.type == "ARMATURE"]
    if not armatures:
        raise RuntimeError(f"template missing armature: {template}")
    return armatures[0]


def bind_mesh(
    mesh_obj: bpy.types.Object,
    arm_obj: bpy.types.Object,
    *,
    mode: BindMode = BIND_FALLBACK,
) -> str:
    align_armature_to_mesh(arm_obj, mesh_obj)
    if mode == BIND_FALLBACK:
        return bind_mesh_with_weight_fallback(mesh_obj, arm_obj, widen_envelopes=widen_bone_envelopes)
    widen_bone_envelopes(arm_obj, mesh_obj)
    clear_mesh_skin(mesh_obj)
    if mode == BIND_PROXIMITY:
        paint_proximity_weights(mesh_obj, arm_obj)
        mesh_obj.parent = arm_obj
        return BIND_PROXIMITY
    paint_automatic_weights(mesh_obj, arm_obj)
    mesh_obj.parent = arm_obj
    return BIND_AUTOMATIC


def export_skinned_glb(
    mesh_obj: bpy.types.Object,
    arm_obj: bpy.types.Object,
    output_path: Path,
) -> Path:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="DESELECT")
    mesh_obj.select_set(True)
    arm_obj.select_set(True)
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.export_scene.gltf(
        filepath=str(output_path),
        export_format="GLB",
        use_selection=True,
        export_apply=False,
        export_skins=True,
        export_animations=False,
        export_yup=True,
    )
    return output_path


def weight_metrics(mesh_obj: bpy.types.Object, arm_obj: bpy.types.Object):
    return evaluate_bpy_weights(mesh_obj, arm_obj)
