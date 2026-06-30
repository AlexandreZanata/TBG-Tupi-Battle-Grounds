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
    detail = client.get("/api/characters/tank_human_99")
    assert detail.json()["characterName"] == "Test"


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
    assert "TBG Character Studio" in response.text
