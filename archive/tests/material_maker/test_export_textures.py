"""Tests for texture export script and graph descriptors."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
TOOLS_DIR = ROOT / "tools" / "material-maker"
sys.path.insert(0, str(TOOLS_DIR))

from faction_graphs import FACTION_GRAPHS, PILOT_UNITS, load_graph  # noqa: E402
from texture_generator import export_unit_textures  # noqa: E402


def test_all_faction_graph_files_exist() -> None:
    for filename in FACTION_GRAPHS.values():
        path = TOOLS_DIR / filename
        assert path.is_file(), f"missing graph descriptor: {path}"


@pytest.mark.parametrize("faction", sorted(FACTION_GRAPHS))
def test_load_graph_matches_faction(faction: str) -> None:
    graph = load_graph(faction)
    assert graph.faction == faction
    assert "albedo" in graph.outputs
    assert "orm" in graph.outputs


def test_export_writes_expected_png_names(tmp_path: Path) -> None:
    graph = load_graph("Humans")
    paths = export_unit_textures("test_unit", graph, tmp_path, size=64)
    names = {path.name for path in paths}
    assert names == {
        "test_unit_albedo.png",
        "test_unit_normal.png",
        "test_unit_orm.png",
    }
    for path in paths:
        with Image.open(path) as image:
            assert image.size == (64, 64)


def test_undead_export_includes_emissive(tmp_path: Path) -> None:
    graph = load_graph("Undead")
    paths = export_unit_textures("dps_undead_01", graph, tmp_path, size=32)
    names = {path.name for path in paths}
    assert "dps_undead_01_emissive.png" in names


def test_pilot_roster_covers_all_factions() -> None:
    assert set(PILOT_UNITS.values()) == set(FACTION_GRAPHS)


def test_export_script_all_pilots(tmp_path: Path) -> None:
    script = TOOLS_DIR / "export_textures.py"
    import subprocess

    result = subprocess.run(
        [
            sys.executable,
            str(script),
            "--all-pilots",
            "--size",
            "32",
            "--output-dir",
            str(tmp_path),
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    for unit_id in PILOT_UNITS:
        assert (tmp_path / f"{unit_id}_albedo.png").is_file()
        assert (tmp_path / f"{unit_id}_orm.png").is_file()
