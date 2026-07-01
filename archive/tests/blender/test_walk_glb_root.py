"""Contract: baked Walk GLB — TBG_Root rotation stays on bind axis (no bob leak)."""

import struct
import json
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
SCOUT_GAME = REPO / "content/characters/meshes/game/demo_scout_01.glb"


def _parse_glb(path: Path):
    data = path.read_bytes()
    off = 12
    json_chunk = bin_chunk = None
    while off < len(data):
        length = struct.unpack_from("<I", data, off)[0]
        off += 4
        chunk_type = data[off : off + 4]
        off += 4
        chunk = data[off : off + length]
        off += length
        if chunk_type == b"JSON":
            json_chunk = json.loads(chunk)
        elif chunk_type == b"BIN\x00":
            bin_chunk = chunk
    return json_chunk, bin_chunk


def _read_vec4_accessor(gltf: dict, blob: bytes, accessor_idx: int) -> list[tuple[float, ...]]:
    acc = gltf["accessors"][accessor_idx]
    bv = gltf["bufferViews"][acc["bufferView"]]
    start = bv.get("byteOffset", 0) + acc.get("byteOffset", 0)
    out = []
    for i in range(acc["count"]):
        out.append(struct.unpack_from("<ffff", blob, start + i * 16))
    return out


def test_scout_walk_root_rotation_has_no_bob_quaternion_leak():
    assert SCOUT_GAME.is_file(), "run export_game_unit.py demo_scout_01 after rebake"
    gltf, blob = _parse_glb(SCOUT_GAME)
    names = {i: n.get("name", "?") for i, n in enumerate(gltf["nodes"])}
    walk = next(a for a in gltf["animations"] if a["name"] == "Walk")
    for channel in walk["channels"]:
        if gltf["nodes"][channel["target"]["node"]].get("name") != "TBG_Root":
            continue
        if channel["target"]["path"] != "rotation":
            continue
        sampler = walk["samplers"][channel["sampler"]]
        quats = _read_vec4_accessor(gltf, blob, sampler["output"])
        for x, y, z, w in quats:
            assert abs(y) < 1e-5, f"root quat Y leak {y}"
            assert abs(z) < 1e-5, f"root quat Z leak {z}"
            assert abs(x + 0.7071067) < 0.001
            assert abs(w - 0.7071067) < 0.001
        return
    raise AssertionError("Walk missing TBG_Root rotation track")
