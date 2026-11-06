"""Check the saved Celestial Citadel layout and both animated auxiliary seats."""

import json
from pathlib import Path
import sys

import bpy

HERE = Path(__file__).resolve().parent
sys.path[:0] = [str(HERE),str(HERE.parent/"blender_forest_arena"),str(HERE.parent/"blender_tundra_arena")]
from offshore_checks import (
    SAMPLES, merchant_action, mine_action, validate_auxiliary_clearance, validate_hull_clearance, validate_layout,
)
from offshore_sites import SITES
from celestial_study import ARENA, BOARD, COAST, HOLDS, ROAD, SCENIC_LAYERS, SITES_LAYER


def check_waterfalls(scene):
    coast = scene.collection.children[COAST]
    bases = {tuple(obj.location):obj for obj in coast.objects
             if obj.instance_collection.name=="Waterfall islet"}
    falls = [obj for obj in coast.objects if obj.instance_collection.name=="Azure cascade"]
    assert len(bases)==len(falls)==3
    for fall in falls:
        base = bases[tuple(fall.location)]
        assert tuple(base.scale)==tuple(fall.scale) and base.rotation_euler.z==fall.rotation_euler.z
    return len(falls)


def main():
    scene = bpy.data.scenes[ARENA]
    bpy.context.window.scene = scene
    assets = [group for group in bpy.data.collections if group.asset_data]
    assert len(assets)==27 and len(bpy.data.scenes[BOARD].objects)>30
    sites = {group.name:group for group in scene.collection.children[SITES_LAYER].children}
    assert sites.keys()=={name for name,*_ in SITES}
    scenic = validate_layout(scene,sites,ROAD,HOLDS,"Celestial flagstone","Celestial Stronghold",SCENIC_LAYERS)
    auxiliary = validate_auxiliary_clearance(scene,sites)
    miners = {name:mine_action(scene,sites[name],"Ivory quarry portal pivot")
              for name in ("Mineiro A","Mineiro B")}
    merchants = {name:merchant_action(scene,sites[name],"Gilded market canopy pivot",
                                     "Ceremonial cutter prow pivot") for name in ("Mercador A","Mercador B")}
    assert miners["Mineiro A"]==miners["Mineiro B"] and merchants["Mercador A"]==merchants["Mercador B"]
    hull = validate_hull_clearance(scene,sites,COAST,
                                  ("Waterfall islet","Azure cascade","Citadel island foundation"),
                                  ("Celestial prow ",),7)
    result = {"blender":bpy.app.version_string,"asset_collections":len(assets),"flagstones":72,
              "waterfall_pairs":check_waterfalls(scene),"scenic_bounds":scenic,
              "auxiliary_sample_times":list(SAMPLES),"auxiliary_bounds":auxiliary,
              "mine_pick_contacts_local":miners,"merchant_berth_travel":merchants,"clear_boat_mesh_samples":hull}
    if "--" in sys.argv:
        Path(sys.argv[sys.argv.index("--")+1]).write_text(json.dumps(result,indent=2)+"\n")
    print(json.dumps(result),flush=True)


if __name__=="__main__":
    main()
