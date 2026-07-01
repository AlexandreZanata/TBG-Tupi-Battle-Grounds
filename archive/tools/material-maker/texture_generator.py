"""Procedural PBR map generation for TBG character textures."""

from __future__ import annotations

import math
from pathlib import Path

from PIL import Image

from faction_graphs import GraphDescriptor


def export_unit_textures(
    unit_id: str,
    graph: GraphDescriptor,
    output_dir: Path,
    size: int,
) -> list[Path]:
    output_dir.mkdir(parents=True, exist_ok=True)
    written: list[Path] = []

    albedo_path = output_dir / f"{unit_id}_albedo.png"
    normal_path = output_dir / f"{unit_id}_normal.png"
    orm_path = output_dir / f"{unit_id}_orm.png"

    _write_albedo(albedo_path, graph, size)
    _write_normal(normal_path, graph, size)
    _write_orm(orm_path, graph, size)
    written.extend([albedo_path, normal_path, orm_path])

    if "emissive" in graph.outputs and graph.emissive_strength > 0:
        emissive_path = output_dir / f"{unit_id}_emissive.png"
        _write_emissive(emissive_path, graph, size)
        written.append(emissive_path)

    return written


def _write_albedo(path: Path, graph: GraphDescriptor, size: int) -> None:
    pixels = []
    for y in range(size):
        for x in range(size):
            t = _pattern(graph, x, y, size)
            color = _lerp_rgb(graph.primary_rgb, graph.secondary_rgb, t)
            pixels.append(color)
    image = Image.new("RGB", (size, size))
    image.putdata(pixels)
    image.save(path)


def _write_normal(path: Path, graph: GraphDescriptor, size: int) -> None:
    pixels = []
    for y in range(size):
        for x in range(size):
            hx = _height(graph, x + 1, y, size) - _height(graph, x - 1, y, size)
            hy = _height(graph, x, y + 1, size) - _height(graph, x, y - 1, size)
            nx = -hx * 2.0
            ny = -hy * 2.0
            nz = 1.0
            length = math.sqrt(nx * nx + ny * ny + nz * nz)
            r = int(((nx / length) * 0.5 + 0.5) * 255)
            g = int(((ny / length) * 0.5 + 0.5) * 255)
            b = int(((nz / length) * 0.5 + 0.5) * 255)
            pixels.append((r, g, b))
    image = Image.new("RGB", (size, size))
    image.putdata(pixels)
    image.save(path)


def _write_orm(path: Path, graph: GraphDescriptor, size: int) -> None:
    pixels = []
    for y in range(size):
        for x in range(size):
            t = _pattern(graph, x, y, size)
            ao = int(graph.ao_strength * (0.75 + 0.25 * t) * 255)
            rough = int(graph.roughness_base * (0.85 + 0.15 * (1.0 - t)) * 255)
            metal = int(graph.metallic_base * (0.7 + 0.3 * t) * 255)
            pixels.append((ao, rough, metal))
    image = Image.new("RGB", (size, size))
    image.putdata(pixels)
    image.save(path)


def _write_emissive(path: Path, graph: GraphDescriptor, size: int) -> None:
    pixels = []
    for y in range(size):
        for x in range(size):
            vein = _vein_mask(graph, x, y, size)
            value = int(vein * graph.emissive_strength * 255)
            pixels.append((value, value, value))
    image = Image.new("RGB", (size, size))
    image.putdata(pixels)
    image.save(path)


def _pattern(graph: GraphDescriptor, x: int, y: int, size: int) -> float:
    scale = graph.pattern_scale / size
    u = x * scale
    v = y * scale
    weave = 0.5 + 0.5 * math.sin(u * math.pi * 2) * math.cos(v * math.pi * 2)
    noise = _hash(x, y)
    return max(0.0, min(1.0, weave * 0.65 + noise * 0.35))


def _height(graph: GraphDescriptor, x: int, y: int, size: int) -> float:
    return _pattern(graph, x % size, y % size, size)


def _vein_mask(graph: GraphDescriptor, x: int, y: int, size: int) -> float:
    scale = graph.pattern_scale / size
    u = x * scale
    v = y * scale
    wave = abs(math.sin(u * 3.1 + math.sin(v * 2.7) * 1.5))
    return max(0.0, min(1.0, (1.0 - wave) ** 3))


def _hash(x: int, y: int) -> float:
    value = math.sin(x * 127.1 + y * 311.7) * 43758.5453
    return value - math.floor(value)


def _lerp_rgb(
    a: tuple[int, int, int],
    b: tuple[int, int, int],
    t: float,
) -> tuple[int, int, int]:
    return (
        int(a[0] + (b[0] - a[0]) * t),
        int(a[1] + (b[1] - a[1]) * t),
        int(a[2] + (b[2] - a[2]) * t),
    )
