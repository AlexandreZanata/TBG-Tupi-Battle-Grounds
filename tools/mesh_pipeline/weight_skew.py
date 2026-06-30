"""Skin weight dominance metrics from glTF (mirrors rig_weight_paint gate)."""

from __future__ import annotations

import struct
import tempfile
from collections import Counter
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path

from mesh_pipeline.glb_inspect import load_glb

SINGLE_BONE_DOMINANCE = 0.85


@dataclass(frozen=True)
class WeightSkewReport:
    vertex_count: int
    dominant_bone: str | None
    dominant_share: float
    skewed: bool
    unweighted_vertices: int
    bones_with_weights: int
    joint_names: tuple[str, ...] = ()
    bone_max_weight: tuple[tuple[str, float], ...] = ()

    @property
    def passes_gate(self) -> bool:
        return not self.skewed and self.unweighted_vertices == 0 and self.vertex_count > 0


def weights_skewed_to_one_bone(report: WeightSkewReport, threshold: float = SINGLE_BONE_DOMINANCE) -> bool:
    return report.skewed or report.vertex_count == 0


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


def analyze_weight_skew(path: Path, *, threshold: float = SINGLE_BONE_DOMINANCE) -> WeightSkewReport:
    with _readable_glb(path) as readable:
        return _analyze_weight_skew_plain(readable, threshold=threshold)


def _analyze_weight_skew_plain(path: Path, *, threshold: float) -> WeightSkewReport:
    gltf, bin_chunk = load_glb(path)
    skins = gltf.get("skins", [])
    if not skins or bin_chunk is None:
        return WeightSkewReport(0, None, 0.0, True, 0, 0, (), ())

    nodes = gltf.get("nodes", [])
    skin_joint_nodes = skins[0].get("joints", [])
    joint_names = tuple(_node_name(nodes, node_index) for node_index in skin_joint_nodes)
    counts: Counter[str] = Counter()
    unweighted = 0
    bones_used: set[str] = set()
    bone_max: dict[str, float] = {name: 0.0 for name in joint_names}
    vertex_count = 0

    for mesh in gltf.get("meshes", []):
        for primitive in mesh.get("primitives", []):
            attrs = primitive.get("attributes", {})
            joints_attr = attrs.get("JOINTS_0")
            weights_attr = attrs.get("WEIGHTS_0")
            if joints_attr is None or weights_attr is None:
                continue
            joint_rows = _read_accessor_vec4i(gltf, bin_chunk, joints_attr)
            weight_rows = _read_accessor_vec4f(gltf, bin_chunk, weights_attr)
            for joints, weights in zip(joint_rows, weight_rows, strict=True):
                vertex_count += 1
                total = sum(weights)
                if total <= 1e-8:
                    unweighted += 1
                    continue
                norm = [w / total for w in weights]
                dominant_idx = max(range(4), key=lambda i: norm[i])
                if norm[dominant_idx] <= 1e-6:
                    unweighted += 1
                    continue
                bone = joint_names[joints[dominant_idx]] if joints[dominant_idx] < len(joint_names) else "?"
                counts[bone] += 1
                for idx, weight in zip(joints, norm, strict=True):
                    if idx >= len(joint_names):
                        continue
                    name = joint_names[idx]
                    if weight > 0.01:
                        bones_used.add(name)
                    bone_max[name] = max(bone_max.get(name, 0.0), weight)

    if vertex_count == 0:
        return _make_report(0, None, 0.0, True, unweighted, bones_used, joint_names, bone_max, threshold)

    if not counts:
        return _make_report(vertex_count, None, 1.0, True, unweighted, bones_used, joint_names, bone_max, threshold)

    dominant_bone, dominant_count = counts.most_common(1)[0]
    share = dominant_count / vertex_count
    return _make_report(
        vertex_count,
        dominant_bone,
        share,
        share >= threshold,
        unweighted,
        bones_used,
        joint_names,
        bone_max,
        threshold,
    )


def _make_report(
    vertex_count: int,
    dominant_bone: str | None,
    share: float,
    skewed: bool,
    unweighted: int,
    bones_used: set[str],
    joint_names: tuple[str, ...],
    bone_max: dict[str, float],
    threshold: float,
) -> WeightSkewReport:
    _ = threshold
    max_items = tuple(sorted(bone_max.items()))
    return WeightSkewReport(
        vertex_count=vertex_count,
        dominant_bone=dominant_bone,
        dominant_share=share,
        skewed=skewed,
        unweighted_vertices=unweighted,
        bones_with_weights=len(bones_used),
        joint_names=joint_names,
        bone_max_weight=max_items,
    )


def _node_name(nodes: list[dict], index: int) -> str:
    if index < 0 or index >= len(nodes):
        return "?"
    return nodes[index].get("name") or f"node_{index}"


def _read_accessor_vec4f(gltf: dict, bin_chunk: bytes, accessor_index: int) -> list[tuple[float, float, float, float]]:
    accessor = gltf["accessors"][accessor_index]
    if accessor["type"] != "VEC4" or accessor["componentType"] != 5126:
        raise ValueError(f"expected FLOAT VEC4 accessor, got {accessor}")
    return _read_interleaved(gltf, bin_chunk, accessor, struct.Struct("<4f"))


def _read_accessor_vec4i(gltf: dict, bin_chunk: bytes, accessor_index: int) -> list[tuple[int, int, int, int]]:
    accessor = gltf["accessors"][accessor_index]
    if accessor["type"] != "VEC4":
        raise ValueError(f"expected VEC4 joints accessor, got {accessor}")
    component = accessor["componentType"]
    if component == 5121:
        return _read_interleaved(gltf, bin_chunk, accessor, struct.Struct("<4B"))
    if component == 5123:
        return _read_interleaved(gltf, bin_chunk, accessor, struct.Struct("<4H"))
    raise ValueError(f"unsupported joints componentType {component}")


def _read_interleaved(
    gltf: dict,
    bin_chunk: bytes,
    accessor: dict,
    item_struct: struct.Struct,
) -> list:
    if "bufferView" not in accessor:
        return []
    buffer_views = gltf["bufferViews"]
    view = buffer_views[accessor["bufferView"]]
    base = view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
    stride = view.get("byteStride") or item_struct.size
    count = accessor["count"]
    rows = []
    for i in range(count):
        offset = base + i * stride
        rows.append(item_struct.unpack_from(bin_chunk, offset))
    return rows
