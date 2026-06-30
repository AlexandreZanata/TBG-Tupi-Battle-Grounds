"""Blender-only game unit export (no host deps — safe inside bpy)."""

from __future__ import annotations

import sys
from pathlib import Path

from animation_sets.validation import AnimationSet, validate_animation_set

PILOT_UNIT_ID = "demo_sentinel_01"
DEFAULT_ANIMATION_SET = "humanoid_heavy_combat_v1"


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

    sculpt_to_game_mesh(showcase_mesh, low_mesh)
    attach_tbg_rig(low_mesh, body_class, skinned_mesh)
    retarget_clip_library(skinned_mesh, animation_set_id, raw_output, repo)
    low_mesh.unlink(missing_ok=True)
    skinned_mesh.unlink(missing_ok=True)
    return raw_output
