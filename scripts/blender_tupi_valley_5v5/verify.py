"""Check a saved Valley assembly and the matching embedded triangle meshes."""

import json
import sys
from pathlib import Path
from struct import iter_unpack

import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from build import BRIDGES, camps, towers


def placements(layer, prefixes):
    scene = bpy.data.scenes["01  Tupi Valley 5v5"]
    objects = scene.collection.children[layer].objects
    return sorted(
        (obj.instance_collection.name, tuple(round(v, 3) for v in obj.location.xy))
        for obj in objects
        if obj.instance_collection and obj.instance_collection.name.startswith(prefixes)
    )


def main():
    out, runtime = map(Path, sys.argv[sys.argv.index("--") + 1:])
    assert placements("02  Three lanes, bridges and plaza", ("Tupi bridge",)) == sorted(
        ("Tupi bridge deck", at) for at, _ in BRIDGES
    )
    assert placements("04  Towers and team pennants", ("Tupi tower",)) == sorted(
        (f"Tupi {part} {team}", at)
        for team, _, _, at in towers()
        for part in ("tower", "tower crystal")
    )
    assert placements("05  Jungle camps", ("Tupi camp",)) == sorted(
        (f"Tupi camp {kind}", at) for at, kind in camps()
    )
    board = bpy.data.scenes["02  Tupi Valley component board"]
    samples = [obj.instance_collection for obj in board.objects if obj.instance_collection]
    assert len(samples) == 24 and all(asset.asset_data for asset in samples)

    expected = json.loads((out / "metrics.json").read_text())["runtime_triangles"]
    paths = sorted(runtime.glob("*.tbgpart"))
    assert {path.stem for path in paths} == set(expected)
    for path in paths:
        data = path.read_bytes()
        assert len(data) == 27 * expected[path.stem], path
        vertices = [Vector(row[:3]) for row in iter_unpack("<hhhBBB", data)]
        for a, b, c in zip(vertices[::3], vertices[1::3], vertices[2::3], strict=True):
            assert (b - a).cross(c - a).length_squared > 0, path
    for damage in range(3):
        assert expected[f"tower-blue-{damage}"] == expected[f"tower-red-{damage}"]
    assert expected["tower-blue-0"] > expected["tower-blue-1"] > expected["tower-blue-2"]
    print(f"Validated two scenes, 24 board assets, 2 bridges, 22 Towers, 8 Camps and {len(paths)} meshes.")


if __name__ == "__main__":
    main()
