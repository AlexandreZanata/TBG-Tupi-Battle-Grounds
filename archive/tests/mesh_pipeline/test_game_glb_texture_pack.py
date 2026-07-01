"""Tests for ORM texture packing in game GLBs."""

from __future__ import annotations

from io import BytesIO
from pathlib import Path

from PIL import Image

from mesh_pipeline.game_glb_texture_pack import pack_orm_textures
from mesh_pipeline.glb_inspect import load_glb


def _png_bytes(size: tuple[int, int], rgb: tuple[int, int, int]) -> bytes:
    buf = BytesIO()
    Image.new("RGB", size, rgb).save(buf, format="PNG")
    return buf.getvalue()


def _minimal_glb(tmp_path: Path, mr_png: bytes, occ_png: bytes | None = None) -> Path:
    import json
    import struct

    from mesh_pipeline.glb_inspect import GLB_MAGIC, JSON_CHUNK_TYPE, BIN_CHUNK_TYPE

    images_blob = mr_png + (occ_png or b"")
    occ_offset = len(mr_png)
    gltf = {
        "asset": {"version": "2.0"},
        "buffers": [{"byteLength": len(images_blob)}],
        "bufferViews": [
            {"buffer": 0, "byteOffset": 0, "byteLength": len(mr_png)},
        ],
        "images": [
            {"mimeType": "image/png", "bufferView": 0, "width": 4, "height": 4},
        ],
        "textures": [{"source": 0, "sampler": 0}],
        "materials": [
            {
                "pbrMetallicRoughness": {"metallicRoughnessTexture": {"index": 0}},
            }
        ],
    }
    if occ_png:
        gltf["bufferViews"].append(
            {"buffer": 0, "byteOffset": occ_offset, "byteLength": len(occ_png)}
        )
        gltf["images"].append(
            {"mimeType": "image/png", "bufferView": 1, "width": 4, "height": 4}
        )
        gltf["textures"].append({"source": 1, "sampler": 0})
        gltf["materials"][0]["occlusionTexture"] = {"index": 1}

    json_bytes = json.dumps(gltf).encode("utf-8")
    json_padded = json_bytes + b" " * ((4 - len(json_bytes) % 4) % 4)
    bin_padded = images_blob + b"\x00" * ((4 - len(images_blob) % 4) % 4)
    total = 12 + 8 + len(json_padded) + 8 + len(bin_padded)
    header = struct.pack("<III", GLB_MAGIC, 2, total)
    path = tmp_path / "input.glb"
    path.write_bytes(
        header
        + struct.pack("<II", len(json_padded), JSON_CHUNK_TYPE)
        + json_padded
        + struct.pack("<II", len(bin_padded), BIN_CHUNK_TYPE)
        + bin_padded
    )
    return path


def test_pack_orm_merges_mr_and_occlusion(tmp_path: Path) -> None:
    mr = _png_bytes((4, 4), (0, 160, 200))
    occ = _png_bytes((4, 4), (64, 64, 64))
    path = _minimal_glb(tmp_path, mr, occ)
    before, _ = load_glb(path)
    assert len(before["textures"]) == 2

    pack_orm_textures(path)
    after, _ = load_glb(path)
    material = after["materials"][0]
    mr_index = material["pbrMetallicRoughness"]["metallicRoughnessTexture"]["index"]
    occ_index = material["occlusionTexture"]["index"]
    assert mr_index == occ_index
    assert len(after["textures"]) == 1
