"""Bake editable Blender collections into small vertex-colored meshes for Bevy."""

import struct

import bpy
from mathutils import Matrix

VERTEX = struct.Struct("<hhhBBB")


def srgb_byte(linear):
    linear = max(0.0, min(1.0, linear))
    value = linear * 12.92 if linear <= 0.0031308 else 1.055 * linear**(1 / 2.4) - 0.055
    return round(value * 255)


def bake(collection, depsgraph, transform=Matrix.Identity(4), include=lambda name: True, white=False):
    data = bytearray()
    for obj in collection.objects:
        if not include(obj.name):
            continue
        placement = transform @ obj.matrix_world
        if obj.instance_type == "COLLECTION":
            nested = obj.instance_collection
            data.extend(bake(nested, depsgraph, placement @ Matrix.Translation(-nested.instance_offset)))
        elif obj.type == "MESH":
            evaluated = obj.evaluated_get(depsgraph)
            mesh = evaluated.to_mesh()
            mesh.calc_loop_triangles()
            colors = [tuple(map(srgb_byte, mat.diffuse_color[:3])) for mat in mesh.materials]
            for tri in mesh.loop_triangles:
                tint = (255, 255, 255) if white else colors[tri.material_index]
                faces = [tri.vertices]
                if obj.name.startswith("Tower pennant"):
                    faces.append(tuple(reversed(tri.vertices)))
                for face in faces:
                    for index in face:
                        x, y, z = placement @ mesh.vertices[index].co
                        data.extend(VERTEX.pack(round(x * 256), round(z * 256), round(-y * 256), *tint))
            evaluated.to_mesh_clear()
    for child in collection.children:
        data.extend(bake(child, depsgraph, transform, include, white))
    return data


def tower_parts(obj_name, damage):
    if " crown merlon " not in obj_name:
        return True
    index = int(obj_name.rsplit(" ", 1)[-1])
    return damage == 0 or (index % 3 != 0 and (damage == 1 or index % 2 != 0))


def export(output, assets):
    output.mkdir(parents=True, exist_ok=True)
    graph = bpy.context.evaluated_depsgraph_get()
    metrics = {}

    def save(slug, name, **options):
        data = bake(assets[name], graph, **options)
        assert data and len(data) % (3 * VERTEX.size) == 0, name
        (output / f"{slug}.tbgpart").write_bytes(data)
        metrics[slug] = len(data) // (3 * VERTEX.size)

    save("bridge", "Tupi bridge deck")
    save("plaza", "Tupi plaza island")
    save("plaza-crystal", "Tupi plaza crystal", white=True)
    save("nexus-plinth", "Tupi Nexus plinth")
    save("tower-crystal", "Tupi tower crystal Blue", white=True,
         transform=Matrix.Translation((0, 0, -7.6)))
    for kind in ("Wolves", "Golems", "Jaguar", "Frogs"):
        save(f"camp-{kind.lower()}", f"Tupi camp {kind}")
    for team in ("Blue", "Red"):
        for damage in range(3):
            save(f"tower-{team.lower()}-{damage}", f"Tupi tower {team}",
                 include=lambda name: tower_parts(name, damage))
    return metrics
