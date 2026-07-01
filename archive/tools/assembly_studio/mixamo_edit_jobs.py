"""Mixamo Edit background jobs — Blender skin swap + TBG export."""

from __future__ import annotations

import json
import threading
import uuid
from dataclasses import asdict, dataclass, field
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from character_studio.paths import find_repo_root

REPO = find_repo_root()
MIXAMO_DIR = "content/characters/imports/mixamo"


@dataclass
class MixamoEditJob:
    id: str
    unit_id: str
    status: str = "queued"
    created_at: str = field(default_factory=lambda: datetime.now(UTC).isoformat())
    finished_at: str | None = None
    report: dict[str, Any] | None = None
    error: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


_lock = threading.Lock()
_jobs: dict[str, MixamoEditJob] = {}


def mixamo_edit_status(repo_root: Path | None = None) -> dict[str, Any]:
    root = repo_root or REPO
    base = root / MIXAMO_DIR
    raw = sorted(p.name for p in (base / "raw").glob("*.fbx") if p.is_file())
    built = sorted(p.stem for p in (base / "edit").glob("*.glb") if p.is_file())
    return {
        "rawFbx": raw,
        "builtUnits": built,
        "meshesDir": str(base / "meshes"),
        "editDir": str(base / "edit"),
    }


def create_mixamo_edit_job(payload: dict[str, Any], *, repo_root: Path | None = None) -> MixamoEditJob:
    unit_id = str(payload["unitId"])
    job = MixamoEditJob(id=uuid.uuid4().hex[:12], unit_id=unit_id)
    root = repo_root or REPO
    with _lock:
        _jobs[job.id] = job
    threading.Thread(target=_run_job, args=(job.id, payload, root), daemon=True).start()
    return job


def get_mixamo_edit_job(job_id: str) -> MixamoEditJob | None:
    with _lock:
        return _jobs.get(job_id)


def load_edit_manifest(repo_root: Path | None = None) -> dict[str, Any]:
    root = repo_root or REPO
    path = root / MIXAMO_DIR / "edit-manifest.json"
    if not path.is_file():
        return {"projects": []}
    return json.loads(path.read_text(encoding="utf-8"))


def save_edit_manifest(data: dict[str, Any], repo_root: Path | None = None) -> None:
    root = repo_root or REPO
    path = root / MIXAMO_DIR / "edit-manifest.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def delete_edit_project(unit_id: str, repo_root: Path | None = None) -> dict[str, Any]:
    root = repo_root or REPO
    base = root / MIXAMO_DIR
    removed: list[str] = []
    for rel in (f"edit/{unit_id}.glb", f"meshes/{unit_id}.glb"):
        path = base / rel
        if path.is_file():
            path.unlink()
            removed.append(rel)
    data = load_edit_manifest(root)
    projects = [p for p in data.get("projects", []) if p.get("id") != unit_id]
    if len(projects) == len(data.get("projects", [])) and not removed:
        raise ValueError(f"project not found: {unit_id}")
    save_edit_manifest({"projects": projects}, root)
    return {"unitId": unit_id, "removedFiles": removed}


def _run_job(job_id: str, payload: dict[str, Any], repo_root: Path) -> None:
    from assembly_studio.mixamo_edit_runner import dispatch_mixamo_edit  # noqa: PLC0415

    with _lock:
        job = _jobs[job_id]
        job.status = "running"
    try:
        report = dispatch_mixamo_edit(payload, repo_root)
        _upsert_manifest(report, repo_root)
        with _lock:
            job.report = report
            job.status = "completed"
            job.finished_at = datetime.now(UTC).isoformat()
    except Exception as exc:  # noqa: BLE001 — surface to UI
        with _lock:
            job.error = str(exc)
            job.status = "failed"
            job.finished_at = datetime.now(UTC).isoformat()


def _upsert_manifest(report: dict[str, Any], repo_root: Path) -> None:
    unit_id = str(report.get("unitId", ""))
    if not unit_id:
        return
    data = load_edit_manifest(repo_root)
    projects = [p for p in data.get("projects", []) if p.get("id") != unit_id]
    projects.insert(0, {
        "id": unit_id,
        "label": report.get("label", unit_id),
        "mixamoFbx": report.get("mixamoFbx"),
        "customMesh": report.get("customMesh"),
        "heightScale": report.get("heightScale", 1.0),
        "meshUrl": f"/api/content/characters/imports/mixamo/edit/{unit_id}.glb",
        "clipNames": report.get("clipNames", []),
    })
    save_edit_manifest({"projects": projects}, repo_root)
