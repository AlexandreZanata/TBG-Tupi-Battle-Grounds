"""Contract tests for Phase 02 behavior profile JSON files."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
BEHAVIORS_DIR = REPO_ROOT / "content" / "characters" / "behaviors"
ANIMATIONS_DIR = REPO_ROOT / "content" / "characters" / "animations"

ARCHETYPE_PROFILES = {
    "Tank": "tank_melee_v1",
    "DPS": "dps_melee_v1",
    "Support": "support_cast_v1",
    "Ranged": "ranged_shot_v1",
    "Control": "control_cast_v1",
}
REQUIRED_ACTIONS = ("idle", "spawn", "attack", "death")


def _load_profile(profile_id: str) -> dict:
    path = BEHAVIORS_DIR / f"{profile_id}.json"
    assert path.is_file(), f"missing behavior profile: {path}"
    return json.loads(path.read_text(encoding="utf-8"))


@pytest.mark.parametrize("archetype,profile_id", list(ARCHETYPE_PROFILES.items()))
def test_archetype_behavior_profile_exists(archetype: str, profile_id: str) -> None:
    data = _load_profile(profile_id)
    assert data["id"] == profile_id
    assert data["archetype"] == archetype
    assert data["clipSource"] in {"embedded", "external"}


def test_actions_map_character_action_states() -> None:
    tank = _load_profile("tank_melee_v1")
    actions = tank["actions"]
    for key in REQUIRED_ACTIONS:
        assert actions[key]
    assert actions["attack"] == "AttackHeavy"
    assert actions["walk"] == "Walk"
    assert actions["ultimate"] == "Ultimate"
    assert tank["triggers"]["attack"] == "Attack"


def test_all_profiles_reference_existing_controllers() -> None:
    for profile_id in ARCHETYPE_PROFILES.values():
        data = _load_profile(profile_id)
        controller = REPO_ROOT / data["animatorController"]
        assert controller.is_file(), f"missing controller for {profile_id}: {controller}"


def test_body_class_controllers_exist() -> None:
    for name in ("HumanoidHeavy", "HumanoidLight", "Mechanical", "Organic"):
        path = ANIMATIONS_DIR / f"{name}.controller"
        assert path.is_file(), f"missing animator controller: {path}"


def test_timings_are_positive_integers() -> None:
    for profile_id in ARCHETYPE_PROFILES.values():
        timings = _load_profile(profile_id)["timings"]
        assert timings["attackWindupMs"] > 0
        assert timings["attackHitMs"] > timings["attackWindupMs"]
        assert timings["ultimateWindupMs"] > 0
        assert timings["ultimateDurationMs"] > timings["ultimateWindupMs"]
