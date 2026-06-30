"""Contract tests for Phase 03 dual-tier mesh schema fields."""

from __future__ import annotations

import pytest

from character_studio.defaults import create_default_definition
from character_studio.models import CharacterDefinition, GameLod
from character_studio.validation import ValidationError, validate_definition


def _dual_tier_definition() -> CharacterDefinition:
    base = create_default_definition("demo_sentinel_01", "Support", "Humans")
    return CharacterDefinition.from_dict(
        {
            **base.to_dict(),
            "assetMode": "mesh3d",
            "meshShowcase": "content/characters/meshes/showcase/demo_sentinel_01.glb",
            "mesh": "content/characters/meshes/game/demo_sentinel_01.glb",
            "animationSet": "humanoid_heavy_combat_v1",
            "materialProfile": "humans_pbr_v1",
            "behaviorProfile": "support_cast_v1",
        }
    )


def test_dual_mesh_paths_and_animation_set_accepted() -> None:
    defn = _dual_tier_definition()
    validate_definition(defn)
    assert defn.meshShowcase.endswith(".glb")
    assert defn.mesh.endswith("meshes/game/demo_sentinel_01.glb")
    assert defn.animationSet == "humanoid_heavy_combat_v1"


def test_mesh_showcase_requires_glb_extension() -> None:
    defn = _dual_tier_definition()
    defn.meshShowcase = "content/characters/meshes/showcase/demo_sentinel_01.fbx"
    with pytest.raises(ValidationError, match="meshShowcase must be a \\.glb"):
        validate_definition(defn)


def test_animation_set_requires_mesh_when_mesh3d() -> None:
    defn = _dual_tier_definition()
    defn.mesh = None
    with pytest.raises(ValidationError, match="requires mesh path"):
        validate_definition(defn)


def test_animation_set_rejects_invalid_id() -> None:
    defn = _dual_tier_definition()
    defn.animationSet = "HumanoidHeavy"
    with pytest.raises(ValidationError, match="animationSet must be a lowercase identifier"):
        validate_definition(defn)


def test_round_trip_serializes_dual_tier_fields() -> None:
    defn = _dual_tier_definition()
    restored = CharacterDefinition.from_dict(defn.to_dict())
    assert restored.meshShowcase == defn.meshShowcase
    assert restored.mesh == defn.mesh
    assert restored.animationSet == defn.animationSet


def test_game_lod_accepts_valid_overrides() -> None:
    defn = _dual_tier_definition()
    defn.gameLod = GameLod(triTarget=6000, textureSize=512)
    validate_definition(defn)
    payload = defn.to_dict()
    assert payload["gameLod"]["triTarget"] == 6000
    assert payload["gameLod"]["textureSize"] == 512
