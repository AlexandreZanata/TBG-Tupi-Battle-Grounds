"""Seed minimum bone coverage after anatomical proximity paint."""

from __future__ import annotations

try:
    import bpy
except ImportError as exc:
    raise SystemExit("rig_weight_coverage requires Blender bpy") from exc

try:
    from mathutils import Vector
except ImportError as exc:
    raise SystemExit("rig_weight_coverage requires mathutils") from exc

from rig_weight_paint import MAX_INFLUENCES, MIN_BONE_WEIGHT, _point_segment_distance

COVERAGE_SEED_WEIGHT = 0.15
_ROOT_PRIORITY = ("TBG_Root", "TBG_Spine", "TBG_Spine1")


def ensure_bone_coverage(
    mesh_obj: bpy.types.Object,
    groups: dict,
    segments: list[tuple[str, Vector, Vector]],
    arm_obj: bpy.types.Object,
) -> None:
    seg_map = {name: (head, tail) for name, head, tail in segments}
    pinned: dict[int, set[str]] = {}
    bones = sorted(
        (bone for bone in arm_obj.data.bones if bone.name.startswith("TBG_")),
        key=lambda bone: (_priority(bone.name), bone.name),
    )
    for bone in bones:
        if _bone_peak_weight(mesh_obj, groups, bone.name) >= MIN_BONE_WEIGHT:
            continue
        segment = seg_map.get(bone.name)
        if segment is None:
            continue
        head, tail = segment
        vert_index = _closest_vertex(mesh_obj, head, tail, reserved=pinned.keys())
        pinned.setdefault(vert_index, set()).add(bone.name)
        _seed_vertex_weight(groups, vert_index, bone.name, COVERAGE_SEED_WEIGHT, pinned[vert_index])


def _priority(bone_name: str) -> int:
    if bone_name in _ROOT_PRIORITY:
        return _ROOT_PRIORITY.index(bone_name)
    return len(_ROOT_PRIORITY)


def _bone_peak_weight(mesh_obj: bpy.types.Object, groups: dict, bone_name: str) -> float:
    group = groups.get(bone_name)
    if group is None:
        return 0.0
    peak = 0.0
    for vert in mesh_obj.data.vertices:
        for item in vert.groups:
            if item.group == group.index:
                peak = max(peak, item.weight)
    return peak


def _closest_vertex(
    mesh_obj: bpy.types.Object,
    head: Vector,
    tail: Vector,
    reserved: set[int],
) -> int:
    ranked: list[tuple[float, int]] = []
    for vert_index, vert in enumerate(mesh_obj.data.vertices):
        world = mesh_obj.matrix_world @ vert.co
        ranked.append((_point_segment_distance(world, head, tail), vert_index))
    ranked.sort(key=lambda item: item[0])
    for _, vert_index in ranked:
        if vert_index not in reserved:
            return vert_index
    return ranked[0][1]


def _seed_vertex_weight(
    groups: dict,
    vert_index: int,
    bone_name: str,
    weight: float,
    pinned: set[str],
) -> None:
    current = _vertex_weights(groups, vert_index)
    for name in pinned:
        current[name] = max(current.get(name, 0.0), COVERAGE_SEED_WEIGHT)
    current[bone_name] = max(current.get(bone_name, 0.0), weight)
    normalized = _normalize_top4(current, pinned)
    for name in groups:
        groups[name].remove([vert_index])
    for name, value in normalized.items():
        if value > 1e-6:
            groups[name].add([vert_index], value, "REPLACE")


def _vertex_weights(groups: dict, vert_index: int) -> dict[str, float]:
    weights: dict[str, float] = {}
    for name, group in groups.items():
        try:
            weights[name] = group.weight(vert_index)
        except RuntimeError:
            continue
    return weights


def _normalize_top4(raw: dict[str, float], required: set[str] | None = None) -> dict[str, float]:
    required = required or set()
    selected: dict[str, float] = {
        name: raw[name] for name in required if name in raw and raw[name] > 1e-6
    }
    slots = max(0, MAX_INFLUENCES - len(selected))
    others = sorted(
        ((name, weight) for name, weight in raw.items() if name not in selected and weight > 1e-6),
        key=lambda item: -item[1],
    )
    for name, weight in others[:slots]:
        selected[name] = weight
    total = sum(selected.values())
    if total <= 0:
        return {}
    return {name: weight / total for name, weight in selected.items()}
