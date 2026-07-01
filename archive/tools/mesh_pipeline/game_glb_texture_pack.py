"""Pack metallic/roughness/occlusion into a single ORM texture in game GLBs."""

from __future__ import annotations

import json
import struct
from io import BytesIO
from pathlib import Path

from PIL import Image

from mesh_pipeline.glb_inspect import GLB_MAGIC, JSON_CHUNK_TYPE, BIN_CHUNK_TYPE, load_glb

ORM_TEXTURE_NAME = "TBG_ORM"


def pack_orm_textures(path: Path) -> Path:
    """Merge MR + optional AO into one ORM map; drop orphaned images."""
    gltf, bin_chunk = load_glb(path)
    bin_data = bytearray(bin_chunk or b"")
    changed = False

    for material in gltf.get("materials", []):
        if _pack_material_orm(gltf, material, bin_data):
            changed = True

    if not changed:
        return path

    _compact_textures(gltf)
    _compact_images(gltf)
    if gltf.get("buffers"):
        gltf["buffers"][0]["byteLength"] = len(bin_data)
    _write_glb(path, gltf, bytes(bin_data))
    return path


def _pack_material_orm(gltf: dict, material: dict, bin_data: bytearray) -> bool:
    pbr = material.get("pbrMetallicRoughness") or {}
    mr_tex = (pbr.get("metallicRoughnessTexture") or {}).get("index")
    occ_tex = (material.get("occlusionTexture") or {}).get("index")
    if mr_tex is None and occ_tex is None:
        return False

    mr_bytes = _texture_bytes(gltf, bin_data, mr_tex) if mr_tex is not None else None
    occ_bytes = _texture_bytes(gltf, bin_data, occ_tex) if occ_tex is not None else None
    orm_png = _build_orm_png(mr_bytes, occ_bytes)
    orm_index = _append_texture(gltf, bin_data, orm_png)

    pbr["metallicRoughnessTexture"] = {"index": orm_index}
    material["occlusionTexture"] = {"index": orm_index, "strength": 1.0}
    material["pbrMetallicRoughness"] = pbr
    return True


def _build_orm_png(mr_bytes: bytes | None, occ_bytes: bytes | None) -> bytes:
    size = _image_size(mr_bytes) or _image_size(occ_bytes) or (1024, 1024)
    rough, metal = _roughness_metallic(mr_bytes, size)
    ao = _channel(occ_bytes, size, default=255)
    orm = Image.merge("RGB", (ao, rough, metal))
    buf = BytesIO()
    orm.save(buf, format="PNG")
    return buf.getvalue()


def _roughness_metallic(mr_bytes: bytes | None, size: tuple[int, int]) -> tuple[Image.Image, Image.Image]:
    if not mr_bytes:
        return Image.new("L", size, 128), Image.new("L", size, 0)
    with Image.open(BytesIO(mr_bytes)) as image:
        rgb = image.convert("RGB").resize(size, Image.Resampling.BILINEAR)
        rough = rgb.getchannel("G")
        metal = rgb.getchannel("B")
    return rough, metal


def _channel(payload: bytes | None, size: tuple[int, int], default: int) -> Image.Image:
    if not payload:
        return Image.new("L", size, default)
    with Image.open(BytesIO(payload)) as image:
        return image.convert("L").resize(size, Image.Resampling.BILINEAR)


def _image_size(payload: bytes | None) -> tuple[int, int] | None:
    if not payload:
        return None
    with Image.open(BytesIO(payload)) as image:
        return image.size


def _texture_bytes(gltf: dict, bin_data: bytes | bytearray, texture_index: int) -> bytes | None:
    texture = gltf["textures"][texture_index]
    return _read_image_bytes(gltf, bin_data, texture["source"])


def _read_image_bytes(gltf: dict, bin_data: bytes | bytearray, image_index: int) -> bytes | None:
    image = gltf["images"][image_index]
    if "bufferView" not in image:
        return None
    view = gltf["bufferViews"][image["bufferView"]]
    start = view.get("byteOffset", 0)
    end = start + view["byteLength"]
    return bytes(bin_data[start:end])


