"""Contract tests for glTF export sanitization."""

from __future__ import annotations

from pathlib import Path

from assembly_studio.library.glb_inspect import load_glb
from assembly_studio.library.gltf_sanitize import (
    max_texcoord_layer,
    promote_skinned_meshes_to_root,
    sanitize_glb,
    skinned_mesh_has_parent,
    strip_extra_texcoords,
)

REPO = Path(__file__).resolve().parents[2]


def _minimal_skinned_gltf() -> dict:
    return {
        "scenes": [{"nodes": [1]}],
        "nodes": [
            {"mesh": 0, "name": "body", "skin": 0},
            {"children": [0], "name": "armature"},
        ],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0, "TEXCOORD_0": 1, "TEXCOORD_1": 2}}]}],
        "skins": [{"joints": [1]}],
    }


def test_promote_skinned_mesh_to_scene_root() -> None:
    gltf = _minimal_skinned_gltf()
    assert skinned_mesh_has_parent(gltf)
    assert promote_skinned_meshes_to_root(gltf) == 1
    assert not skinned_mesh_has_parent(gltf)
    assert 0 in gltf["scenes"][0]["nodes"]
    assert 0 not in gltf["nodes"][1].get("children", [])


def test_strip_extra_texcoords_keeps_primary_uv() -> None:
    gltf = _minimal_skinned_gltf()
    removed = strip_extra_texcoords(gltf, keep=1)
    attrs = gltf["meshes"][0]["primitives"][0]["attributes"]
    assert removed == 1
    assert "TEXCOORD_0" in attrs
    assert "TEXCOORD_1" not in attrs


def test_sanitize_live_preview_glb_if_present() -> None:
    path = REPO / "content/characters/library/exports/mblab_live_preview.glb"
    if not path.is_file():
        return
    backup = path.read_bytes()
    try:
        before, _ = load_glb(path)
        had_extra_uv = max_texcoord_layer(before) >= 1
        had_parent = skinned_mesh_has_parent(before)
        report = sanitize_glb(path)
        after, _ = load_glb(path)
        if had_extra_uv:
            assert report["texCoordsStripped"] >= 1
        if had_parent:
            assert report["skinnedPromoted"] >= 1
            assert not skinned_mesh_has_parent(after)
        assert max_texcoord_layer(after) == 0
    finally:
        path.write_bytes(backup)


def test_gltf_export_kwargs_includes_tangents() -> None:
    prep = REPO / "tools/assembly_studio/blender/gltf_export_prep.py"
    text = prep.read_text(encoding="utf-8")
    assert '"export_tangents": True' in text or "'export_tangents': True" in text
