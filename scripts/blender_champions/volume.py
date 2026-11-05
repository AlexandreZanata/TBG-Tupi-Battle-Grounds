"""Voxel reconstruction and automatic two-joint bind weights."""

from mathutils import Vector, kdtree
import bpy

from geometry import SWATCH, linear
from rig import INDEX, NAMES, PARENT, TO_BLENDER


def generate(model, rest):
    source=[p for p in model.parts if p.fuse]
    vertices, faces, tags = [], [], []
    for p in source:
        offset=len(vertices)
        vertices.extend(v.co+rest[p.joint] for v in p.obj.data.vertices)
        tags.extend([(INDEX[p.joint],p.swatch)]*len(p.obj.data.vertices))
        faces.extend(tuple(offset+i for i in f.vertices) for f in p.obj.data.polygons)
    tree=kdtree.KDTree(len(vertices))
    for i,v in enumerate(vertices):
        tree.insert(v,i)
    tree.balance()

    mesh=bpy.data.meshes.new("Voxel reconstruction")
    mesh.from_pydata(vertices,[],faces)
    mesh.update()
    obj=bpy.data.objects.new("Generated body · fused volumes",mesh)
    model.collection.objects.link(obj)
    bpy.context.view_layer.objects.active=obj
    obj.select_set(True)
    mesh.remesh_voxel_size=.0165
    bpy.ops.object.voxel_remesh()
    bpy.ops.object.shade_smooth()
    smoother=obj.modifiers.new("Relax voxel contours","SMOOTH")
    smoother.factor=.60
    smoother.iterations=2
    bpy.ops.object.modifier_apply(modifier=smoother.name)
    obj.data.calc_loop_triangles()
    if len(obj.data.loop_triangles)>4300:
        reducer=obj.modifiers.new("Compact render mesh","DECIMATE")
        reducer.ratio=4300/len(obj.data.loop_triangles)
        bpy.ops.object.modifier_apply(modifier=reducer.name)
    obj.select_set(False)
    weights=[]
    colors=[]
    for vertex in obj.data.vertices:
        candidates=tree.find_n(vertex.co,24)
        _,i,d0=candidates[0]
        main, swatch=tags[i]
        alternate=main
        other_weight=0
        for _,j,distance in candidates[1:]:
            other,_=tags[j]
            if other!=main and (PARENT[main]==other or PARENT[other]==main or {other,main}=={1,3}):
                if distance-d0 < .025:
                    alternate=other
                    other_weight=.48*(1-max(0,distance-d0)/.025)
                break
        weights.append((main,alternate,1-other_weight))
        colors.append(tuple(SWATCH[swatch][:3]))
    for p in source:
        bpy.data.objects.remove(p.obj,do_unlink=True)
    model.parts=[p for p in model.parts if not p.fuse]
    return obj,weights,colors


def bind(obj, arm, weights, colors):
    groups=[obj.vertex_groups.new(name=name) for name in NAMES[:17]]
    for i,(main,other,weight) in enumerate(weights):
        groups[main].add([i],weight,"REPLACE")
        if other!=main:
            groups[other].add([i],1-weight,"REPLACE")
    attr=obj.data.color_attributes.new(name="BakedColor",type="FLOAT_COLOR",domain="POINT")
    for item,rgb in zip(attr.data,colors,strict=True):
        item.color=(*[linear(v) for v in rgb],1)
    obj.data.color_attributes.active_color=attr
    mat=bpy.data.materials.new("Generated body · vertex color")
    mat.use_nodes=True
    node=mat.node_tree.nodes.new("ShaderNodeVertexColor")
    node.layer_name=attr.name
    mat.node_tree.links.new(node.outputs["Color"],mat.node_tree.nodes["Principled BSDF"].inputs["Base Color"])
    obj.data.materials.append(mat)
    obj.data.transform(TO_BLENDER)
    obj.parent=arm
    deform=obj.modifiers.new("TBG rig v2","ARMATURE")
    deform.object=arm
    obj["Method"]="Voxel union, relaxation, decimation and automatic 2-joint weights"
