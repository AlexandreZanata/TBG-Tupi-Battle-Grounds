"""Unit tests for weight_quality gate."""

from __future__ import annotations

import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
if str(REPO / "tools") not in sys.path:
    sys.path.insert(0, str(REPO / "tools"))

from mesh_pipeline.weight_quality import validate_weight_quality  # noqa: E402
from mesh_pipeline.weight_skew import WeightSkewReport  # noqa: E402
from mesh_pipeline.weight_quality import _bones_missing_coverage, _collect_errors  # noqa: E402


def test_collect_errors_flags_skew_and_unweighted():
    skew = WeightSkewReport(
        vertex_count=10,
        dominant_bone="TBG_Spine",
        dominant_share=0.9,
        skewed=True,
        unweighted_vertices=2,
        bones_with_weights=3,
        joint_names=("TBG_Spine", "TBG_Head"),
        bone_max_weight=(("TBG_Spine", 0.9), ("TBG_Head", 0.05)),
    )
    errors = _collect_errors(skew, 0.85)
    assert any("weight skew" in item for item in errors)
    assert any("zero weight" in item for item in errors)


def test_bones_missing_coverage_lists_tbg_joints():
    skew = WeightSkewReport(
        vertex_count=5,
        dominant_bone="TBG_Head",
        dominant_share=0.5,
        skewed=False,
        unweighted_vertices=0,
        bones_with_weights=2,
        joint_names=("TBG_Spine", "TBG_Head"),
        bone_max_weight=(("TBG_Spine", 0.0), ("TBG_Head", 0.8)),
    )
    missing = _bones_missing_coverage(skew)
    assert missing == ("TBG_Spine",)


def test_validate_cli_flag_documented():
    text = (REPO / "tools/mesh-pipeline/validate_game_glb.py").read_text(encoding="utf-8")
    assert "--check-weights" in text
    assert "TBG_WEIGHT_GATE" in text
