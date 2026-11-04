"""Mesh primitives and the shared miniature palette."""

from math import cos, sin, tau
import bpy
from mathutils import Vector

PALETTE = {
    "sand": "b79b72", "sand-dark": "9c815f", "sand-light": "c5ab83",
    "path-bed": "938064", "curb": "807c68", "curb-light": "a39a7d",
    "turf": "6a854a", "turf-light": "799654", "turf-shade": "526d41", "moss": "8a9957",
    "cliff": "656e68", "cliff-shadow": "435954", "cliff-light": "909387",
    "stratum": "788377", "stone": "8d9183", "stone-cool": "717f79",
    "stone-light": "aeb19e", "mortar": "555f57", "iron": "444d4d",
    "oak": "66513c", "wood-end": "a08156", "wood": "8a6747", "wood-dark": "4e4235",
    "fir-shadow": "274b3b", "fir": "42654a", "fir-light": "608053",
    "leaf-deep": "426b48", "leaf": "63894e", "leaf-light": "7e9e59",
    "grass": "627c43", "grass-light": "90a85c", "petal": "d9c790", "poppy": "b77663",
    "water": "226372", "water-dark": "1a5465", "water-light": "2a7080",
    "shelf-deep": "2e7f89", "shelf": "44969a", "foam": "9fc8ba", "ripple": "64a5a4",
    "blue": "426e96", "red": "984e43", "linen": "d6c29a", "trim": "c3a266",
    "ember": "ab5536", "fire": "e88438", "glow": "ffd987",
    "distant": "536f70", "distant-light": "6d8280",
    "tile": "8d4934", "tile-light": "ad6348", "tile-dark": "673b34",
    "gunmetal": "4c5b60", "silver": "9aa9a2", "soot": "272f30",
    "bronze": "a47a47", "gold": "d4a954", "patina": "4e7671",
    "skin": "a97755", "hood": "5b6e4b",
}


def color(hex_value):
    srgb = [int(hex_value[i:i+2],16)/255 for i in (0,2,4)]
    return tuple(v/12.92 if v < .04045 else ((v+.055)/1.055)**2.4 for v in srgb)+(1,)


def material(key):
    existing = bpy.data.materials.get(key)
    if existing:
        return existing
    mat = bpy.data.materials.new(key)
    mat.diffuse_color = color(PALETTE[key])
    mat.use_nodes = True
    shader = mat.node_tree.nodes.get("Principled BSDF")
    shader.inputs["Base Color"].default_value = mat.diffuse_color
    shader.inputs["Roughness"].default_value = .31 if key.startswith(("water","shelf")) else .85
    if key in {"gunmetal","silver","bronze","gold","patina"}:
        shader.inputs["Metallic"].default_value = .44
        shader.inputs["Roughness"].default_value = .45
    if key in {"fire","glow"}:
        shader.inputs["Emission Color"].default_value = mat.diffuse_color
        shader.inputs["Emission Strength"].default_value = 1.3
    return mat


def surface(group, name, vertices, faces, colors, *, smooth=False):
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(vertices, [], faces)
    mesh.update()
    keys = list(dict.fromkeys(colors))
    for key in keys:
        mesh.materials.append(material(key))
    slots = {key: i for i,key in enumerate(keys)}
    for polygon, key in zip(mesh.polygons, colors, strict=True):
        polygon.material_index = slots[key]
        polygon.use_smooth = smooth
    obj = bpy.data.objects.new(name,mesh)
    group.objects.link(obj)
    return obj


def box(group, name, center, size, key, bevel=0, angle=0):
    x,y,z = center
    a,b,c = (v/2 for v in size)
    outline = [(-a,-b),(a,-b),(a,b),(-a,b)]
    vertices = [(x+u*cos(angle)-v*sin(angle),y+u*sin(angle)+v*cos(angle),z+w)
                for w in (-c,c) for u,v in outline]
    faces = [(3,2,1,0),(4,5,6,7),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7)]
    obj = surface(group,name,vertices,faces,[key]*6)
    if bevel:
        modifier = obj.modifiers.new("Worn edges","BEVEL")
        modifier.width = bevel
        modifier.segments = 1
    return obj


def tube(group, name, points, radii, key, sides=8, accent=None):
    points = [Vector(point) for point in points]
    vertices = []
    for i,(point,radius) in enumerate(zip(points,radii,strict=True)):
        axis = (points[min(i+1,len(points)-1)]-points[max(i-1,0)]).normalized()
        ref = Vector((1,0,0)) if abs(axis.x) < .9 else Vector((0,1,0))
        a = axis.cross(ref).normalized()
        b = axis.cross(a)
        vertices.extend(point+radius*(cos(tau*j/sides)*a+sin(tau*j/sides)*b) for j in range(sides))
    faces = [(i*sides+j,i*sides+(j+1)%sides,(i+1)*sides+(j+1)%sides,(i+1)*sides+j)
             for i in range(len(points)-1) for j in range(sides)]
    colors = [accent if accent and i%sides in (1,2) else key for i in range(len(faces))]
    faces += [tuple(reversed(range(sides))), tuple(range(len(vertices)-sides,len(vertices)))]
    return surface(group,name,vertices,faces,colors+[key,key])


def pebble(group, name, center, size, key, sides=7, accent=None, phase=0):
    x,y,z = center
    rx,ry,rz = size
    outline = [tau*i/sides+phase for i in range(sides)]
    vertices = [(x+rx*r*cos(a)*(1+.12*sin(a*5)),y+ry*r*sin(a),
                 z+rz*(h+.12*cos(a*3))) for r,h in ((.67,-.60),(1,-.13),(.64,.58)) for a in outline]
    faces = [(row*sides+i,row*sides+(i+1)%sides,(row+1)*sides+(i+1)%sides,(row+1)*sides+i)
             for row in (0,1) for i in range(sides)]
    faces += [tuple(reversed(range(sides))),tuple(range(2*sides,3*sides))]
    colors = [accent if accent and (i%5 in (2,3) or i==len(faces)-1) else key for i in range(len(faces))]
    return surface(group,name,vertices,faces,colors)


def prototype(name):
    collection = bpy.data.collections.new(name)
    collection.asset_mark()
    return collection


def instance(group, asset, name, at, scale=1, angle=0):
    obj = bpy.data.objects.new(name,None)
    obj.instance_type = "COLLECTION"
    obj.instance_collection = asset
    group.objects.link(obj)
    obj.location = at
    obj.scale = (scale,)*3 if isinstance(scale,(int,float)) else scale
    obj.rotation_euler.z = angle
    return obj