def _append_texture(gltf: dict, bin_data: bytearray, png_bytes: bytes) -> int:
    offset = _pad4(len(bin_data))
    if offset > len(bin_data):
        bin_data.extend(b"\x00" * (offset - len(bin_data)))
    bin_data.extend(png_bytes)

    images = gltf.setdefault("images", [])
    textures = gltf.setdefault("textures", [])
    buffer_views = gltf.setdefault("bufferViews", [])
    buffers = gltf.setdefault("buffers", [{"byteLength": 0}])

    view_index = len(buffer_views)
    buffer_views.append({"buffer": 0, "byteOffset": offset, "byteLength": len(png_bytes)})
    with Image.open(BytesIO(png_bytes)) as image:
        image_index = len(images)
        images.append(
            {
                "name": ORM_TEXTURE_NAME,
                "mimeType": "image/png",
                "bufferView": view_index,
                "width": image.width,
                "height": image.height,
            }
        )
    texture_index = len(textures)
    textures.append({"source": image_index, "sampler": 0})
    buffers[0]["byteLength"] = len(bin_data)
    return texture_index


def _compact_textures(gltf: dict) -> None:
    used = _collect_texture_indices(gltf)
    if not used:
        return
    ordered = sorted(used)
    remap = {old: new for new, old in enumerate(ordered)}
    gltf["textures"] = [gltf["textures"][index] for index in ordered]
    _remap_texture_refs(gltf, remap)


def _collect_texture_indices(gltf: dict) -> set[int]:
    used: set[int] = set()
    for material in gltf.get("materials", []):
        pbr = material.get("pbrMetallicRoughness") or {}
        for key in ("baseColorTexture", "metallicRoughnessTexture"):
            ref = pbr.get(key)
            if ref and "index" in ref:
                used.add(ref["index"])
        for key in ("normalTexture", "occlusionTexture", "emissiveTexture"):
            ref = material.get(key)
            if ref and "index" in ref:
                used.add(ref["index"])
    return used


def _remap_texture_refs(gltf: dict, remap: dict[int, int]) -> None:
    for material in gltf.get("materials", []):
        pbr = material.get("pbrMetallicRoughness") or {}
        for key in ("baseColorTexture", "metallicRoughnessTexture"):
            ref = pbr.get(key)
            if ref and ref["index"] in remap:
                ref["index"] = remap[ref["index"]]
        for key in ("normalTexture", "occlusionTexture", "emissiveTexture"):
            ref = material.get(key)
            if ref and ref["index"] in remap:
                ref["index"] = remap[ref["index"]]


def _compact_images(gltf: dict) -> None:
    used = {texture["source"] for texture in gltf.get("textures", [])}
    if not used:
        return
    ordered = sorted(used)
    remap = {old: new for new, old in enumerate(ordered)}
    gltf["images"] = [gltf["images"][index] for index in ordered]
    for texture in gltf.get("textures", []):
        texture["source"] = remap[texture["source"]]


def _pad4(length: int) -> int:
    return length + ((4 - length % 4) % 4)


def _write_glb(path: Path, gltf: dict, bin_data: bytes) -> None:
    json_bytes = json.dumps(gltf, separators=(",", ":")).encode("utf-8")
    json_padded = json_bytes + b" " * ((4 - len(json_bytes) % 4) % 4)
    bin_padded = bin_data + b"\x00" * ((4 - len(bin_data) % 4) % 4)
    total = 12 + 8 + len(json_padded) + 8 + len(bin_padded)
    header = struct.pack("<III", GLB_MAGIC, 2, total)
    json_header = struct.pack("<II", len(json_padded), JSON_CHUNK_TYPE)
    bin_header = struct.pack("<II", len(bin_padded), BIN_CHUNK_TYPE)
    path.write_bytes(header + json_header + json_padded + bin_header + bin_padded)
