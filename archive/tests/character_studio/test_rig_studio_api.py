"""Rig Studio API contract tests."""

from __future__ import annotations

from fastapi.testclient import TestClient

from character_studio.rig_jobs import launch_command, session_meta
from character_studio.web_app import create_app


def test_rig_status_endpoint() -> None:
    client = TestClient(create_app())
    response = client.get("/api/rig/status")
    assert response.status_code == 200
    payload = response.json()
    assert "blenderAvailable" in payload
    assert "sessionsRoot" in payload


def test_rig_unit_config_for_pilot() -> None:
    client = TestClient(create_app())
    response = client.get("/api/rig/units/demo_scout_01/config")
    assert response.status_code == 200
    payload = response.json()
    assert payload["unitId"] == "demo_scout_01"
    assert payload["bodyClass"] in {"HumanoidLight", "HumanoidHeavy"}


def test_session_meta_missing() -> None:
    meta = session_meta("nonexistent_unit_xyz")
    assert meta["blend"]["exists"] is False
    assert meta["preview"]["exists"] is False


def test_launch_command_includes_blend_path() -> None:
    cmd = launch_command("demo_scout_01")
    assert "session.blend" in cmd["blendPath"]
    assert "blender" in cmd["native"].lower() or "Blender" in cmd["flatpak"]
