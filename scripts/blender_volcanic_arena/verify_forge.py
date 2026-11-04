"""Check the saved volcanic arena, mine motions and barge approach lanes."""

import json
from pathlib import Path
import sys

import bpy

HERE=Path(__file__).resolve().parent
sys.path[:0]=[str(HERE),str(HERE.parent/"blender_forest_arena"),str(HERE.parent/"blender_tundra_arena")]
from forge_study import ARENA, BOARD, COAST, HOLDS, ROAD, SCENIC_LAYERS, SITES_LAYER
from offshore_checks import SAMPLES, bounds, frame, merchant_action, mine_action, validate_auxiliary_clearance, validate_layout
from offshore_sites import SITES, pivot

APPROACH=(23.1,23.4,23.92,24.28,24.82,25.15,25.62,26.30)


def rectangle(corners):
    x,y,_=zip(*corners,strict=True)
    return min(x),max(x),min(y),max(y)


def overlaps(a,b):
    return a[0]<b[1] and a[1]>b[0] and a[2]<b[3] and a[3]>b[2]


def check_barge_channel(scene,sites):
    frame(scene,APPROACH[0])
    graph=bpy.context.evaluated_depsgraph_get()
    coast=scene.collection.children[COAST]
    crust=[rectangle(bounds(item.object,item.matrix_world)) for item in graph.object_instances
           if item.object.type=="MESH" and item.parent
           and item.parent.original.name.startswith("Drifting slag crust")
           and coast in item.parent.original.users_collection[:]]
    assert crust
    count=0
    for name in ("Mercador A","Mercador B"):
        ship=pivot(sites[name],"Merchant caravel pivot")
        hull=[o for o in ship.children_recursive if o.type=="MESH" and (
            o.name.startswith("Open carvel-planked hull") or o.name.startswith("Forged prow shroud"))]
        assert len(hull)==2
        for time in APPROACH:
            frame(scene,time)
            graph=bpy.context.evaluated_depsgraph_get()
            for mesh in hull:
                obj=mesh.evaluated_get(graph)
                extent=rectangle(bounds(obj,obj.matrix_world))
                assert not any(overlaps(extent,rock) for rock in crust),(name,time,extent)
                count+=1
    return count


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
            "clear_barge_hull_samples":check_barge_channel(scene,sites)}
    if "--" in sys.argv:
        Path(sys.argv[sys.argv.index("--")+1]).write_text(json.dumps(result,indent=2)+"\n")
    print(json.dumps(result),flush=True)


if __name__=="__main__":
    main()
