"""Detect fragmented meshes (floating islands) after decimation."""

from __future__ import annotations

import array
import struct
import tempfile
from dataclasses import dataclass
from pathlib import Path

from mesh_pipeline.glb_inspect import load_glb

MAX_SIGNIFICANT_ISLANDS = 5
MIN_LARGEST_ISLAND_RATIO = 0.75
MIN_ISLAND_VERTS = 15
POSITION_EPSILON = 1e-4


@dataclass(frozen=True)
class MeshIntegrity:
    island_count: int
    vertex_count: int
    largest_island_ratio: float

    @property
    def ok(self) -> bool:
        if self.vertex_count == 0:
            return False
        if self.island_count > MAX_SIGNIFICANT_ISLANDS:
            return False
        return self.largest_island_ratio >= MIN_LARGEST_ISLAND_RATIO


def mesh_integrity(path: Path) -> MeshIntegrity:
    gltf, _ = load_glb(path)
    if "KHR_draco_mesh_compression" in (gltf.get("extensionsUsed") or []):
        return _integrity_draco(path)
    return _integrity_plain(path)


def integrity_error(path: Path) -> str | None:
    stats = mesh_integrity(path)
    if stats.ok:
        return None
    if stats.vertex_count == 0:
        return "mesh has no vertices"
    if stats.island_count > MAX_SIGNIFICANT_ISLANDS:
        return f"mesh has {stats.island_count} significant disconnected islands (max {MAX_SIGNIFICANT_ISLANDS})"
    return (
        f"mesh fragmented: largest island is {stats.largest_island_ratio:.0%} of vertices "
        f"(min {MIN_LARGEST_ISLAND_RATIO:.0%})"
    )


def _integrity_draco(path: Path) -> MeshIntegrity:
    from mesh_pipeline.game_glb_transform import run_gltf_transform

    with tempfile.NamedTemporaryFile(suffix=".glb", delete=False) as handle:
        decoded = Path(handle.name)
    try:
        run_gltf_transform(["copy", str(path), str(decoded)])
        return _integrity_plain(decoded)
    finally:
        decoded.unlink(missing_ok=True)


def _integrity_plain(path: Path) -> MeshIntegrity:
    gltf, bin_chunk = load_glb(path)
    if bin_chunk is None:
        return MeshIntegrity(0, 0, 0.0)

    island_sizes, total_unique = _significant_island_sizes(gltf, bin_chunk)
    if not island_sizes or total_unique == 0:
        return MeshIntegrity(0, total_unique, 0.0)
    largest = max(island_sizes)
    ratio = largest / total_unique
    return MeshIntegrity(len(island_sizes), total_unique, ratio)


def _significant_island_sizes(gltf: dict, bin_chunk: bytes) -> tuple[list[int], int]:
    accessors = gltf.get("accessors", [])
    buffer_views = gltf.get("bufferViews", [])
    all_island_keys: dict[tuple[int, int, int], set[tuple[int, int, int]]] = {}

    for mesh in gltf.get("meshes", []):
        for primitive in mesh.get("primitives", []):
            position = primitive.get("attributes", {}).get("POSITION")
            indices_ref = primitive.get("indices")
            if position is None or indices_ref is None:
                continue
            positions = _read_vec3(gltf, bin_chunk, accessors, buffer_views, position)
            indices = _read_indices(gltf, bin_chunk, accessors, buffer_views, indices_ref)
            _merge_islands(all_island_keys, positions, indices)

    if not all_island_keys:
        return [], 0

    unique_sizes = [len(keys) for keys in all_island_keys.values()]
    total_unique = len({key for keys in all_island_keys.values() for key in keys})
    significant = sorted((size for size in unique_sizes if size >= MIN_ISLAND_VERTS), reverse=True)
    return significant, total_unique


def _merge_islands(
    islands: dict[tuple[int, int, int], set[tuple[int, int, int]]],
    positions: list[tuple[float, float, float]],
    indices: array.array,
) -> None:
    if not positions or not indices:
        return

    parent: dict[tuple[int, int, int], tuple[int, int, int]] = {}

    def find(key: tuple[int, int, int]) -> tuple[int, int, int]:
        root = key
        while parent.get(root, root) != root:
            root = parent[root]
        return root

    def union(a: tuple[int, int, int], b: tuple[int, int, int]) -> None:
        ra, rb = find(a), find(b)
        if ra != rb:
            parent[rb] = ra

    def key_at(vertex_index: int) -> tuple[int, int, int]:
        return _position_key(positions[vertex_index])

    for i in range(0, len(indices), 3):
        keys = (key_at(indices[i]), key_at(indices[i + 1]), key_at(indices[i + 2]))
        union(keys[0], keys[1])
        union(keys[1], keys[2])

    for i in range(0, len(indices), 3):
        for vi in (indices[i], indices[i + 1], indices[i + 2]):
            root = find(key_at(vi))
            pos_key = key_at(vi)
            islands.setdefault(root, set()).add(pos_key)


def _read_indices(
    gltf: dict,
    bin_chunk: bytes,
    accessors: list[dict],
    buffer_views: list[dict],
    index_ref: int,
) -> array.array:
    accessor = accessors[index_ref]
    if "bufferView" not in accessor:
        return array.array("I")
    view = buffer_views[accessor["bufferView"]]
    start = view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
    count = accessor["count"]
    type_map = {5121: "B", 5123: "H", 5125: "I"}
    raw = bin_chunk[start : start + count * array.array(type_map[accessor["componentType"]]).itemsize]
    return array.array(type_map[accessor["componentType"]], raw)


def _read_vec3(
    gltf: dict,
    bin_chunk: bytes,
    accessors: list[dict],
    buffer_views: list[dict],
    position_ref: int,
) -> list[tuple[float, float, float]]:
    accessor = accessors[position_ref]
    if "bufferView" not in accessor:
        return []
    view = buffer_views[accessor["bufferView"]]
    start = view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
    count = accessor["count"]
    raw = bin_chunk[start : start + count * 12]
    return [struct.unpack_from("<3f", raw, i * 12) for i in range(count)]


def _position_key(pos: tuple[float, float, float]) -> tuple[int, int, int]:
    scale = 1.0 / POSITION_EPSILON
    return (int(round(pos[0] * scale)), int(round(pos[1] * scale)), int(round(pos[2] * scale)))
