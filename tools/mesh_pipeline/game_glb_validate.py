"""Validate game-tier GLB files against mobile budgets."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from mesh_pipeline.budgets import (
    HARD_MAX_FILE_BYTES,
    MAX_BONES,
    MAX_CLIPS_SKINNED,
    MAX_MATERIALS,
    MAX_TEXTURE_SIZE,
    MAX_TRIANGLES,
    MIN_CLIPS_SKINNED,
    TARGET_FILE_BYTES,
)
from mesh_pipeline.glb_inspect import GlbStats, inspect_glb


@dataclass(frozen=True)
class ValidationResult:
    stats: GlbStats
    errors: list[str]
    warnings: list[str]

    @property
    def ok(self) -> bool:
        return not self.errors


def validate_game_glb(path: Path) -> ValidationResult:
    stats = inspect_glb(path)
    errors: list[str] = []
    warnings: list[str] = []

    if stats.triangles > MAX_TRIANGLES:
        errors.append(f"triangles {stats.triangles} exceed limit {MAX_TRIANGLES}")

    if stats.materials > MAX_MATERIALS:
        errors.append(f"materials {stats.materials} exceed limit {MAX_MATERIALS}")

    if stats.max_texture_size > MAX_TEXTURE_SIZE:
        errors.append(
            f"max texture dimension {stats.max_texture_size} exceeds limit {MAX_TEXTURE_SIZE}"
        )

    if stats.file_bytes > HARD_MAX_FILE_BYTES:
        errors.append(
            f"file size {stats.file_bytes} bytes exceeds hard limit {HARD_MAX_FILE_BYTES}"
        )
    elif stats.file_bytes > TARGET_FILE_BYTES:
        warnings.append(
            f"file size {stats.file_bytes} bytes exceeds target {TARGET_FILE_BYTES}"
        )

    if stats.skins > 0:
        if stats.bones > MAX_BONES:
            errors.append(f"bones {stats.bones} exceed limit {MAX_BONES}")
        if stats.animations < MIN_CLIPS_SKINNED:
            errors.append(
                f"animations {stats.animations} below minimum {MIN_CLIPS_SKINNED} for skinned mesh"
            )
        elif stats.animations > MAX_CLIPS_SKINNED:
            errors.append(
                f"animations {stats.animations} exceed maximum {MAX_CLIPS_SKINNED}"
            )

    return ValidationResult(stats=stats, errors=errors, warnings=warnings)
