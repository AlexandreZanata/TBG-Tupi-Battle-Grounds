"""API tests for MB-Lab Studio."""

from __future__ import annotations

import pytest
from fastapi.testclient import TestClient

from character_studio.paths import find_repo_root
from character_studio.web_app import create_app

REPO = find_repo_root()


@pytest.fixture(scope="module")
def client() -> TestClient:
    return TestClient(create_app(REPO))


def test_mblab_status_lists_templates(client: TestClient) -> None:
    res = client.get("/api/assembly/mblab/status")
    assert res.status_code == 200
    data = res.json()
    assert data["mblabVersion"] == "1.8.1"
    assert set(data["templates"]) == {"male", "female", "small", "large"}
    assert set(data["libraryBases"]) == {"male", "female", "small", "large"}
    assert "previewReady" in data
    assert isinstance(data["ready"], bool)


def test_base_preview_route(client: TestClient) -> None:
    res = client.head("/api/assembly/bases/male/preview")
    if res.status_code == 404:
        pytest.skip("preview.glb not baked yet")
    assert res.status_code == 200


def test_mblab_setup_accepts(client: TestClient) -> None:
    res = client.post("/api/assembly/mblab/setup")
    assert res.status_code == 202
    assert res.json()["unit_id"] == "_mblab_setup"


def test_mblab_generate_rejects_missing_template(client: TestClient) -> None:
    template = REPO / "content/characters/mblab/projects/male.blend"
    if template.is_file():
        pytest.skip("male.blend exists — missing-template path not testable")
    res = client.post(
        "/api/assembly/mblab/generate",
        json={"unitId": "mblab_api_test", "baseId": "male", "morphs": {"age": 0.3}},
    )
    assert res.status_code == 400
    assert "template missing" in res.json()["detail"].lower()


def test_mblab_generate_validates_unit_id(client: TestClient) -> None:
    res = client.post(
        "/api/assembly/mblab/generate",
        json={"unitId": "Bad-Id", "baseId": "male"},
    )
    assert res.status_code == 422


def test_head_morph_config_served(client: TestClient) -> None:
    res = client.get("/api/content/characters/library/mblab_studio_morphs.json")
    assert res.status_code == 200
    data = res.json()
    assert data["neutral"] == 0.5
    assert any(s["id"] == "head_width" for s in data["sliders"])


def test_mblab_generate_accepts_head_morphs(client: TestClient) -> None:
    template = REPO / "content/characters/mblab/projects/male.blend"
    if not template.is_file():
        pytest.skip("male.blend missing")
    res = client.post(
        "/api/assembly/mblab/generate",
        json={
            "unitId": "mblab_head_test",
            "baseId": "male",
            "morphs": {"head_width": 0.7, "jaw_prominence": 0.35},
        },
    )
    assert res.status_code == 202
    assert res.json()["unit_id"] == "mblab_head_test"
