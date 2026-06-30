"""Contract tests for studio demo sculpt roster."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

from character_studio.models import CharacterDefinition
from character_studio.validation import validate_definition
from mesh_pipeline.game_glb_validate import validate_game_glb

REPO_ROOT = Path(__file__).resolve().parents[2]
DEFINITIONS_DIR = REPO_ROOT / "content/characters/definitions"
MESHES_DIR = REPO_ROOT / "content/characters/meshes"
SHOWCASE_DIR = MESHES_DIR / "showcase"
ROSTER_PATH = REPO_ROOT / "content/characters/pilot_roster.json"
MANIFEST_PATH = REPO_ROOT / "content/characters/fixed_test_imports.json"
GAME_MESH_DIR = REPO_ROOT / "content/characters/meshes/game"


def _fixed_test_ids() -> tuple[str, ...]:
    data = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    return tuple(data["showcaseOrder"])


FIXED_TEST_IDS = _fixed_test_ids()


@pytest.fixture(scope="module")
def pilot_roster() -> list[dict]:
    data = json.loads(ROSTER_PATH.read_text(encoding="utf-8"))
    return list(data["pilots"])


def test_pilot_roster_manifest_has_fifteen_fixed_test_sculpts(pilot_roster: list[dict]) -> None:
    assert len(pilot_roster) == 15
    assert {entry["id"] for entry in pilot_roster} == set(FIXED_TEST_IDS)


@pytest.mark.parametrize("pilot_id", FIXED_TEST_IDS)
def test_demo_sculpt_definition_is_mesh3d_with_glb(pilot_id: str) -> None:
    path = DEFINITIONS_DIR / f"{pilot_id}.json"
    defn = CharacterDefinition.from_dict(json.loads(path.read_text(encoding="utf-8")))
    validate_definition(defn)
    assert defn.assetMode == "mesh3d"
    assert defn.meshShowcase == f"content/characters/meshes/showcase/{pilot_id}.glb"
    assert defn.mesh == f"content/characters/meshes/game/{pilot_id}.glb"
    assert defn.animationSet is not None
    showcase_glb = REPO_ROOT / defn.meshShowcase
    assert showcase_glb.is_file(), f"missing showcase glb: {showcase_glb}"
    assert showcase_glb.stat().st_size > 1_000_000
    game_glb = REPO_ROOT / defn.mesh
    assert game_glb.is_file(), f"missing game glb: {game_glb}"
    result = validate_game_glb(game_glb)
    assert result.ok, result.errors


def test_pilot_definitions_match_roster_manifest(pilot_roster: list[dict]) -> None:
    for entry in pilot_roster:
        pilot_id = entry["id"]
        path = DEFINITIONS_DIR / f"{pilot_id}.json"
        defn = CharacterDefinition.from_dict(json.loads(path.read_text(encoding="utf-8")))
        assert defn.behaviorProfile == entry["behaviorProfile"]
        assert defn.materialProfile == entry["materialProfile"]
        assert defn.animatorController == entry["animatorController"]


def test_provision_pilot_meshes_script_skips_existing_demo_glb() -> None:
    script = REPO_ROOT / "tools/unity/provision_pilot_meshes.py"
    result = subprocess.run(
        [sys.executable, str(script)],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    for pilot_id in FIXED_TEST_IDS:
        assert (SHOWCASE_DIR / f"{pilot_id}.glb").is_file()
