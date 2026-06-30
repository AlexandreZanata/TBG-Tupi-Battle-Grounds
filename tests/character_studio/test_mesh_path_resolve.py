"""Tests for showcase vs game mesh path resolution."""

from __future__ import annotations

from pathlib import Path

import pytest
from fastapi.testclient import TestClient

from character_studio.content_assets import resolve_showcase_mesh_path
from character_studio.models import CharacterDefinition
from character_studio.web_app import create_app

REPO_ROOT = Path(__file__).resolve().parents[2]


@pytest.fixture
def client() -> TestClient:
    return TestClient(create_app(REPO_ROOT))


def test_to_mesh_info_url_preserves_tier_query() -> None:
    # ponytail: inline check mirrors gltfLoader.toMeshInfoUrl contract
    mesh_url = "/api/characters/demo_scout_01/mesh?tier=showcase"
    path, _, query = mesh_url.partition("?")
    info = path.replace("/mesh", "/mesh-info") + ("?" + query if query else "")
    assert info == "/api/characters/demo_scout_01/mesh-info?tier=showcase"


def test_resolve_showcase_prefers_existing_showcase_subfolder() -> None:
    defn = CharacterDefinition.from_dict(
        {
            "id": "demo_scout_01",
            "characterName": "Scout",
            "archetype": "Ranged",
            "faction": "Humans",
            "heightScale": 1.0,
            "bodyWidthScale": 1.0,
            "headScale": 1.0,
            "limbLengthScale": 1.0,
            "primaryColor": "#FFFFFF",
            "secondaryColor": "#FFFFFF",
            "assetMode": "mesh3d",
            "meshShowcase": "content/characters/meshes/showcase/demo_scout_01.glb",
            "mesh": "content/characters/meshes/game/demo_scout_01.glb",
        }
    )
    resolved = resolve_showcase_mesh_path(defn, REPO_ROOT)
    assert resolved is not None
    assert "showcase/demo_scout_01.glb" in resolved.replace("\\", "/")


def test_mesh_showcase_tier_api_serves_all_demo_roster(client: TestClient) -> None:
    for unit_id in (
        "demo_scout_01",
        "demo_striker_01",
        "demo_guard_01",
        "demo_vanguard_01",
        "demo_reaver_01",
        "demo_hunter_01",
        "demo_bruiser_01",
        "demo_sentinel_01",
    ):
        info = client.get(f"/api/characters/{unit_id}/mesh-info?tier=showcase")
        assert info.status_code == 200, unit_id
        mesh = client.get(f"/api/characters/{unit_id}/mesh?tier=showcase")
        assert mesh.status_code == 200, unit_id
        assert len(mesh.content) > 1_000_000


def test_resolve_showcase_falls_back_to_legacy_flat_path(tmp_path: Path) -> None:
    meshes = tmp_path / "content" / "characters" / "meshes"
    meshes.mkdir(parents=True)
    legacy = meshes / "demo_legacy_01.glb"
    legacy.write_bytes(b"x" * 2000)

    defn = CharacterDefinition.from_dict(
        {
            "id": "demo_legacy_01",
            "characterName": "Legacy",
            "archetype": "Tank",
            "faction": "Humans",
            "heightScale": 1.0,
            "bodyWidthScale": 1.0,
            "headScale": 1.0,
            "limbLengthScale": 1.0,
            "primaryColor": "#FFFFFF",
            "secondaryColor": "#FFFFFF",
            "assetMode": "mesh3d",
            "meshShowcase": "content/characters/meshes/showcase/demo_legacy_01.glb",
            "mesh": "content/characters/meshes/game/demo_legacy_01.glb",
        }
    )
    resolved = resolve_showcase_mesh_path(defn, tmp_path)
    assert resolved == "content/characters/meshes/demo_legacy_01.glb"
