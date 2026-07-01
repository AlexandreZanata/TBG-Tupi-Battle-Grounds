"""Contract for MB-Lab adaptive hair part."""

from __future__ import annotations

import json
import struct
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HAIR_DIR = REPO / "content/characters/library/parts/hair"
HAIR_ID = "hair_mblab_01"


def _load_glb(path: Path) -> dict:
    data = path.read_bytes()
    offset = 12
    while offset + 8 <= len(data):
        ln, typ = struct.unpack_from("<II", data, offset)
        offset += 8
        chunk = data[offset : offset + ln]
        offset += ln
        if typ == 0x4E4F534A:
            return json.loads(chunk)
    raise ValueError("no JSON chunk")


def test_hair_mblab_manifest() -> None:
    manifest = HAIR_DIR / f"{HAIR_ID}.json"
    assert manifest.is_file()
    data = json.loads(manifest.read_text(encoding="utf-8"))
    assert data["id"] == HAIR_ID
    assert data["socket"] == "TBG_Head"
    assert data["source"] == "mblab_shrinkwrap_bake"
    assert len(data.get("morphTargets", [])) >= 12


def test_hair_mblab_glb_has_morph_targets() -> None:
    glb = HAIR_DIR / f"{HAIR_ID}.glb"
    if not glb.is_file():
        return
    assert glb.stat().st_size > 50_000
    gltf = _load_glb(glb)
    prim = gltf["meshes"][0]["primitives"][0]
    assert len(prim.get("targets", [])) >= 12
