"""Check the saved Ancient Ruins scene and both sets of auxiliary motions."""

import json
from pathlib import Path
import sys

import bpy

HERE=Path(__file__).resolve().parent
sys.path[:0]=[str(HERE),str(HERE.parent/"blender_forest_arena"),str(HERE.parent/"blender_tundra_arena")]
from offshore_checks import (
    SAMPLES, merchant_action, mine_action, validate_auxiliary_clearance, validate_hull_clearance, validate_layout,
)
from offshore_sites import SITES
from ruins_study import ARENA, BOARD, COAST, HOLDS, ROAD, SCENIC_LAYERS, SITES_LAYER


def main():
    scene=bpy.data.scenes[ARENA]
    bpy.context.window.scene=scene
    assets=[g for g in bpy.data.collections if g.asset_data]
    assert len(assets)==26 and len(bpy.data.scenes[BOARD].objects)>30
    sites={g.name:g for g in scene.collection.children[SITES_LAYER].children}
    assert sites.keys()=={name for name,*_ in SITES}
    static=validate_layout(scene,sites,ROAD,HOLDS,"Limestone flagstone","Ruins Stronghold",SCENIC_LAYERS)
    moving=validate_auxiliary_clearance(scene,sites)
    miners={name:mine_action(scene,sites[name],"Ruin adit stonework pivot")
            for name in ("Mineiro A","Mineiro B")}
    merchants={name:merchant_action(scene,sites[name],"Reed market canopy pivot","Carved lagoon prow pivot")
               for name in ("Mercador A","Mercador B")}
    assert miners["Mineiro A"]==miners["Mineiro B"] and merchants["Mercador A"]==merchants["Mercador B"]
    hull=validate_hull_clearance(scene,sites,COAST,
        ("Sanctuary islet","Flooded limestone steps","Submerged column drum","Lagoon lilies"),
        ("Lagoon prow ",),7)
    result={"blender":bpy.app.version_string,"asset_collections":len(assets),"flagstones":72,
            "scenic_bounds":static,"auxiliary_sample_times":list(SAMPLES),"auxiliary_bounds":moving,
            "mine_pick_contacts_local":miners,"merchant_berth_travel":merchants,
            "clear_boat_mesh_samples":hull}
    if "--" in sys.argv:
        Path(sys.argv[sys.argv.index("--")+1]).write_text(json.dumps(result,indent=2)+"\n")
    print(json.dumps(result),flush=True)


if __name__=="__main__":
    main()
