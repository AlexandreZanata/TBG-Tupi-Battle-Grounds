"""Contract tests for game unit export orchestrator."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

from mesh_pipeline.export_game_unit_core import (
    BLENDER_CMD,
    blender_available,
    export_game_unit_python,
    load_export_config,
)
from mesh_pipeline.game_glb_assemble import assemble_skinned_game_glb
from mesh_pipeline.game_glb_validate import validate_game_glb

REPO_ROOT = Path(__file__).resolve().parents[2]
EXPORT_CLI = REPO_ROOT / "tools/blender/export_game_unit.py"
PILOT_UNIT_ID = "demo_sentinel_01"
GAME_MESH = REPO_ROOT / "content/characters/meshes/game/demo_sentinel_01.glb"


def test_load_export_config_reads_guard_unit() -> None:
    config = load_export_config("demo_guard_01", REPO_ROOT)
    assert config.unit_id == "demo_guard_01"
    assert config.body_class == "HumanoidHeavy"
    assert config.showcase_mesh.name == "demo_guard_01.glb"
    assert "showcase" in str(config.showcase_mesh)


def test_assembled_skinned_glb_passes_budget(tmp_path: Path) -> None:
    output = tmp_path / "pilot.glb"
    assemble_skinned_game_glb(output, ["Idle", "Walk", "Spawn", "Attack", "Attack2", "Ultimate", "Death"])
    result = validate_game_glb(output)
    assert result.ok, result.errors
    assert result.stats.skins == 1
    assert result.stats.animations == 7
    assert result.stats.bones == 19
    assert result.stats.triangles == 12


def test_export_game_unit_python_writes_valid_pilot_mesh(tmp_path: Path) -> None:
    showcase = REPO_ROOT / "content/characters/meshes/showcase/demo_sentinel_01.glb"
    definition = json.loads(
        (REPO_ROOT / "content/characters/definitions/demo_sentinel_01.json").read_text(encoding="utf-8")
    )
    config = load_export_config(PILOT_UNIT_ID, REPO_ROOT)
    config = type(config)(
        unit_id=config.unit_id,
        showcase_mesh=showcase,
        game_mesh=tmp_path / "game" / "demo_sentinel_01.glb",
        animation_set_id=definition.get("animationSet", "humanoid_heavy_combat_v1"),
        body_class=config.body_class,
    )
    output = export_game_unit_python(config, REPO_ROOT)
    result = validate_game_glb(output)
    assert result.ok, result.errors
    assert output.is_file()


def test_blender_wrapper_is_executable() -> None:
    assert BLENDER_CMD.is_file()
    assert blender_available()


@pytest.fixture(scope="module")
def exported_pilot_mesh() -> Path:
    if not GAME_MESH.is_file():
        proc = subprocess.run(
            [sys.executable, str(EXPORT_CLI), PILOT_UNIT_ID],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        assert proc.returncode == 0, proc.stderr or proc.stdout
    return GAME_MESH


def test_export_cli_produces_valid_game_mesh(exported_pilot_mesh: Path) -> None:
    result = validate_game_glb(exported_pilot_mesh)
    assert result.ok, result.errors
    assert result.stats.triangles <= 8000
    assert result.stats.animations >= 6
    if blender_available():
        assert result.stats.triangles > 100, "expected Blender-decimated sculpt, not proxy cube"
