"""Contract tests for modular character library — Phase 05 §1–§2."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from assembly_studio.library.glb_inspect import has_tbg_root, inspect_glb, load_glb
from assembly_studio.library.glb_write import roots_have_display_flip
from assembly_studio.library.schema import load_json, validate_base_manifest, validate_part_manifest
from assembly_studio.library.sockets import BASE_IDS, SHOWCASE_TRI_CAP

REPO = Path(__file__).resolve().parents[2]
LIBRARY = REPO / "content" / "characters" / "library"


@pytest.fixture(scope="module", autouse=True)
def _ensure_library_seeded() -> None:
    male_body = LIBRARY / "bases" / "male" / "body.glb"
    if not male_body.is_file():
        from assembly_studio.library.seed_fixtures import main as seed

        seed()


def test_base_manifests_validate() -> None:
    for base_id in BASE_IDS:
        base_dir = LIBRARY / "bases" / base_id
        data = load_json(base_dir / "manifest.json")
        errors = validate_base_manifest(data, base_dir=base_dir)
        assert errors == [], f"{base_id}: {errors}"


def test_base_glb_has_tbg_root_and_tri_budget() -> None:
    for base_id in BASE_IDS:
        body = LIBRARY / "bases" / base_id / "body.glb"
        gltf, _bin = load_glb(body)
        assert has_tbg_root(gltf), f"{base_id} missing TBG_Root"
        assert roots_have_display_flip(gltf), f"{base_id} body not display-corrected"
        stats = inspect_glb(body)
        assert stats.triangles <= SHOWCASE_TRI_CAP, f"{base_id} tri budget exceeded"


def test_rig_glb_exists_per_base() -> None:
    for base_id in BASE_IDS:
        rig = LIBRARY / "rigs" / base_id / "armature.glb"
        assert rig.is_file()
        gltf, _bin = load_glb(rig)
        assert has_tbg_root(gltf)


def test_part_manifests_validate() -> None:
    for manifest_path in sorted(LIBRARY.glob("parts/*/*.json")):
        part_dir = manifest_path.parent
        data = load_json(manifest_path)
        errors = validate_part_manifest(data, part_dir=part_dir)
        assert errors == [], f"{manifest_path.name}: {errors}"


def test_mvp_part_counts() -> None:
    counts = {
        "heads": len(list((LIBRARY / "parts/heads").glob("*.json"))),
        "hair": len(list((LIBRARY / "parts/hair").glob("*.json"))),
        "armor": len(list((LIBRARY / "parts/armor").glob("*.json"))),
        "weapons": len(list((LIBRARY / "parts/weapons").glob("*.json"))),
        "hats": len(list((LIBRARY / "parts/hats").glob("*.json"))),
    }
    assert counts["heads"] >= 3
    assert counts["hair"] >= 2
    assert counts["armor"] >= 2
    assert counts["weapons"] >= 2
    assert counts["hats"] >= 1


def test_json_schemas_exist() -> None:
    assert (LIBRARY / "_schema.part.json").is_file()
    assert (LIBRARY / "_schema.base.json").is_file()
    sockets = json.loads((LIBRARY / "sockets.json").read_text(encoding="utf-8"))
    assert "head" in sockets["sockets"]
    assert sockets["sockets"]["head"]["bone"] == "TBG_Head"
