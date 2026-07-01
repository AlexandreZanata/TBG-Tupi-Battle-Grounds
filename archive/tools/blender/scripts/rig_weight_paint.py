"""Anatomical proximity skin weights for TBG armatures (Blender bpy)."""

from __future__ import annotations

from collections import Counter
from dataclasses import dataclass

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
MIN_BONE_WEIGHT = 0.01

REGION_TAGS: dict[str, tuple[str, ...]] = {
    "head": ("Head", "Neck"),
    "torso": ("Spine", "Root"),
    "arm": ("Shoulder", "UpperArm", "LowerArm", "Hand"),
    "leg": ("UpperLeg", "LowerLeg", "Foot", "Toe"),
}


@dataclass(frozen=True)
class AnatomicalLandmarks:
    center_x: float
    hip_y: float
    shoulder_y: float
    neck_y: float
    torso_half_width: float


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
    """Assign up to 4 bones per vertex using anatomical regions + proximity."""
    mesh_obj.vertex_groups.clear()
    groups = {bone.name: mesh_obj.vertex_groups.new(name=bone.name) for bone in arm_obj.data.bones}
    segments = _bone_segments(arm_obj)
    landmarks = _landmarks_from_armature(arm_obj, mesh_obj)

    for vert_index, vert in enumerate(mesh_obj.data.vertices):
        world = mesh_obj.matrix_world @ vert.co
        region = _classify_region(world, landmarks)
        side = ".L" if world.x < landmarks.center_x else ".R"
        region_segments = _segments_for_region(segments, region, side)
        influences = _top_influences(world, region_segments)
        _apply_influences(groups, vert_index, influences)

    from rig_weight_coverage import ensure_bone_coverage  # noqa: PLC0415

    ensure_bone_coverage(mesh_obj, groups, segments, arm_obj)
    _ensure_armature_modifier(mesh_obj, arm_obj)


def _landmarks_from_armature(arm_obj: bpy.types.Object, mesh_obj: bpy.types.Object) -> AnatomicalLandmarks:
    bones = arm_obj.data.bones
    hip_y = (arm_obj.matrix_world @ bones["TBG_Spine"].head_local).y
    shoulder_y = (arm_obj.matrix_world @ bones["TBG_Shoulder.L"].head_local).y
    neck_y = (arm_obj.matrix_world @ bones["TBG_Neck"].head_local).y
    min_x, max_x, _, _, _, _ = _mesh_world_bounds(mesh_obj)
    center_x = (min_x + max_x) * 0.5
    torso_half_width = max(max_x - min_x, 0.01) * 0.22
    return AnatomicalLandmarks(center_x, hip_y, shoulder_y, neck_y, torso_half_width)


def _mesh_world_bounds(mesh_obj: bpy.types.Object) -> tuple[float, float, float, float, float, float]:
    corners = [mesh_obj.matrix_world @ Vector(corner) for corner in mesh_obj.bound_box]
    xs = [corner.x for corner in corners]
    ys = [corner.y for corner in corners]
    zs = [corner.z for corner in corners]
    return min(xs), max(xs), min(ys), max(ys), min(zs), max(zs)


def _classify_region(point: Vector, landmarks: AnatomicalLandmarks) -> str:
    lateral = abs(point.x - landmarks.center_x)
    if point.y >= landmarks.neck_y:
        return "head"
    if point.y >= landmarks.shoulder_y and lateral > landmarks.torso_half_width:
        return "arm"
    if point.y < landmarks.hip_y * 0.95:
        return "leg"
    return "torso"


def _segments_for_region(
    segments: list[tuple[str, Vector, Vector]],
    region: str,
    side: str,
) -> list[tuple[str, Vector, Vector]]:
    tags = REGION_TAGS[region]
    picked = [segment for segment in segments if any(tag in segment[0] for tag in tags)]
    if region in ("arm", "leg"):
        sided = [segment for segment in picked if segment[0].endswith(side)]
        picked = sided if sided else picked
    return picked if picked else segments


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
