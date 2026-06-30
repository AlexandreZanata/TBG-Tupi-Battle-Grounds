"""Batch game unit export — one Blender session + parallel post-process."""

from __future__ import annotations

import json
import os
import subprocess
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

from mesh_pipeline.export_game_unit_core import (
    BLENDER_CMD,
    blender_available,
    load_export_config,
    post_process_game_glb,
    repo_root,
)
from mesh_pipeline.game_glb_simplify import prepare_unit_low_meshes

BATCH_BPY_SCRIPT = (
    Path(__file__).resolve().parents[1] / "blender" / "scripts" / "batch_export_game_bpy.py"
)


def roster_unit_ids(repo: Path | None = None) -> list[str]:
    repo = repo or repo_root()
    roster_path = repo / "content/characters/pilot_roster.json"
    data = json.loads(roster_path.read_text(encoding="utf-8"))
    return [str(entry["id"]) for entry in data.get("pilots", [])]


def _prepare_low_meshes(unit_ids: list[str], repo: Path, workers: int) -> list[str]:
    failed: list[str] = []

    def _prep(unit_id: str) -> tuple[str, str | None]:
        config = load_export_config(unit_id, repo)
        try:
            prepare_unit_low_meshes(
                config.unit_id,
                config.showcase_mesh,
                repo,
                config.tri_target,
                config.texture_size,
            )
            return unit_id, None
        except Exception as exc:  # noqa: BLE001
            return unit_id, str(exc)

    worker_count = max(1, min(workers, len(unit_ids)))
    if worker_count == 1:
        for unit_id in unit_ids:
            uid, error = _prep(unit_id)
            if error:
                print(f"simplify FAILED {uid}: {error}")
                failed.append(uid)
            else:
                print(f"simplify OK {uid}")
        return failed

    with ThreadPoolExecutor(max_workers=worker_count) as pool:
        futures = {pool.submit(_prep, uid): uid for uid in unit_ids}
        for future in as_completed(futures):
            uid, error = future.result()
            if error:
                print(f"simplify FAILED {uid}: {error}")
                failed.append(uid)
            else:
                print(f"simplify OK {uid}")
    return failed


def run_blender_batch(unit_ids: list[str], repo: Path | None = None) -> None:
    repo = repo or repo_root()
    if not blender_available():
        raise RuntimeError("Blender not available — run tools/blender/setup.sh")

    script = BATCH_BPY_SCRIPT.resolve()
    proc = subprocess.run(
        [str(BLENDER_CMD.resolve()), "--background", "--python", str(script), "--", *unit_ids],
        cwd=repo,
        capture_output=True,
        text=True,
        check=False,
    )
    combined = f"{proc.stdout}\n{proc.stderr}".strip()
    if combined:
        print(combined)
    if proc.returncode != 0 or "Traceback" in combined:
        raise RuntimeError("Blender batch export failed — see output above")


def post_process_raw_exports(
    unit_ids: list[str], repo: Path | None = None, *, workers: int = 1
) -> list[str]:
    repo = repo or repo_root()
    failed: list[str] = []

    def _process(unit_id: str) -> tuple[str, str | None]:
        config = load_export_config(unit_id, repo)
        raw = config.game_mesh.with_suffix(".blender-raw.glb")
        if not raw.is_file():
            return unit_id, f"missing raw export: {raw}"
        try:
            post_process_game_glb(raw, config.game_mesh, config.texture_size)
            raw.unlink(missing_ok=True)
            return unit_id, None
        except Exception as exc:  # noqa: BLE001 — collect per-unit errors
            return unit_id, str(exc)

    worker_count = max(1, min(workers, len(unit_ids)))
    if worker_count == 1:
        for unit_id in unit_ids:
            uid, error = _process(unit_id)
            if error:
                print(f"post-process FAILED {uid}: {error}")
                failed.append(uid)
            else:
                print(f"post-process OK {uid}")
        return failed

    with ThreadPoolExecutor(max_workers=worker_count) as pool:
        futures = {pool.submit(_process, uid): uid for uid in unit_ids}
        for future in as_completed(futures):
            uid, error = future.result()
            if error:
                print(f"post-process FAILED {uid}: {error}")
                failed.append(uid)
            else:
                print(f"post-process OK {uid}")
    return failed


def batch_export_game_units(
    unit_ids: list[str] | None = None,
    repo: Path | None = None,
    *,
    workers: int | None = None,
) -> list[str]:
    """Export all units; returns list of failed unit ids."""
    repo = repo or repo_root()
    ids = unit_ids or roster_unit_ids(repo)
    if not ids:
        return []

    parallel = workers if workers is not None else int(os.environ.get("TBG_BATCH_WORKERS", "4"))
    print(f"batch export: {len(ids)} units (workers={parallel})")
    prep_failed = _prepare_low_meshes(ids, repo, parallel)
    if prep_failed:
        return prep_failed
    run_blender_batch(ids, repo)
    return post_process_raw_exports(ids, repo, workers=parallel)
