"""Verify GLB export keeps JOINTS_0/WEIGHTS_0 skinning."""
import json
import struct
import sys
from pathlib import Path

import bpy

ROOT = Path(__file__).resolve().parents[3]
GLB = ROOT / "content/characters/imports/mixamo/edit/ybot_skin_01.glb"
OUT = ROOT / "content/characters/imports/mixamo/edit/_export_test.glb"

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(GLB))
mesh = next(o for o in bpy.context.scene.objects if o.type == "MESH")
arm = next(o for o in bpy.context.scene.objects if o.type == "ARMATURE")
bpy.ops.object.select_all(action="DESELECT")
mesh.select_set(True)
arm.select_set(True)
bpy.context.view_layer.objects.active = arm
bpy.ops.export_scene.gltf(
    filepath=str(OUT),
    export_format="GLB",
    use_selection=True,
    export_apply=False,
    export_skins=True,
    export_animations=True,
    export_yup=True,
)
print("exported", OUT, "groups", len(mesh.vertex_groups))

data = OUT.read_bytes()
off = 12
chunk_len, _ = struct.unpack_from("<II", data, off)
gltf = json.loads(data[off + 8 : off + 8 + chunk_len])
attrs = gltf["meshes"][0]["primitives"][0]["attributes"]
print("attrs", list(attrs.keys()))
print("has skin", "JOINTS_0" in attrs and "WEIGHTS_0" in attrs)
