"""MB-Lab generation background jobs."""

from __future__ import annotations

import threading
import uuid
from dataclasses import asdict, dataclass, field
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from assembly_studio.mblab_setup import _is_mblab_body
from assembly_studio.library.sockets import BASE_IDS
from character_studio.paths import find_repo_root

REPO = find_repo_root()


@dataclass
class MblabJob:
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
_jobs: dict[str, MblabJob] = {}


def create_mblab_job(recipe_data: dict[str, Any], *, repo_root: Path | None = None) -> MblabJob:
    unit_id = str(recipe_data["unitId"])
    job = MblabJob(id=uuid.uuid4().hex[:12], unit_id=unit_id)
    root = repo_root or REPO
    with _lock:
        _jobs[job.id] = job
    threading.Thread(target=_run_job, args=(job.id, recipe_data, root), daemon=True).start()
    return job


def get_mblab_job(job_id: str) -> MblabJob | None:
    with _lock:
        return _jobs.get(job_id)


def mblab_status(repo_root: Path | None = None) -> dict[str, Any]:
    root = repo_root or REPO
    projects = root / "content" / "characters" / "mblab" / "projects"
    library = root / "content" / "characters" / "library" / "bases"
    templates = {base_id: (projects / f"{base_id}.blend").is_file() for base_id in BASE_IDS}
    return {
        "mblabVersion": "1.8.1",
        "mblabTag": "1_8_1",
        "mblabVendor": str(root / "third_party" / "MB-Lab"),
        "vendorScript": "./scripts/vendor-mblab.sh",
        "installScript": "./scripts/install-mblab.sh",
        "templates": templates,
        "templatesReady": all(templates.values()),
        "libraryBases": {base_id: _is_mblab_body(library / base_id / "body.glb") for base_id in BASE_IDS},
        "libraryBasesReady": all(_is_mblab_body(library / bid / "body.glb") for bid in BASE_IDS),
        "ready": all(templates.values()) and all(_is_mblab_body(library / bid / "body.glb") for bid in BASE_IDS),
        "projectsDir": str(projects),
        "previewUrls": {bid: f"/api/assembly/bases/{bid}/preview" for bid in BASE_IDS},
        "previewReady": {
            bid: (library / bid / "preview.glb").is_file() for bid in BASE_IDS
        },
    }


def create_mblab_setup_job(*, repo_root: Path | None = None) -> MblabJob:
    job = MblabJob(id=uuid.uuid4().hex[:12], unit_id="_mblab_setup")
    root = repo_root or REPO
    with _lock:
        _jobs[job.id] = job
    threading.Thread(target=_run_setup_job, args=(job.id, root), daemon=True).start()
    return job


def _run_setup_job(job_id: str, repo_root: Path) -> None:
    from assembly_studio.mblab_setup import run_full_setup  # noqa: PLC0415

    with _lock:
        job = _jobs[job_id]
        job.status = "running"
    try:
        report = run_full_setup(repo_root)
        with _lock:
            job.report = report
            job.status = "completed"
            job.finished_at = datetime.now(UTC).isoformat()
    except Exception as exc:  # noqa: BLE001
        with _lock:
            job.error = str(exc)
            job.status = "failed"
            job.finished_at = datetime.now(UTC).isoformat()


def _run_job(job_id: str, recipe_data: dict[str, Any], repo_root: Path) -> None:
    from assembly_studio.mblab_job_runner import dispatch_mblab  # noqa: PLC0415

    with _lock:
        job = _jobs[job_id]
        job.status = "running"
    try:
        report = dispatch_mblab(recipe_data, repo_root)
        with _lock:
            job.report = report
            job.status = "completed"
            job.finished_at = datetime.now(UTC).isoformat()
    except Exception as exc:  # noqa: BLE001 — surface to UI
        with _lock:
            job.error = str(exc)
            job.status = "failed"
            job.finished_at = datetime.now(UTC).isoformat()
