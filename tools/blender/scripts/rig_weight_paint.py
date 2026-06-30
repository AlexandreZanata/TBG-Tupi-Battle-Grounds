"""Proximity envelope skin weights for TBG armatures (Blender bpy)."""

from __future__ import annotations

from collections import Counter

try:
    import bpy
except ImportError as exc:
    raise SystemExit("rig_weight_paint requires Blender bpy") from exc

try:
    from mathutils import Vector
except ImportError as exc:
    raise SystemExit("rig_weight_paint requires mathutils") from exc

MAX_INFLUENCES = 4
SINGLE_BONE_DOMINANCE = 0.85


def weights_skewed_to_one_bone(mesh_obj: bpy.types.Object, threshold: float = SINGLE_BONE_DOMINANCE) -> bool:
    if not mesh_obj.vertex_groups:
        return True
    counts: Counter[str] = Counter()
    for vert in mesh_obj.data.vertices:
        if not vert.groups:
            return True
        dominant = max(vert.groups, key=lambda item: item.weight)
        counts[mesh_obj.vertex_groups[dominant.group].name] += 1
    top_share = counts.most_common(1)[0][1] / max(1, len(mesh_obj.data.vertices))
    return top_share >= threshold


def clear_mesh_skin(mesh_obj: bpy.types.Object) -> None:
    if mesh_obj.parent is not None:
        mesh_obj.parent = None
    mesh_obj.vertex_groups.clear()
    for modifier in list(mesh_obj.modifiers):
        if modifier.type == "ARMATURE":
            mesh_obj.modifiers.remove(modifier)


def paint_proximity_weights(mesh_obj: bpy.types.Object, arm_obj: bpy.types.Object) -> None:
    """Assign up to 4 bones per vertex by inverse distance to bone segments."""
    mesh_obj.vertex_groups.clear()
    groups = {bone.name: mesh_obj.vertex_groups.new(name=bone.name) for bone in arm_obj.data.bones}
    segments = _bone_segments(arm_obj)
    hip_y = (arm_obj.matrix_world @ arm_obj.data.bones["TBG_Spine"].head_local).y

    for vert_index, vert in enumerate(mesh_obj.data.vertices):
        world = mesh_obj.matrix_world @ vert.co
        region_segments = _segments_for_region(segments, world, hip_y)
        influences = _top_influences(world, region_segments)
        _apply_influences(groups, vert_index, influences)

    _ensure_armature_modifier(mesh_obj, arm_obj)


def _segments_for_region(
    segments: list[tuple[str, Vector, Vector]],
    point: Vector,
    hip_y: float,
) -> list[tuple[str, Vector, Vector]]:
    if point.y < hip_y * 0.9:
        leg = [s for s in segments if "Leg" in s[0] or "Foot" in s[0]]
        return leg if leg else segments
    if point.y > hip_y * 1.2:
        upper = [s for s in segments if any(tag in s[0] for tag in ("Arm", "Hand", "Shoulder", "Head", "Neck"))]
        return upper if upper else segments
    torso = [s for s in segments if any(tag in s[0] for tag in ("Spine", "Root", "Shoulder"))]
    return torso if torso else segments


def _bone_segments(arm_obj: bpy.types.Object) -> list[tuple[str, Vector, Vector]]:
    segments: list[tuple[str, Vector, Vector]] = []
    for bone in arm_obj.data.bones:
        head = arm_obj.matrix_world @ bone.head_local
        tail = arm_obj.matrix_world @ bone.tail_local
        segments.append((bone.name, head, tail))
    return segments


def _top_influences(point: Vector, segments: list[tuple[str, Vector, Vector]]) -> list[tuple[str, float]]:
    scored: list[tuple[str, float]] = []
    for name, head, tail in segments:
        dist = _point_segment_distance(point, head, tail)
        scored.append((name, 1.0 / max(dist * dist, 1e-8)))
    scored.sort(key=lambda item: item[1], reverse=True)
    top = scored[:MAX_INFLUENCES]
    total = sum(weight for _, weight in top)
    if total <= 0:
        return [(segments[0][0], 1.0)]
    return [(name, weight / total) for name, weight in top]


def _point_segment_distance(point: Vector, start: Vector, end: Vector) -> float:
    edge = end - start
    length_sq = edge.length_squared
    if length_sq <= 1e-12:
        return (point - start).length
    t = max(0.0, min(1.0, (point - start).dot(edge) / length_sq))
    return (point - (start + edge * t)).length


def _apply_influences(groups: dict, vert_index: int, influences: list[tuple[str, float]]) -> None:
    for name, weight in influences:
        if weight <= 1e-6:
            continue
        groups[name].add([vert_index], weight, "ADD")


def _ensure_armature_modifier(mesh_obj: bpy.types.Object, arm_obj: bpy.types.Object) -> None:
    modifier = mesh_obj.modifiers.get("Armature")
    if modifier is None:
        modifier = mesh_obj.modifiers.new(name="Armature", type="ARMATURE")
    modifier.object = arm_obj
