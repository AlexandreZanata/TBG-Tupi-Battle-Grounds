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
INSECT_KITS = ("leafcutter", "blastbeetle", "needlewing", "nectarnurse", "stridulator")
EXTENDED_KITS = {
    "ravager", "acidmaw", "crowcaller", "rattletick", "solarsentinel", "celestialarcher", "hornet",
    *INSECT_KITS,
}
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
    "jawguard": {"WeaponR": "Head", "OffhandL": "Head"},
    "mantis": {"BackSpring": "Hips"},
    "spinehurler": {"HeadGear": "WeaponR"},
    "hornet": {
        "BackSpring": "Hips", "HeadGear": "Chest", "Tabard": "Chest",
        **dict.fromkeys(("OffhandL", "Nock", "StringTop", "StringBottom"), "Root"),
    },
    "weaver": {
        **dict.fromkeys(("UpperArmL", "ForearmL", "ThighL", "ShinL",
                         "UpperArmR", "ForearmR", "ThighR", "ShinR", "Chest"), "Hips"),
        "WeaponR": "Head", **dict.fromkeys(("BackSpring","HeadGear","Tabard"), "Chest"),
    },
    "drillnest": {
        **dict.fromkeys(("Chest","Head","BackSpring","HeadGear","Tabard"), "Hips"),
        "WeaponR": "Chest",
    },
    "capybara": {"Tabard": "Chest"},
    "trident": {"WeaponR": "Chest"},
    "gustdrummer": {"WeaponR": "Chest"},
    "luminousoracle": {"HeadGear": "WeaponR"},
    "celestialarcher": dict.fromkeys(("BackSpring", "Tabard", "Back"), "Chest"),
    "rattletick": {
        "WeaponR": "Chest", "Tabard": "Chest",
        **dict.fromkeys(("HeadGear", "Back", "StringTop", "Nock"), "BackSpring"),
    },
}
for insect in INSECT_KITS:
    PARENT_OVERRIDES[insect] = {
        "BackSpring": "Hips", "Tabard": "Hips", "HeadGear": "Chest",
        "StringTop": "BackSpring", "StringBottom": "Tabard",
        "Back": "StringTop", "Nock": "StringBottom",
    }

PARENT_OVERRIDES["needlewing"].update(WeaponR="Chest", HeadGear="HandR")

