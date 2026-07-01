"""Write and transform minimal GLB fixtures without Blender."""

from __future__ import annotations

import json
import struct
from pathlib import Path

from assembly_studio.library.glb_inspect import BIN_CHUNK_TYPE, GLB_MAGIC, JSON_CHUNK_TYPE, load_glb


def write_glb(path: Path, gltf: dict, bin_chunk: bytes | None) -> None:
    json_bytes = json.dumps(gltf, separators=(",", ":")).encode("utf-8")
    json_pad = (4 - len(json_bytes) % 4) % 4
    json_bytes += b" " * json_pad
    chunks = [(JSON_CHUNK_TYPE, json_bytes)]
    if bin_chunk is not None:
        bin_pad = (4 - len(bin_chunk) % 4) % 4
        chunks.append((BIN_CHUNK_TYPE, bin_chunk + b"\x00" * bin_pad))
    total = 12 + sum(8 + len(payload) for _ctype, payload in chunks)
    out = bytearray()
    out.extend(struct.pack("<III", GLB_MAGIC, 2, total))
    for chunk_type, payload in chunks:
        out.extend(struct.pack("<II", len(payload), chunk_type))
        out.extend(payload)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(out)


def copy_scaled_body(src: Path, dst: Path, scale: tuple[float, float, float]) -> None:
    gltf, bin_chunk = load_glb(src)
    if bin_chunk is None:
        raise ValueError(f"{src}: missing BIN chunk")
    bin_chunk = bytearray(bin_chunk)
    buffer_views = gltf.get("bufferViews", [])
    accessors = gltf.get("accessors", [])
    sx, sy, sz = scale
    for mesh in gltf.get("meshes", []):
        for primitive in mesh.get("primitives", []):
            pos_idx = primitive.get("attributes", {}).get("POSITION")
            if pos_idx is None:
                continue
            _scale_accessor_positions(accessors, buffer_views, bin_chunk, pos_idx, sx, sy, sz)
    write_glb(dst, gltf, bin_chunk)


def extract_armature_only(body_path: Path, arm_path: Path) -> None:
    gltf, bin_chunk = load_glb(body_path)
    gltf = dict(gltf)
    gltf.pop("meshes", None)
    for node in gltf.get("nodes", []):
        node.pop("mesh", None)
    write_glb(arm_path, gltf, bin_chunk)


def write_box_part(path: Path, size: tuple[float, float, float]) -> None:
    """Static box mesh at origin — attached at socket during assembly."""
    sx, sy, sz = size
    hx, hy, hz = sx / 2, sy / 2, sz / 2
    positions = [
        -hx, -hy, hz, hx, -hy, hz, hx, hy, hz, -hx, hy, hz,
        -hx, -hy, -hz, hx, -hy, -hz, hx, hy, -hz, -hx, hy, -hz,
    ]
    indices = [0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 0, 4, 5, 0, 5, 1,
               2, 6, 7, 2, 7, 3, 0, 3, 7, 0, 7, 4, 1, 5, 6, 1, 6, 2]
    pos_bytes = struct.pack(f"<{len(positions)}f", *positions)
    idx_bytes = struct.pack(f"<{len(indices)}H", *indices)
    bin_chunk = pos_bytes + idx_bytes
    gltf = {
        "asset": {"version": "2.0", "generator": "tbg-assembly-fixture"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0, "name": path.stem}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0}, "indices": 1, "mode": 4}]}],
        "accessors": [
            {"bufferView": 0, "componentType": 5126, "count": 8, "type": "VEC3",
             "max": [hx, hy, hz], "min": [-hx, -hy, -hz]},
            {"bufferView": 1, "componentType": 5123, "count": len(indices), "type": "SCALAR"},
        ],
        "bufferViews": [
            {"buffer": 0, "byteOffset": 0, "byteLength": len(pos_bytes), "target": 34962},
            {"buffer": 0, "byteOffset": len(pos_bytes), "byteLength": len(idx_bytes), "target": 34963},
        ],
        "buffers": [{"byteLength": len(bin_chunk)}],
    }
    write_glb(path, gltf, bin_chunk)


def _scale_accessor_positions(
    accessors: list[dict],
    buffer_views: list[dict],
    bin_chunk: bytearray,
    accessor_idx: int,
    sx: float,
    sy: float,
    sz: float,
) -> None:
    accessor = accessors[accessor_idx]
    view = buffer_views[accessor["bufferView"]]
    start = view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
    count = accessor["count"]
    stride = 12
    values = list(struct.unpack_from(f"<{count * 3}f", bin_chunk, start))
    for i in range(0, len(values), 3):
        values[i] *= sx
        values[i + 1] *= sy
        values[i + 2] *= sz
    struct.pack_into(f"<{count * 3}f", bin_chunk, start, *values)
    accessor["min"] = [accessor["min"][0] * sx, accessor["min"][1] * sy, accessor["min"][2] * sz]
    accessor["max"] = [accessor["max"][0] * sx, accessor["max"][1] * sy, accessor["max"][2] * sz]
