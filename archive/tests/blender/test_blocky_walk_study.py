"""Contract: blocky walk study character assets."""

from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
OPS = REPO / "tools/blender/scripts/blocky_human_ops.py"
DEFN = REPO / "content/characters/definitions/demo_blocky_walk_01.json"


def test_blocky_parts_cover_walk_bones():
    text = OPS.read_text(encoding="utf-8")
    for bone in ("TBG_UpperLeg.L", "TBG_LowerLeg.L", "TBG_Foot.L", "TBG_Head", "ANATOMICAL_BONES"):
        assert bone in text


def test_blocky_rebuilds_anatomical_armature():
    text = OPS.read_text(encoding="utf-8")
    assert "rebuild_anatomical_armature" in text
    assert "_cube_on_bone" in text


def test_blocky_definition_is_mesh3d():
    assert '"demo_blocky_walk_01"' in DEFN.read_text(encoding="utf-8")
    assert '"assetMode": "mesh3d"' in DEFN.read_text(encoding="utf-8")


def test_build_script_exports_walk():
    text = (REPO / "tools/blender/scripts/build_blocky_walk_study.py").read_text(encoding="utf-8")
    assert "humanoid_blocky_v1" in text
    assert "create_blocky_human" in text
