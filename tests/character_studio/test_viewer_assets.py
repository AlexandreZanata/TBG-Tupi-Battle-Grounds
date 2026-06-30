"""Tests for browser 3D preview asset API."""

from __future__ import annotations

from pathlib import Path

import pytest
from fastapi.testclient import TestClient

from character_studio.content_assets import MAX_ASSET_BYTES, resolve_character_asset
from character_studio.web_app import create_app

REPO_ROOT = Path(__file__).resolve().parents[2]


@pytest.fixture
def client() -> TestClient:
    return TestClient(create_app(REPO_ROOT))


def test_resolve_character_asset_rejects_traversal(tmp_path: Path) -> None:
    (tmp_path / "content" / "characters" / "meshes").mkdir(parents=True)
    fixture = tmp_path / "content" / "characters" / "meshes" / "safe.glb"
    fixture.write_bytes(b"glb")

    resolved = resolve_character_asset(tmp_path, "content/characters/meshes/safe.glb")
    assert resolved == fixture.resolve()

    with pytest.raises(Exception):
        resolve_character_asset(tmp_path, "content/characters/../../etc/passwd")


def test_mesh_head_and_get_serve_demo_glb(client: TestClient) -> None:
    head = client.head("/api/characters/demo_striker_01/mesh")
    assert head.status_code == 200

    response = client.get("/api/characters/demo_striker_01/mesh")
    assert response.status_code == 200
    assert response.headers["content-type"] == "model/gltf-binary"
    assert len(response.content) > 1_000_000


def test_mesh_info_reports_size(client: TestClient) -> None:
    response = client.get("/api/characters/demo_guard_01/mesh-info")
    assert response.status_code == 200
    body = response.json()
    assert body["size"] > 1_000_000
    assert body["contentType"] == "model/gltf-binary"


def test_index_includes_threejs_importmap(client: TestClient) -> None:
    response = client.get("/")
    assert response.status_code == 200
    assert "importmap" in response.text
    assert "/js/studio/app.js" in response.text


def test_preview_asset_limit_is_150mb() -> None:
    assert MAX_ASSET_BYTES == 150 * 1024 * 1024


def test_demo_sculpt_units_serve_mesh(client: TestClient) -> None:
    for unit_id in (
        "demo_scout_01",
        "demo_striker_01",
        "demo_guard_01",
        "demo_vanguard_01",
        "demo_reaver_01",
    ):
        detail = client.get(f"/api/characters/{unit_id}")
        assert detail.status_code == 200
        assert detail.json()["assetMode"] == "mesh3d"

        mesh = client.get(f"/api/characters/{unit_id}/mesh")
        assert mesh.status_code == 200
        assert mesh.headers["content-type"] == "model/gltf-binary"
        assert len(mesh.content) > 1_000_000


def test_studio_lists_demo_sculpts_and_procedural_fixture(client: TestClient) -> None:
    ids = {entry["id"] for entry in client.get("/api/characters").json()}
    assert ids == {
        "demo_scout_01",
        "demo_striker_01",
        "demo_guard_01",
        "demo_vanguard_01",
        "demo_reaver_01",
        "test_procedural_spawn",
    }


def test_showcase_page_is_served(client: TestClient) -> None:
    response = client.get("/showcase.html")
    assert response.status_code == 200
    assert "showcase-track" in response.text
    assert "/js/showcase/showcase.js" in response.text
    assert "modalPrev" in response.text
