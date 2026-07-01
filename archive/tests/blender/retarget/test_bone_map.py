"""Contract: CMU → TBG bone map covers RIG-SPEC deformation bones."""

from tools.blender.animation.retarget.bone_map import load_bone_map, required_tbg_bones


def test_bone_map_covers_rig_spec_deformation_bones():
    bone_map = load_bone_map()
    mapped = set(bone_map["mapping"].keys())
    for bone in required_tbg_bones():
        assert bone in mapped, f"missing map entry for {bone}"


def test_bone_map_has_cmu_source_for_each_tbg_bone():
    bone_map = load_bone_map()
    for tbg_bone, cmu_joint in bone_map["mapping"].items():
        assert cmu_joint, f"empty CMU joint for {tbg_bone}"
        assert isinstance(cmu_joint, str)
