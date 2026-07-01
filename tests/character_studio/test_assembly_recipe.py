"""Contract tests for assemblyRecipe on CharacterDefinition."""

from __future__ import annotations

from character_studio.models import CharacterDefinition
from character_studio.validation import ValidationError, validate_definition


def _base_definition(**overrides) -> CharacterDefinition:
    data = {
        "id": "assembled_tank_01",
        "characterName": "Assembled Tank",
        "archetype": "Tank",
        "faction": "Humans",
        "heightScale": 1.0,
        "bodyWidthScale": 1.0,
        "headScale": 1.0,
        "limbLengthScale": 1.0,
        "primaryColor": "#FFFFFF",
        "secondaryColor": "#CCCCCC",
        "assetMode": "mesh3d",
        "assemblyRecipe": {
            "unitId": "test_male_head_sword",
            "baseId": "male",
            "parts": ["head_round_01"],
            "materialId": "default",
        },
    }
    data.update(overrides)
    return CharacterDefinition.from_dict(data)


def test_assembly_recipe_round_trip() -> None:
    defn = _base_definition()
    restored = CharacterDefinition.from_dict(defn.to_dict())
    assert restored.assemblyRecipe is not None
    assert restored.assemblyRecipe.unitId == "test_male_head_sword"
    assert restored.assemblyRecipe.baseId == "male"


def test_assembly_recipe_validates_base_id() -> None:
    defn = _base_definition(
        assemblyRecipe={
            "unitId": "x",
            "baseId": "invalid",
            "parts": [],
            "materialId": "default",
        },
    )
    try:
        validate_definition(defn)
    except ValidationError as exc:
        assert any("baseId" in err for err in exc.errors)
    else:
        raise AssertionError("expected validation error")
