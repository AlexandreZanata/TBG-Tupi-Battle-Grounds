"""Editable Blender components and their authoring materials."""

from dataclasses import dataclass
from math import cos, sin, tau
import bpy
import bmesh
from mathutils import Euler, Vector

# The last two numbers are metallic and roughness in Blender.
SWATCH = {
    "moss": (43, 95, 83, 0.0, 0.77), "pine": (30, 56, 54, 0.0, 0.85),
    "wine": (103, 40, 64, 0.0, 0.78), "plum": (58, 38, 62, 0.0, 0.78),
    "cloth": (53, 70, 78, 0.0, 0.90), "linen": (201, 175, 116, 0.0, 0.77),
    "leather": (82, 49, 38, 0.0, 0.70), "hide": (47, 32, 31, 0.0, 0.82),
    "wood": (121, 76, 45, 0.0, 0.68), "darkwood": (65, 41, 32, 0.0, 0.79),
    "brass": (198, 131, 57, 0.65, 0.38), "gold": (239, 182, 78, 0.63, 0.32),
    "iron": (70, 91, 100, 0.66, 0.46), "edge": (158, 188, 191, 0.67, 0.34),
    "patina": (40, 113, 112, 0.25, 0.69), "slag": (39, 52, 59, 0.45, 0.52),
    "bone": (214, 188, 134, 0.0, 0.70), "oldbone": (150, 121, 81, 0.0, 0.83),
    "skin": (192, 123, 82, 0.0, 0.65), "ink": (31, 27, 26, 0.0, 0.87),
    "soul": (104, 255, 207, 0.0, 0.53),
    "steel": (119, 131, 147, 0.63, 0.45), "steel_dark": (63, 72, 86, 0.54, 0.58),
    "violet": (85, 58, 110, 0.0, 0.82), "violet_dark": (50, 34, 65, 0.0, 0.89),
    "ritual": (199, 122, 255, 0.0, 0.53),
    "copper": (163, 82, 48, 0.58, 0.53), "forge": (255, 154, 65, 0.0, 0.48),
    "leaf": (78, 122, 57, 0.0, 0.74), "sprout": (149, 167, 81, 0.0, 0.75),
    "petal": (215, 118, 145, 0.0, 0.81), "petal_light": (241, 170, 176, 0.0, 0.81),
    "sap": (251, 205, 98, 0.0, 0.48),
    "ogre": (131, 147, 87, 0.0, 0.72), "ogre_shade": (85, 104, 63, 0.0, 0.77),
    "mushroom": (161, 98, 64, 0.0, 0.84),
    "wicker": (176, 133, 78, 0.0, 0.83), "slate": (111, 114, 112, 0.10, 0.77),
    "venom": (159, 229, 88, 0.0, 0.50),
    "ochre_cloth": (156, 104, 50, 0.0, 0.82), "field_slate": (69, 87, 102, 0.0, 0.86),
    "oilskin": (193, 144, 52, 0.0, 0.58), "oilshade": (137, 104, 47, 0.0, 0.66),
    "fish_silver": (186, 208, 205, 0.22, 0.50), "fish_back": (68, 109, 132, 0.13, 0.57),
    "finn_storm": (42, 76, 105, 0.0, 0.56), "finn_spray": (67, 119, 148, 0.0, 0.64),
    "herb_amber": (181, 112, 43, 0.05, 0.74), "brew_violet": (98, 63, 128, 0.0, 0.75),
    "ember_stone": (225, 123, 54, 0.10, 0.71),
    "furnace_verdigris": (64, 124, 111, 0.39, 0.62),
    "furnace_jade": (99, 153, 132, 0.36, 0.57),
}
GLOW_SWATCHES = {"soul", "ritual", "forge", "sap", "venom"}


def linear(byte):
    t = byte / 255
    return t / 12.92 if t < 0.04045 else ((t + 0.055) / 1.055) ** 2.4


@dataclass
class Part:
    obj: object
    joint: str
    swatch: str
    fuse: bool


def material(swatch):
    name = f"TBG · {swatch}"
    if mat := bpy.data.materials.get(name):
        return mat
    r, g, b, metal, rough = SWATCH[swatch]
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    color = (linear(r), linear(g), linear(b), 1)
    mat.diffuse_color = color
    shader = mat.node_tree.nodes.get("Principled BSDF")
    shader.inputs["Base Color"].default_value = color
    shader.inputs["Metallic"].default_value = metal
    shader.inputs["Roughness"].default_value = rough
    if swatch in GLOW_SWATCHES:
        shader.inputs["Emission Color"].default_value = color
        shader.inputs["Emission Strength"].default_value = 1.35
    return mat


