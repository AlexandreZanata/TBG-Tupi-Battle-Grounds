"""Export pilot game meshes — Blender subprocess + Python fallback."""

from __future__ import annotations

import json
import os
import subprocess
from dataclasses import dataclass
from pathlib import Path

from animation_sets.validation import AnimationSet, validate_animation_set
from mesh_pipeline.game_glb_assemble import assemble_skinned_game_glb
from mesh_pipeline.game_glb_simplify import prepare_unit_low_meshes
from mesh_pipeline.game_glb_transform import run_gltf_transform
from mesh_pipeline.game_glb_validate import ValidationResult, validate_game_glb
from mesh_pipeline.glb_inspect import inspect_glb
from mesh_pipeline.glb_mesh_integrity import integrity_error

DEFAULT_ANIMATION_SET = "humanoid_heavy_combat_v1"
DEFAULT_TRI_TARGET = 8000
DEFAULT_TEXTURE_SIZE = 1024
BLENDER_CMD = Path(__file__).resolve().parents[1] / "blender" / "blender_cmd.sh"
BLENDER_BPY_SCRIPT = (
    Path(__file__).resolve().parents[1] / "blender" / "scripts" / "export_game_unit_bpy.py"
)


@dataclass(frozen=True)
class GameUnitExportConfig:
    unit_id: str
    showcase_mesh: Path
    game_mesh: Path
    animation_set_id: str
    body_class: str
    tri_target: int = DEFAULT_TRI_TARGET
    texture_size: int = DEFAULT_TEXTURE_SIZE


def repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def blender_available() -> bool:
    return BLENDER_CMD.is_file() and os.access(BLENDER_CMD, os.X_OK)


def _load_pilot_body_class(unit_id: str, repo: Path) -> str:
    roster_path = repo / "content/characters/pilot_roster.json"
    data = json.loads(roster_path.read_text(encoding="utf-8"))
    for entry in data.get("pilots", []):
        if entry.get("id") == unit_id:
            return str(entry["bodyClass"])
    raise ValueError(f"{unit_id}: not found in pilot_roster.json")


def load_export_config(
    unit_id: str,
    root: Path | None = None,
    *,
    body_class_override: str | None = None,
) -> GameUnitExportConfig:
    repo = root or repo_root()
    definition_path = repo / "content/characters/definitions" / f"{unit_id}.json"
    if not definition_path.is_file():
        raise FileNotFoundError(definition_path)

    definition = json.loads(definition_path.read_text(encoding="utf-8"))
    showcase = definition.get("meshShowcase") or definition.get("mesh")
    if not showcase:
        raise ValueError(f"{unit_id}: missing showcase mesh path")

    game_lod = definition.get("gameLod") or {}
    body_class = body_class_override or _load_pilot_body_class(unit_id, repo)
    return GameUnitExportConfig(
        unit_id=unit_id,
        showcase_mesh=repo / showcase,
        game_mesh=repo / "content/characters/meshes/game" / f"{unit_id}.glb",
        animation_set_id=definition.get("animationSet", DEFAULT_ANIMATION_SET),
        body_class=body_class,
        tri_target=int(game_lod.get("triTarget", DEFAULT_TRI_TARGET)),
        texture_size=int(game_lod.get("textureSize", DEFAULT_TEXTURE_SIZE)),
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


def post_process_game_glb(raw_path: Path, output_path: Path, texture_size: int = 1024) -> Path:
    """Resize textures, pack ORM, strip preview-hostile extensions, Draco-compress."""
    from mesh_pipeline.game_glb_texture_pack import pack_orm_textures
    from mesh_pipeline.game_glb_strip_extensions import strip_preview_extensions

    mesh_error = integrity_error(raw_path)
    if mesh_error:
        raise RuntimeError(f"mesh integrity failed for {raw_path}: {mesh_error}")

    tmp = output_path.with_suffix(".post.glb")
    run_gltf_transform(
        ["resize", str(raw_path), str(tmp), "--width", str(texture_size), "--height", str(texture_size)]
    )
    pack_orm_textures(tmp)
    strip_preview_extensions(tmp)
    run_gltf_transform(["draco", str(tmp), str(output_path)])
    tmp.unlink(missing_ok=True)
    return output_path


def run_blender_export(
    unit_id: str,
    repo: Path | None = None,
    *,
    body_class_override: str | None = None,
) -> Path:
    repo = repo or repo_root()
    if not blender_available():
        raise RuntimeError("Blender not available — run tools/blender/setup.sh")

    config = load_export_config(unit_id, repo, body_class_override=body_class_override)
    print(f"export: simplifying {unit_id} via gltf-transform")
    prepare_unit_low_meshes(
        config.unit_id,
        config.showcase_mesh,
        repo,
        config.tri_target,
        config.texture_size,
    )

    script = BLENDER_BPY_SCRIPT.resolve()
    proc = subprocess.run(
        [str(BLENDER_CMD.resolve()), "--background", "--python", str(script), "--", unit_id],
        cwd=repo,
        capture_output=True,
        text=True,
        check=False,
    )
    combined = f"{proc.stdout}\n{proc.stderr}".strip()
    if combined:
        print(combined)
    if proc.returncode != 0 or "Traceback" in combined:
        raise RuntimeError("Blender export failed — see traceback above")

    raw = config.game_mesh.with_suffix(".blender-raw.glb")
    if not raw.is_file():
        raise RuntimeError(f"Blender export produced no output: {raw}")
    from mesh_pipeline.game_glb_simplify import trim_to_tri_budget

    if inspect_glb(raw).triangles > config.tri_target:
        trim_to_tri_budget(raw, config.tri_target)
    post_process_game_glb(raw, config.game_mesh, config.texture_size)
    raw.unlink(missing_ok=True)
    return config.game_mesh


def export_game_unit_python(config: GameUnitExportConfig, repo: Path | None = None) -> Path:
    """Build a game GLB without Blender (proxy skinned mesh + clip stubs)."""
    repo = repo or repo_root()
    if not config.showcase_mesh.is_file():
        raise FileNotFoundError(f"showcase mesh missing: {config.showcase_mesh}")

    clip_names = _load_animation_names(config, repo)
    tmp = config.game_mesh.with_suffix(".raw.glb")
    config.game_mesh.parent.mkdir(parents=True, exist_ok=True)
    assemble_skinned_game_glb(tmp, clip_names)
    run_gltf_transform(["draco", str(tmp), str(config.game_mesh)])
    tmp.unlink(missing_ok=True)
    return config.game_mesh


def export_game_unit(
    unit_id: str,
    repo: Path | None = None,
    *,
    prefer_blender: bool = True,
    body_class_override: str | None = None,
) -> ValidationResult:
    repo = repo or repo_root()
    config = load_export_config(unit_id, repo, body_class_override=body_class_override)

    if prefer_blender and blender_available():
        print("export: using Blender pipeline")
        run_blender_export(unit_id, repo, body_class_override=body_class_override)
    else:
        print("export: using Python fallback (install Blender — tools/blender/setup.sh)")
        export_game_unit_python(config, repo)

    result = validate_game_glb(config.game_mesh)
    if not result.ok:
        details = "; ".join(result.errors)
        raise RuntimeError(f"game GLB validation failed for {config.game_mesh}: {details}")
    return result
