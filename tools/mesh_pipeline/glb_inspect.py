"""Inspect GLB assets for mobile game budget validation."""

from __future__ import annotations

import json
import struct
from dataclasses import dataclass
from io import BytesIO
from pathlib import Path

from PIL import Image

GLB_MAGIC = 0x46546C67
JSON_CHUNK_TYPE = 0x4E4F534A
BIN_CHUNK_TYPE = 0x004E4942


@dataclass(frozen=True)
class GlbStats:
    triangles: int
    bones: int
    skins: int
    animations: int
    materials: int
    textures: int
    max_texture_size: int
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
    gltf, bin_chunk = load_glb(path)
    return GlbStats(
        triangles=count_triangles(gltf),
        bones=count_bones(gltf),
        skins=len(gltf.get("skins", [])),
        animations=len(gltf.get("animations", [])),
        materials=len(gltf.get("materials", [])),
        textures=len(gltf.get("textures", [])),
        max_texture_size=max_texture_dimension(gltf, bin_chunk),
        file_bytes=path.stat().st_size,
    )


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


def max_texture_dimension(gltf: dict, bin_chunk: bytes | None) -> int:
    images = gltf.get("images", [])
    if not images:
        return 0

    buffer_views = gltf.get("bufferViews", [])
    max_dim = 0
    for image in images:
        width = image.get("width")
        height = image.get("height")
        if width and height:
            max_dim = max(max_dim, int(width), int(height))
            continue
        if "bufferView" not in image or bin_chunk is None:
            continue
        view = buffer_views[image["bufferView"]]
        start = view.get("byteOffset", 0)
        end = start + view["byteLength"]
        max_dim = max(max_dim, *_image_dimensions(bin_chunk[start:end]))
    return max_dim


def _primitive_triangles(primitive: dict, accessors: list[dict]) -> int:
    indices = primitive.get("indices")
    if indices is not None:
        return accessors[indices]["count"] // 3

    position = primitive.get("attributes", {}).get("POSITION")
    if position is None:
        return 0
    return accessors[position]["count"] // 3


def _image_dimensions(image_bytes: bytes) -> tuple[int, int]:
    with Image.open(BytesIO(image_bytes)) as image:
        return image.size


def skinned_mesh_has_weights(path: Path) -> bool:
    """Return True when skinned meshes expose joint/weight attributes."""
    gltf, _bin = load_glb(path)
    skins = gltf.get("skins", [])
    if not skins:
        return True
    for mesh in gltf.get("meshes", []):
        for primitive in mesh.get("primitives", []):
            attrs = primitive.get("attributes", {})
            if "JOINTS_0" in attrs and "WEIGHTS_0" in attrs:
                return True
    return False


def base_color_texture_is_valid(path: Path, *, min_avg: float = 8.0) -> bool:
    """Detect empty/black albedo maps (failed texture bake)."""
    gltf, bin_chunk = load_glb(path)
    if bin_chunk is None:
        return True
    pbr = (gltf.get("materials") or [{}])[0].get("pbrMetallicRoughness") or {}
    tex_index = (pbr.get("baseColorTexture") or {}).get("index")
    if tex_index is None:
        return True
    texture = gltf["textures"][tex_index]
    image = gltf["images"][texture["source"]]
    if "bufferView" not in image:
        return True
    view = gltf["bufferViews"][image["bufferView"]]
    start = view.get("byteOffset", 0)
    payload = bin_chunk[start : start + view["byteLength"]]
    with Image.open(BytesIO(payload)) as img:
        rgb = img.convert("RGB")
        pixels = rgb.getdata()
        if not pixels:
            return False
        total = [0.0, 0.0, 0.0]
        for r, g, b in pixels:
            total[0] += r
            total[1] += g
            total[2] += b
        count = len(pixels)
        avg = sum(total) / (3 * count)
        return avg >= min_avg
