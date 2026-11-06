"""Assemble the next modular Blender arena and render the two auxiliary sites per seat."""

import argparse
import json
from math import cos, radians, sin, tan, tau
from pathlib import Path
from random import Random
import sys

import bpy

HERE = Path(__file__).resolve().parent
sys.path[:0] = [str(HERE),str(HERE.parent/"blender_forest_arena")]
import build as forest
from auxiliary_study import timeline
from aux_parts import FPS
from mesh import box, instance, prototype
from terrain import HOLD_CENTERS, LANE_LENGTH, LANE_WIDTH
from tundra_auxiliaries import place
import tundra_props as props
import tundra_terrain as terrain

ARENA = "01  Frozen Tundra"
BOARD = "02  Winter component board"
OVERVIEW_TIME = 24.37


def kit():
    makers={
        "Snow plateau":terrain.plateau, "Frost-cut escarpment":terrain.escarpment,
        "Cold sea":terrain.sea, "Fractured ice shelf":terrain.shelf,
        "Slate road bed":terrain.lane_bed, "Frost curb":terrain.shoulder,
        "Winter Stronghold":props.stronghold, "Beacon lantern":props.beacon,
        "Snowbound rail":props.rail, "Granite and snow":props.boulder,
        "Crystal vein":props.frost_crystals, "Frozen cascade":props.icefall,
        "Trail cairn":props.cairn, "Low snowdrift":props.drift, "Frost reeds":props.reeds,
        "Glacier ridge":props.ridge, "Drift ice":props.floe, "Snow-loaded spruce":props.fir,
        "Young spruce":lambda group: props.fir(group,young=True),
        "Blue Stronghold banners":lambda group: props.standard(group,"blue"),
        "Red Stronghold banners":lambda group: props.standard(group,"red"),
    }
    for i in range(4):
        makers[f"Slate flagstone {i+1}"]=lambda group,i=i: terrain.flagstone(group,i)
    assets={name:prototype(name) for name in makers}
    for name,make in makers.items():
        make(assets[name])
    return assets


def assemble(scene,assets):
    rng=Random(245)

    def put(group,name,at,scale=1,angle=0):
        return instance(group,assets[name],name,at,scale,angle)

    place_lane(scene,put,rng)
    place_paths(scene,put)
    place_treeline(scene,put,rng)
    place_understory(scene,put,rng)
    place_coast(scene,put,rng)
    place(forest.group(scene,"08  Mineiros and Mercadores"))


def place_lane(scene,put,rng):
    terrain_layer=forest.group(scene,"01  Snow, slate cliffs and shelf ice")
    road=forest.group(scene,"02  BattleLine and frost curbs")
    castles=forest.group(scene,"03  Winter Strongholds")
    for name in ("Snow plateau","Frost-cut escarpment","Cold sea","Fractured ice shelf"):
        put(terrain_layer,name,(0,0,0))
    put(road,"Slate road bed",(LANE_LENGTH/2,0,0))
    for i in range(18):
        x=i-.50
        for j in range(4):
            variant=1+rng.choices(range(4),(7,2,5,1))[0]
            put(road,f"Slate flagstone {variant}",(x+(j%2)*.009,(j-1.5)*1.238,0),
                angle=rng.uniform(-.007,.007))
        for sign in (-1,1):
            put(road,"Frost curb",(x,sign*(LANE_WIDTH/2+.086),0),
                angle=(tau/2 if sign<0 else 0)+sign*.011)
    for x,seat in zip(HOLD_CENTERS,("Blue","Red"),strict=True):
        put(castles,"Winter Stronghold",(x,0,0))
        put(castles,f"{seat} Stronghold banners",(x,0,0))


def place_paths(scene,put):
    fixtures=forest.group(scene,"04  Path lanterns and timber")
    for sign in (-1,1):
        for x in (.92,5.23,10.77,15.08):
            put(fixtures,"Beacon lantern",(x,sign*2.98,-.018),.93)
        for x in (2.35,3.55,12.45,13.65):
            put(fixtures,"Snowbound rail",(x,sign*3.12,-.028),angle=sign*.033*sin(x))
        for x in (6.45,9.55):
            put(fixtures,"Trail cairn",(x,sign*3.03,-.019),.92)


