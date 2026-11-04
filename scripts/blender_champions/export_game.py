"""Export the selected modular Blender models in compact joint-local runtime form."""

from pathlib import Path
from struct import pack
import sys

import bpy

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from export_study import FORMS, UNITS as STUDY_UNITS, collection, groups_for
from acidmaw import acidmaw
from antidotebrewer import antidotebrewer
from bloodfurnace import bloodfurnace
from bloodweaver import bloodweaver
from bloomkeeper import bloomkeeper
from bombard import bombard
from bonehorn import bonehorn
from bulwark import bulwark
from capybara import capybara
from cargocrab import cargocrab
from celestialarcher import celestialarcher
from crowcaller import crowcaller
from demolisher import demolisher
from drumguard import drumguard
from farwatch import farwatch
from geometry import Model
from hexer import hexer
from justiciar import justiciar
from leechroot import leechroot
from longaxe import longaxe
from luminousoracle import luminousoracle
from nester import nester
from oldfinn import oldfinn
from oilpress import oilpress
from puppetmaster import puppetmaster
from rampart import rampart
from rattletick import rattletick
from ravager import ravager
from rig import NAMES, rest_positions
from rootcaller import rootcaller
from rotcap import rotcap
from sawbot import sawbot
from shatterback import plate as shatter_plate, shatterback
from shieldbearer import shieldbearer
from siegewalker import siegewalker
from smuggler import smuggler
from solarsentinel import solarsentinel
from trigger import trigger
from trident import trident
from whisperer import whisperer
from jawguard import jawguard
from mantis import mantis
from hornet import hornet
from weaver import weaver
from pyrewarden import pyrewarden
from drillnest import drillnest
from gravecutter import gravecutter
from lastlightcantor import lastlightcantor
from cragclaw import cragclaw
from hushfang import hushfang

OUTPUT = HERE.parents[1] / "crates/tbg-client/assets/champions/blender"
OWNERSHIP = {(63, 169, 245), (242, 84, 91)}
GAME_UNITS = {
    **STUDY_UNITS,
    "shieldbearer": shieldbearer,
    "hexer": hexer,
    "sawbot": sawbot,
    "bloomkeeper": bloomkeeper,
    "siegewalker": siegewalker,
    "rootcaller": rootcaller,
    "nester": nester,
    "rampart": rampart,
    "longaxe": longaxe,
    "farwatch": farwatch,
    "bombard": bombard,
    "rotcap": rotcap,
    "oldfinn": oldfinn,
    "antidotebrewer": antidotebrewer,
    "shatterback": shatterback,
    "bloodfurnace": bloodfurnace,
    "bloodweaver": bloodweaver,
    "oilpress": oilpress,
    "drumguard": drumguard,
    "leechroot": leechroot,
    "cargocrab": cargocrab,
    "puppetmaster": puppetmaster,
    "smuggler": smuggler,
    "bonehorn": bonehorn,
    "ravager": ravager,
    "acidmaw": acidmaw,
    "whisperer": whisperer,
    "bulwark": bulwark,
    "demolisher": demolisher,
    "crowcaller": crowcaller,
    "trigger": trigger,
    "rattletick": rattletick,
    "capybara": capybara,
    "trident": trident,
    "solarsentinel": solarsentinel,
    "luminousoracle": luminousoracle,
    "celestialarcher": celestialarcher,
    "justiciar": justiciar,
    "jawguard": jawguard,
    "mantis": mantis,
    "hornet": hornet,
    "weaver": weaver,
    "pyrewarden": pyrewarden,
    "drillnest": drillnest,
    "gravecutter": gravecutter,
    "lastlightcantor": lastlightcantor,
    "cragclaw": cragclaw,
    "hushfang": hushfang,
}


def quantize(values, scale):
    return [round(value * scale) for value in values]


def form_payload(groups, rest):
    positions = quantize([value for position in rest.values() for value in position], 4096)
    result = bytearray(pack(f"<{len(positions)}hB", *positions, len(groups)))
    for (layer, node), (vertices, indices) in sorted(groups.items()):
        assert layer in (0, 1) and node < len(rest) and len(vertices) <= 65535
        result.extend(pack("<BBHI", layer, node, len(vertices), len(indices)))
        for position, normal, color, team, _ in vertices:
            assert not team and color not in OWNERSHIP, "use the ground circle for ownership"
            result.extend(pack("<6h3B", *quantize(position, 4096), *quantize(normal, 32767), *color))
        result.extend(pack(f"<{len(indices)}H", *indices))
    return result


def export(unit, name, make):
    payloads = []
    for label, tier, branch in FORMS:
        bpy.ops.wm.read_factory_settings(use_empty=True)
        scene = collection(f"{name} {label}", bpy.context.scene.collection)
        model = Model(scene)
        make(model, tier, branch)
        groups = groups_for(model)
        rest = rest_positions(unit, production=True, tier=tier, branch=branch)
        payloads.append(form_payload(groups, rest))
        print(name, label, sum(len(indices)//3 for _,indices in groups.values()), flush=True)
    magic = b"TBG4" if len(rest) == len(NAMES) else b"TBG5"
    data = magic + pack("<5I", *(len(payload) for payload in payloads)) + b"".join(payloads)
    (OUTPUT / f"{name}.tbgmesh").write_bytes(data)
    print(f"{name}: {len(data)} bytes", flush=True)


def main(units):
    OUTPUT.mkdir(parents=True, exist_ok=True)
    for unit in units:
        make = GAME_UNITS[unit]
        export(unit, unit, lambda model, tier, branch: make(model, tier, branch, production=True))
        if unit == "shatterback":
            for index in range(4):
                export(unit, f"{unit}-plate-{index}",
                       lambda model, tier, branch: shatter_plate(model, tier, branch, index))


if __name__ == "__main__":
    units = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    main(units or GAME_UNITS)
