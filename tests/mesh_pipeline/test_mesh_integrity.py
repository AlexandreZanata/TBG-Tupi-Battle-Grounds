"""Contract tests for mesh integrity validation."""

from __future__ import annotations

from pathlib import Path

from mesh_pipeline.glb_mesh_integrity import mesh_integrity

REPO_ROOT = Path(__file__).resolve().parents[2]


def test_champion_game_mesh_fragmentation_is_detected() -> None:
    """Regression: decimated game meshes may fragment — gate must flag demo_champion_01."""
    path = REPO_ROOT / "content/characters/meshes/game/demo_champion_01.glb"
    if not path.is_file():
        return
    stats = mesh_integrity(path)
    assert not stats.ok, (
        f"expected fragmentation flag: islands={stats.island_count} "
        f"largest={stats.largest_island_ratio:.0%}"
    )


def test_stalker_game_mesh_fragmentation_is_detected() -> None:
    path = REPO_ROOT / "content/characters/meshes/game/demo_stalker_01.glb"
    if not path.is_file():
        return
    stats = mesh_integrity(path)
    assert not stats.ok
