"""API tests for Character Studio web app."""

from __future__ import annotations

import pytest
from fastapi.testclient import TestClient

from character_studio.web_app import create_app


@pytest.fixture
def client(tmp_path):
    (tmp_path / "content" / "characters" / "definitions").mkdir(parents=True)
    (tmp_path / "content" / "characters" / "cards").mkdir(parents=True)
    (tmp_path / "content" / "characters" / "previews").mkdir(parents=True)
    app = create_app(tmp_path)
    return TestClient(app)


def test_health(client: TestClient) -> None:
    response = client.get("/api/health")
    assert response.status_code == 200
    assert response.json()["status"] == "ok"


def test_create_list_and_get(client: TestClient) -> None:
    create = client.post(
        "/api/characters",
        json={"id": "tank_human_99", "archetype": "Tank", "faction": "Humans", "characterName": "Test"},
    )
    assert create.status_code == 201
    listing = client.get("/api/characters")
    assert listing.json()[0]["id"] == "tank_human_99"
    assert listing.json()[0]["assetMode"] == "procedural"
    detail = client.get("/api/characters/tank_human_99")
    assert detail.json()["characterName"] == "Test"
    assert detail.json()["assetMode"] == "procedural"


def test_card_png(client: TestClient) -> None:
    client.post(
        "/api/characters",
        json={"id": "dps_undead_99", "archetype": "DPS", "faction": "Undead"},
    )
    card = client.get("/api/characters/dps_undead_99/card")
    assert card.status_code == 200
    assert card.headers["content-type"] == "image/png"
    assert len(card.content) > 500


def test_index_html(client: TestClient) -> None:
    response = client.get("/")
    assert response.status_code == 200
    assert "Character Studio" in response.text
    assert "mesh3d" in response.text.lower()
    assert 'href="/css/studio.css"' in response.text
    assert 'src="/js/studio/app.js"' in response.text


def test_list_includes_mesh_for_mesh3d_units(client: TestClient) -> None:
    client.put(
        "/api/characters/mesh3d_unit",
        json={
            "id": "mesh3d_unit",
            "characterName": "Mesh Unit",
            "archetype": "Tank",
            "faction": "Humans",
            "heightScale": 1.0,
            "bodyWidthScale": 1.0,
            "headScale": 1.0,
            "limbLengthScale": 1.0,
            "primaryColor": "#FFFFFF",
            "secondaryColor": "#CCCCCC",
            "accessories": [],
            "evolutionStage": 0,
            "assetMode": "mesh3d",
            "mesh": "content/characters/meshes/tank_human_01.glb",
        },
    )
    listing = client.get("/api/characters").json()
    entry = next(item for item in listing if item["id"] == "mesh3d_unit")
    assert entry["assetMode"] == "mesh3d"
    assert entry["mesh"] == "content/characters/meshes/tank_human_01.glb"


def test_static_assets(client: TestClient) -> None:
    assert client.get("/css/studio.css").status_code == 200
    assert client.get("/js/studio/app.js").status_code == 200
    assert client.get("/css/showcase.css").status_code == 200
    assert client.get("/js/showcase/showcase.js").status_code == 200
