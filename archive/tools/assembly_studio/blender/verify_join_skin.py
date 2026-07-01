"""Full mixamo edit build + verify skin weights before export."""
import json
import struct
import sys
from pathlib import Path

_SCRIPT = Path(__file__).resolve().parent
if str(_SCRIPT) not in sys.path:
    sys.path.insert(0, str(_SCRIPT))

from mixamo_edit_ops import _build_manual_align

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "content/characters/imports/mixamo/edit/_join_skin_test.glb"

report = _build_manual_align(
    ROOT / "content/characters/imports/mixamo/raw/Y Bot.fbx",
    OUT,
    ROOT / "content/characters/imports/mixamo/meshes/ybot_skin_01.glb",
    (0.0, 0.0, 0.0),
    1.0,
    1.0,
    None,
)

import bpy

mesh = next(o for o in bpy.context.scene.objects if o.type == "MESH")
print("mesh groups", len(mesh.vertex_groups), "names", [g.name for g in mesh.vertex_groups[:8]])
mods = [m.type for m in mesh.modifiers]
print("modifiers", mods)

data = OUT.read_bytes()
off = 12
chunk_len, _ = struct.unpack_from("<II", data, off)
gltf = json.loads(data[off + 8 : off + 8 + chunk_len])
attrs = gltf["meshes"][0]["primitives"][0]["attributes"]
print("export attrs", list(attrs.keys()))
print("report", report)