def place_treeline(scene,put,rng):
    pines=forest.group(scene,"05  Snow-loaded treeline")
    for i in range(21):
        x=-1.55+i*.955
        y=3.90+1.45*sin((x+2.6)/21.5*3.14)+rng.uniform(-.18,.22)
        height=rng.uniform(.74,1.21)
        put(pines,"Snow-loaded spruce" if i%3 else "Young spruce",(x,y,.004),
            height,rng.uniform(-tau,tau))
    for i in range(8):
        x=.74+i*2.08
        put(pines,"Young spruce",(x,3.61+.19*sin(i),-.018),rng.uniform(.53,.73),i*.61)


def place_understory(scene,put,rng):
    drifts=forest.group(scene,"06  Frost understory")
    for i in range(62):
        x,y=rng.uniform(-1.58,17.58),rng.uniform(3.37,5.72)
        sign=1 if i%4==0 else -1
        shore_distance=((x-8)/11.80)**2+(y/6.10)**2
        market_access=sign<0 and min(abs(x-1.42),abs(x-14.58))<.91 and y>4.18
        if shore_distance>.91 or market_access:
            continue
        name=("Granite and snow" if i%12==0 else "Crystal vein" if i%9==0
              else "Low snowdrift" if i%3==0 else "Frost reeds")
        put(drifts,name,(x,sign*y,-.014+(y-3.36)*.028),rng.uniform(.62,1.03),rng.random()*tau)
    for x,y,name in ((3.54,-4.81,"Granite and snow"),(5.32,-5.34,"Crystal vein"),
                     (8.07,-5.49,"Granite and snow"),(10.73,-5.24,"Crystal vein"),
                     (12.31,-4.87,"Granite and snow"),(2.78,3.46,"Crystal vein"),
                     (13.24,3.50,"Crystal vein")):
        put(drifts,name,(x,y,.026),1.04,rng.random()*tau)


def place_coast(scene,put,rng):
    coast=forest.group(scene,"07  Glaciers and sea ice")
    for x,y,s in ((-5.36,10.2,1.43),(.45,11.58,1.97),(6.98,12.75,2.39),
                  (13.30,11.19,1.97),(20.66,9.81,1.55)):
        put(coast,"Glacier ridge",(x,y,-1.22),(s,s,s*.82),rng.uniform(-.13,.15))
    for x,y,scale in ((4.83,-6.03,.86),(9.77,-6.15,1.05),(11.13,-5.99,.64)):
        put(coast,"Frozen cascade",(x,y,.032),scale,rng.uniform(-.16,.19))
    for x,y,s in ((-5.7,-2.1,1.18),(-3.6,-5.0,1.04),(4.3,-7.42,.91),(6.7,-8.46,1.02),
                  (8.5,-7.98,.73),(10.9,-7.44,1.08),(18.78,-5.0,1.04),(22.1,-1.8,1.23),
                  (-4.4,5.75,.84),(21.8,5.22,1.03),(2.38,8.61,1.13),(15.48,9.0,.84)):
        put(coast,"Drift ice",(x,y,terrain.WATERLINE+.062),s,rng.random()*tau)


def lighting(scene):
    forest.lighting(scene)
    scene.world.node_tree.nodes["Background"].inputs["Color"].default_value=(.55,.68,.84,1)
    scene.world.node_tree.nodes["Background"].inputs["Strength"].default_value=.42
    sun=next(obj.data for obj in scene.objects if obj.type=="LIGHT" and obj.data.type=="SUN")
    sun.color=(1.0,.88,.73)
    sun.energy=2.15


def cameras(scene):
    distance=max(13.5/(tan(radians(17.5))*16/9),7.1/tan(radians(17.5)))
    return {
        "Overview":forest.camera(scene,"Overview", (4.3,-25.6,24.7),(8,1.66,-.28),35.1),
        "Match":forest.camera(scene,"Desktop match framing",
             (8,-distance*cos(radians(55)),distance*sin(radians(55))),(8,0,0)),
        "Mineiro A":forest.camera(scene,"Mineiro A",(-3.05,-2.52,4.25),(-4.35,2.53,.37),4.68),
        "Mineiro B":forest.camera(scene,"Mineiro B",(19.05,-2.52,4.25),(20.35,2.53,.37),4.68),
        "Mercador A":forest.camera(scene,"Mercador A",(3.80,-11.72,4.32),(1.47,-7.11,.07),5.26),
        "Mercador B":forest.camera(scene,"Mercador B",(12.20,-11.72,4.32),(14.53,-7.11,.07),5.26),
    }