REST_OVERRIDES = {
    "spinehurler": {"HeadGear": (0,0,0)},
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
    "jawguard": {
        "Head": (0,.30,.16), "WeaponR": (-.13,-.08,.16), "OffhandL": (.13,-.08,.16),
        "BackSpring": (0,.23,-.08),
    },
    "mantis": {"BackSpring": (0,.02,-.12)},
    "hornet": {"BackSpring": (0,0,-.14), "HeadGear": (.10,.26,-.09), "Tabard": (-.10,.26,-.09)},
    "weaver": {
        "Hips": (0,.31,0), "Chest": (0,.06,-.18), "Head": (0,-.10,.43), "WeaponR": (0,0,.11),
        **dict.fromkeys(("BackSpring","HeadGear","Tabard"), (0,0,0)),
    },
    "drillnest": {
        "Hips": (0,.24,0), "Chest": (0,.15,0), "Head": (0,.29,.05), "WeaponR": (0,.05,.18),
        **dict.fromkeys(("BackSpring","HeadGear","Tabard"), (0,0,0)),
    },
    "hushfang": {
        "Hips": (0,.385,-.24), "Spine": (0,0,.12), "Chest": (0,.015,.27),
        "Head": (0,.07,.24), "BackSpring": (0,.02,-.10),
    },
    "capybara": {"Head": (0,.38,.17), "Tabard": (0,.13,.14)},
    "gustdrummer": {"WeaponR": (0,.12,.29)},
    "trident": {"HeadGear": (0,-.075,.13), "BackSpring": (0,.27,-.18)},
    "solarsentinel": {
        "HeadGear": (0,.15,-.06), "StringTop": (-.15,0,0), "StringBottom": (.15,0,0),
    },
    "luminousoracle": {"HeadGear": (0,.55,0)},
    "celestialarcher": {
        "BackSpring": (.16,.28,-.13), "Tabard": (-.16,.28,-.13),
        "HeadGear": (0,.18,-.05), "Back": (0,.12,-.15),
    },
    "justiciar": {"BackSpring": (0,.19,.19), "HeadGear": (0,-.02,0)},
    "rattletick": {
        "BackSpring": (.28,.42,-.05), "HeadGear": (0,.22,0), "Tabard": (0,.20,-.22),
        "Back": (.09,.12,0), "StringTop": (-.09,.12,0), "Nock": (-.09,.035,.18),
    },
}
for side, sign in (("L",1), ("R",-1)):
    REST_OVERRIDES["weaver"].update({
        f"{joint}{side}": (sign*.14,0,z) for joint,z in
        (("UpperArm",.20),("Forearm",.06),("Thigh",-.08),("Shin",-.22))
    })
    for unit, offsets in (
        ("hushfang", ((sign*.14,-.01,.03),(0,-.18,0),(0,-.18,0),
                      (sign*.14,.005,-.04),(0,-.18,0),(0,-.18,0))),
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
    "capybara": (.25, .48, .32, .24, .1365, .20),
    "trident": (.46, .48, .42, .26, .1365, .35),
    "solarsentinel": (.38, .36, .34, .24, .1365, .17),
    "luminousoracle": (.38, .36 * .82, .34, .24, .1365, .17),
    "celestialarcher": (.38, .36 * .85, .34, .24, .1365, .20),
    "justiciar": (.38, .46 * 1.10, .34, .24, .1365, .22),
    "jawguard": (.28, .46, .30, .24, .1365, .18),
    "mantis": (.36, .36 * .68, .34, .24, .1365, .23),
    "hornet": (.38, .36 * .88, .34, .24, .1365, .23),
    "weaver": (.38, .36, .34, .24, .1365, .17),
    "pyrewarden": (.38, .36 * 1.2, .34, .24, .1365, .27),
    "drillnest": (.38, .36, .34, .24, .1365, .17),
    "gravecutter": (.38, .36, .34, .24, .1365, .17),
    "lastlightcantor": (.38, .36, .34, .24, .1365, .17),
    "cragclaw": (.38, .36, .34, .24, .1365, .17 * 1.15),
    "hushfang": (.36, .36, .34, .24, .1365, .18),
    "spinehurler": (.38, .36, .34, .24, .1365, .17),
    "miretender": (.38 * .85, .36 * 1.10, .34, .24, .1365, .17),
    "gorgehowl": (.38, .36, .34, .24, .1365, .17),
    "sunlancer": (.38, .36, .34, .24, .1365, .20),
    "eclipsedancer": (.38, .36, .34, .24, .1365, .17),
    "starbreaker": (.38, .36, .34, .24, .1365, .20),
    "dawnmender": (.38, .36, .34, .24, .1365, .19),
    "prismancer": (.38, .36, .34, .24, .1365, .17),
    **dict.fromkeys(INSECT_KITS, (.38, .36 * .83, .34, .24, .1365, .17 * .95)),
    "basaltbulwark": (.35, .48, .34, .24, .145, .235),
    "cindermaul": (.35, .44, .34, .24, .145, .235),
    "hookbrawler": (.43, .43, .34, .24, .145, .235),
    "flintrunner": (.48, .34, .34, .24, .145, .235),
    "claybinder": (.37, .43, .34, .24, .145, .235),
    "gustdrummer": (.39, .47, .34, .24, .145, .235),
    "hidemender": (.38, .41, .34, .24, .145, .235),
}
for insect in INSECT_KITS:
    leg, width, *_ = BODY_PLANS[insect]
    REST_OVERRIDES[insect] = {
        "BackSpring": (width*.45,-.02,-.08), "Tabard": (-width*.45,-.02,-.08),
        "HeadGear": (0,0,0),
        **dict.fromkeys(("StringTop","StringBottom","Back","Nock"), (0,-leg*.55,0)),
    }

REST_OVERRIDES["needlewing"]["HeadGear"] = (0,0,.13)

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
    parents = parents_for(unit)
    world = {}

    def resolve(name):
        if name not in world:
            parent = parents[name]
            offset = Vector(offsets[name])
            world[name] = offset if parent is None else resolve(parent) + offset
        return world[name]

    # Weapon-mounted springs can refer to a socket later in the export order.
    return {name: resolve(name) for name in parents}


def armature(unit, rest):
    data = bpy.data.armatures.new("TBG v2 · 17 body joints, springs and sockets")
    obj = bpy.data.objects.new(f"TBG v2 · {unit}", data)
    bpy.context.scene.collection.objects.link(obj)
    bpy.context.view_layer.objects.active = obj
    obj.show_in_front = True
    obj.select_set(True)
    bpy.ops.object.mode_set(mode="EDIT")
    parents = parents_for(unit)
    for name in parents:
        bone = data.edit_bones.new(name)
        bone.head = TO_BLENDER @ rest[name]
        bone.tail = bone.head + Vector((0, 0, 0.066 if INDEX[name] < 17 else 0.041))
    for name, parent in parents.items():
        if parent is not None:
            data.edit_bones[name].parent = data.edit_bones[parent]
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
