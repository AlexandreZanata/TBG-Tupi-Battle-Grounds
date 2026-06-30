"""Bpy-side skin weight metrics and automatic weight painting."""

from __future__ import annotations

from collections import Counter
from dataclasses import dataclass

try:
    import bpy
except ImportError as exc:
    raise SystemExit("rig_weight_eval requires Blender bpy") from exc

from rig_weight_paint import MIN_BONE_WEIGHT, SINGLE_BONE_DOMINANCE, _ensure_armature_modifier


@dataclass(frozen=True)
class BpyWeightMetrics:
    dominant_bone: str | None
    dominant_share: float
    unweighted_vertices: int
    bones_missing: tuple[str, ...]

    @property
    def passes_gate(self) -> bool:
        return (
            self.unweighted_vertices == 0
            and self.dominant_share < SINGLE_BONE_DOMINANCE
            and not self.bones_missing
        )


def paint_automatic_weights(mesh_obj: bpy.types.Object, arm_obj: bpy.types.Object) -> None:
    bpy.ops.object.select_all(action="DESELECT")
    mesh_obj.select_set(True)
    arm_obj.select_set(True)
    bpy.context.view_layer.objects.active = mesh_obj
    bpy.ops.object.parent_set(type="ARMATURE_AUTO")
    _ensure_armature_modifier(mesh_obj, arm_obj)


def evaluate_bpy_weights(mesh_obj: bpy.types.Object, arm_obj: bpy.types.Object) -> BpyWeightMetrics:
    counts: Counter[str] = Counter()
    unweighted = 0
    bone_max: dict[str, float] = {
        bone.name: 0.0 for bone in arm_obj.data.bones if bone.name.startswith("TBG_")
    }
    for vert in mesh_obj.data.vertices:
        if not vert.groups:
            unweighted += 1
            continue
        total = sum(group.weight for group in vert.groups)
        if total <= 1e-8:
            unweighted += 1
            continue
        dominant = max(vert.groups, key=lambda item: item.weight)
        name = mesh_obj.vertex_groups[dominant.group].name
        counts[name] += 1
        for group in vert.groups:
            weight = group.weight / total
            vg_name = mesh_obj.vertex_groups[group.group].name
            if vg_name in bone_max:
                bone_max[vg_name] = max(bone_max[vg_name], weight)
    if not counts:
        missing = tuple(name for name, peak in bone_max.items() if peak < MIN_BONE_WEIGHT)
        return BpyWeightMetrics(None, 1.0, unweighted, missing)
    dominant_bone, dominant_count = counts.most_common(1)[0]
    share = dominant_count / max(1, len(mesh_obj.data.vertices))
    missing = tuple(name for name, peak in bone_max.items() if peak < MIN_BONE_WEIGHT)
    return BpyWeightMetrics(dominant_bone, share, unweighted, missing)


def is_better_metrics(candidate: BpyWeightMetrics, current: BpyWeightMetrics) -> bool:
    if candidate.passes_gate and not current.passes_gate:
        return True
    if current.passes_gate and not candidate.passes_gate:
        return False
    if candidate.dominant_share != current.dominant_share:
        return candidate.dominant_share < current.dominant_share
    return len(candidate.bones_missing) < len(current.bones_missing)


def capture_vertex_weights(mesh_obj: bpy.types.Object) -> dict[str, list[tuple[int, float]]]:
    captured: dict[str, list[tuple[int, float]]] = {group.name: [] for group in mesh_obj.vertex_groups}
    for vert_index, vert in enumerate(mesh_obj.data.vertices):
        for item in vert.groups:
            captured[mesh_obj.vertex_groups[item.group].name].append((vert_index, item.weight))
    return captured


def apply_captured_weights(
    mesh_obj: bpy.types.Object,
    arm_obj: bpy.types.Object,
    captured: dict[str, list[tuple[int, float]]],
) -> None:
    mesh_obj.vertex_groups.clear()
    groups = {name: mesh_obj.vertex_groups.new(name=name) for name in captured}
    for name, entries in captured.items():
        for vert_index, weight in entries:
            if weight > 1e-6:
                groups[name].add([vert_index], weight, "REPLACE")
    _ensure_armature_modifier(mesh_obj, arm_obj)
    mesh_obj.parent = arm_obj
