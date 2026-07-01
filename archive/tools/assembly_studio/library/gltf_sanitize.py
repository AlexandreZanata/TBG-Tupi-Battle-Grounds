"""Post-export glTF fixes for MB-Lab body GLBs (no Blender required)."""

from __future__ import annotations

from pathlib import Path

from assembly_studio.library.glb_inspect import load_glb
from assembly_studio.library.glb_write import write_glb


def promote_skinned_meshes_to_root(gltf: dict) -> int:
    """Move skinned mesh nodes to scene roots (fixes NODE_SKINNED_MESH_NON_ROOT)."""
    nodes = gltf.setdefault("nodes", [])
    parent_of: dict[int, int] = {}
    for idx, node in enumerate(nodes):
        for child in node.get("children", []):
            parent_of[child] = idx

    scenes = gltf.get("scenes") or [{"nodes": []}]
    scene = scenes[0]
    root_nodes = scene.setdefault("nodes", [])
    root_set = set(root_nodes)
    promoted = 0

    for idx, node in enumerate(nodes):
        if "skin" not in node:
            continue
        parent_idx = parent_of.get(idx)
        if parent_idx is None:
            continue
        children = nodes[parent_idx].get("children", [])
        if idx not in children:
            continue
        children.remove(idx)
        if children:
            nodes[parent_idx]["children"] = children
        else:
            nodes[parent_idx].pop("children", None)
        if idx not in root_set:
            root_nodes.append(idx)
            root_set.add(idx)
        promoted += 1

    gltf["scenes"] = scenes
    return promoted


def strip_extra_texcoords(gltf: dict, *, keep: int = 1) -> int:
    """Drop TEXCOORD_N attributes not used by exported materials."""
    removed = 0
    for mesh in gltf.get("meshes", []):
        for prim in mesh.get("primitives", []):
            attrs = prim.get("attributes", {})
            for key in list(attrs.keys()):
                if not key.startswith("TEXCOORD_"):
                    continue
                try:
                    layer = int(key.split("_", 1)[1])
                except ValueError:
                    continue
                if layer >= keep:
                    del attrs[key]
                    removed += 1
    return removed


def skinned_mesh_has_parent(gltf: dict) -> bool:
    nodes = gltf.get("nodes", [])
    parent_of: dict[int, int] = {}
    for idx, node in enumerate(nodes):
        for child in node.get("children", []):
            parent_of[child] = idx
    return any("skin" in nodes[i] and i in parent_of for i in range(len(nodes)))


def max_texcoord_layer(gltf: dict) -> int:
    highest = -1
    for mesh in gltf.get("meshes", []):
        for prim in mesh.get("primitives", []):
            for key in prim.get("attributes", {}):
                if key.startswith("TEXCOORD_"):
                    highest = max(highest, int(key.split("_", 1)[1]))
    return highest


def sanitize_glb(path: Path, *, keep_texcoords: int = 1) -> dict[str, int]:
    gltf, bin_chunk = load_glb(path)
    report = {
        "skinnedPromoted": promote_skinned_meshes_to_root(gltf),
        "texCoordsStripped": strip_extra_texcoords(gltf, keep=keep_texcoords),
    }
    write_glb(path, gltf, bin_chunk)
    return report
