"""Rig Studio job store and session paths."""

from __future__ import annotations

import threading
import uuid
from dataclasses import asdict, dataclass, field
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from tools.experimental.auto_rig.rig_bone_names import normalize_bones_payload

from mesh_pipeline.export_game_unit_core import load_export_config, repo_root
from mesh_pipeline.prep_ai_mesh_run import blender_available

REPO = repo_root()
BLENDER_CMD = REPO / "tools/blender/blender_cmd.sh"
RIG_STUDIO_BPY = REPO / "tools/experimental/auto_rig/rig_studio_bpy.py"
PROMOTE_SCRIPT = REPO / "tools/experimental/auto_rig/promote_arena_pilot.py"
PROMOTE_WALK_SCRIPT = REPO / "tools/experimental/auto_rig/promote_rig_session_walk.py"
SESSIONS_ROOT = REPO / ".local/rig-studio/sessions"
JOB_KINDS = frozenset({"prep", "finalize", "auto", "bake_walk"})


@dataclass
class RigJob:
    id: str
    unit_id: str
    kind: str
    status: str = "queued"
    created_at: str = field(default_factory=lambda: datetime.now(UTC).isoformat())
    finished_at: str | None = None
    report: dict[str, Any] | None = None
    error: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


_lock = threading.Lock()
_jobs: dict[str, RigJob] = {}


def session_blend_path(unit_id: str) -> Path:
    return SESSIONS_ROOT / unit_id / "session.blend"


def session_preview_path(unit_id: str) -> Path:
    return SESSIONS_ROOT / unit_id / "preview.glb"


def session_bones_path(unit_id: str) -> Path:
    return SESSIONS_ROOT / unit_id / "bones.json"


def _file_meta(path: Path) -> dict[str, Any]:
    if not path.is_file():
        return {"exists": False, "path": str(path)}
    stat = path.stat()
    return {
        "exists": True,
        "path": str(path),
        "size": stat.st_size,
        "modifiedAt": datetime.fromtimestamp(stat.st_mtime, UTC).isoformat(),
    }


def session_meta(unit_id: str) -> dict[str, Any]:
    return {
        "blend": _file_meta(session_blend_path(unit_id)),
        "preview": _file_meta(session_preview_path(unit_id)),
        "bones": _file_meta(session_bones_path(unit_id)),
    }


def load_bones_payload(unit_id: str) -> dict[str, Any]:
    path = session_bones_path(unit_id)
    if not path.is_file():
        raise FileNotFoundError(f"bones.json missing for {unit_id}")
    import json

    return json.loads(path.read_text(encoding="utf-8"))


def save_bones_payload(unit_id: str, payload: dict[str, Any]) -> dict[str, Any]:
    path = session_bones_path(unit_id)
    path.parent.mkdir(parents=True, exist_ok=True)
    import json

    normalized = normalize_bones_payload(payload)
    path.write_text(json.dumps(normalized, indent=2), encoding="utf-8")
    return _file_meta(path)


def unit_rig_config(unit_id: str) -> dict[str, Any]:
    config = load_export_config(unit_id, REPO)
    return {
        "unitId": config.unit_id,
        "bodyClass": config.body_class,
        "animationSet": config.animation_set_id,
        "showcaseMesh": str(config.showcase_mesh),
        "gameMesh": str(config.game_mesh),
        "showcaseExists": config.showcase_mesh.is_file(),
    }


def launch_command(unit_id: str) -> dict[str, str]:
    blend = session_blend_path(unit_id).resolve()
    root = REPO.resolve()
    native = f'blender "{blend}"'
    flatpak = (
        f'flatpak run --filesystem="{root}" --filesystem=home '
        f'org.blender.Blender "{blend}"'
    )
    script = REPO / "tools/blender/open_blend.sh"
    helper = f'"{script}" "{blend}"'
    return {"native": native, "flatpak": flatpak, "helper": helper, "blendPath": str(blend)}


def create_job(unit_id: str, kind: str, *, fast: bool = True) -> RigJob:
    if kind not in JOB_KINDS:
        raise ValueError(f"invalid job kind: {kind}")
    job = RigJob(id=uuid.uuid4().hex[:12], unit_id=unit_id, kind=kind)
    with _lock:
        _jobs[job.id] = job
    threading.Thread(target=_run_job, args=(job.id, fast), daemon=True).start()
    return job


def get_job(job_id: str) -> RigJob | None:
    with _lock:
        return _jobs.get(job_id)


def rig_status() -> dict[str, Any]:
    return {
        "blenderAvailable": blender_available(),
        "blenderCmd": str(BLENDER_CMD),
        "sessionsRoot": str(SESSIONS_ROOT),
    }


def save_session_blend(unit_id: str, data: bytes) -> dict[str, Any]:
    if len(data) > 500 * 1024 * 1024:
        raise ValueError("blend file exceeds 500 MB limit")
    path = session_blend_path(unit_id)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return session_meta(unit_id)


def _run_job(job_id: str, fast: bool) -> None:
    from .rig_job_runner import dispatch_rig_job  # noqa: PLC0415

    with _lock:
        job = _jobs[job_id]
        job.status = "running"
    try:
        report = dispatch_rig_job(job.unit_id, job.kind, fast=fast)
        with _lock:
            job.report = report
            job.status = "completed"
            job.finished_at = datetime.now(UTC).isoformat()
    except Exception as exc:  # noqa: BLE001 — surface to UI
        with _lock:
            job.error = str(exc)
            job.status = "failed"
            job.finished_at = datetime.now(UTC).isoformat()
