"""Encode evaluated Blender meshes for the shared Bevy renderer."""

from mathutils import Matrix
from io import BytesIO
import struct
import zlib

# Blender Z up, Bevy Y up. The rotation preserves triangle winding.
BASIS = Matrix.Rotation(-1.5707963267948966, 4, "X")
INVERSE = BASIS.inverted()
IDENTITY = Matrix.Identity(4)


def srgb(value):
    value = max(0, min(1, value))
    return round(255 * (12.92 * value if value <= .0031308 else 1.055 * value ** (1 / 2.4) - .055))


def pose(matrix):
    point, rotation, scale = (BASIS @ matrix @ INVERSE).decompose()
    return (*point, rotation.x, rotation.y, rotation.z, rotation.w, *scale)


class Batch:
    def __init__(self, transforms=(pose(IDENTITY),), *, units=4096):
        self.transforms = transforms
        self.units = units
        self.socket = (0.0, 0.0, 0.0)
        self.vertices = {}
        self.indices = []

    def add(self, obj, matrix, *, keep=lambda _: True):
        mesh = obj.data
        mesh.calc_loop_triangles()
        matrix = BASIS @ matrix
        normal_matrix = matrix.to_3x3().inverted().transposed()
        normals = mesh.corner_normals
        colors = mesh.color_attributes.active_color
        palette = [tuple(srgb(c) for c in mat.diffuse_color[:3]) for mat in mesh.materials]
        palette = palette or [(255, 255, 255)]
        mirrored = matrix.determinant() < 0
        for triangle in mesh.loop_triangles:
            if not keep(triangle):
                continue
            loops = triangle.loops
            if mirrored:
                loops = (loops[0], loops[2], loops[1])
            for corner in loops:
                index = mesh.loops[corner].vertex_index
                pos = matrix @ mesh.vertices[index].co
                normal = (normal_matrix @ normals[corner].vector).normalized()
                tint = palette[triangle.material_index]
                if colors and colors.domain in {"POINT", "CORNER"}:
                    tint = tuple(srgb(c) for c in colors.data[index if colors.domain == "POINT" else corner].color[:3])
                position = [round(self.units * v) for v in pos]
                if not all(-32768 <= v <= 32767 for v in position):
                    raise ValueError(f"{obj.name}: vertex {tuple(pos)} outside the pack range")
                vertex = struct.pack("<3h3b3B", *position, *(round(127 * n) for n in normal), *tint)
                self.indices.append(self.vertices.setdefault(vertex, len(self.vertices)))

    def write(self, stream, name):
        label = name.encode("ascii")
        stream.write(struct.pack("<B", len(label)) + label)
        poses = [struct.pack("<10f", *(round(v, 5) for v in frame)) for frame in self.transforms]
        ticks = [
            i for i, frame in enumerate(poses)
            if i == 0 or frame != poses[i - 1] or (i + 1 < len(poses) and frame != poses[i + 1])
        ]
        stream.write(struct.pack("<HII3f", len(ticks), len(self.vertices), len(self.indices), *self.socket))
        for tick in ticks:
            stream.write(struct.pack("<H", tick) + poses[tick])
        stream.writelines(self.vertices)
        index_type = "H" if len(self.vertices) <= 65535 else "I"
        stream.write(struct.pack(f"<{len(self.indices)}{index_type}", *self.indices))



def write(path, batches, origin=(0, 0, 0)):
    batches = {name: batch for name, batch in batches.items() if batch.indices}
    if not batches:
        raise ValueError(f"No geometry exported to {path}")
    stream = BytesIO()
    stream.write(b"TBGD" + struct.pack("<HH3f", len(batches), next(iter(batches.values())).units, *origin))
    for name, batch in batches.items():
        batch.write(stream, name)
    path.write_bytes(zlib.compress(stream.getvalue(), level=9))
    print(path.name, sum(len(batch.indices) // 3 for batch in batches.values()), path.stat().st_size, flush=True)
