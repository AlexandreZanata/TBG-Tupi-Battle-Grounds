"""Assemble and render the editable Verdant Forest arena study."""

import argparse
import json
from math import cos, radians, sin, tan, tau
from pathlib import Path
from random import Random
import sys

import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from mesh import box, instance, material, prototype
import scenery
import structures
import terrain


def group(scene,name):
    collection = bpy.data.collections.new(name)
    scene.collection.children.link(collection)
    return collection


def kit():
    makers = {
        "Meadow": terrain.meadow, "Layered coast": terrain.coast, "Sea and tidal shelf": terrain.ocean,
        "BattleLine bed": terrain.lane_bed, "Road shoulder": terrain.shoulder,
        "Classic Stronghold": structures.stronghold, "Brazier": structures.torch,
        "Split-rail fence": structures.fence, "Supplies": structures.cargo,
        "Lighthouse": structures.lighthouse, "Timber jetty": structures.jetty, "Sailboat": structures.sailboat,
        "Coastal oak": scenery.oak, "Mossy rocks": scenery.rock, "Shrub": scenery.shrub,
        "Meadow grass": scenery.grass, "Fern": scenery.fern, "Wildflowers": scenery.flowers,
        "Waterfall": scenery.cascade, "Tide rocks": scenery.reef,
    }
    assets = {}
    for label,make in makers.items():
        make(assets.setdefault(label,prototype(label)))
    for i in range(4):
        label = f"Sandstone flagstone {i+1}"
        terrain.paver(assets.setdefault(label,prototype(label)),i)
    for label,key in (("Blue Stronghold banners","blue"),("Red Stronghold banners","red")):
        structures.heraldry(assets.setdefault(label,prototype(label)),key)
    for label,dense in (("Full pine",True),("Sparse pine",False)):
        scenery.fir(assets.setdefault(label,prototype(label)),dense)
    for label,tall in (("Beacon island",False),("Cascade island",True)):
        scenery.islet(assets.setdefault(label,prototype(label)),tall)
    return assets


def assemble(scene,assets):
    rng = Random(804)
    land = group(scene,"01  Meadow, cliffs and water")
    road = group(scene,"02  BattleLine and sandstone shoulders")
    castles = group(scene,"03  Strongholds and architectural banners")
    props = group(scene,"04  Lane-side props")
    forest = group(scene,"05  Forest edge")
    understory = group(scene,"06  Shoreline understory")
    offshore = group(scene,"07  Beacon, sea stacks and cascade")
    put = lambda layer,label,at,scale=1,angle=0: instance(layer,assets[label],label,at,scale,angle)

    for name in ("Meadow","Layered coast","Sea and tidal shelf"):
        put(land,name,(0,0,0))
    put(road,"BattleLine bed",(terrain.LANE_LENGTH/2,0,0))
    for i in range(18):
        x = -.5+i
        for j in range(4):
            y = (j-1.5)*1.238
            variant = 1+rng.choices(range(4),(8,3,5,1))[0]
            put(road,f"Sandstone flagstone {variant}",(x+(j%2)*.009,y,0),angle=rng.uniform(-.008,.008))
        for sign in (-1,1):
            put(road,"Road shoulder",(x,sign*(terrain.LANE_WIDTH/2+.088),0),angle=sign*.013)
    for x,seat in zip(terrain.HOLD_CENTERS,("Blue","Red"),strict=True):
        put(castles,"Classic Stronghold",(x,0,0))
        put(castles,f"{seat} Stronghold banners",(x,0,0))

    for sign in (-1,1):
        for x in (1.28,14.72):
            put(props,"Brazier",(x,sign*3.04,.015),.89)
        for x in (3.0,4.25,11.75,13.0):
            put(props,"Split-rail fence",(x,sign*3.10,.015),angle=sign*.027*sin(x))
    for x,y,angle in ((-.67,-3.20,.30),(16.92,-3.10,-.19),(17.45,2.81,-.06),(.26,3.35,.3)):
        put(props,"Supplies",(x,y,.023),.88,angle)
    put(props,"Brazier",(-.30,-2.95,0),.75)
    put(props,"Brazier",(16.25,-2.95,0),.75)

    for i in range(23):
        x = -1.6+i*.88
        y = 3.42+1.79*sin((x+3)/22*3.14)+rng.uniform(-.20,.19)
        tree = "Coastal oak" if i in (1,8,15,20) else ("Full pine" if i%3 else "Sparse pine")
        scale = rng.uniform(.64,1.18)*(1.06 if i%5==0 else 1)
        put(forest,tree,(x,y,.015),scale,rng.uniform(-tau,tau))
    for i in range(11):
        x = .49+i*1.55
        y = 3.65+.36*sin(i*.9)
        tree = "Sparse pine" if i%3 else "Coastal oak"
        put(forest,tree,(x,y,.025),rng.uniform(.57,.79),rng.random()*tau)

    for i in range(115):
        x = rng.uniform(-2.53,18.52)
        side = -1 if i%4 else 1
        y = side*rng.uniform(3.53,5.79)
        if ((x-8)/11.73)**2+(y/5.99)**2 > .97:
            continue
        if i%12==0:
            item,scale = "Mossy rocks",rng.uniform(.78,1.14)
        elif i%5==0:
            item,scale = "Shrub",rng.uniform(.57,.96)
        elif i%7==0:
            item,scale = "Fern",rng.uniform(.69,1.14)
        elif i%3==0:
            item,scale = "Wildflowers",rng.uniform(.69,1.18)
        else:
            item,scale = "Meadow grass",rng.uniform(.52,1.18)
        put(understory,item,(x,y,.025+(abs(y)-3.55)*.015),scale,rng.random()*tau)
    for x,y,s in ((-1.9,-3.8,.76),(2.75,-4.73,1.32),(7.5,-5.27,1.0),(11.53,-4.87,1.24),(17.6,-3.4,1.25)):
        put(understory,"Mossy rocks",(x,y,.055),s,rng.random()*tau)
        put(understory,"Fern",(x+.42,y-.34,.04),.78,rng.random()*tau)
        put(understory,"Wildflowers",(x-.57,y-.14,.045),1.04,rng.random()*tau)

    put(offshore,"Beacon island",(-3.11,7.29,0),1.18,.22)
    put(offshore,"Lighthouse",(-3.28,7.27,.45),.93,-.17)
    put(offshore,"Sparse pine",(-2.28,7.68,.40),.48)
    put(offshore,"Timber jetty",(-3.56,5.72,-.51),.80,.45)
    put(offshore,"Sailboat",(-4.48,4.23,-1.12),.76,-1.29)
    put(offshore,"Cascade island",(19.94,7.44,0),1.13,-.04)
    put(offshore,"Waterfall",(19.84,5.78,.03),1.09,-.16)
    for x,y,s in ((18.90,7.6,.63),(20.18,8.03,.77),(21.15,7.74,.56)):
        put(offshore,"Full pine",(x,y,1.51),s,rng.random()*tau)
    for x,y,s in ((1.1,9.4,.72),(12.7,11.3,.91),(22.81,12.2,.65)):
        put(offshore,"Beacon island",(x,y,-.03),s,rng.random()*tau)
        put(offshore,"Coastal oak",(x,y,.27*s),s*.67,rng.random()*tau)
    for x,y,s in ((-4.55,-1.54,1.06),(-2.77,-4.57,.95),(3.33,-6.32,1.26),(9,-6.81,.91),
                  (15.24,-6.19,.76),(19.06,-3.38,1.18),(21.48,2.64,.8),(16.98,7.51,.71),(-.34,7.9,.6)):
        put(offshore,"Tide rocks",(x,y,-1.28*(1-s)),s,rng.random()*tau)


