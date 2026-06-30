"""Blender-only game unit export (no host deps — safe inside bpy)."""

from __future__ import annotations

import sys
from pathlib import Path

from animation_sets.validation import AnimationSet, validate_animation_set

DEFAULT_ANIMATION_SET = "humanoid_heavy_combat_v1"
DEFAULT_TRI_TARGET = 8000
DEFAULT_TEXTURE_SIZE = 1024
LOD1_TRI_RATIO = 0.5


def repo_root_from_script() -> Path:
    return Path(__file__).resolve().parents[2]


def load_animation_set(animation_set_id: str, repo: Path) -> AnimationSet:
    set_path = repo / "content/characters/animation_sets" / f"{animation_set_id}.json"
    import json

    defn = AnimationSet.from_dict(json.loads(set_path.read_text(encoding="utf-8")))
    validate_animation_set(defn, repo)
    return defn


def export_game_unit_blender(
    unit_id: str,
    showcase_mesh: Path,
    game_mesh: Path,
    animation_set_id: str,
    body_class: str,
    repo: Path,
    tri_target: int = DEFAULT_TRI_TARGET,
    texture_size: int = DEFAULT_TEXTURE_SIZE,
) -> Path:
    tools_dir = repo / "tools"
    scripts_dir = tools_dir / "blender" / "scripts"
    for path in (tools_dir, scripts_dir):
        if str(path) not in sys.path:
            sys.path.insert(0, str(path))

    from attach_tbg_rig import attach_tbg_rig  # noqa: PLC0415
    from retarget_clip_library import retarget_clip_library  # noqa: PLC0415
    from sculpt_to_game_mesh import sculpt_to_game_mesh  # noqa: PLC0415

    tmp_dir = repo / "content/characters/meshes/.tmp"
    tmp_dir.mkdir(parents=True, exist_ok=True)
    low_mesh = tmp_dir / f"{unit_id}_low.glb"
    skinned_mesh = tmp_dir / f"{unit_id}_skinned.glb"
    raw_output = game_mesh.with_suffix(".blender-raw.glb")
    lod1_dir = repo / "content/characters/meshes/game/lod1"
    lod1_dir.mkdir(parents=True, exist_ok=True)
    lod1_mesh = lod1_dir / f"{unit_id}.glb"
    lod1_target = max(500, int(tri_target * LOD1_TRI_RATIO))
    lod1_tex = max(256, texture_size // 2)

    # ponytail: low_mesh + lod1 pre-built on host via gltf-transform (see game_glb_simplify.py)
    tmp_dir = repo / "content/characters/meshes/.tmp"
    low_mesh = tmp_dir / f"{unit_id}_low.glb"
    if not low_mesh.is_file():
        sculpt_to_game_mesh(showcase_mesh, low_mesh, tri_target=tri_target, texture_size=texture_size)
        sculpt_to_game_mesh(low_mesh, lod1_mesh, tri_target=lod1_target, texture_size=lod1_tex)
    attach_tbg_rig(low_mesh, body_class, skinned_mesh)
    retarget_clip_library(skinned_mesh, animation_set_id, raw_output, repo)
    low_mesh.unlink(missing_ok=True)
    skinned_mesh.unlink(missing_ok=True)
    return raw_output
