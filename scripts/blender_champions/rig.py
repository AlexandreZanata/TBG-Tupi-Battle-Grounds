"""Mirror the named TBG rig v2 rest transforms for the Blender study."""

from mathutils import Matrix, Vector
import bpy

NAMES = (
    "Root", "Hips", "Spine", "Chest", "Head",
    "UpperArmL", "ForearmL", "HandL", "UpperArmR", "ForearmR", "HandR",
    "ThighL", "ShinL", "FootL", "ThighR", "ShinR", "FootR",
    "BackSpring", "HeadGear", "Tabard", "WeaponR", "OffhandL", "Back",
)
PARENT = (None, 0, 1, 2, 3, 3, 5, 6, 3, 8, 9, 1, 11, 12, 1, 14, 15, 3, 4, 1, 10, 7, 17)
INDEX = {name: i for i, name in enumerate(NAMES)}
BODY_PLANS = {
    "archer": (0.38, 0.36, 0.34, 0.24, 0.1365, 0.20),
    "bonebrute": (0.36, 0.50, 0.38, 0.28, 0.14, 0.25),
    "shieldbearer": (0.38, 0.36 * 1.08, 0.34, 0.24, 0.1365, 0.17),
    "hexer": (0.38, 0.36, 0.34, 0.24, 0.1365, 0.17),
    "sawbot": (0.38, 0.36 * 0.95, 0.34, 0.24, 0.1365, 0.17 * 1.1),
    "bloomkeeper": (0.38, 0.36, 0.34, 0.24, 0.1365, 0.17),
    "siegewalker": (0.34, 0.48, 0.36, 0.30, 0.13, 0.25),
    "rootcaller": (0.30, 0.36, 0.36, 0.26, 0.15, 0.21),
    "nester": (0.38 * 0.85, 0.36 * 1.25, 0.34, 0.24 * 1.3, 0.1365, 0.20),
    "rampart": (0.38, 0.46 * 1.1, 0.34, 0.24, 0.1365, 0.17),
    "longaxe": (0.38, 0.36, 0.34, 0.24, 0.1365, 0.26),
    "farwatch": (0.38, 0.36, 0.34, 0.24, 0.1365, 0.23),
    "bombard": (0.38 * 0.86, 0.36 * 1.2, 0.34, 0.24 * 1.2, 0.1365, 0.35),
    "rotcap": (0.38 * 0.8, 0.36 * 1.1, 0.34, 0.24, 0.1365, 0.17),
    "oldfinn": (0.36, 0.40, 0.34, 0.27, 0.1365, 0.21),
    "antidotebrewer": (0.38, 0.36, 0.34, 0.24, 0.1365, 0.17),
    "shatterback": (0.38 * 0.6, 0.46 * 1.35, 0.16, 0.24 * 1.9, 0.1365 * 0.85, 0.26),
    "bloodfurnace": (0.38, 0.36 * 1.1, 0.34, 0.24, 0.1365, 0.25),
}
# +Y up and +Z forward in TBG, +Z up and -Y forward in Blender.
TO_BLENDER = Matrix(((1, 0, 0, 0), (0, 0, -1, 0), (0, 1, 0, 0), (0, 0, 0, 1)))


def rest_positions(unit, *, production=False, tier=0, branch=0):
    leg, width, height, depth, head, arm = BODY_PLANS[unit]
    if production and unit=="bonebrute" and tier==2 and branch==1:
        arm=.32
    a = width / 2 + 0.05
    l = -leg * 0.5
    local = (
        (0, 0, 0), (0, leg + 0.06, 0), (0, 0.02, 0), (0, 0, 0),
        (0, .20, .33) if unit == "shatterback" else (0, height + head * 0.85, 0),
        (a, height * 0.9, 0), (0, -arm, 0), (0, -arm * 0.9, 0),
        (-a, height * 0.9, 0), (0, -arm, 0), (0, -arm * 0.9, 0),
        (width * 0.25, 0, 0), (0, l, 0), (0, l, 0),
        (-width * 0.25, 0, 0), (0, l, 0), (0, l, 0),
        (0, height * 0.65, -depth * 0.65), (0, 0, 0), (0, 0, depth * 0.55),
        (0, 0, 0), (0, 0, 0), (0, 0, 0),
    )
    world = []
    for parent, offset in zip(PARENT, local, strict=True):
        world.append(Vector(offset) if parent is None else world[parent] + Vector(offset))
    return dict(zip(NAMES, world, strict=True))


def armature(unit, rest):
    data = bpy.data.armatures.new("TBG v2 · 17 body joints, springs and sockets")
    obj = bpy.data.objects.new(f"TBG v2 · {unit}", data)
    bpy.context.scene.collection.objects.link(obj)
    bpy.context.view_layer.objects.active = obj
    obj.show_in_front = True
    obj.select_set(True)
    bpy.ops.object.mode_set(mode="EDIT")
    for name, parent in zip(NAMES, PARENT, strict=True):
        bone = data.edit_bones.new(name)
        bone.head = TO_BLENDER @ rest[name]
        bone.tail = bone.head + Vector((0, 0, 0.066 if INDEX[name] < 17 else 0.041))
        if parent is not None:
            bone.parent = data.edit_bones[NAMES[parent]]
    bpy.ops.object.mode_set(mode="OBJECT")
    obj.select_set(False)
    obj["Rig source"] = "TBG art/rig.rs and champion body plan"
    return obj


def bind_piece(obj, arm, joint, rest):
    obj.data.transform(TO_BLENDER)
    world = TO_BLENDER @ rest[joint]
    obj.location = world
    bpy.context.view_layer.update()
    matrix = obj.matrix_world.copy()
    obj.parent = arm
    obj.parent_type = "BONE"
    obj.parent_bone = joint
    obj.matrix_world = matrix
    obj["TBG joint"] = joint
