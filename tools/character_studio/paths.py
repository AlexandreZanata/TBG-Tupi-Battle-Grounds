"""Resolve versioned character content paths relative to repository root."""

from __future__ import annotations

from pathlib import Path


def find_repo_root(start: Path | None = None) -> Path:
    current = (start or Path(__file__)).resolve()
    for parent in [current, *current.parents]:
        if (parent / "content" / "characters").is_dir():
            return parent
    raise FileNotFoundError("Repository root not found (missing content/characters)")


def character_content_root(repo_root: Path | None = None) -> Path:
    root = repo_root or find_repo_root()
    return root / "content" / "characters"


def definitions_dir(repo_root: Path | None = None) -> Path:
    return character_content_root(repo_root) / "definitions"


def cards_dir(repo_root: Path | None = None) -> Path:
    return character_content_root(repo_root) / "cards"


def previews_dir(repo_root: Path | None = None) -> Path:
    return character_content_root(repo_root) / "previews"