def aim(obj, at):
    obj.rotation_euler = (Vector(at)-obj.location).to_track_quat("-Z","Y").to_euler()


def camera(scene,name,origin,target,ortho=None):
    cam = bpy.data.cameras.new(name)
    obj = bpy.data.objects.new(name,cam)
    scene.collection.objects.link(obj)
    obj.location = origin
    aim(obj,target)
    cam.clip_end = 180
    if ortho:
        cam.type = "ORTHO"
        cam.ortho_scale = ortho
    else:
        cam.sensor_fit = "VERTICAL"
        cam.angle_y = radians(35)
    return obj


def lighting(scene,studio=False):
    world = bpy.data.worlds.new(scene.name+" ambient")
    scene.world = world
    world.use_nodes = True
    background = world.node_tree.nodes.get("Background")
    background.inputs["Color"].default_value = (.42,.53,.68,1)
    background.inputs["Strength"].default_value = .43 if studio else .34
    lights = group(scene,"Lighting rig")
    sun = bpy.data.lights.new("Warm afternoon sun","SUN")
    obj = bpy.data.objects.new(sun.name,sun)
    lights.objects.link(obj)
    obj.location = (-5,-12,20)
    aim(obj,(4,1,0))
    sun.energy, sun.angle = 2.35,radians(8)
    sun.color = (1,.86,.70)
    fill = bpy.data.lights.new("Cool sky bounce","AREA")
    obj = bpy.data.objects.new(fill.name,fill)
    lights.objects.link(obj)
    obj.location = (13,1,12)
    aim(obj,(8,0,0))
    fill.energy,fill.shape,fill.size = 700,"DISK",14
    fill.color = (.72,.85,1)


def render_setup(scene,width,height,samples):
    scene.render.engine = "CYCLES"
    scene.cycles.samples = samples
    scene.cycles.use_denoising = True
    scene.cycles.max_bounces = 4
    scene.render.resolution_x,scene.render.resolution_y = width,height
    scene.render.image_settings.file_format = "PNG"
    scene.view_settings.view_transform = "AgX"
    scene.view_settings.look = "AgX - Medium High Contrast"
    scene.view_settings.exposure = .33


