"""Assembly Studio REST routes."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from fastapi import APIRouter, HTTPException
from fastapi.responses import FileResponse
from pydantic import BaseModel, Field

from assembly_studio.assembly_jobs import assembly_status, create_job, get_job
from assembly_studio.library.recipe import build_recipe, validate_recipe
from assembly_studio.library_catalog import load_library_catalog


class ExportRecipeRequest(BaseModel):
    unitId: str = Field(pattern=r"^[a-z0-9_]+$")
    baseId: str = Field(pattern=r"^(male|female|small|large)$")
    parts: list[str] = Field(default_factory=list)
    materialId: str = Field(pattern=r"^[a-z0-9_]+$")


def create_assembly_router(repo_root: Path) -> APIRouter:
    router = APIRouter(prefix="/api/assembly", tags=["assembly"])

    @router.get("/library")
    def get_library() -> dict[str, Any]:
        return load_library_catalog(repo_root)

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

    @router.get("/exports/{unit_id}/mesh")
    def get_export_mesh(unit_id: str) -> FileResponse:
        path = repo_root / "content" / "characters" / "library" / "exports" / f"{unit_id}.glb"
        if not path.is_file():
            raise HTTPException(status_code=404, detail="export glb not found")
        return FileResponse(path, media_type="model/gltf-binary", filename=f"{unit_id}.glb")

    return router
