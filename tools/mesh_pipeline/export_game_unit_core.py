"""Export pilot game meshes — Blender subprocess + Python fallback."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path

from animation_sets.validation import AnimationSet, validate_animation_set
from mesh_pipeline.game_glb_assemble import assemble_skinned_game_glb
from mesh_pipeline.game_glb_validate import ValidationResult, validate_game_glb

PILOT_UNIT_ID = "demo_sentinel_01"
DEFAULT_ANIMATION_SET = "humanoid_heavy_combat_v1"
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


def repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def blender_available() -> bool:
    return BLENDER_CMD.is_file() and os.access(BLENDER_CMD, os.X_OK)


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


def _run_gltf_transform(args: list[str]) -> None:
    npx = shutil.which("npx")
    if npx is None:
        raise RuntimeError("npx is required for gltf-transform post-process")
    subprocess.run([npx, "--yes", "@gltf-transform/cli", *args], check=True)


def post_process_game_glb(raw_path: Path, output_path: Path, texture_size: int = 1024) -> Path:
    """Resize textures and Draco-compress a Blender-exported game GLB."""
    tmp = output_path.with_suffix(".post.glb")
    _run_gltf_transform(["resize", str(raw_path), str(tmp), "--width", str(texture_size), "--height", str(texture_size)])
    _run_gltf_transform(["draco", str(tmp), str(output_path)])
    tmp.unlink(missing_ok=True)
    return output_path


def run_blender_export(unit_id: str, repo: Path | None = None) -> Path:
    repo = repo or repo_root()
    if not blender_available():
        raise RuntimeError("Blender not available — run tools/blender/setup.sh")

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
        if "Traceback" in combined:
            raise RuntimeError("Blender export failed — see traceback above")
        if proc.returncode != 0:
            raise RuntimeError("Blender export failed")

    config = load_export_config(unit_id, repo)
    raw = config.game_mesh.with_suffix(".blender-raw.glb")
    if not raw.is_file():
        raise RuntimeError(f"Blender export produced no output: {raw}")
    post_process_game_glb(raw, config.game_mesh)
    raw.unlink(missing_ok=True)
    return config.game_mesh


def export_game_unit_python(config: GameUnitExportConfig, repo: Path | None = None) -> Path:
    """Build a pilot game GLB without Blender (proxy skinned mesh + clip stubs)."""
    repo = repo or repo_root()
    if not config.showcase_mesh.is_file():
        raise FileNotFoundError(f"showcase mesh missing: {config.showcase_mesh}")

    clip_names = _load_animation_names(config, repo)
    tmp = config.game_mesh.with_suffix(".raw.glb")
    assemble_skinned_game_glb(tmp, clip_names)
    _run_gltf_transform(["draco", str(tmp), str(config.game_mesh)])
    tmp.unlink(missing_ok=True)
    return config.game_mesh


def export_game_unit(unit_id: str, repo: Path | None = None, *, prefer_blender: bool = True) -> ValidationResult:
    repo = repo or repo_root()
    config = load_export_config(unit_id, repo)

    if prefer_blender and blender_available():
        print("export: using Blender pipeline")
        run_blender_export(unit_id, repo)
    else:
        print("export: using Python fallback (install Blender — tools/blender/setup.sh)")
        export_game_unit_python(config, repo)

    result = validate_game_glb(config.game_mesh)
    if not result.ok:
        details = "; ".join(result.errors)
        raise RuntimeError(f"game GLB validation failed for {config.game_mesh}: {details}")
    return result
