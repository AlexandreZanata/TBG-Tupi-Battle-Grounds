"""Export the selected modular Blender models in compact joint-local runtime form."""

from pathlib import Path
from struct import pack
import sys

import bpy

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from export_study import FORMS, UNITS as STUDY_UNITS, collection, groups_for
from antidotebrewer import antidotebrewer
from bloomkeeper import bloomkeeper
from bombard import bombard
from farwatch import farwatch
from geometry import Model
from hexer import hexer
from longaxe import longaxe
from nester import nester
from oldfinn import oldfinn
from rampart import rampart
from rig import NAMES, rest_positions
from rootcaller import rootcaller
from rotcap import rotcap
from sawbot import sawbot
from shieldbearer import shieldbearer
from siegewalker import siegewalker

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
}


def quantize(values, scale):
    return [round(value * scale) for value in values]


def form_payload(groups, rest):
    result = bytearray(pack("<69hB", *quantize([v for name in NAMES for v in rest[name]], 4096), len(groups)))
    for (layer, node), (vertices, indices) in sorted(groups.items()):
        assert layer in (0, 1) and len(vertices) <= 65535
        result.extend(pack("<BBHI", layer, node, len(vertices), len(indices)))
        for position, normal, color, team, _ in vertices:
            assert not team and color not in OWNERSHIP, "use the ground circle for ownership"
            result.extend(pack("<6h3B", *quantize(position, 4096), *quantize(normal, 32767), *color))
        result.extend(pack(f"<{len(indices)}H", *indices))
    return result


def main(units):
    OUTPUT.mkdir(parents=True, exist_ok=True)
    for unit in units:
        make = GAME_UNITS[unit]
        payloads = []
        for label, tier, branch in FORMS:
            bpy.ops.wm.read_factory_settings(use_empty=True)
            scene = collection(f"{unit} {label}", bpy.context.scene.collection)
            model = Model(scene)
            make(model, tier, branch, production=True)
            groups = groups_for(model)
            rest = rest_positions(unit, production=True, tier=tier, branch=branch)
            payloads.append(form_payload(groups, rest))
            print(unit, label, sum(len(indices)//3 for _,indices in groups.values()), flush=True)
        data = b"TBG4" + pack("<5I", *(len(payload) for payload in payloads)) + b"".join(payloads)
        (OUTPUT / f"{unit}.tbgmesh").write_bytes(data)
        print(f"{unit}: {len(data)} bytes", flush=True)


if __name__ == "__main__":
    units = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    main(units or GAME_UNITS)
