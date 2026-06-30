"""Export pilot game meshes — Python fallback and Blender orchestration hooks."""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

from animation_sets.validation import AnimationSet, validate_animation_set
from mesh_pipeline.game_glb_assemble import assemble_skinned_game_glb
from mesh_pipeline.game_glb_validate import ValidationResult, validate_game_glb

PILOT_UNIT_ID = "demo_sentinel_01"
DEFAULT_ANIMATION_SET = "humanoid_heavy_combat_v1"


@dataclass(frozen=True)
class GameUnitExportConfig:
    unit_id: str
    showcase_mesh: Path
    game_mesh: Path
    animation_set_id: str
    body_class: str


def repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def load_export_config(unit_id: str, root: Path | None = None) -> GameUnitExportConfig:
    if unit_id != PILOT_UNIT_ID:
        raise ValueError(f"unsupported unit id: {unit_id} (Step C pilot: {PILOT_UNIT_ID} only)")

    repo = root or repo_root()
    definition_path = repo / "content/characters/definitions" / f"{unit_id}.json"
    if not definition_path.is_file():
        raise FileNotFoundError(definition_path)

    definition = json.loads(definition_path.read_text(encoding="utf-8"))
    showcase = definition.get("meshShowcase") or definition.get("mesh")
    if not showcase:
        raise ValueError(f"{unit_id}: missing showcase mesh path")

    animation_set_id = definition.get("animationSet", DEFAULT_ANIMATION_SET)
    return GameUnitExportConfig(
        unit_id=unit_id,
        showcase_mesh=repo / showcase,
        game_mesh=repo / "content/characters/meshes/game" / f"{unit_id}.glb",
        animation_set_id=animation_set_id,
        body_class="HumanoidHeavy",
    )


def _load_animation_names(config: GameUnitExportConfig, repo: Path) -> list[str]:
    set_path = repo / "content/characters/animation_sets" / f"{config.animation_set_id}.json"
    data = json.loads(set_path.read_text(encoding="utf-8"))
    defn = AnimationSet.from_dict(data)
    validate_animation_set(defn, repo)
    seen: list[str] = []
    for clip_name in defn.clips.values():
        if clip_name not in seen:
            seen.append(clip_name)
    return seen


def _compress_draco(input_path: Path, output_path: Path) -> None:
    npx = shutil.which("npx")
    if npx is None:
        raise RuntimeError("npx is required to Draco-compress game GLB output")
    subprocess.run(
        [npx, "--yes", "@gltf-transform/cli", "draco", str(input_path), str(output_path)],
        check=True,
    )


def export_game_unit_python(config: GameUnitExportConfig, repo: Path | None = None) -> ValidationResult:
    """Build a pilot game GLB without Blender (proxy skinned mesh + clip stubs)."""
    repo = repo or repo_root()
    if not config.showcase_mesh.is_file():
        raise FileNotFoundError(f"showcase mesh missing: {config.showcase_mesh}")

    clip_names = _load_animation_names(config, repo)
    tmp = config.game_mesh.with_suffix(".raw.glb")
    assemble_skinned_game_glb(tmp, clip_names)
    _compress_draco(tmp, config.game_mesh)
    tmp.unlink(missing_ok=True)
    return validate_game_glb(config.game_mesh)


def export_game_unit_blender(config: GameUnitExportConfig, repo: Path | None = None) -> ValidationResult:
    """Run Blender pipeline scripts when bpy is available."""
    repo = repo or repo_root()
    tools_dir = repo / "tools"
    scripts_dir = tools_dir / "blender" / "scripts"
    for path in (tools_dir, scripts_dir):
        if str(path) not in sys.path:
            sys.path.insert(0, str(path))

    from sculpt_to_game_mesh import sculpt_to_game_mesh  # noqa: PLC0415
    from attach_tbg_rig import attach_tbg_rig  # noqa: PLC0415
    from retarget_clip_library import retarget_clip_library  # noqa: PLC0415

    low_mesh = repo / "content" / "characters" / "meshes" / ".tmp" / f"{config.unit_id}_low.glb"
    skinned_mesh = low_mesh.with_name(f"{config.unit_id}_skinned.glb")
    low_mesh.parent.mkdir(parents=True, exist_ok=True)

    sculpt_to_game_mesh(config.showcase_mesh, low_mesh)
    attach_tbg_rig(low_mesh, config.body_class, skinned_mesh)
    retarget_clip_library(skinned_mesh, config.animation_set_id, config.game_mesh, repo)
    low_mesh.unlink(missing_ok=True)
    skinned_mesh.unlink(missing_ok=True)
    return validate_game_glb(config.game_mesh)


def export_game_unit(unit_id: str, repo: Path | None = None) -> ValidationResult:
    repo = repo or repo_root()
    config = load_export_config(unit_id, repo)
    try:
        import bpy  # noqa: F401

        result = export_game_unit_blender(config, repo)
    except ImportError:
        result = export_game_unit_python(config, repo)

    if not result.ok:
        details = "; ".join(result.errors)
        raise RuntimeError(f"game GLB validation failed for {config.game_mesh}: {details}")
    return result
