"""Contract: GLB skin weight dominance matches rig_weight_paint gate threshold."""

from __future__ import annotations

from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[2]
TOOLS = REPO / "tools"
if str(TOOLS) not in __import__("sys").path:
    __import__("sys").path.insert(0, str(TOOLS))

from mesh_pipeline.weight_skew import (  # noqa: E402
    SINGLE_BONE_DOMINANCE,
    analyze_weight_skew,
    weights_skewed_to_one_bone,
)

PILOT_GAME = REPO / "content/characters/meshes/game/demo_sentinel_01.glb"


def test_single_bone_dominance_matches_rig_weight_paint():
    source = (REPO / "tools/blender/scripts/rig_weight_paint.py").read_text(encoding="utf-8")
    assert "SINGLE_BONE_DOMINANCE = 0.85" in source
    assert SINGLE_BONE_DOMINANCE == 0.85


@pytest.mark.skipif(not PILOT_GAME.is_file(), reason="pilot game GLB not built")
def test_sentinel_game_glb_has_skin_weights():
    report = analyze_weight_skew(PILOT_GAME)
    assert report.vertex_count > 0
    assert report.dominant_bone is not None
    assert 0.0 < report.dominant_share <= 1.0


@pytest.mark.skipif(not PILOT_GAME.is_file(), reason="pilot game GLB not built")
def test_sentinel_weight_skew_report_shape():
    report = analyze_weight_skew(PILOT_GAME)
    assert report.unweighted_vertices == 0
    assert report.bones_with_weights >= 10
    assert weights_skewed_to_one_bone(report) == report.skewed
