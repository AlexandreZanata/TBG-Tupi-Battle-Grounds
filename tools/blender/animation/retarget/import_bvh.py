"""Blender BVH import → TBG GaitPose (bpy only)."""

from __future__ import annotations

from pathlib import Path

from tools.blender.animation.retarget.bone_map import load_bone_map
from tools.blender.animation.walk.types import GaitPose


def sample_gait_pose_from_bvh(bvh_path: Path | str, frame: int, bone_map: dict | None = None) -> GaitPose:
    try:
        import bpy
    except ImportError as exc:
        raise RuntimeError("sample_gait_pose_from_bvh requires Blender bpy") from exc

    cfg = bone_map or load_bone_map()
    arm_obj = _import_bvh_armature(bpy, Path(bvh_path))
    bpy.context.scene.frame_set(frame)
    bpy.context.view_layer.update()
    return _pose_from_armature(arm_obj, cfg)


def _import_bvh_armature(bpy, path: Path):
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete()
    bpy.ops.import_anim.bvh(filepath=str(path), rotate_mode="NATIVE", global_scale=0.01)
    armatures = [obj for obj in bpy.context.scene.objects if obj.type == "ARMATURE"]
    if not armatures:
        raise RuntimeError(f"no armature after BVH import: {path}")
    return armatures[0]


def _pose_from_armature(arm_obj, bone_map: dict) -> GaitPose:
    flex = bone_map["leg_flexion"]
    pose: GaitPose = {}
    for tbg_bone, cmu_joint in bone_map["mapping"].items():
        pbone = arm_obj.pose.bones.get(cmu_joint)
        if pbone is None:
            continue
        if tbg_bone == "TBG_Root":
            hips = arm_obj.pose.bones.get("Hips")
            pose[tbg_bone] = tuple(hips.location) if hips else (0.0, 0.0, 0.0)
            continue
        euler = pbone.matrix_basis.to_euler("XYZ")
        if "Leg" in tbg_bone or "Foot" in tbg_bone:
            scale = flex["right_scale"] if tbg_bone.endswith(".R") else flex["left_scale"]
            if "LowerLeg" in tbg_bone:
                scale = flex["knee_scale"]
            if "Foot" in tbg_bone:
                scale = flex["ankle_scale"]
            pose[tbg_bone] = (scale * euler.x, 0.0, 0.0)
        else:
            pose[tbg_bone] = (euler.x, euler.y, euler.z)
    return pose
