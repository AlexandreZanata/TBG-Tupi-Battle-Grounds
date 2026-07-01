"""Rig Studio REST routes."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from fastapi import APIRouter, HTTPException, UploadFile
from fastapi.responses import FileResponse
from pydantic import BaseModel, Field

from .rig_jobs import (
    create_job,
    get_job,
    load_bones_payload,
    rig_status,
    save_bones_payload,
    save_session_blend,
    session_bones_path,
    session_meta,
    session_preview_path,
    unit_rig_config,
)

router = APIRouter(prefix="/api/rig", tags=["rig"])


class RigJobRequest(BaseModel):
    unitId: str
    kind: str = Field(pattern="^(prep|finalize|auto|bake_walk)$")
    fast: bool = True


class BonesPayload(BaseModel):
    version: int = 1
    armature: str | None = None
    bones: dict[str, Any]


@router.get("/status")
def get_rig_status() -> dict:
    return rig_status()


@router.get("/units/{unit_id}/config")
def get_rig_unit_config(unit_id: str) -> dict:
    try:
        return unit_rig_config(unit_id)
    except (FileNotFoundError, ValueError) as exc:
        raise HTTPException(status_code=404, detail=str(exc)) from exc


@router.get("/sessions/{unit_id}")
def get_rig_session(unit_id: str) -> dict:
    return {"unitId": unit_id, "session": session_meta(unit_id)}


@router.get("/sessions/{unit_id}/preview")
def get_session_preview(unit_id: str) -> FileResponse:
    path = session_preview_path(unit_id)
    if not path.is_file():
        raise HTTPException(status_code=404, detail="preview glb not found — run prep first")
    return FileResponse(path, media_type="model/gltf-binary", filename="preview.glb")


@router.get("/sessions/{unit_id}/bones")
def get_session_bones(unit_id: str) -> dict:
    try:
        return load_bones_payload(unit_id)
    except FileNotFoundError as exc:
        raise HTTPException(status_code=404, detail=str(exc)) from exc


@router.put("/sessions/{unit_id}/bones")
def put_session_bones(unit_id: str, body: BonesPayload) -> dict:
    meta = save_bones_payload(unit_id, body.model_dump())
    return {"saved": True, "bones": meta}


@router.get("/sessions/{unit_id}/blend")
def download_session_blend(unit_id: str) -> FileResponse:
    meta = session_meta(unit_id)["blend"]
    if not meta.get("exists"):
        raise HTTPException(status_code=404, detail="session blend not found")
    path = Path(meta["path"])
    return FileResponse(path, media_type="application/octet-stream", filename="session.blend")


@router.put("/sessions/{unit_id}/blend")
async def upload_session_blend(unit_id: str, file: UploadFile) -> dict:
    data = await file.read()
    try:
        return save_session_blend(unit_id, data)
    except ValueError as exc:
        raise HTTPException(status_code=413, detail=str(exc)) from exc


@router.post("/jobs", status_code=202)
def start_rig_job(body: RigJobRequest) -> dict:
    if not rig_status()["blenderAvailable"]:
        raise HTTPException(status_code=503, detail="Blender not available — run tools/blender/setup.sh")
    try:
        unit_rig_config(body.unitId)
    except (FileNotFoundError, ValueError) as exc:
        raise HTTPException(status_code=404, detail=str(exc)) from exc
    job = create_job(body.unitId, body.kind, fast=body.fast)
    return job.to_dict()


@router.get("/jobs/{job_id}")
def get_rig_job(job_id: str) -> dict:
    job = get_job(job_id)
    if job is None:
        raise HTTPException(status_code=404, detail="job not found")
    return job.to_dict()
