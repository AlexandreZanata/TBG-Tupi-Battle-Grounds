"""CI contract for demo_sentinel_01 game mesh Walk/Ultimate (mirrors Unity EditMode smoke)."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from character_studio.models import CharacterDefinition
from character_studio.validation import validate_definition

REPO_ROOT = Path(__file__).resolve().parents[2]
SENTINEL_ID = "demo_sentinel_01"
GAME_GLB = REPO_ROOT / "content/characters/meshes/game/demo_sentinel_01.glb"
BEHAVIOR_JSON = REPO_ROOT / "content/characters/behaviors/support_cast_v1.json"
CONTROLLER = (
    REPO_ROOT / "unity/Packages/com.tbg.character-library/Animations/HumanoidHeavy.controller"
)


@pytest.fixture(scope="module")
def sentinel_definition() -> CharacterDefinition:
    path = REPO_ROOT / "content/characters/definitions" / f"{SENTINEL_ID}.json"
    defn = CharacterDefinition.from_dict(json.loads(path.read_text(encoding="utf-8")))
    validate_definition(defn)
    return defn


def test_sentinel_definition_points_at_game_mesh(sentinel_definition: CharacterDefinition) -> None:
    assert sentinel_definition.mesh == f"content/characters/meshes/game/{SENTINEL_ID}.glb"
    assert sentinel_definition.meshShowcase is not None
    assert sentinel_definition.behaviorProfile == "support_cast_v1"
    assert sentinel_definition.animationSet == "humanoid_heavy_combat_v1"


def test_sentinel_game_glb_has_walk_and_ultimate_clips() -> None:
    assert GAME_GLB.is_file()
    from mesh_pipeline.glb_inspect import inspect_glb, load_glb

    stats = inspect_glb(GAME_GLB)
    assert stats.animations >= 6
    gltf, _ = load_glb(GAME_GLB)
    clip_names = {anim.get("name", "") for anim in gltf.get("animations", [])}
    assert "Walk" in clip_names
    assert "Ultimate" in clip_names


def test_support_cast_profile_maps_walk_and_ultimate() -> None:
    profile = json.loads(BEHAVIOR_JSON.read_text(encoding="utf-8"))
    assert profile["actions"]["walk"] == "Walk"
    assert profile["actions"]["ultimate"] == "Ultimate"
    assert profile["triggers"]["walk"] == "Walk"
    assert profile["triggers"]["ultimate"] == "Ultimate"
    assert profile["timings"]["ultimateDurationMs"] > profile["timings"]["ultimateWindupMs"]


def test_humanoid_heavy_controller_exposes_walk_and_ultimate_triggers() -> None:
    yaml_text = CONTROLLER.read_text(encoding="utf-8")
    assert "m_Name: Walk" in yaml_text
    assert "m_Name: Ultimate" in yaml_text
    assert "m_ConditionEvent: Walk" in yaml_text
    assert "m_ConditionEvent: Ultimate" in yaml_text
