"""Validate character definitions against Phase 01 contract."""

from __future__ import annotations

from .models import (
    ACCESSORIES,
    ARCHETYPES,
    EVOLUTION_MAX,
    EVOLUTION_MIN,
    FACTIONS,
    PROPORTION_MAX,
    PROPORTION_MIN,
    CharacterDefinition,
)


class ValidationError(Exception):
    def __init__(self, errors: list[str]) -> None:
        self.errors = errors
        super().__init__("; ".join(errors))


def validate_definition(defn: CharacterDefinition, existing_ids: set[str] | None = None) -> None:
    errors: list[str] = []

    if not defn.id or defn.id.startswith("_"):
        errors.append("id must be non-empty and not a schema file")

    if existing_ids and defn.id in existing_ids:
        errors.append(f"duplicate id: {defn.id}")

    if defn.archetype not in ARCHETYPES:
        errors.append(f"invalid archetype: {defn.archetype}")

    if defn.faction not in FACTIONS:
        errors.append(f"invalid faction: {defn.faction}")

    for accessory in defn.accessories:
        if accessory not in ACCESSORIES:
            errors.append(f"invalid accessory: {accessory}")

    if not (EVOLUTION_MIN <= defn.evolutionStage <= EVOLUTION_MAX):
        errors.append(f"evolutionStage must be {EVOLUTION_MIN}–{EVOLUTION_MAX}")

    for field_name in ("heightScale", "bodyWidthScale", "headScale", "limbLengthScale"):
        value = getattr(defn, field_name)
        if not (PROPORTION_MIN <= value <= PROPORTION_MAX):
            errors.append(f"{field_name} out of range ({PROPORTION_MIN}–{PROPORTION_MAX})")

    for color_name in ("primaryColor", "secondaryColor"):
        color = getattr(defn, color_name)
        if not _is_hex_color(color):
            errors.append(f"{color_name} must be #RRGGBB")

    if errors:
        raise ValidationError(errors)


def _is_hex_color(value: str) -> bool:
    if len(value) != 7 or not value.startswith("#"):
        return False
    try:
        int(value[1:], 16)
    except ValueError:
        return False
    return True