def label(collection,body,at):
    data = bpy.data.curves.new(body,"FONT")
    data.body, data.size, data.align_x = body,.20,"CENTER"
    obj = bpy.data.objects.new(body,data)
    collection.objects.link(obj)
    obj.location = at
    data.materials.append(material("linen"))


def component_scene(assets,samples):
    scene = bpy.data.scenes.new("02  Modular asset board")
    collection = group(scene,"Editable asset instances")
    samples_to_show = (
        ("Classic Stronghold",.62),("Lighthouse",.77),("Full pine",.78),("Coastal oak",.80),
        ("Sailboat",.85),("Timber jetty",.83),("Split-rail fence",1.20),("Brazier",1.65),
        ("Mossy rocks",1.20),("Shrub",1.20),("Fern",1.55),("Wildflowers",1.82),
        ("Sandstone flagstone 1",1.28),("Road shoulder",1.52),("Supplies",1.20),("Meadow grass",1.58),
    )
    box(collection,"Studio tabletop",(0,0,-.35),(200,200,.30),"cliff-shadow")
    for i,(name,scale) in enumerate(samples_to_show):
        x,y = (i%4-1.5)*3.27,(1.5-i//4)*2.90
        box(collection,"Sample platform",(x,y,-.105),(2.76,2.34,.20),"distant",.075)
        angle = {"Classic Stronghold": -.13, "Sailboat": -1.16}.get(name,.27)
        instance(collection,assets[name],name,(x,y+.13,0),scale,angle)
        if name=="Classic Stronghold":
            instance(collection,assets["Blue Stronghold banners"],"Sample banner",(x,y+.13,0),scale,-.13)
        label(collection,name.replace(" 1",""),(x,y-1.02,.006))
    scene.camera = camera(scene,"Asset board",(0,-12.9,24),(0,0,.15),15.1)
    lighting(scene,studio=True)
    render_setup(scene,1500,1330,samples)
    return scene


def measure(scene,assets):
    bpy.context.window.scene = scene
    depsgraph = bpy.context.evaluated_depsgraph_get()
    count = 0
    by_mesh = {}
    layers = {}
    for inst in depsgraph.object_instances:
        obj = inst.object
        if obj.type != "MESH":
            continue
        count += 1
        if obj.data.name not in by_mesh:
            obj.data.calc_loop_triangles()
            by_mesh[obj.data.name] = len(obj.data.loop_triangles)
        parent = inst.parent.original if inst.parent else obj.original
        layer = parent.users_collection[0].name
        layers[layer] = layers.get(layer,0) + by_mesh[obj.data.name]
    return {"modular_collections":len(assets),"placed_mesh_instances":count,
            "evaluated_triangles":sum(layers.values()),"unique_evaluated_meshes":len(by_mesh),
            "triangles_by_layer":dict(sorted(layers.items()))}


def main():
    args = argparse.ArgumentParser()
    args.add_argument("output",type=Path)
    args.add_argument("--samples",type=int,default=24)
    args.add_argument("--draft",action="store_true")
    options = args.parse_args(sys.argv[sys.argv.index("--")+1:])
    out = options.output.resolve()
    out.mkdir(parents=True,exist_ok=True)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    arena = bpy.context.scene
    arena.name = "01  Verdant Forest"
    assets = kit()
    assemble(arena,assets)
    hero = camera(arena,"Diorama overview",(4.7,-23.5,23.0),(8,1.0,-.33),30.5)
    distance = max(13.5/(tan(radians(17.5))*16/9),7.1/tan(radians(17.5)))
    match = camera(arena,"Desktop match framing",(8,-distance*cos(radians(55)),distance*sin(radians(55))),(8,0,0))
    arena.camera = hero
    lighting(arena)
    render_setup(arena,1700,1062,options.samples)
    board = component_scene(assets,options.samples)
    metrics = measure(arena,assets)
    (out/"metrics.json").write_text(json.dumps(metrics,indent=2)+"\n")
    print(metrics,flush=True)
    for area in bpy.context.screen.areas:
        if area.type == "VIEW_3D":
            area.spaces.active.region_3d.view_perspective = "CAMERA"
    bpy.ops.wm.save_as_mainfile(filepath=str(out/"verdant-forest.blend"),compress=True)
    for scene,cam,file in ((arena,hero,"forest-hero"),(arena,match,"forest-match"),(board,board.camera,"forest-kit")):
        bpy.context.window.scene = scene
        scene.camera = cam
        if file=="forest-match":
            scene.render.resolution_x,scene.render.resolution_y = 1600,900
        scene.render.resolution_percentage = 52 if options.draft else 100
        scene.render.filepath = str(out/(file+".png"))
        bpy.ops.render.render(write_still=True)


if __name__ == "__main__":
    main()
