"""Ensure repo-root imports work from Blender bpy entry scripts."""

from __future__ import annotations

import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
BLENDER_ROOT = Path(__file__).resolve().parent


def ensure_repo_paths() -> None:
    repo = str(REPO_ROOT)
    if repo not in sys.path:
        sys.path.insert(0, repo)


def resolve_repo_path(path: Path | str) -> Path:
    """Resolve user paths against repo root (Docker cwd is not the project)."""
    candidate = Path(path)
    if candidate.is_file():
        return candidate.resolve()
    rooted = REPO_ROOT / candidate
    if rooted.is_file():
        return rooted.resolve()
    if candidate.is_absolute():
        return candidate.resolve()
    return rooted.resolve()
