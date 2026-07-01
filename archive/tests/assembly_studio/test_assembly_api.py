"""API tests for Assembly Studio — Phase 05 §4."""

from __future__ import annotations

import time

import pytest
from fastapi.testclient import TestClient

from character_studio.paths import find_repo_root
from character_studio.web_app import create_app

REPO = find_repo_root()


@pytest.fixture(scope="module")
def client() -> TestClient:
    return TestClient(create_app(REPO))


@pytest.fixture(scope="module", autouse=True)
def _ensure_library() -> None:
    male = REPO / "content/characters/library/bases/male/body.glb"
    if not male.is_file():
        from assembly_studio.library.seed_fixtures import main as seed

        seed()


def test_assembly_library_lists_bases_and_parts(client: TestClient) -> None:
    res = client.get("/api/assembly/library")
    assert res.status_code == 200
    data = res.json()
    assert len(data["bases"]) >= 4
    assert len(data["parts"]["heads"]) >= 3
    assert any(m["id"] == "default" for m in data["materials"])


def test_assembly_export_job_completes(client: TestClient) -> None:
    unit_id = "api_test_assembled_01"
    export_path = REPO / "content/characters/library/exports" / f"{unit_id}.glb"
    export_path.unlink(missing_ok=True)
    res = client.post(
        "/api/assembly/export",
        json={
            "unitId": unit_id,
            "baseId": "male",
            "parts": ["head_round_01", "weapon_sword_01"],
            "materialId": "default",
        },
    )
    assert res.status_code == 202
    job_id = res.json()["id"]
    final = None
    for _ in range(40):
        poll = client.get(f"/api/assembly/jobs/{job_id}")
        assert poll.status_code == 200
        final = poll.json()
        if final["status"] in ("completed", "failed"):
            break
        time.sleep(0.25)
    assert final is not None
    assert final["status"] == "completed", final.get("error")
    assert export_path.is_file()
    mesh = client.get(f"/api/assembly/exports/{unit_id}/mesh")
    assert mesh.status_code == 200
    assert mesh.headers["content-type"] == "model/gltf-binary"


def test_assembly_export_rejects_invalid_base(client: TestClient) -> None:
    res = client.post(
        "/api/assembly/export",
        json={"unitId": "bad_unit", "baseId": "invalid", "parts": [], "materialId": "default"},
    )
    assert res.status_code == 422
