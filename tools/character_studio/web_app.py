"""FastAPI web server for Character Studio browser UI."""

from __future__ import annotations

from pathlib import Path

from fastapi import FastAPI, HTTPException
from fastapi.responses import FileResponse, Response
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel, Field

from .card_renderer import render_card
from .content_assets import (
    build_mesh_info,
    mesh_media_type,
    resolve_character_asset,
    resolve_mesh_path,
    texture_media_type,
)
from .defaults import create_default_definition
from .models import DEFAULT_ASSET_MODE, CharacterDefinition
from .paths import find_repo_root
from .repository import CharacterRepository
from .validation import ValidationError

STATIC_DIR = Path(__file__).parent / "static"
STUDIO_BUILD_ID = "20260701a"


class CreateCharacterRequest(BaseModel):
    id: str
    archetype: str
    faction: str
    characterName: str | None = None


class CharacterTexturesPayload(BaseModel):
    albedo: str
    normal: str
    orm: str
    emissive: str | None = None


class CharacterPayload(BaseModel):
    id: str
    characterName: str
    archetype: str
    faction: str
    heightScale: float = Field(ge=0.1, le=3.0)
    bodyWidthScale: float = Field(ge=0.1, le=3.0)
    headScale: float = Field(ge=0.1, le=3.0)
    limbLengthScale: float = Field(ge=0.1, le=3.0)
    primaryColor: str
    secondaryColor: str
    accessories: list[str] = Field(default_factory=list)
    evolutionStage: int = Field(ge=0, le=2)
    assetMode: str = DEFAULT_ASSET_MODE
    mesh: str | None = None
    textures: CharacterTexturesPayload | None = None
    materialProfile: str | None = None
    behaviorProfile: str | None = None
    animatorController: str | None = None


def create_app(repo_root: Path | None = None) -> FastAPI:
    root = repo_root or find_repo_root()
    repo = CharacterRepository(root)
    app = FastAPI(title="TBG Character Studio", version="0.1.0")

    @app.get("/api/health")
    def health() -> dict[str, str]:
        return {"status": "ok", "studioBuild": STUDIO_BUILD_ID}

    @app.middleware("http")
    async def disable_static_cache(request, call_next):
        response = await call_next(request)
        path = request.url.path
        if path.endswith((".html", ".js", ".css", ".mjs")):
            response.headers["Cache-Control"] = "no-cache, must-revalidate"
        return response

    @app.get("/favicon.ico", include_in_schema=False, response_model=None)
    def favicon() -> FileResponse | Response:
        icon = STATIC_DIR / "favicon.ico"
        if icon.is_file():
            return FileResponse(icon, media_type="image/x-icon")
        return Response(status_code=204)

    @app.get("/api/characters")
    def list_characters() -> list[dict[str, str]]:
        items: list[dict[str, str]] = []
        for unit_id in repo.list_ids():
            defn = repo.load(unit_id)
            items.append(
                {
                    "id": unit_id,
                    "assetMode": defn.assetMode,
                    "cardUrl": f"/api/characters/{unit_id}/card",
                    **({"mesh": defn.mesh} if defn.mesh else {}),
                }
            )
        return items

    @app.get("/api/characters/{unit_id}")
    def get_character(unit_id: str) -> dict:
        try:
            return repo.load(unit_id).to_dict()
        except FileNotFoundError as exc:
            raise HTTPException(status_code=404, detail=str(exc)) from exc

    @app.post("/api/characters", status_code=201)
    def create_character(body: CreateCharacterRequest) -> dict:
        defn = create_default_definition(
            body.id, body.archetype, body.faction, body.characterName
        )
        try:
            repo.create(defn)
            render_card(defn, repo.card_path(defn.id))
        except ValidationError as exc:
            raise HTTPException(status_code=400, detail=exc.errors) from exc
        return defn.to_dict()

    @app.put("/api/characters/{unit_id}")
    def update_character(unit_id: str, body: CharacterPayload) -> dict:
        if unit_id != body.id:
            raise HTTPException(status_code=400, detail="path id and body id must match")
        defn = CharacterDefinition.from_dict(body.model_dump())
        try:
            repo.save(defn)
            render_card(defn, repo.card_path(defn.id))
        except ValidationError as exc:
            raise HTTPException(status_code=400, detail=exc.errors) from exc
        return defn.to_dict()

    @app.delete("/api/characters/{unit_id}", status_code=204)
    def delete_character(unit_id: str) -> None:
        try:
            repo.delete(unit_id)
        except FileNotFoundError as exc:
            raise HTTPException(status_code=404, detail=str(exc)) from exc

    @app.get("/api/characters/{unit_id}/card")
    def get_card(unit_id: str) -> FileResponse:
        path = repo.card_path(unit_id)
        if not path.is_file():
            defn = repo.load(unit_id)
            render_card(defn, path)
        return FileResponse(path, media_type="image/png")

    @app.post("/api/characters/{unit_id}/export-card")
    def export_card(unit_id: str) -> dict[str, str]:
        defn = repo.load(unit_id)
        path = render_card(defn, repo.card_path(unit_id))
        return {"path": str(path)}

    @app.get("/api/characters/{unit_id}/mesh-info")
    def get_character_mesh_info(unit_id: str, tier: str = "showcase") -> dict:
        try:
            defn = repo.load(unit_id)
        except FileNotFoundError as exc:
            raise HTTPException(status_code=404, detail=str(exc)) from exc
        if defn.assetMode != "mesh3d":
            raise HTTPException(status_code=404, detail="unit has no mesh3d asset")
        return build_mesh_info(defn, root, tier)

    @app.get("/api/characters/{unit_id}/mesh")
    @app.head("/api/characters/{unit_id}/mesh")
    def get_character_mesh(unit_id: str, tier: str = "showcase") -> FileResponse:
        try:
            defn = repo.load(unit_id)
        except FileNotFoundError as exc:
            raise HTTPException(status_code=404, detail=str(exc)) from exc
        if defn.assetMode != "mesh3d":
            raise HTTPException(status_code=404, detail="unit has no mesh3d asset")
        relative = resolve_mesh_path(defn, tier, root)
        if not relative:
            raise HTTPException(status_code=404, detail=f"unit has no {tier} mesh")
        path = resolve_character_asset(root, relative)
        return FileResponse(path, media_type=mesh_media_type(path))

    @app.get("/api/content/{asset_path:path}")
    def get_content_asset(asset_path: str) -> FileResponse:
        path = resolve_character_asset(root, asset_path)
        media_type = mesh_media_type(path)
        if path.suffix.lower() in {".png", ".jpg", ".jpeg"}:
            media_type = texture_media_type(path)
        return FileResponse(path, media_type=media_type)

    app.mount("/", StaticFiles(directory=STATIC_DIR, html=True), name="static")
    return app


app = create_app()
