"""Validate character definitions against Phase 01 + Phase 02 mesh3d contract."""

from __future__ import annotations

from .models import (
    ACCESSORIES,
    ARCHETYPES,
    ASSET_MODES,
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

    if defn.assetMode not in ASSET_MODES:
        errors.append(f"invalid assetMode: {defn.assetMode}")

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

    errors.extend(_validate_mesh3d_fields(defn))

    if errors:
        raise ValidationError(errors)


def _validate_mesh3d_fields(defn: CharacterDefinition) -> list[str]:
    errors: list[str] = []

    if defn.assetMode == "mesh3d":
        if not defn.mesh:
            errors.append("mesh3d assetMode requires mesh path")
        elif not defn.mesh.lower().endswith(".glb"):
            errors.append("mesh path must be a .glb file")

    errors.extend(_validate_optional_glb_path(defn.meshShowcase, "meshShowcase"))
    errors.extend(_validate_animation_set(defn.animationSet))

    if defn.textures is not None:
        for slot in ("albedo", "normal", "orm"):
            if not getattr(defn.textures, slot):
                errors.append(f"textures.{slot} is required when textures object is present")
        if defn.textures.emissive is not None and not defn.textures.emissive.strip():
            errors.append("textures.emissive must be non-empty when provided")

    for field_name in ("materialProfile", "behaviorProfile", "animatorController"):
        value = getattr(defn, field_name)
        if value is not None and not value.strip():
            errors.append(f"{field_name} must be non-empty when provided")

    return errors


def _validate_optional_glb_path(value: str | None, field_name: str) -> list[str]:
    if value is None:
        return []
    if not value.strip():
        return [f"{field_name} must be non-empty when provided"]
    if not value.lower().endswith(".glb"):
        return [f"{field_name} must be a .glb file"]
    return []


def _validate_animation_set(value: str | None) -> list[str]:
    if value is None:
        return []
    if not value.strip():
        return ["animationSet must be non-empty when provided"]
    if not _is_animation_set_id(value):
        return ["animationSet must be a lowercase identifier (e.g. humanoid_heavy_combat_v1)"]
    return []


def _is_animation_set_id(value: str) -> bool:
    parts = value.split("_")
    return bool(parts) and all(part and part[0].isalpha() and part.islower() and part.isascii() for part in parts)


def _is_hex_color(value: str) -> bool:
    if len(value) != 7 or not value.startswith("#"):
        return False
    try:
        int(value[1:], 16)
    except ValueError:
        return False
    return True
