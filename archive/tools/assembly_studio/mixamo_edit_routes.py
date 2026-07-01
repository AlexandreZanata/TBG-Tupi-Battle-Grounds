"""Mixamo Edit REST routes — register on assembly router."""

from __future__ import annotations

import re
from pathlib import Path
from typing import Any

from fastapi import APIRouter, File, HTTPException, UploadFile
from pydantic import BaseModel, Field

from assembly_studio.mixamo_edit_jobs import (
    create_mixamo_edit_job,
    delete_edit_project,
    get_mixamo_edit_job,
    load_edit_manifest,
    mixamo_edit_status,
)

_UNIT_RE = re.compile(r"^[a-z0-9_]+$")


class BoneProportions(BaseModel):
    hipWidth: float = Field(default=1.0, ge=0.5, le=2.0)
    shoulderWidth: float = Field(default=1.0, ge=0.5, le=2.0)
    torsoDepth: float = Field(default=1.0, ge=0.5, le=2.0)


class MixamoEditBuildRequest(BaseModel):
    unitId: str = Field(pattern=r"^[a-z0-9_]+$")
    mixamoFbx: str = Field(min_length=1, max_length=120)
    label: str | None = Field(default=None, max_length=80)
    customMesh: str | None = Field(default=None, max_length=120)
    heightScale: float = Field(default=1.0, ge=0.25, le=3.0)
    rigPosition: list[float] = Field(default_factory=lambda: [0.0, 0.0, 0.0])
    rigScale: float = Field(default=1.0, ge=0.25, le=3.0)
    manualAlign: bool = False
    boneProportions: BoneProportions | None = None


def register_mixamo_edit_routes(router: APIRouter, repo_root: Path) -> None:
    meshes_dir = repo_root / "content/characters/imports/mixamo/meshes"
    meshes_dir.mkdir(parents=True, exist_ok=True)
    edit_dir = repo_root / "content/characters/imports/mixamo/edit"
    edit_dir.mkdir(parents=True, exist_ok=True)

    @router.get("/mixamo-edit/status")
    def get_mixamo_edit_status() -> dict[str, Any]:
        return mixamo_edit_status(repo_root)

    @router.get("/mixamo-edit/manifest")
    def get_mixamo_edit_manifest() -> dict[str, Any]:
        return load_edit_manifest(repo_root)

    @router.post("/mixamo-edit/upload-mesh")
    async def upload_custom_mesh(unitId: str, file: UploadFile = File(...)) -> dict[str, str]:
        if not _UNIT_RE.match(unitId):
            raise HTTPException(status_code=400, detail="invalid unitId")
        if not file.filename or not file.filename.lower().endswith(".glb"):
            raise HTTPException(status_code=400, detail="upload must be a .glb file")
        dest = meshes_dir / f"{unitId}.glb"
        data = await file.read()
        if len(data) > 150 * 1024 * 1024:
            raise HTTPException(status_code=413, detail="mesh exceeds 150 MB limit")
        dest.write_bytes(data)
        return {"unitId": unitId, "customMesh": dest.name}

    @router.post("/mixamo-edit/build", status_code=202)
    def build_mixamo_edit(body: MixamoEditBuildRequest) -> dict[str, Any]:
        fbx = repo_root / "content/characters/imports/mixamo/raw" / Path(body.mixamoFbx).name
        if not fbx.is_file():
            raise HTTPException(status_code=404, detail=f"Mixamo FBX not found: {body.mixamoFbx}")
        if body.customMesh:
            mesh = meshes_dir / Path(body.customMesh).name
            if not mesh.is_file():
                raise HTTPException(status_code=404, detail=f"custom mesh not found: {body.customMesh}")
        job = create_mixamo_edit_job(body.model_dump(), repo_root=repo_root)
        return job.to_dict()

    @router.get("/mixamo-edit/jobs/{job_id}")
    def poll_mixamo_edit_job(job_id: str) -> dict[str, Any]:
        job = get_mixamo_edit_job(job_id)
        if job is None:
            raise HTTPException(status_code=404, detail="job not found")
        return job.to_dict()

    @router.delete("/mixamo-edit/projects/{unit_id}")
    def remove_mixamo_edit_project(unit_id: str) -> dict[str, Any]:
        if not _UNIT_RE.match(unit_id):
            raise HTTPException(status_code=400, detail="invalid unitId")
        try:
            return delete_edit_project(unit_id, repo_root=repo_root)
        except ValueError as exc:
            raise HTTPException(status_code=404, detail=str(exc)) from exc
