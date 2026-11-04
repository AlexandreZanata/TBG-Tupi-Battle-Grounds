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
EXTENDED = ("StringTop", "StringBottom", "Nock")
EXTENDED_KITS = {"ravager", "acidmaw", "crowcaller", "rattletick"}
INDEX = {name: i for i, name in enumerate(NAMES + EXTENDED)}
PARENT_OVERRIDES = {
    "ravager": {
        "WeaponR": "Hips", "OffhandL": "Hips", "Back": "WeaponR",
        "StringBottom": "Back", "Nock": "StringTop",
    },
    "acidmaw": dict.fromkeys(("WeaponR", "OffhandL", "Back", "StringTop"), "Head"),
    "whisperer": {
        **dict.fromkeys(("ThighL", "ThighR", "WeaponR", "OffhandL"), "Chest"),
        "Back": "Head",
    },
    "crowcaller": {"HeadGear": "BackSpring", "Nock": "WeaponR"},
    "trigger": {"BackSpring": "Hips"},
    "rattletick": {
        "WeaponR": "Chest", "Tabard": "Chest",
        **dict.fromkeys(("HeadGear", "Back", "StringTop", "Nock"), "BackSpring"),
    },
}
REST_OVERRIDES = {
    "bonehorn": {
        "Hips": (0,.70,-.34), "Spine": (0,.02,.18), "Chest": (0,.02,.38),
        "Head": (0,.02,.47), "BackSpring": (0,0,0),
    },
    "ravager": {
        "Hips": (0,.51,-.15), "Spine": (0,0,.08), "Chest": (0,0,.27),
        "Head": (0,-.06,.30), "WeaponR": (.20,0,-.27), "OffhandL": (-.20,0,-.27),
        "Back": (.17,-.20,-.08), "StringTop": (-.17,-.20,-.08),
        "StringBottom": (.09,-.28,-.06), "Nock": (-.09,-.28,-.06),
    },
    "acidmaw": {"Head": (0,.70,.08), "BackSpring": (0,.42,.09)},
    "whisperer": {
        "Hips": (0,.92,0), "Head": (0,.18,.23),
        "WeaponR": (.29,.04,0), "OffhandL": (-.29,.04,0),
    },
    "crowcaller": {
        "BackSpring": (.22,.35,-.06), "HeadGear": (0,.075,.055),
        "Tabard": (0,.30,-.15), "Nock": (0,.035,0),
    },
    "trigger": {"BackSpring": (0,-.02,-.14), "Back": (0,.04,-.30)},
    "rattletick": {
        "BackSpring": (.28,.42,-.05), "HeadGear": (0,.22,0), "Tabard": (0,.20,-.22),
        "Back": (.09,.12,0), "StringTop": (-.09,.12,0), "Nock": (-.09,.035,.18),
    },
}
for side, sign in (("L",1), ("R",-1)):
    for unit, offsets in (
        ("whisperer", ((sign*.16,.01,.15),(0,-.17,0),(0,-.18,0),
                       (sign*.16,.01,-.15),(0,-.17,0),(0,-.18,0))),
        ("bonehorn", ((sign*.29,-.02,.03),(0,-.32,0),(0,-.32,0),
                      (sign*.25,-.02,-.06),(0,-.30,0),(0,-.32,0))),
        ("ravager", ((sign*.24,0,.15),(sign*.18,-.20,.08),(sign*.08,-.27,.09),
                     (sign*.21,0,-.02),(sign*.17,-.20,0),(sign*.09,-.28,.02))),
    ):
        REST_OVERRIDES[unit].update({
            f"{joint}{side}": offset for joint, offset in
            zip(("UpperArm","Forearm","Hand","Thigh","Shin","Foot"), offsets, strict=True)
        })
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
    "bloodweaver": (0.38, 0.36, 0.34, 0.24, 0.1365, 0.17),
    "oilpress": (0.38, 0.46 * 1.15, 0.34 * 1.05, 0.24 * 1.35, 0.1365 * 0.8, 0.17 * 1.15),
    "drumguard": (0.38, 0.36, 0.34, 0.24, 0.1365, 0.21),
    "leechroot": (0.38, 0.36 * 1.22, 0.34, 0.24 * 1.15, 0.1365, 0.17 * 1.3),
    "cargocrab": (.28, .70, .22, .56, .09, .24),
    "puppetmaster": (.38 * 1.12, .36 * .9, .34, .24, .1365, .17 * 1.15),
    "smuggler": (.38, .36, .34, .24, .1365, .20),
    "bonehorn": (.64, .46, .34, .24, .1365, .34),
    "ravager": (.45, .36, .34, .24, .1365, .17),
    "acidmaw": (.48, .25, .34, .24, .1365, .12),
    "whisperer": (.38, .36, .34, .24, .1365, .17),
    "bulwark": (.30, .46, .29, .24, .1365, .19),
    "demolisher": (.32, .46, .34, .24, .1365, .25),
    "crowcaller": (.38, .36 * .82, .34, .24, .1365, .21),
    "trigger": (.38, .36 * .85, .34, .24, .1365, .18),
    "rattletick": (.38, .46 * 1.10, .34, .24, .1365 * .90, .17),
}
# +Y up and +Z forward in TBG, +Z up and -Y forward in Blender.
TO_BLENDER = Matrix(((1, 0, 0, 0), (0, 0, -1, 0), (0, 1, 0, 0), (0, 0, 0, 1)))


def parents_for(unit):
    extended = unit in EXTENDED_KITS
    names = NAMES + EXTENDED if extended else NAMES
    parents = PARENT + (21, 21, 21) if extended else PARENT
    overrides = PARENT_OVERRIDES.get(unit, {})
    return {
        name: overrides.get(name, NAMES[parent] if parent is not None else None)
        for name, parent in zip(names, parents, strict=True)
    }


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
    offsets = dict(zip(NAMES, local, strict=True))
    offsets.update(dict.fromkeys(EXTENDED, (0,0,0)))
    offsets.update(REST_OVERRIDES.get(unit, {}))
    world = {}
    for name, parent in parents_for(unit).items():
        offset = Vector(offsets[name])
        world[name] = offset if parent is None else world[parent] + offset
    return world


def armature(unit, rest):
    data = bpy.data.armatures.new("TBG v2 · 17 body joints, springs and sockets")
    obj = bpy.data.objects.new(f"TBG v2 · {unit}", data)
    bpy.context.scene.collection.objects.link(obj)
    bpy.context.view_layer.objects.active = obj
    obj.show_in_front = True
    obj.select_set(True)
    bpy.ops.object.mode_set(mode="EDIT")
    for name, parent in parents_for(unit).items():
        bone = data.edit_bones.new(name)
        bone.head = TO_BLENDER @ rest[name]
        bone.tail = bone.head + Vector((0, 0, 0.066 if INDEX[name] < 17 else 0.041))
        if parent is not None:
            bone.parent = data.edit_bones[parent]
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
