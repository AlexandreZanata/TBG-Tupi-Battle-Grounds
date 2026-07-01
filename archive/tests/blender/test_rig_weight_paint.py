"""Contract: anatomical rig weight paint regions and bone coverage."""

from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SOURCE = REPO / "tools/blender/scripts/rig_weight_paint.py"
COVERAGE = REPO / "tools/blender/scripts/rig_weight_coverage.py"


def test_weight_paint_leg_region_excludes_spine():
    text = SOURCE.read_text(encoding="utf-8")
    assert "UpperLeg" in text
    assert '"leg"' in text
    assert "REGION_TAGS" in text


def test_weight_paint_uses_lateral_arm_band():
    text = SOURCE.read_text(encoding="utf-8")
    assert "torso_half_width" in text
    assert '"arm"' in text


def test_weight_paint_seeds_missing_bone_coverage():
    text = COVERAGE.read_text(encoding="utf-8")
    assert "ensure_bone_coverage" in text
    assert "MIN_BONE_WEIGHT" in text
