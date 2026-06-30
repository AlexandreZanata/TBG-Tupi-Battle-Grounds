"""Axis-aligned bounds from glTF POSITION accessors (Draco-aware, node transforms applied)."""

from __future__ import annotations

import struct
import tempfile
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path

from mesh_pipeline.glb_inspect import load_glb


@dataclass(frozen=True)
class MeshBounds:
    min_x: float
    max_x: float
    min_y: float
    max_y: float
    min_z: float
    max_z: float

    @property
    def height(self) -> float:
        return self.max_y - self.min_y

    @property
    def center_x(self) -> float:
        return (self.min_x + self.max_x) * 0.5

    @property
    def center_z(self) -> float:
        return (self.min_z + self.max_z) * 0.5


def mesh_bounds(path: Path) -> MeshBounds:
    with _readable_glb(path) as readable:
        return _mesh_bounds_plain(readable)


@contextmanager
def _readable_glb(path: Path):
    gltf, _ = load_glb(path)
    if "KHR_draco_mesh_compression" not in (gltf.get("extensionsUsed") or []):
        yield path
        return
    from mesh_pipeline.game_glb_transform import run_gltf_transform

    with tempfile.NamedTemporaryFile(suffix=".glb", delete=False) as handle:
        decoded = Path(handle.name)
    try:
        run_gltf_transform(["copy", str(path), str(decoded)])
        yield decoded
    finally:
        decoded.unlink(missing_ok=True)


def _mesh_bounds_plain(path: Path) -> MeshBounds:
    gltf, bin_chunk = load_glb(path)
    if bin_chunk is None:
        raise ValueError(f"{path}: missing binary chunk")

    nodes = gltf.get("nodes", [])
    scene = gltf["scenes"][gltf["scene"]]
    world_points: list[tuple[float, float, float]] = []
    for root in scene.get("nodes", []):
        _collect_mesh_points(gltf, bin_chunk, nodes, root, _identity_matrix(), world_points)

    if not world_points:
        raise ValueError(f"{path}: no POSITION data")
    xs = [point[0] for point in world_points]
    ys = [point[1] for point in world_points]
    zs = [point[2] for point in world_points]
    return MeshBounds(min(xs), max(xs), min(ys), max(ys), min(zs), max(zs))


def _collect_mesh_points(
    gltf: dict,
    bin_chunk: bytes,
    nodes: list[dict],
    node_index: int,
    parent: list[list[float]],
    out: list[tuple[float, float, float]],
) -> None:
    node = nodes[node_index]
    world = _multiply_mat(parent, _node_matrix(node))
    mesh_index = node.get("mesh")
    if mesh_index is not None:
        for x, y, z in _mesh_local_positions(gltf, bin_chunk, mesh_index):
            out.append(_transform_point(world, x, y, z))
    for child in node.get("children", []):
        _collect_mesh_points(gltf, bin_chunk, nodes, child, world, out)


def _mesh_local_positions(gltf: dict, bin_chunk: bytes, mesh_index: int) -> list[tuple[float, float, float]]:
    points: list[tuple[float, float, float]] = []
    for primitive in gltf["meshes"][mesh_index].get("primitives", []):
        position = primitive.get("attributes", {}).get("POSITION")
        if position is not None:
            points.extend(_read_vec3(gltf, bin_chunk, position))
    return points


def _identity_matrix() -> list[list[float]]:
    return [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]


def _node_matrix(node: dict) -> list[list[float]]:
    if "matrix" in node:
        flat = node["matrix"]
        return [list(flat[i : i + 4]) for i in range(0, 16, 4)]
    return _trs_matrix(node.get("translation"), node.get("rotation"), node.get("scale"))


def _trs_matrix(
    translation: list[float] | None,
    rotation: list[float] | None,
    scale: list[float] | None,
) -> list[list[float]]:
    matrix = _identity_matrix()
    if scale:
        matrix[0][0], matrix[1][1], matrix[2][2] = scale[0], scale[1], scale[2]
    if rotation:
        matrix = _multiply_mat(matrix, _quat_matrix(rotation))
    if translation:
        matrix[0][3], matrix[1][3], matrix[2][3] = translation[0], translation[1], translation[2]
    return matrix


def _quat_matrix(quat: list[float]) -> list[list[float]]:
    x, y, z, w = quat
    return [
        [1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w), 0.0],
        [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w), 0.0],
        [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y), 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]


def _multiply_mat(a: list[list[float]], b: list[list[float]]) -> list[list[float]]:
    out = [[0.0] * 4 for _ in range(4)]
    for row in range(4):
        for col in range(4):
            out[row][col] = sum(a[row][k] * b[k][col] for k in range(4))
    return out


def _transform_point(matrix: list[list[float]], x: float, y: float, z: float) -> tuple[float, float, float]:
    return (
        matrix[0][0] * x + matrix[0][1] * y + matrix[0][2] * z + matrix[0][3],
        matrix[1][0] * x + matrix[1][1] * y + matrix[1][2] * z + matrix[1][3],
        matrix[2][0] * x + matrix[2][1] * y + matrix[2][2] * z + matrix[2][3],
    )


def _read_vec3(gltf: dict, bin_chunk: bytes, accessor_index: int) -> list[tuple[float, float, float]]:
    accessor = gltf["accessors"][accessor_index]
    if accessor["type"] != "VEC3" or accessor["componentType"] != 5126:
        raise ValueError(f"expected FLOAT VEC3 POSITION, got {accessor}")
    if "bufferView" not in accessor:
        return []
    view = gltf["bufferViews"][accessor["bufferView"]]
    base = view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
    stride = view.get("byteStride") or 12
    count = accessor["count"]
    return [struct.unpack_from("<3f", bin_chunk, base + i * stride) for i in range(count)]
