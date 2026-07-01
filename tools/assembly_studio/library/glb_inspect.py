"""Inspect GLB assets for modular library contract tests."""

from __future__ import annotations

import json
import struct
from dataclasses import dataclass
from pathlib import Path

GLB_MAGIC = 0x46546C67
JSON_CHUNK_TYPE = 0x4E4F534A
BIN_CHUNK_TYPE = 0x004E4942


@dataclass(frozen=True)
class GlbStats:
    triangles: int
    bones: int
    skins: int
    file_bytes: int


def load_glb(path: Path) -> tuple[dict, bytes | None]:
    data = path.read_bytes()
    if len(data) < 12:
        raise ValueError(f"{path}: file too small for GLB header")
    magic, _version, _length = struct.unpack_from("<III", data, 0)
    if magic != GLB_MAGIC:
        raise ValueError(f"{path}: not a GLB file")
    offset = 12
    gltf: dict | None = None
    bin_chunk: bytes | None = None
    while offset + 8 <= len(data):
        chunk_length, chunk_type = struct.unpack_from("<II", data, offset)
        offset += 8
        chunk_data = data[offset : offset + chunk_length]
        offset += chunk_length
        if chunk_type == JSON_CHUNK_TYPE:
            gltf = json.loads(chunk_data.decode("utf-8"))
        elif chunk_type == BIN_CHUNK_TYPE:
            bin_chunk = chunk_data
    if gltf is None:
        raise ValueError(f"{path}: missing JSON chunk")
    return gltf, bin_chunk


def inspect_glb(path: Path) -> GlbStats:
    gltf, _bin = load_glb(path)
    return GlbStats(
        triangles=count_triangles(gltf),
        bones=count_bones(gltf),
        skins=len(gltf.get("skins", [])),
        file_bytes=path.stat().st_size,
    )


def list_node_names(gltf: dict) -> list[str]:
    return [str(node.get("name", "")) for node in gltf.get("nodes", [])]


def list_tbg_bones(gltf: dict) -> list[str]:
    return sorted(name for name in list_node_names(gltf) if name.startswith("TBG_"))


def has_tbg_root(gltf: dict) -> bool:
    return "TBG_Root" in list_node_names(gltf)


def count_triangles(gltf: dict) -> int:
    accessors = gltf.get("accessors", [])
    total = 0
    for mesh in gltf.get("meshes", []):
        for primitive in mesh.get("primitives", []):
            total += _primitive_triangles(primitive, accessors)
    return total


def count_bones(gltf: dict) -> int:
    skins = gltf.get("skins", [])
    if not skins:
        return 0
    return max(len(skin.get("joints", [])) for skin in skins)


def _primitive_triangles(primitive: dict, accessors: list[dict]) -> int:
    indices = primitive.get("indices")
    if indices is not None:
        return accessors[indices]["count"] // 3
    position = primitive.get("attributes", {}).get("POSITION")
    if position is None:
        return 0
    return accessors[position]["count"] // 3
