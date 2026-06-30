"""Contract tests for animation set manifests."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from animation_sets.validation import (
    REQUIRED_CLIP_KEYS,
    AnimationSet,
    AnimationSetValidationError,
    clip_library_path,
    validate_animation_set,
)

REPO_ROOT = Path(__file__).resolve().parents[2]
SETS_DIR = REPO_ROOT / "content/characters/animation_sets"
HUMANOID_HEAVY_SET = SETS_DIR / "humanoid_heavy_combat_v1.json"


def test_schema_example_matches_required_clip_keys() -> None:
    example = json.loads((SETS_DIR / "_schema.example.json").read_text(encoding="utf-8"))
    for key in REQUIRED_CLIP_KEYS:
        assert example["clips"][key]


def test_humanoid_heavy_combat_set_resolves_library_clips() -> None:
    data = json.loads(HUMANOID_HEAVY_SET.read_text(encoding="utf-8"))
    defn = AnimationSet.from_dict(data)
    validate_animation_set(defn, REPO_ROOT)
    assert defn.bodyClass == "HumanoidHeavy"
    assert defn.clips["walk"] == "Walk"
    assert clip_library_path(REPO_ROOT, defn.bodyClass, "Ultimate").is_file()


def test_missing_library_clip_fails_validation() -> None:
    defn = AnimationSet.from_dict(json.loads(HUMANOID_HEAVY_SET.read_text(encoding="utf-8")))
    defn = AnimationSet(
        id=defn.id,
        bodyClass=defn.bodyClass,
        clipSource=defn.clipSource,
        clips={**defn.clips, "walk": "MissingWalkClip"},
    )
    with pytest.raises(AnimationSetValidationError, match="missing library clip"):
        validate_animation_set(defn, REPO_ROOT)
