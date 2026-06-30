"""Contract tests for game GLB budget validation."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

from mesh_pipeline.budgets import MAX_TRIANGLES
from mesh_pipeline.game_glb_validate import validate_game_glb
from mesh_pipeline.glb_inspect import inspect_glb

REPO_ROOT = Path(__file__).resolve().parents[2]
FIXTURE = REPO_ROOT / "content/characters/meshes/_test_fixture.glb"
DEMO_GUARD = REPO_ROOT / "content/characters/meshes/demo_guard_01.glb"
VALIDATE_CLI = REPO_ROOT / "tools/mesh-pipeline/validate_game_glb.py"
OPTIMIZE_CLI = REPO_ROOT / "tools/mesh-pipeline/optimize_glb.sh"


def test_fixture_reports_expected_triangle_count() -> None:
    stats = inspect_glb(FIXTURE)
    assert stats.triangles == 12
    assert stats.skins == 0
    assert stats.animations == 0


def test_showcase_sculpt_fails_triangle_budget() -> None:
    result = validate_game_glb(DEMO_GUARD)
    assert not result.ok
    assert any("triangles" in error for error in result.errors)


def test_validate_cli_reports_failure_for_showcase_sculpt() -> None:
    proc = subprocess.run(
        [sys.executable, str(VALIDATE_CLI), str(DEMO_GUARD)],
        capture_output=True,
        text=True,
        check=False,
    )
    assert proc.returncode == 1
    assert "FAIL" in proc.stdout


def test_optimize_game_mode_produces_budget_mesh(tmp_path: Path) -> None:
    output = tmp_path / "game_fixture.glb"
    proc = subprocess.run(
        [
            str(OPTIMIZE_CLI),
            "--mode",
            "game",
            "--tri",
            str(MAX_TRIANGLES),
            "--tex",
            "1024",
            str(FIXTURE),
            str(output),
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr
    assert output.is_file()

    result = validate_game_glb(output)
    assert result.ok, result.errors
    assert result.stats.triangles <= MAX_TRIANGLES
