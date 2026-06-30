"""Contract tests for mesh integrity validation."""

from __future__ import annotations

from pathlib import Path

from mesh_pipeline.glb_mesh_integrity import mesh_integrity
from mesh_pipeline.glb_inspect import inspect_glb

REPO_ROOT = Path(__file__).resolve().parents[2]


def test_champion_game_mesh_is_not_fragmented() -> None:
    path = REPO_ROOT / "content/characters/meshes/game/demo_champion_01.glb"
    if not path.is_file():
        return
    stats = mesh_integrity(path)
    assert stats.ok, (
        f"islands={stats.island_count} largest={stats.largest_island_ratio:.0%} "
        f"tris={inspect_glb(path).triangles}"
    )


def test_stalker_game_mesh_is_not_fragmented() -> None:
    path = REPO_ROOT / "content/characters/meshes/game/demo_stalker_01.glb"
    if not path.is_file():
        return
    stats = mesh_integrity(path)
    assert stats.ok
