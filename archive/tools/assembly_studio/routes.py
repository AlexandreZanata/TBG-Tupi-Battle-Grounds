"""Assembly Studio REST routes."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from fastapi import APIRouter, HTTPException
from fastapi.responses import FileResponse
from pydantic import BaseModel, ConfigDict, Field

from assembly_studio.assembly_jobs import assembly_status, create_job, get_job
from assembly_studio.mblab_jobs import create_mblab_job, create_mblab_setup_job, get_mblab_job, mblab_status
from assembly_studio.library.recipe import build_recipe, validate_recipe
from assembly_studio.library_catalog import load_library_catalog
from assembly_studio.library.sockets import CATEGORY_SOCKETS
from assembly_studio.mixamo_edit_routes import register_mixamo_edit_routes


class AttachmentPayload(BaseModel):
    partId: str
    socket: str
    translation: list[float] = Field(default_factory=lambda: [0.0, 0.0, 0.0])
    rotation: list[float] = Field(default_factory=lambda: [0.0, 0.0, 0.0, 1.0])
    scale: list[float] = Field(default_factory=lambda: [1.0, 1.0, 1.0])


class ExportRecipeRequest(BaseModel):
    unitId: str = Field(pattern=r"^[a-z0-9_]+$")
    baseId: str = Field(pattern=r"^(male|female|small|large)$")
    parts: list[str] = Field(default_factory=list)
    materialId: str = Field(pattern=r"^[a-z0-9_]+$")
    attachments: list[AttachmentPayload] = Field(default_factory=list)


class MblabMorphs(BaseModel):
    model_config = ConfigDict(extra="allow")

    age: float | None = Field(default=None, ge=0.0, le=1.0)
    body_mass: float | None = Field(default=None, ge=0.0, le=1.0)
    muscles: float | None = Field(default=None, ge=0.0, le=1.0)
    stature: float | None = Field(default=None, ge=0.0, le=1.0)
    head_width: float | None = Field(default=None, ge=0.0, le=1.0)
    head_length: float | None = Field(default=None, ge=0.0, le=1.0)
    jaw_prominence: float | None = Field(default=None, ge=0.0, le=1.0)
    jaw_width: float | None = Field(default=None, ge=0.0, le=1.0)
    cheek_volume: float | None = Field(default=None, ge=0.0, le=1.0)
    cheek_bones: float | None = Field(default=None, ge=0.0, le=1.0)


class MblabGenerateRequest(BaseModel):
    unitId: str = Field(pattern=r"^[a-z0-9_]+$")
    baseId: str = Field(pattern=r"^(male|female|small|large)$")
    label: str | None = None
    promoteToLibrary: bool = False
    morphs: MblabMorphs | None = None


def create_assembly_router(repo_root: Path) -> APIRouter:
    router = APIRouter(prefix="/api/assembly", tags=["assembly"])

    @router.get("/library")
    def get_library() -> dict[str, Any]:
        return load_library_catalog(repo_root)

    @router.get("/skeleton")
    def get_skeleton() -> dict[str, Any]:
        sockets_path = repo_root / "content/characters/library/sockets.json"
        sockets = json.loads(sockets_path.read_text(encoding="utf-8"))
        bones = sorted(set(CATEGORY_SOCKETS.values()))
        return {"bones": bones, "sockets": sockets.get("sockets", {}), "categories": CATEGORY_SOCKETS}

    @router.get("/status")
    def get_status() -> dict:
        return assembly_status(repo_root)

    @router.post("/export", status_code=202)
    def export_character(body: ExportRecipeRequest) -> dict:
        data = body.model_dump()
        errors = validate_recipe(data)
        if errors:
            raise HTTPException(status_code=400, detail=errors)
        try:
            build_recipe(data, repo_root=repo_root)
        except (FileNotFoundError, ValueError) as exc:
            raise HTTPException(status_code=400, detail=str(exc)) from exc
        job = create_job(data, repo_root=repo_root)
        return job.to_dict()

    @router.get("/jobs/{job_id}")
    def poll_job(job_id: str) -> dict:
        job = get_job(job_id)
        if job is None:
            raise HTTPException(status_code=404, detail="job not found")
        return job.to_dict()

    @router.api_route("/exports/{unit_id}/mesh", methods=["GET", "HEAD"])
    def get_export_mesh(unit_id: str) -> FileResponse:
        path = repo_root / "content/characters/library" / "exports" / f"{unit_id}.glb"
        if not path.is_file():
            raise HTTPException(status_code=404, detail="export glb not found")
        return FileResponse(path, media_type="model/gltf-binary", filename=f"{unit_id}.glb")

    @router.api_route("/bases/{base_id}/preview", methods=["GET", "HEAD"])
    def get_base_preview(base_id: str) -> FileResponse:
        if base_id not in ("male", "female", "small", "large"):
            raise HTTPException(status_code=404, detail="unknown base")
        path = repo_root / "content/characters/library/bases" / base_id / "preview.glb"
        if not path.is_file():
            raise HTTPException(
                status_code=404,
                detail="preview glb not found — re-run POST /api/assembly/mblab/setup",
            )
        return FileResponse(path, media_type="model/gltf-binary", filename=f"{base_id}_preview.glb")

    @router.api_route("/bases/{base_id}/mesh", methods=["GET", "HEAD"])
    def get_base_mesh(base_id: str) -> FileResponse:
        if base_id not in ("male", "female", "small", "large"):
            raise HTTPException(status_code=404, detail="unknown base")
        path = repo_root / "content/characters/library/bases" / base_id / "body.glb"
        if not path.is_file():
            raise HTTPException(status_code=404, detail="base glb not found — POST /api/assembly/mblab/setup")
        return FileResponse(path, media_type="model/gltf-binary", filename=f"{base_id}_body.glb")

    @router.get("/mblab/status")
    def get_mblab_status() -> dict[str, Any]:
        return mblab_status(repo_root)

    @router.post("/mblab/setup", status_code=202)
    def setup_mblab_pipeline() -> dict:
        """Vendor + install + bootstrap templates + export library bases (headless Blender)."""
        job = create_mblab_setup_job(repo_root=repo_root)
        return job.to_dict()

    @router.post("/mblab/generate", status_code=202)
    def generate_mblab(body: MblabGenerateRequest) -> dict:
        data = body.model_dump(exclude_none=True)
        morphs = data.pop("morphs", None)
        if morphs:
            data["morphs"] = morphs
        template = repo_root / "content/characters/mblab/projects" / f"{data['baseId']}.blend"
        if not template.is_file():
            from assembly_studio.mblab_setup import bootstrap_template  # noqa: PLC0415

            try:
                bootstrap_template(str(data["baseId"]), repo_root)
            except Exception as exc:
                raise HTTPException(status_code=500, detail=str(exc)) from exc
        job = create_mblab_job(data, repo_root=repo_root)
        return job.to_dict()

    @router.get("/mblab/jobs/{job_id}")
    def poll_mblab_job(job_id: str) -> dict:
        job = get_mblab_job(job_id)
        if job is None:
            raise HTTPException(status_code=404, detail="job not found")
        return job.to_dict()

    register_mixamo_edit_routes(router, repo_root)

    return router
