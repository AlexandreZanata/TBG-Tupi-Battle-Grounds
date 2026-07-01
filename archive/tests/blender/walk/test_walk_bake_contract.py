"""Contract: walk bake — pelvis bob on TBG_Root location only."""

from pathlib import Path

REPO = Path(__file__).resolve().parents[3]


def test_walk_bake_root_uses_location_not_rotation():
    text = (REPO / "tools/blender/animation/walk/bake.py").read_text(encoding="utf-8")
    assert "bone.name == \"TBG_Root\"" in text
    assert "bone.rotation_euler = (0.0, 0.0, 0.0)" in text
    assert "bone.location = (0.0, root[1], root[2])" in text
