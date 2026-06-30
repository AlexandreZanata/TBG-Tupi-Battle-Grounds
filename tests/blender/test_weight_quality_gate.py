"""Contract tests for skin weight quality gate."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[2]
TOOLS = REPO / "tools"
if str(TOOLS) not in sys.path:
    sys.path.insert(0, str(TOOLS))

from mesh_pipeline.weight_quality import MIN_BONE_WEIGHT, validate_weight_quality  # noqa: E402
from mesh_pipeline.weight_skew import SINGLE_BONE_DOMINANCE  # noqa: E402

PILOT_GAME = REPO / "content/characters/meshes/game/demo_sentinel_01.glb"
SKIPPED_GAME = REPO / "content/characters/meshes/game/demo_reaver_01.glb"


def test_dominance_threshold_matches_rig_weight_paint():
    source = (REPO / "tools/blender/scripts/rig_weight_paint.py").read_text(encoding="utf-8")
    assert "SINGLE_BONE_DOMINANCE = 0.85" in source
    assert SINGLE_BONE_DOMINANCE == 0.85
    assert MIN_BONE_WEIGHT == 0.01


def test_attach_tbg_rig_uses_weight_fallback():
    text = (REPO / "tools/blender/scripts/attach_tbg_rig.py").read_text(encoding="utf-8")
    assert "bind_mesh_with_weight_fallback" in text


@pytest.mark.skipif(not PILOT_GAME.is_file(), reason="pilot game GLB not built")
def test_sentinel_passes_skew_and_unweighted_contract():
    result = validate_weight_quality(PILOT_GAME)
    assert not result.skew.skewed, result.errors
    assert result.skew.unweighted_vertices == 0
    assert result.skew.dominant_share < SINGLE_BONE_DOMINANCE


@pytest.mark.skipif(not SKIPPED_GAME.is_file(), reason="pilot game GLB not built")
def test_reaver_fails_weight_skew_gate():
    result = validate_weight_quality(SKIPPED_GAME)
    assert not result.passes_gate
    assert any("weight skew" in error for error in result.errors)
