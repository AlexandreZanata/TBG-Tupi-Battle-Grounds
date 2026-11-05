"""Check the saved Desert Relics layout, temple assembly and auxiliary motions."""

import json
from math import isclose
from pathlib import Path
import sys

import bpy

HERE = Path(__file__).resolve().parent
sys.path[:0] = [str(HERE),str(HERE.parent/"blender_forest_arena"),str(HERE.parent/"blender_tundra_arena")]
from desert_study import ARENA, BOARD, COAST, HOLDS, ROAD, SCENIC_LAYERS, SITES_LAYER
from offshore_checks import (
    SAMPLES, merchant_action, mine_action, validate_auxiliary_clearance, validate_hull_clearance, validate_layout,
)
from offshore_sites import SITES


def check_temple(scene):
    objects = scene.collection.children[COAST].objects
    temple = [obj for obj in objects if obj.instance_collection.name=="Stepped sun temple"]
    island = [obj for obj in objects if obj.instance_collection.name=="Sun temple island"]
    guardians = [obj for obj in objects if obj.instance_collection.name=="Reclining sandstone guardian"]
    assert len(temple)==len(island)==1 and len(guardians)==2
    center = temple[0].location.x
    assert isclose(island[0].location.x,center,abs_tol=1e-6)
    assert isclose(sum(obj.location.x for obj in guardians),center*2,abs_tol=1e-5)
    assert guardians[0].location.y==guardians[1].location.y
    assert tuple(guardians[0].scale)==tuple(guardians[1].scale)
    assert isclose(guardians[0].rotation_euler.z,-guardians[1].rotation_euler.z,abs_tol=1e-6)
    return len(guardians)


def main():
    scene = bpy.data.scenes[ARENA]
    bpy.context.window.scene = scene
    assets = [group for group in bpy.data.collections if group.asset_data]
    assert len(assets)==28 and len(bpy.data.scenes[BOARD].objects)>30
    sites = {group.name:group for group in scene.collection.children[SITES_LAYER].children}
    assert sites.keys()=={name for name,*_ in SITES}
    scenic = validate_layout(scene,sites,ROAD,HOLDS,"Desert flagstone","Desert Stronghold",SCENIC_LAYERS)
    auxiliary = validate_auxiliary_clearance(scene,sites)
    miners = {name:mine_action(scene,sites[name],"Carved sandstone portal pivot")
              for name in ("Mineiro A","Mineiro B")}
    merchants = {name:merchant_action(scene,sites[name],"Reed oasis canopy pivot","Carved oasis prow pivot")
                 for name in ("Mercador A","Mercador B")}
    assert miners["Mineiro A"]==miners["Mineiro B"] and merchants["Mercador A"]==merchants["Mercador B"]
    hull = validate_hull_clearance(scene,sites,COAST,("Sun temple island","Sandstone skerry"),
                                  ("Oasis prow ",),7)
    result = {"blender":bpy.app.version_string,"asset_collections":len(assets),"flagstones":72,
              "temple_guardians":check_temple(scene),"scenic_bounds":scenic,
              "auxiliary_sample_times":list(SAMPLES),"auxiliary_bounds":auxiliary,
              "mine_pick_contacts_local":miners,"merchant_berth_travel":merchants,"clear_boat_mesh_samples":hull}
    if "--" in sys.argv:
        Path(sys.argv[sys.argv.index("--")+1]).write_text(json.dumps(result,indent=2)+"\n")
    print(json.dumps(result),flush=True)


if __name__=="__main__":
    main()
