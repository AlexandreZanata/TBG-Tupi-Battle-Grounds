"""Check the saved volcanic arena, mine motions and barge approach lanes."""

import json
from pathlib import Path
import sys

import bpy

HERE=Path(__file__).resolve().parent
sys.path[:0]=[str(HERE),str(HERE.parent/"blender_forest_arena"),str(HERE.parent/"blender_tundra_arena")]
from forge_study import ARENA, BOARD, COAST, HOLDS, ROAD, SCENIC_LAYERS, SITES_LAYER
from offshore_checks import (
    SAMPLES, merchant_action, mine_action, validate_auxiliary_clearance, validate_hull_clearance, validate_layout,
)
from offshore_sites import SITES

def main():
    scene=bpy.data.scenes[ARENA]
    bpy.context.window.scene=scene
    assets=[g for g in bpy.data.collections if g.asset_data]
    assert len(assets)==24 and len(bpy.data.scenes[BOARD].objects)>30
    sites={g.name:g for g in scene.collection.children[SITES_LAYER].children}
    assert sites.keys()=={name for name,*_ in SITES}
    static=validate_layout(scene,sites,ROAD,HOLDS,"Charcoal flagstone","Forge Stronghold",SCENIC_LAYERS)
    moving=validate_auxiliary_clearance(scene,sites)
    miners={name:mine_action(scene,sites[name],"Slag mine vent pivot") for name in ("Mineiro A","Mineiro B")}
    merchants={name:merchant_action(scene,sites[name],"Foundry market shelter pivot","Barge heat shield pivot")
               for name in ("Mercador A","Mercador B")}
    assert miners["Mineiro A"]==miners["Mineiro B"] and merchants["Mercador A"]==merchants["Mercador B"]
    result={"blender":bpy.app.version_string,"asset_collections":len(assets),"flagstones":72,
            "scenic_bounds":static,"auxiliary_sample_times":list(SAMPLES),"auxiliary_bounds":moving,
            "mine_pick_contacts_local":miners,"merchant_berth_travel":merchants,
            "clear_barge_hull_samples":validate_hull_clearance(
                scene,sites,COAST,("Drifting slag crust",),("Forged prow shroud",),2)}
    if "--" in sys.argv:
        Path(sys.argv[sys.argv.index("--")+1]).write_text(json.dumps(result,indent=2)+"\n")
    print(json.dumps(result),flush=True)


if __name__=="__main__":
    main()
