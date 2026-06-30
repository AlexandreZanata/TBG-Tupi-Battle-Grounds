"""Contract: Mixamo → TBG bone map covers RIG-SPEC deformation bones."""

from tools.blender.animation.retarget.bone_map import REQUIRED_TBG_BONES
from tools.blender.animation.retarget.mixamo_bone_map import (
    load_mixamo_bone_map,
    normalize_mixamo_bone_name,
    required_tbg_bones_with_toes,
    tbg_bone_for_mixamo,
)


def test_mixamo_bone_map_covers_rig_spec_deformation_bones():
    bone_map = load_mixamo_bone_map()
    mapped = set(bone_map["mapping"].keys())
    if bone_map.get("root_policy") == "inject_from_spine":
        assert "TBG_Root" not in mapped
        assert bone_map["mapping"]["TBG_Spine"] == "Hips"
    for bone in required_tbg_bones_with_toes():
        if bone == "TBG_Root" and bone_map.get("root_policy") == "inject_from_spine":
            continue
        assert bone in mapped, f"missing map entry for {bone}"


def test_mixamo_bone_map_has_source_for_each_tbg_bone():
    bone_map = load_mixamo_bone_map()
    for tbg_bone, mixamo_joint in bone_map["mapping"].items():
        assert mixamo_joint, f"empty Mixamo joint for {tbg_bone}"
        assert isinstance(mixamo_joint, str)


def test_mixamo_bone_map_covers_required_without_toes():
    bone_map = load_mixamo_bone_map()
    mapped = set(bone_map["mapping"].keys())
    for bone in REQUIRED_TBG_BONES:
        if bone == "TBG_Root" and bone_map.get("root_policy") == "inject_from_spine":
            continue
        assert bone in mapped


def test_normalize_mixamo_prefixed_bone_names():
    assert normalize_mixamo_bone_name("mixamorig:Hips") == "Hips"
    assert normalize_mixamo_bone_name("mixamorig_Hips") == "Hips"


def test_tbg_bone_for_mixamo_aliases():
    bone_map = load_mixamo_bone_map()
    assert tbg_bone_for_mixamo(bone_map, "mixamorig:Hips") == "TBG_Spine"
    assert tbg_bone_for_mixamo(bone_map, "LeftArm") == "TBG_UpperArm.L"
