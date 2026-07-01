"""Contract tests for modular character assembly — Phase 05 §3."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

import pytest

from assembly_studio.library.assemble import assemble_from_recipe
from assembly_studio.library.assemble_gltf import assemble_gltf
from assembly_studio.library.glb_inspect import has_tbg_root, inspect_glb, load_glb
from assembly_studio.library.glb_write import roots_have_display_flip
from assembly_studio.library.recipe import resolve_recipe, validate_recipe

REPO = Path(__file__).resolve().parents[2]
LIBRARY = REPO / "content" / "characters" / "library"
FIXTURE_RECIPE = LIBRARY / "exports" / "_test_male_head_sword.recipe.json"
EXPORT_GLB = LIBRARY / "exports" / "test_male_head_sword.glb"
BLENDER_CMD = REPO / "tools" / "assembly_studio" / "blender" / "blender_cmd.sh"
ASSEMBLE_SCRIPT = REPO / "tools" / "assembly_studio" / "blender" / "assemble_character.py"


@pytest.fixture(scope="module", autouse=True)
def _ensure_library_seeded() -> None:
    male_body = LIBRARY / "bases" / "male" / "body.glb"
    if not male_body.is_file():
        from assembly_studio.library.seed_fixtures import main as seed

        seed()


@pytest.fixture(scope="module")
def fixture_recipe() -> Path:
    FIXTURE_RECIPE.parent.mkdir(parents=True, exist_ok=True)
    FIXTURE_RECIPE.write_text(
        json.dumps(
            {
                "unitId": "test_male_head_sword",
                "baseId": "male",
                "parts": ["head_round_01", "weapon_sword_01"],
                "materialId": "default",
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    return FIXTURE_RECIPE


def test_recipe_schema_validates(fixture_recipe: Path) -> None:
    data = json.loads(fixture_recipe.read_text(encoding="utf-8"))
    assert validate_recipe(data) == []
    recipe = resolve_recipe(fixture_recipe, REPO)
    assert recipe.base_id == "male"
    assert recipe.part_ids == ("head_round_01", "weapon_sword_01")


def test_assemble_male_head_sword_gltf_fallback(fixture_recipe: Path) -> None:
    base_stats = inspect_glb(LIBRARY / "bases" / "male" / "body.glb")
    recipe = resolve_recipe(fixture_recipe, REPO)
    output = assemble_gltf(recipe)
    assert output == EXPORT_GLB
    assert output.is_file()
    out_stats = inspect_glb(output)
    assert out_stats.bones == base_stats.bones
    assert out_stats.triangles > base_stats.triangles
    gltf, _out_bin = load_glb(output)
    assert has_tbg_root(gltf)
    assert roots_have_display_flip(gltf)


def test_assemble_from_recipe_entrypoint(fixture_recipe: Path, tmp_path: Path) -> None:
    if EXPORT_GLB.is_file():
        EXPORT_GLB.unlink()
    output = assemble_from_recipe(fixture_recipe, repo_root=REPO)
    assert output.is_file()
    assert inspect_glb(output).bones == inspect_glb(LIBRARY / "bases" / "male" / "body.glb").bones


@pytest.mark.skipif(not BLENDER_CMD.is_file(), reason="blender_cmd.sh missing")
def test_blender_cli_assembly(fixture_recipe: Path) -> None:
    export_path = LIBRARY / "exports" / "test_blender_cli.glb"
    if export_path.is_file():
        export_path.unlink()
    cli_recipe = fixture_recipe.parent / "test_blender_cli.recipe.json"
    cli_recipe.write_text(
        json.dumps(
            {
                "unitId": "test_blender_cli",
                "baseId": "male",
                "parts": ["head_round_01", "weapon_sword_01"],
                "materialId": "default",
            }
        ),
        encoding="utf-8",
    )
    result = subprocess.run(
        [
            str(BLENDER_CMD),
            "--background",
            "--python",
            str(ASSEMBLE_SCRIPT),
            "--",
            "--recipe",
            str(cli_recipe),
        ],
        cwd=REPO,
        capture_output=True,
        text=True,
        timeout=180,
    )
    if result.returncode != 0 and "Blender not found" in (result.stderr or ""):
        pytest.skip("Blender not available")
    assert result.returncode == 0, result.stderr
    assert export_path.is_file()
    base_bones = inspect_glb(LIBRARY / "bases" / "male" / "body.glb").bones
    assert inspect_glb(export_path).bones == base_bones
