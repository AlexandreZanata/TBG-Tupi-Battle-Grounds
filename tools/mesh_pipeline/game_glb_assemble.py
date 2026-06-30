"""Assemble a budget-compliant skinned game GLB with named animation stubs."""

from __future__ import annotations

import struct
from pathlib import Path

from animation_sets.placeholder_glb import _CUBE_INDICES, _CUBE_POSITIONS, _pack_glb

TBG_BONES = (
    "TBG_Root",
    "TBG_Spine",
    "TBG_Spine1",
    "TBG_Neck",
    "TBG_Head",
    "TBG_Shoulder.L",
    "TBG_UpperArm.L",
    "TBG_LowerArm.L",
    "TBG_Hand.L",
    "TBG_Shoulder.R",
    "TBG_UpperArm.R",
    "TBG_LowerArm.R",
    "TBG_Hand.R",
    "TBG_UpperLeg.L",
    "TBG_LowerLeg.L",
    "TBG_Foot.L",
    "TBG_UpperLeg.R",
    "TBG_LowerLeg.R",
    "TBG_Foot.R",
)


def assemble_skinned_game_glb(path: Path, animation_names: list[str]) -> None:
    """Write a minimal skinned mesh with TBG bone names and clip name stubs."""
    bone_count = len(TBG_BONES)
    mesh_node_index = bone_count
    position_bytes = struct.pack(f"<{len(_CUBE_POSITIONS)}f", *_CUBE_POSITIONS)
    index_bytes = struct.pack(f"<{len(_CUBE_INDICES)}H", *_CUBE_INDICES)
    joints_bytes = bytes(32)  # 8 vertices × 4 joint indices (root bone)
    weights_bytes = struct.pack("<32f", *([1.0, 0.0, 0.0, 0.0] * 8))
    bind_bytes = struct.pack(f"<{bone_count * 16}f", *([1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1] * bone_count))

    pos_len = len(position_bytes)
    idx_len = len(index_bytes)
    joint_len = len(joints_bytes)
    weight_len = len(weights_bytes)
    bind_len = len(bind_bytes)
    bin_chunk = position_bytes + index_bytes + joints_bytes + weights_bytes + bind_bytes

    nodes: list[dict] = [{"name": TBG_BONES[0], "children": list(range(1, bone_count)) + [mesh_node_index]}]
    nodes.extend({"name": name} for name in TBG_BONES[1:])
    nodes.append({"name": "Body", "mesh": 0, "skin": 0})

    gltf = {
        "asset": {"version": "2.0", "generator": "TBG game mesh assembler"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": nodes,
        "skins": [
            {
                "inverseBindMatrices": 4,
                "joints": list(range(bone_count)),
                "skeleton": 0,
            }
        ],
        "meshes": [
            {
                "name": "Body",
                "primitives": [
                    {
                        "attributes": {"POSITION": 0, "JOINTS_0": 2, "WEIGHTS_0": 3},
                        "indices": 1,
                    }
                ],
            }
        ],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5126,
                "count": 8,
                "type": "VEC3",
                "min": [-0.5, -0.5, -0.5],
                "max": [0.5, 0.5, 0.5],
            },
            {"bufferView": 1, "componentType": 5123, "count": 36, "type": "SCALAR"},
            {"bufferView": 2, "componentType": 5121, "count": 32, "type": "VEC4"},
            {"bufferView": 3, "componentType": 5126, "count": 32, "type": "VEC4"},
            {"bufferView": 4, "componentType": 5126, "count": bone_count, "type": "MAT4"},
        ],
        "bufferViews": [
            {"buffer": 0, "byteOffset": 0, "byteLength": pos_len},
            {"buffer": 0, "byteOffset": pos_len, "byteLength": idx_len},
            {"buffer": 0, "byteOffset": pos_len + idx_len, "byteLength": joint_len},
            {"buffer": 0, "byteOffset": pos_len + idx_len + joint_len, "byteLength": weight_len},
            {"buffer": 0, "byteOffset": pos_len + idx_len + joint_len + weight_len, "byteLength": bind_len},
        ],
        "buffers": [{"byteLength": len(bin_chunk)}],
        "materials": [{"name": "PilotMaterial", "pbrMetallicRoughness": {"baseColorFactor": [0.72, 0.68, 0.62, 1.0]}}],
        "animations": [
            {"name": name, "channels": [], "samplers": []} for name in animation_names
        ],
    }

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(_pack_glb(gltf, bin_chunk))
