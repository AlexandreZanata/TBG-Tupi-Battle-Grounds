"""Contract tests for batch game mesh export script."""

from __future__ import annotations

import json
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
BATCH_SCRIPT = REPO_ROOT / "tools/mesh-pipeline/batch_seed_to_game.sh"
ROSTER_PATH = REPO_ROOT / "content/characters/pilot_roster.json"
GAME_MESH_DIR = REPO_ROOT / "content/characters/meshes/game"


def test_batch_script_exists_and_is_executable() -> None:
    assert BATCH_SCRIPT.is_file()
    assert BATCH_SCRIPT.stat().st_mode & 0o111


def test_roster_ids_have_game_mesh_paths_in_definitions() -> None:
    roster = json.loads(ROSTER_PATH.read_text(encoding="utf-8"))
    for entry in roster["pilots"]:
        pilot_id = entry["id"]
        definition_path = REPO_ROOT / "content/characters/definitions" / f"{pilot_id}.json"
        definition = json.loads(definition_path.read_text(encoding="utf-8"))
        assert definition["mesh"] == f"content/characters/meshes/game/{pilot_id}.glb"


def test_all_roster_game_meshes_exist_and_validate() -> None:
    from mesh_pipeline.game_glb_validate import validate_game_glb

    roster = json.loads(ROSTER_PATH.read_text(encoding="utf-8"))
    for entry in roster["pilots"]:
        game_glb = GAME_MESH_DIR / f"{entry['id']}.glb"
        assert game_glb.is_file(), f"missing game glb: {game_glb}"
        result = validate_game_glb(game_glb)
        assert result.ok, result.errors