class Model:
    def __init__(self, collection):
        self.collection = collection
        self.parts = []

    def mesh(self, name, joint, swatch, vertices, faces, *, smooth=True, fuse=False):
        mesh = bpy.data.meshes.new(name)
        mesh.from_pydata(vertices, [], faces)
        mesh.update()
        obj = bpy.data.objects.new(name, mesh)
        self.collection.objects.link(obj)
        mesh.materials.append(material(swatch))
        for face in mesh.polygons:
            face.use_smooth = smooth
        self.parts.append(Part(obj, joint, swatch, fuse))
        return obj

    def ell(self, name, joint, swatch, center, radii, *, fuse=False, segments=12, rings=None):
        bm = bmesh.new()
        bmesh.ops.create_uvsphere(bm, u_segments=segments,
                                  v_segments=rings or max(6, segments // 2), radius=1)
        mesh = bpy.data.meshes.new(name)
        bm.to_mesh(mesh)
        bm.free()
        for v in mesh.vertices:
            v.co = Vector(center) + Vector(tuple(x * r for x, r in zip(v.co, radii)))
        obj = bpy.data.objects.new(name, mesh)
        self.collection.objects.link(obj)
        mesh.materials.append(material(swatch))
        for f in mesh.polygons:
            f.use_smooth = True
        self.parts.append(Part(obj, joint, swatch, fuse))
        return obj

    def loft(self, name, joint, swatch, rings, *, n=12, opening=0.0, fuse=False, fold=0.0, transform=None):
        vertices, faces = [], []
        columns = n + int(opening > 0)
        for y, rx, rz, z in rings:
            for i in range(columns):
                a = opening + (tau - 2 * opening) * i / n
                f = 1 + fold * sin(a * 6 + y * 12)
                point = (rx * sin(a) * f, y, z + rz * cos(a) * f)
                vertices.append(transform(point) if transform else point)
        for row in range(len(rings) - 1):
            for col in range(n):
                a = row * columns + col
                b = row * columns + (col + 1) % columns
                faces.append((a, b, b + columns, a + columns))
        if opening == 0:
            faces.extend((tuple(reversed(range(n))), tuple(range(len(vertices) - n, len(vertices)))))
        else:
            # Cloth is thin and readable from both sides on a turntable.
            offset = len(vertices)
            vertices += vertices.copy()
            faces += [tuple(index + offset for index in reversed(face)) for face in faces]
        return self.mesh(name, joint, swatch, vertices, faces, fuse=fuse)

    def tube(self, name, joint, swatch, points, radii, *, n=7, fuse=False):
        points = [Vector(p) for p in points]
        vertices, faces = [], []
        for i, (p, r) in enumerate(zip(points, radii, strict=True)):
            axis = (points[min(i + 1, len(points) - 1)] - points[max(0, i - 1)]).normalized()
            up = Vector((1, 0, 0)) if abs(axis.x) < 0.92 else Vector((0, 0, 1))
            x = axis.cross(up).normalized()
            z = axis.cross(x)
            vertices.extend(tuple(p + (cos(tau*j/n)*x + sin(tau*j/n)*z)*r) for j in range(n))
        for i in range(len(points) - 1):
            for j in range(n):
                a = i*n+j
                b = i*n+(j+1)%n
                faces.append((a, b, b+n, a+n))
        faces.extend((tuple(reversed(range(n))), tuple(range(len(vertices)-n, len(vertices)))))
        return self.mesh(name, joint, swatch, vertices, faces, fuse=fuse)

    def plate(self, name, joint, swatch, outline, depth, *, axis="z", bevel=0.007):
        # Flat outline with supported metal edges, editable as a Blender mesh.
        def vec(u,v,w):
            return (w,u,v) if axis=="x" else (u,v,w)
        vertices = [vec(u,v,w) for w in depth for u,v in outline]
        n = len(outline)
        faces = [tuple(reversed(range(n))), tuple(range(n,2*n))]
        faces += [(i, (i+1)%n, (i+1)%n+n, i+n) for i in range(n)]
        obj = self.mesh(name, joint, swatch, vertices, faces, smooth=False)
        if bevel:
            modifier = obj.modifiers.new("Supported edges", "BEVEL")
            modifier.width = bevel
            modifier.segments = 2
        return obj

    def box(self, name, joint, swatch, center, size, bevel=0.006, rotation=0):
        x,y,z=center
        a,b,c=(v/2 for v in size)
        rot=Euler((0,0,rotation)).to_matrix()
        pts=[tuple(Vector(center)+rot@Vector((u,v,w))) for w in (-c,c)
             for u,v in ((-a,-b),(a,-b),(a,b),(-a,b))]
        obj=self.mesh(name,joint,swatch,pts,[(3,2,1,0),(4,5,6,7),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7)],smooth=False)
        if bevel:
            m=obj.modifiers.new("Soft bevel", "BEVEL")
            m.width=bevel
            m.segments=2
        return obj
