"""Contract tests for placeholder clip GLB generation."""

from __future__ import annotations

from pathlib import Path

from animation_sets.placeholder_glb import write_placeholder_clip_glb
from mesh_pipeline.glb_inspect import load_glb

REPO_ROOT = Path(__file__).resolve().parents[2]


def test_placeholder_clip_contains_named_animation(tmp_path: Path) -> None:
    path = tmp_path / "Walk.glb"
    write_placeholder_clip_glb(path, "Walk")
    gltf, _bin_chunk = load_glb(path)
    names = [anim["name"] for anim in gltf.get("animations", [])]
    assert names == ["Walk"]


def test_humanoid_heavy_walk_clip_on_disk() -> None:
    path = REPO_ROOT / "content/characters/animations/clips/HumanoidHeavy/Walk.glb"
    gltf, _bin_chunk = load_glb(path)
    assert gltf["animations"][0]["name"] == "Walk"
