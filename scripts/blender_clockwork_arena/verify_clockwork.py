"""Check the saved Clockwork Bastion, its drive train and both auxiliary seats."""

import json
from math import isclose
from pathlib import Path
import sys

import bpy

HERE = Path(__file__).resolve().parent
sys.path[:0] = [str(HERE),str(HERE.parent/"blender_forest_arena"),str(HERE.parent/"blender_tundra_arena")]
from clockwork_motion import GEAR_LAYER
from clockwork_study import ARENA, BOARD, COAST, HOLDS, ROAD, SCENIC_LAYERS, SITES_LAYER
from offshore_checks import (
    SAMPLES, frame, merchant_action, mine_action, validate_auxiliary_clearance, validate_hull_clearance,
    validate_layout,
)
from offshore_sites import SITES, pivot


def drive_train(scene):
    gears = scene.collection.children[GEAR_LAYER].objects
    drive = gears["Foundry central drive"]
    followers = [gears[name] for name in ("Foundry left follower","Foundry right follower")]
    assert len(gears)==3
    assert drive.instance_collection.name=="24-tooth brass gear"
    for follower in followers:
        assert follower.instance_collection.name=="16-tooth brass gear"
        assert isclose((drive.location-follower.location).length,1.60,abs_tol=1e-6)
        assert drive.location.y==follower.location.y and drive.location.z==follower.location.z
    frame(scene,0)
    start = [obj.rotation_euler.y for obj in gears]
    previous = {obj.name:obj.rotation_euler.y for obj in gears}
    samples = 0
    for time in (2.2,5.2,10,17,23.1,29):
        frame(scene,time)
        delta = {obj.name:obj.rotation_euler.y-previous[obj.name] for obj in gears}
        assert delta[drive.name]>0
        for follower in followers:
            assert delta[follower.name]<0
            assert abs(delta[drive.name]*24+delta[follower.name]*16)<1e-4
        assert all(obj.animation_data.drivers[0].driver.is_valid for obj in gears)
        previous = {obj.name:obj.rotation_euler.y for obj in gears}
        samples += len(gears)
    travel = [round(obj.rotation_euler.y-angle,4) for obj,angle in zip(gears,start,strict=True)]
    return {"checked_angles":samples,"travel_radians":travel,"axle_distance":1.60}


def main():
    scene = bpy.data.scenes[ARENA]
    bpy.context.window.scene = scene
    assets = [group for group in bpy.data.collections if group.asset_data]
    assert len(assets)==29 and len(bpy.data.scenes[BOARD].objects)>30
    sites = {group.name:group for group in scene.collection.children[SITES_LAYER].children}
    assert sites.keys()=={name for name,*_ in SITES}
    scenic = validate_layout(scene,sites,ROAD,HOLDS,"Clockwork flagstone","Clockwork Stronghold",SCENIC_LAYERS)
    auxiliary = validate_auxiliary_clearance(scene,sites)
    miners = {name:mine_action(scene,sites[name],"Bolted quarry portal pivot")
              for name in ("Mineiro A","Mineiro B")}
    for name in miners:
        assert pivot(sites[name],"Mine cart side armor pivot").parent==pivot(sites[name],"Ore cart pivot")
    merchants = {name:merchant_action(scene,sites[name],"Ribbed foundry canopy pivot",
                                     "Ironclad trade prow pivot") for name in ("Mercador A","Mercador B")}
    assert miners["Mineiro A"]==miners["Mineiro B"] and merchants["Mercador A"]==merchants["Mercador B"]
    hull = validate_hull_clearance(scene,sites,COAST,
                                  ("Foundry island foundation","Crane islet","Waterlogged supply raft"),
                                  ("Clockwork prow ",),7)
    result = {"blender":bpy.app.version_string,"asset_collections":len(assets),"flagstones":72,
              "scenic_bounds":scenic,"auxiliary_sample_times":list(SAMPLES),"auxiliary_bounds":auxiliary,
              "mine_pick_contacts_local":miners,"merchant_berth_travel":merchants,
              "clear_boat_mesh_samples":hull,"drive_train":drive_train(scene)}
    if "--" in sys.argv:
        Path(sys.argv[sys.argv.index("--")+1]).write_text(json.dumps(result,indent=2)+"\n")
    print(json.dumps(result),flush=True)


if __name__=="__main__":
    main()
