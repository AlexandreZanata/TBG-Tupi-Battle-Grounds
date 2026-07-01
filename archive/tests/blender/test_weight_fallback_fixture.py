"""Integration: automatic weight fallback passes synthetic fixture gate."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

import pytest

from mesh_pipeline.prep_ai_mesh_run import blender_available

REPO = Path(__file__).resolve().parents[2]
FIXTURE = REPO / "tools/blender/scripts/weight_fallback_fixture.py"


def test_rig_weight_bind_module_exists():
    text = (REPO / "tools/blender/scripts/rig_weight_bind.py").read_text(encoding="utf-8")
    assert "bind_mesh_with_weight_fallback" in text
    assert "paint_automatic_weights" in (REPO / "tools/blender/scripts/rig_weight_eval.py").read_text(encoding="utf-8")


@pytest.mark.skipif(not blender_available(), reason="Blender not available")
def test_weight_fallback_fixture_picks_lower_skew():
    cmd = [
        str(REPO / "tools/blender/blender_cmd.sh"),
        "--background",
        "--python",
        str(FIXTURE),
    ]
    proc = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True, check=True)
    report_line = next(line for line in reversed(proc.stdout.splitlines()) if line.startswith("{"))
    report = json.loads(report_line)
    assert report["winner"] == "proximity"
    assert report["skew_pass"] is True
    assert report["dominant_share"] < 0.85
    assert not report["bones_missing"]
