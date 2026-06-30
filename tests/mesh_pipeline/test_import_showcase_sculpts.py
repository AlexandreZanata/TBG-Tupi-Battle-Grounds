"""Tests for showcase sculpt import script."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "tools/mesh-pipeline/import_showcase_sculpts.py"
MANIFEST = REPO_ROOT / "content/characters/fixed_test_imports.json"


def _expected_ids() -> set[str]:
    data = json.loads(MANIFEST.read_text(encoding="utf-8"))
    return set(data["imports"].values())


def test_import_mapping_covers_fixed_test_roster() -> None:
    result = subprocess.run(
        [sys.executable, str(SCRIPT), "--dry-run"],
        capture_output=True,
        text=True,
        check=True,
    )
    mapping = json.loads(result.stdout)
    assert set(mapping) == _expected_ids()
    assert len(mapping) == len(_expected_ids())


def test_only_flag_limits_import_subset() -> None:
    result = subprocess.run(
        [
            sys.executable,
            str(SCRIPT),
            "--dry-run",
            "--only",
            "demo_scout_01",
            "demo_reaver_01",
        ],
        capture_output=True,
        text=True,
        check=True,
    )
    mapping = json.loads(result.stdout)
    assert set(mapping) == {"demo_scout_01", "demo_reaver_01"}


def test_imported_sculpts_exist_after_import() -> None:
    meshes = REPO_ROOT / "content/characters/meshes/showcase"
    for pilot_id in sorted(_expected_ids()):
        path = meshes / f"{pilot_id}.glb"
        assert path.is_file(), f"missing imported sculpt: {path}"
        assert path.stat().st_size > 1_000_000
