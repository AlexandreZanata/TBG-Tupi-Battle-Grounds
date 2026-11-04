"""Validate the saved Tundra assembly, both seats, and sampled auxiliary actions."""

import json
from pathlib import Path
import sys

import bpy

HERE=Path(__file__).resolve().parent
sys.path[:0]=[str(HERE),str(HERE.parent/"blender_forest_arena")]
from offshore_checks import SAMPLES, merchant_action, mine_action, validate_auxiliary_clearance, validate_layout
from offshore_sites import SITES
from tundra_study import ARENA, BOARD

SCENIC_LAYERS=("04  Path lanterns and timber","05  Snow-loaded treeline",
               "06  Frost understory","07  Glaciers and sea ice")


def main():
    scene=bpy.data.scenes[ARENA]
    bpy.context.window.scene=scene
    assert len(bpy.data.scenes[BOARD].objects)>30
    assets=[g for g in bpy.data.collections if g.asset_data]
    assert len(assets)==25
    sites={g.name:g for g in scene.collection.children["08  Mineiros and Mercadores"].children}
    assert sites.keys()=={name for name,*_ in SITES}
    static_count=validate_layout(scene,sites,"02  BattleLine and frost curbs","03  Winter Strongholds",
                                 "Slate flagstone","Winter Stronghold",SCENIC_LAYERS)
    moving_count=validate_auxiliary_clearance(scene,sites)
    mining={name:mine_action(scene,sites[name],"Winter mine face pivot") for name in ("Mineiro A","Mineiro B")}
    docking={name:merchant_action(scene,sites[name],"Winter market shelter pivot","Icebreaker's bow shoe pivot") for name in ("Mercador A","Mercador B")}
    assert mining["Mineiro A"]==mining["Mineiro B"] and docking["Mercador A"]==docking["Mercador B"]
    result={"blender":bpy.app.version_string,"scenes":2,"asset_collections":len(assets),
            "flagstones":72,"sites":list(sites),"scenic_mesh_bounds":static_count,
            "auxiliary_sample_times":list(SAMPLES),"auxiliary_mesh_bounds":moving_count,
            "mine_pick_contacts_local":mining,"merchant_berth_travel":docking}
    if "--" in sys.argv:
        Path(sys.argv[sys.argv.index("--")+1]).write_text(json.dumps(result,indent=2)+"\n")
    print(json.dumps(result),flush=True)


if __name__=="__main__":
    main()
