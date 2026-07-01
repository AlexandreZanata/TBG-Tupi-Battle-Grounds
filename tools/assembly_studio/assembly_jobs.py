"""Assembly Studio background job store."""

from __future__ import annotations

import threading
import uuid
from dataclasses import asdict, dataclass, field
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from character_studio.paths import find_repo_root

REPO = find_repo_root()


@dataclass
class AssemblyJob:
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
_jobs: dict[str, AssemblyJob] = {}


def create_job(recipe_data: dict[str, Any], *, repo_root: Path | None = None) -> AssemblyJob:
    unit_id = str(recipe_data["unitId"])
    job = AssemblyJob(id=uuid.uuid4().hex[:12], unit_id=unit_id)
    root = repo_root or REPO
    with _lock:
        _jobs[job.id] = job
    threading.Thread(target=_run_job, args=(job.id, recipe_data, root), daemon=True).start()
    return job


def get_job(job_id: str) -> AssemblyJob | None:
    with _lock:
        return _jobs.get(job_id)


def assembly_status(repo_root: Path | None = None) -> dict[str, Any]:
    root = repo_root or REPO
    blender = root / "tools" / "assembly_studio" / "blender" / "blender_cmd.sh"
    return {
        "assemblerReady": True,
        "blenderCmd": str(blender),
        "exportsDir": str(root / "content" / "characters" / "library" / "exports"),
    }


def _run_job(job_id: str, recipe_data: dict[str, Any], repo_root: Path) -> None:
    from assembly_studio.assembly_job_runner import dispatch_assembly  # noqa: PLC0415

    with _lock:
        job = _jobs[job_id]
        job.status = "running"
    try:
        report = dispatch_assembly(recipe_data, repo_root)
        with _lock:
            job.report = report
            job.status = "completed"
            job.finished_at = datetime.now(UTC).isoformat()
    except Exception as exc:  # noqa: BLE001 — surface to UI
        with _lock:
            job.error = str(exc)
            job.status = "failed"
            job.finished_at = datetime.now(UTC).isoformat()
