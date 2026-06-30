"""Bake tangent normals from high-poly source onto decimated game mesh (Blender bpy)."""

from __future__ import annotations

try:
    import bpy
except ImportError as exc:
    raise SystemExit("bake_game_maps requires Blender bpy") from exc

BAKE_SAMPLES = 16
RAY_DISTANCE = 0.02


def rebake_normal_map(high_obj: bpy.types.Object, low_obj: bpy.types.Object, texture_size: int) -> None:
    """Project high-poly surface detail onto low-poly UVs (mobile game standard)."""
    _ensure_uvs(low_obj)
    image = _normal_image_node(low_obj, texture_size)
    _configure_cycles_bake()
    _select_bake_pair(high_obj, low_obj)
    bpy.ops.object.bake(
        type="NORMAL",
        use_selected_to_active=True,
        normal_space="TANGENT",
        cage_extrusion=RAY_DISTANCE,
        margin=4,
    )
    image.update()


def _ensure_uvs(mesh_obj: bpy.types.Object) -> None:
    if mesh_obj.data.uv_layers:
        return
    bpy.ops.object.select_all(action="DESELECT")
    mesh_obj.select_set(True)
    bpy.context.view_layer.objects.active = mesh_obj
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=66, island_margin=0.02)
    bpy.ops.object.mode_set(mode="OBJECT")


def _normal_image_node(mesh_obj: bpy.types.Object, size: int) -> bpy.types.Image:
    mat = mesh_obj.data.materials[0] if mesh_obj.data.materials else None
    if mat is None:
        mat = bpy.data.materials.new(name="TBG_Bake")
        mesh_obj.data.materials.append(mat)
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    links = mat.node_tree.links
    for node in list(nodes):
        if node.type == "TEX_IMAGE" and node.image and node.image.name.startswith("TBG_NormalBake"):
            nodes.remove(node)
    tex = nodes.new("ShaderNodeTexImage")
    tex.image = bpy.data.images.new("TBG_NormalBake", size, size)
    tex.image.colorspace_settings.name = "Non-Color"
    principled = next((n for n in nodes if n.type == "BSDF_PRINCIPLED"), None)
    if principled:
        normal_in = principled.inputs.get("Normal")
        if normal_in and not normal_in.is_linked:
            normal_map = nodes.new("ShaderNodeNormalMap")
            links.new(tex.outputs["Color"], normal_map.inputs["Color"])
            links.new(normal_map.outputs["Normal"], normal_in)
    nodes.active = tex
    return tex.image


def _configure_cycles_bake() -> None:
    bpy.context.scene.render.engine = "CYCLES"
    bpy.context.scene.cycles.device = "CPU"
    bpy.context.scene.cycles.samples = BAKE_SAMPLES


def _select_bake_pair(high_obj: bpy.types.Object, low_obj: bpy.types.Object) -> None:
    bpy.ops.object.select_all(action="DESELECT")
    high_obj.select_set(True)
    low_obj.select_set(True)
    bpy.context.view_layer.objects.active = low_obj
