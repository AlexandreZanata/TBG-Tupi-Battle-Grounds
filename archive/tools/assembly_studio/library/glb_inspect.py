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


def skinned_mesh_head_below_feet(gltf: dict, bin_chunk: bytes | None) -> bool:
    """True when dominant Head verts sit lower Y than Foot verts (inverted bind)."""
    if bin_chunk is None or not gltf.get("skins"):
        return False
    nodes = gltf.get("nodes", [])
    joint_names = [nodes[j].get("name", "") for j in gltf["skins"][0].get("joints", [])]
    head_y = float("-inf")
    foot_y = float("inf")
    accessors = gltf.get("accessors", [])
    views = gltf.get("bufferViews", [])
    for mesh in gltf.get("meshes", []):
        for prim in mesh.get("primitives", []):
            attrs = prim.get("attributes", {})
            if "JOINTS_0" not in attrs or "WEIGHTS_0" not in attrs:
                continue
            positions = _read_vec3_accessor(accessors, views, bin_chunk, attrs["POSITION"])
            joints = _read_joints_accessor(accessors, views, bin_chunk, attrs["JOINTS_0"])
            weights = _read_vec4_accessor(accessors, views, bin_chunk, attrs["WEIGHTS_0"])
            for i, (x, y, z) in enumerate(positions):
                ji = joints[i]
                wi = weights[i]
                best = 0.0
                bone_idx = ji[0]
                for j in range(4):
                    if wi[j] > best:
                        best = wi[j]
                        bone_idx = ji[j]
                name = joint_names[bone_idx] if bone_idx < len(joint_names) else ""
                if "Head" in name:
                    head_y = max(head_y, y)
                if "Foot" in name:
                    foot_y = min(foot_y, y)
    if not (head_y > float("-inf") and foot_y < float("inf")):
        return False
    return head_y < foot_y


def _read_vec3_accessor(
    accessors: list[dict], views: list[dict], bin_chunk: bytes, accessor_idx: int,
) -> list[tuple[float, float, float]]:
    acc = accessors[accessor_idx]
    view = views[acc["bufferView"]]
    start = view.get("byteOffset", 0) + acc.get("byteOffset", 0)
    count = acc["count"]
    vals = struct.unpack_from(f"<{count * 3}f", bin_chunk, start)
    return [(vals[i], vals[i + 1], vals[i + 2]) for i in range(0, len(vals), 3)]


def _read_vec4_accessor(
    accessors: list[dict], views: list[dict], bin_chunk: bytes, accessor_idx: int,
) -> list[tuple[float, float, float, float]]:
    acc = accessors[accessor_idx]
    view = views[acc["bufferView"]]
    start = view.get("byteOffset", 0) + acc.get("byteOffset", 0)
    count = acc["count"]
    vals = struct.unpack_from(f"<{count * 4}f", bin_chunk, start)
    return [(vals[i], vals[i + 1], vals[i + 2], vals[i + 3]) for i in range(0, len(vals), 4)]


def _read_joints_accessor(
    accessors: list[dict], views: list[dict], bin_chunk: bytes, accessor_idx: int,
) -> list[tuple[int, int, int, int]]:
    acc = accessors[accessor_idx]
    view = views[acc["bufferView"]]
    start = view.get("byteOffset", 0) + acc.get("byteOffset", 0)
    count = acc["count"]
    vals = struct.unpack_from(f"<{count * 4}B", bin_chunk, start)
    return [(vals[i], vals[i + 1], vals[i + 2], vals[i + 3]) for i in range(0, len(vals), 4)]