def component_board(assets,samples):
    scene=bpy.data.scenes.new(BOARD)
    group=forest.group(scene,"Individual winter asset samples")
    box(group,"Slate studio tabletop",(0,0,-.32),(130,130,.26),"frost-basalt")
    examples=(("Winter Stronghold",.61),("Snow-loaded spruce",.83),("Young spruce",.95),
              ("Glacier ridge",.64),("Granite and snow",1.54),("Crystal vein",1.82),
              ("Snowbound rail",1.53),("Beacon lantern",1.52),("Trail cairn",1.72),
              ("Frost reeds",1.95),("Slate flagstone 1",1.23),("Frost curb",1.75),
              ("Low snowdrift",1.53),("Drift ice",1.49),("Frozen cascade",1.12),
              ("Slate flagstone 2",1.23))
    for i,(name,scale) in enumerate(examples):
        x,y=(i%4-1.5)*3.2,(1.5-i//4)*2.90
        box(group,"Component review platform",(x,y,-.112),(2.85,2.48,.210),"road-slate",.061)
        z=.98 if name=="Frozen cascade" else 0
        instance(group,assets[name],name,(x,y+.16,z),scale,.14)
        forest.label(group,name,(x,y-1.05,.014))
    scene.camera=forest.camera(scene,"Component board",(0,-11.4,24),(0,0,.17),15.0)
    lighting(scene)
    forest.render_setup(scene,1536,1320,samples)
    return scene


def render(scene,camera,time,size,samples,path):
    bpy.context.window.scene=scene
    scene.camera=camera
    scene.frame_set(round(time*FPS)+1)
    forest.render_setup(scene,*size,samples)
    scene.render.filepath=str(path)
    bpy.ops.render.render(write_still=True)


def main():
    args=argparse.ArgumentParser()
    args.add_argument("output",type=Path)
    args.add_argument("--samples",type=int,default=20)
    args.add_argument("--draft",action="store_true")
    args.add_argument("--project-only",action="store_true")
    options=args.parse_args(sys.argv[sys.argv.index("--")+1:])
    out=options.output.resolve()
    out.mkdir(parents=True,exist_ok=True)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    terrain.register_palette()
    scene=bpy.context.scene
    scene.name=ARENA
    assets=kit()
    assemble(scene,assets)
    views=cameras(scene)
    lighting(scene)
    timeline(scene)
    scene.frame_set(round(OVERVIEW_TIME*FPS)+1)
    scene.camera=views["Overview"]
    forest.render_setup(scene,1740,1050,options.samples)
    board=component_board(assets,options.samples)
    metrics=forest.measure(scene,assets)
    (out/"metrics.json").write_text(json.dumps(metrics,indent=2)+"\n")
    print(metrics,flush=True)
    for area in bpy.context.screen.areas:
        if area.type=="VIEW_3D":
            area.spaces.active.region_3d.view_perspective="CAMERA"
    bpy.ops.wm.save_as_mainfile(filepath=str(out/"frozen-tundra.blend"),compress=True)
    if options.project_only:
        return

    shots=(
        (scene,"Overview",OVERVIEW_TIME,(1740,1050),"tundra-overview"),
        (scene,"Match",OVERVIEW_TIME,(1600,900),"tundra-match"),
        (board,None,0,(1536,1320),"tundra-kit"),
        (scene,"Mineiro A",24.24,(900,730),"mineiro-a"),
        (scene,"Mineiro B",24.24,(900,730),"mineiro-b"),
        (scene,"Mercador A",24.37,(900,730),"mercador-a"),
        (scene,"Mercador B",24.37,(900,730),"mercador-b"),
    )
    for shot,camera,time,size,filename in shots:
        if options.draft:
            size=tuple(round(d*.53) for d in size)
        render(shot,views[camera] if camera else board.camera,time,size,
               8 if options.draft else options.samples,out/(filename+".png"))


if __name__=="__main__":
    main()
