"""Build minimal GLB files with a named animation stub (dev placeholder clips)."""

from __future__ import annotations

import json
import struct
from pathlib import Path

_CUBE_POSITIONS = (
    -0.5,
    -0.5,
    -0.5,
    0.5,
    -0.5,
    -0.5,
    0.5,
    0.5,
    -0.5,
    -0.5,
    0.5,
    -0.5,
    -0.5,
    -0.5,
    0.5,
    0.5,
    -0.5,
    0.5,
    0.5,
    0.5,
    0.5,
    -0.5,
    0.5,
    0.5,
)
_CUBE_INDICES = (
    0,
    1,
    2,
    2,
    3,
    0,
    1,
    5,
    6,
    6,
    2,
    1,
    7,
    6,
    5,
    5,
    4,
    7,
    4,
    0,
    3,
    3,
    7,
    4,
    4,
    5,
    1,
    1,
    0,
    4,
    3,
    2,
    6,
    6,
    7,
    3,
)


def write_placeholder_clip_glb(path: Path, clip_name: str) -> None:
    """Write a tiny GLB with an empty animation named ``clip_name``."""
    position_bytes = struct.pack(f"<{len(_CUBE_POSITIONS)}f", *_CUBE_POSITIONS)
    index_bytes = struct.pack(f"<{len(_CUBE_INDICES)}H", *_CUBE_INDICES)
    bin_chunk = position_bytes + index_bytes

    gltf = {
        "asset": {"version": "2.0", "generator": "TBG placeholder clip"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"name": "Placeholder", "mesh": 0}],
        "meshes": [
            {
                "name": "Mesh",
                "primitives": [{"attributes": {"POSITION": 0}, "indices": 1}],
            }
        ],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5126,
                "count": 8,
                "type": "VEC3",
                "min": [-0.5, -0.5, -0.5],
                "max": [0.5, 0.5, 0.5],
            },
            {"bufferView": 1, "componentType": 5123, "count": 36, "type": "SCALAR"},
        ],
        "bufferViews": [
            {"buffer": 0, "byteOffset": 0, "byteLength": len(position_bytes)},
            {
                "buffer": 0,
                "byteOffset": len(position_bytes),
                "byteLength": len(index_bytes),
            },
        ],
        "buffers": [{"byteLength": len(bin_chunk)}],
        "animations": [{"name": clip_name, "channels": [], "samplers": []}],
    }

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(_pack_glb(gltf, bin_chunk))


def _pack_glb(gltf: dict, bin_chunk: bytes) -> bytes:
    json_chunk = json.dumps(gltf, separators=(",", ":")).encode("utf-8")
    json_chunk += b" " * ((4 - len(json_chunk) % 4) % 4)
    bin_chunk += b"\x00" * ((4 - len(bin_chunk) % 4) % 4)

    total_length = 12 + 8 + len(json_chunk) + 8 + len(bin_chunk)
    header = struct.pack("<III", 0x46546C67, 2, total_length)
    json_header = struct.pack("<II", len(json_chunk), 0x4E4F534A)
    bin_header = struct.pack("<II", len(bin_chunk), 0x004E4942)
    return header + json_header + json_chunk + bin_header + bin_chunk
