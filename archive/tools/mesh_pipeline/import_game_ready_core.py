"""Orchestrate AI mesh → showcase → game export → weight gate."""

from __future__ import annotations

import json
import os
import subprocess
from dataclasses import dataclass, field
from pathlib import Path

from character_studio.models import DEFAULT_RIG_PROFILE, CharacterDefinition
from mesh_pipeline.export_game_unit_core import export_game_unit, repo_root
from mesh_pipeline.game_glb_validate import validate_game_glb
from mesh_pipeline.mixamo_checklist import mixamo_checklist_lines
from mesh_pipeline.prep_ai_mesh_run import run_prep_ai_mesh
from mesh_pipeline.weight_quality import validate_weight_quality

EXCEPTION_WORKFLOW_DOC = ".local/phases/04b-biped-game-ready-pipeline/EXCEPTION-WORKFLOW.md"
SHOWCASE_REL = "content/characters/meshes/showcase/{unit_id}.glb"
GAME_REL = "content/characters/meshes/game/{unit_id}.glb"


@dataclass(frozen=True)
class ImportGameReadyConfig:
    source: Path
    unit_id: str
    body_class: str | None = None
    animation_set: str | None = None
    mixamo_fallback: bool = False
    skip_prep: bool = False


@dataclass
class ImportGameReadyReport:
    unit_id: str
    steps: list[str] = field(default_factory=list)
    showcase_path: Path | None = None
    game_path: Path | None = None
    weight_gate_pass: bool = False
    weight_errors: tuple[str, ...] = ()
    validate_ok: bool = False
    validate_errors: list[str] = field(default_factory=list)
    blocked: bool = False
    block_reason: str | None = None

    @property
    def ok(self) -> bool:
        return not self.blocked and self.weight_gate_pass and self.validate_ok


def weight_gate_enabled() -> bool:
    return os.environ.get("TBG_WEIGHT_GATE") == "1"


def load_unit_definition(unit_id: str, repo: Path | None = None) -> CharacterDefinition:
    repo = repo or repo_root()
    path = repo / "content/characters/definitions" / f"{unit_id}.json"
    if not path.is_file():
        raise FileNotFoundError(f"missing definition: {path}")
    return CharacterDefinition.from_dict(json.loads(path.read_text(encoding="utf-8")))


def rig_exception_block_message(defn: CharacterDefinition) -> str | None:
    if not defn.rigException:
        return None
    reason = f" ({defn.rigExceptionReason})" if defn.rigExceptionReason else ""
    return (
        f"{defn.id}: rigException is true{reason} — auto pipeline blocked. "
        f"See {EXCEPTION_WORKFLOW_DOC}"
    )


def sync_definition_paths(
    unit_id: str,
    *,
    animation_set: str | None,
    repo: Path | None = None,
) -> Path:
    repo = repo or repo_root()
    path = repo / "content/characters/definitions" / f"{unit_id}.json"
    data = json.loads(path.read_text(encoding="utf-8"))
    data["meshShowcase"] = SHOWCASE_REL.format(unit_id=unit_id)
    data["mesh"] = GAME_REL.format(unit_id=unit_id)
    if animation_set:
        data["animationSet"] = animation_set
    data.setdefault("rigProfile", DEFAULT_RIG_PROFILE)
    data.setdefault("rigException", False)
    data.setdefault("assetMode", "mesh3d")
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    return path


def import_showcase_from_source(
    source: Path,
    unit_id: str,
    *,
    skip_prep: bool,
    repo: Path | None = None,
) -> Path:
    repo = repo or repo_root()
    optimize = repo / "tools/mesh-pipeline/optimize_glb.sh"
    showcase = repo / SHOWCASE_REL.format(unit_id=unit_id)
    showcase.parent.mkdir(parents=True, exist_ok=True)
    pipeline_source = source.resolve()
    prep_tmp: Path | None = None
    if not skip_prep:
        prep_tmp = showcase.with_suffix(".prepped.glb")
        run_prep_ai_mesh(source, prep_tmp)
        pipeline_source = prep_tmp
    try:
        subprocess.run(
            [str(optimize), "--mode", "visual", str(pipeline_source), str(showcase)],
            check=True,
            cwd=repo,
        )
    finally:
        if prep_tmp is not None and prep_tmp.exists():
            prep_tmp.unlink(missing_ok=True)
    return showcase


def gate_failure_lines(
    unit_id: str,
    weight_errors: tuple[str, ...],
    *,
    mixamo_fallback: bool,
    showcase_path: Path | None,
) -> list[str]:
    lines = [f"Weight gate FAILED for {unit_id}:"]
    lines.extend(f"  - {error}" for error in weight_errors)
    if mixamo_fallback and showcase_path is not None:
        lines.append("Mixamo fallback:")
        lines.extend(f"  {line}" for line in mixamo_checklist_lines(showcase_path))
    else:
        lines.append(
            "Re-run with --mixamo-fallback for Mixamo upload steps, "
            "or fix weights manually."
        )
    lines.append(f"Manual exception rigs: {EXCEPTION_WORKFLOW_DOC}")
    return lines


def import_game_ready_unit(
    config: ImportGameReadyConfig,
    repo: Path | None = None,
) -> ImportGameReadyReport:
    repo = repo or repo_root()
    report = ImportGameReadyReport(unit_id=config.unit_id)
    defn = load_unit_definition(config.unit_id, repo)
    block = rig_exception_block_message(defn)
    if block:
        report.blocked = True
        report.block_reason = block
        return report

    sync_definition_paths(config.unit_id, animation_set=config.animation_set, repo=repo)
    report.steps.append("sync_definition")
    report.showcase_path = import_showcase_from_source(
        config.source,
        config.unit_id,
        skip_prep=config.skip_prep,
        repo=repo,
    )
    report.steps.append("showcase_import")

    export_game_unit(
        config.unit_id,
        repo,
        body_class_override=config.body_class,
    )
    report.steps.append("export_game_unit")
    game_path = repo / GAME_REL.format(unit_id=config.unit_id)
    report.game_path = game_path

    weight_result = validate_weight_quality(game_path)
    report.weight_gate_pass = weight_result.passes_gate
    report.weight_errors = weight_result.errors
    report.steps.append("weight_gate")

    validation = validate_game_glb(game_path, check_weights=True)
    report.validate_ok = validation.ok
    report.validate_errors = validation.errors
    report.steps.append("validate_game_glb")
    return report
