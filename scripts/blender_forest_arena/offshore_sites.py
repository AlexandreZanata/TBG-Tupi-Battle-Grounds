"""Place the two mirrored War Auxiliary precincts on the shared coastline."""

from aux_merchant import merchant
from aux_miner import miner
from hold_parts import empty, layer
from mesh import material

# Local bridges run from the pad to the shore. The east models mirror in X.
SITES = (
    ("Mineiro A",miner,(-4.93,2.44,0),(1.07,0,2.52,0),"blue"),
    ("Mineiro B",miner,(20.93,2.44,0),(1.07,0,2.52,0),"red"),
    ("Mercador A",merchant,(1.42,-6.39,0),(0,.70,0,2.10),"blue"),
    ("Mercador B",merchant,(14.58,-6.39,0),(0,.70,0,2.10),"red"),
)

def pivot(group,prefix):
    matches=[obj for obj in group.all_objects if obj.type=="EMPTY" and obj.name.startswith(prefix)]
    if len(matches)!=1:
        raise ValueError(f"{group.name} should contain one {prefix!r}, found {len(matches)}")
    return matches[0]


def place(parent,palette,decorate,design):
    for name,model,at,bridge,seat in SITES:
        group=layer(parent,name)
        root=empty(group,name+" precinct",at)
        root.scale.x=-1 if seat=="red" else 1
        root["Seat"]=name[-1]
        root["Design"]=design
        model(group,root,bridge,seat)
        for obj in group.all_objects:
            for slot in obj.material_slots:
                replacement=palette.get(slot.material.name) if slot.material else None
                if replacement:
                    slot.material=material(replacement)
        decorate(group,model is miner)
