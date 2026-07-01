"""Animation set manifest models and validation."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

CLIP_SOURCES = frozenset({"embedded", "library"})
REQUIRED_CLIP_KEYS = (
    "idle",
    "walk",
    "spawn",
    "attack",
    "attack2",
    "ultimate",
    "death",
)


@dataclass(frozen=True)
class AnimationSet:
    id: str
    bodyClass: str
    clipSource: str
    clips: dict[str, str]

    def to_dict(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "bodyClass": self.bodyClass,
            "clipSource": self.clipSource,
            "clips": dict(self.clips),
        }

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> AnimationSet:
        clips = data.get("clips", {})
        if not isinstance(clips, dict):
            raise ValueError("clips must be an object")
        return cls(
            id=str(data["id"]),
            bodyClass=str(data["bodyClass"]),
            clipSource=str(data.get("clipSource", "library")),
            clips={str(key): str(value) for key, value in clips.items()},
        )


class AnimationSetValidationError(Exception):
    def __init__(self, errors: list[str]) -> None:
        self.errors = errors
        super().__init__("; ".join(errors))


def clip_library_path(repo_root: Path, body_class: str, clip_name: str) -> Path:
    return (
        repo_root
        / "content"
        / "characters"
        / "animations"
        / "clips"
        / body_class
        / f"{clip_name}.glb"
    )


def validate_animation_set(defn: AnimationSet, repo_root: Path) -> None:
    errors: list[str] = []

    if not defn.id or defn.id.startswith("_"):
        errors.append("id must be non-empty and not a schema file")

    if not defn.bodyClass:
        errors.append("bodyClass is required")

    if defn.clipSource not in CLIP_SOURCES:
        errors.append(f"invalid clipSource: {defn.clipSource}")

    for key in REQUIRED_CLIP_KEYS:
        clip_name = defn.clips.get(key, "").strip()
        if not clip_name:
            errors.append(f"clips.{key} is required")
            continue
        if defn.clipSource == "library":
            path = clip_library_path(repo_root, defn.bodyClass, clip_name)
            if not path.is_file():
                errors.append(f"missing library clip: {path.relative_to(repo_root)}")

    if errors:
        raise AnimationSetValidationError(errors)
