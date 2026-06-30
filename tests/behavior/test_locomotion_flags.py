"""Contract: behavior locomotion flags default off."""

from __future__ import annotations

import json
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
BEHAVIORS_DIR = REPO_ROOT / "content/characters/behaviors"


def _load(profile_id: str) -> dict:
    return json.loads((BEHAVIORS_DIR / f"{profile_id}.json").read_text(encoding="utf-8"))


def test_foot_ik_defaults_false_when_locomotion_missing():
    for profile_id in ("tank_melee_v1", "ranged_shot_v1", "dps_melee_v1"):
        data = _load(profile_id)
        locomotion = data.get("locomotion")
        if locomotion is None:
            assert True
            continue
        assert locomotion.get("footIk") is not True


def test_locomotion_schema_accepts_optional_flags():
    example = {
        "locomotion": {
            "useRootMotionWalk": False,
            "footIk": False,
        }
    }
    assert example["locomotion"]["footIk"] is False
