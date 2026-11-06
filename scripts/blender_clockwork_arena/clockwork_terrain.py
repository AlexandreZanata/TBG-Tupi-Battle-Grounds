"""Shape the industrial heath, slate lane and layered foundry coastline."""

from math import cos, sin
from random import Random

from mesh import PALETTE, box, color, material, surface
from terrain import CENTER, LANE_LENGTH, LANE_WIDTH, rim, rows

WATERLINE = -1.28
CLOCKWORK = {
    "cw-earth": "746d60", "cw-earth-light": "867a66", "cw-earth-dark": "625f55",
    "cw-rock": "626a6c", "cw-rock-light": "86908a", "cw-rock-dark": "444e53",
    "cw-stratum": "989484", "cw-slab": "8e9390", "cw-slab-light": "9a9d95",
    "cw-slab-cool": "84908e", "cw-grout": "626764",
    "cw-iron": "35454a", "cw-iron-dark": "27363c", "cw-steel": "63767a",
    "cw-steel-light": "839295", "cw-brass": "b38d53", "cw-brass-light": "d3b578",
    "cw-brass-dark": "8d6b43", "cw-copper": "a56e49", "cw-rust": "815d47",
    "cw-amber": "f2b563", "cw-furnace": "b7693c", "cw-char": "303d3e",
    "cw-patina": "5e827d", "cw-patina-light": "86a19a",
    "cw-oak": "5c4e3e", "cw-wood": "85705a", "cw-grain": "ac916a",
    "cw-canvas": "8b7861", "cw-linen": "c0af90", "cw-workcoat": "6a8076",
    "cw-ore": "bd8d50", "cw-ore-light": "dec28c",
    "cw-grass": "909478", "cw-grass-dark": "72775e", "cw-grass-light": "b2ad8c",
    "water-clockwork": "315b64", "cw-water-deep": "274b5e", "cw-water-light": "538182",
    "cw-shelf": "567c7c", "cw-shallows": "87a29a", "cw-ripple": "b0c6b8",
}


def register_palette():
    PALETTE.update(CLOCKWORK)
    for key in ("cw-iron","cw-iron-dark","cw-steel","cw-steel-light","cw-brass",
                "cw-brass-light","cw-brass-dark","cw-copper","cw-patina"):
        shader = material(key).node_tree.nodes["Principled BSDF"]
        shader.inputs["Metallic"].default_value = .55
        shader.inputs["Roughness"].default_value = .43
    for key,strength in (("cw-amber",1.45),("cw-furnace",.63)):
        mat = material(key)
        shader = mat.node_tree.nodes["Principled BSDF"]
        shader.inputs["Emission Color"].default_value = mat.diffuse_color
        shader.inputs["Emission Strength"].default_value = strength


def ground_height(y,i):
    return -.077+max(0,abs(y)-2.65)*(.029+.021*sin(i*.49))


def plateau(group):
    rings = [[(CENTER[0]+(x-CENTER[0])*s,y*s,ground_height(y*s,i))
              for i,(x,y) in enumerate(rim())] for s in (1,.83,.63,.44,.25,.08)]
    keys = ("cw-earth","cw-earth-light","cw-earth","cw-earth-dark","cw-earth")
    rows(group,"Packed foundry soil",rings,lambda band,i:keys[(i//4+band*2)%len(keys)],floor=True)


def cliffs(group):
    edge = rim()
    profiles = ((1,0),(1.007,-.19),(1.001,-.39),(1.028,-.61),(1.016,-.89),(1.054,-1.60))
    rings = [[(CENTER[0]+(x-CENTER[0])*s,y*s,z+.046*sin(i*.71+b))
              for i,(x,y) in enumerate(edge)] for b,(s,z) in enumerate(profiles)]
    rings[0] = [(x,y,ground_height(y,i)) for i,(x,y) in enumerate(edge)]
    keys = ("cw-rock-light","cw-stratum","cw-rock","cw-rock-dark","cw-rock")
    rows(group,"Banded slate and exposed foundry strata",rings,
         lambda band,i:"cw-rock-dark" if (i+band)%11==0 else keys[band],reverse=True)


def sea(group):
    side = 25
    vertices = [(8+(x-12)*4.2+.15*sin(x+y),(y-11)*4.2+.17*cos(x-y),WATERLINE)
                for y in range(side) for x in range(side)]
    faces = [(i,i+1,i+side+1,i+side) for y in range(side-1) for x in range(side-1)
             for i in (y*side+x,)]
    obj = surface(group,"Cold foundry sea",vertices,faces,["water-clockwork"]*len(faces))
    tones = obj.data.color_attributes.new(name="Water depth",type="FLOAT_COLOR",domain="POINT")
    graph = obj.data.materials[0].node_tree
    attr = graph.nodes.new("ShaderNodeVertexColor")
    attr.layer_name = tones.name
    graph.links.new(attr.outputs["Color"],graph.nodes["Principled BSDF"].inputs["Base Color"])
    low,high = (color(PALETTE[k]) for k in ("cw-water-deep","cw-water-light"))
    for vertex,rgba in zip(obj.data.vertices,tones.data,strict=True):
        x,y,_ = vertex.co
        t = .45+.19*sin(x*.39+y*.41)+.12*cos(x*.54-y*.29)
        rgba.color = tuple(a+(b-a)*t for a,b in zip(low,high,strict=True))


def shelf(group):
    rings = [[(CENTER[0]+(x-CENTER[0])*(s+.007*cos(i+band)),
               y*(s+.008*sin(i*.96+band)),WATERLINE+.014+band*.011)
              for i,(x,y) in enumerate(rim())] for band,s in enumerate((1.15,1.11,1.07,1.024))]
    keys = ("cw-shelf","cw-shallows","cw-rock")
    rows(group,"Foundry coast shallows",rings,lambda band,i:keys[band])
    for i in range(1,len(rings[0]),3):
        x,y,z = rings[1][i]
        a,b,_ = rings[1][(i+1)%len(rings[0])]
        surface(group,"Small broken coastal ripple",[(x,y,z+.016),(a,b,z+.016),
                (a+.027,b+.039,z+.016),(x+.018,y+.021,z+.016)],[(0,1,2,3)],["cw-ripple"])


def road_bed(group):
    box(group,"Compacted slate BattleLine bed",(0,0,-.063),
        (LANE_LENGTH+2.16,LANE_WIDTH+.23,.103),"cw-grout",.040)


def slab(group,variant):
    rng = Random(728+variant)
    x,y = .485,.607
    corners = [(-x+.073,-y),(x-.071,-y),(x,-y+.073),(x,y-.076),
               (x-.080,y),(-x+.072,y),(-x,y-.069),(-x,-y+.078)]
    wear = [(rng.uniform(-.012,.012),rng.uniform(-.010,.010)) for _ in corners]
    vertices = [(a*s+u,b*s+v,z) for z,s in ((-.055,1),(-.020,1),(.004,.972))
                for (a,b),(u,v) in zip(corners,wear,strict=True)]
    faces = [(j*8+i,j*8+(i+1)%8,(j+1)*8+(i+1)%8,(j+1)*8+i) for j in (0,1) for i in range(8)]
    faces.append(tuple(range(16,24)))
    key = ("cw-slab","cw-slab-light","cw-slab","cw-slab-cool")[variant]
    surface(group,"Chamfered quiet slate flagstone",vertices,faces,
            ["cw-grout"]*8+["cw-rock"]*8+[key])


def curb(group):
    box(group,"Seated iron curb",(0,0,-.020),(.93,.247,.103),"cw-iron",.017)
    box(group,"Replaceable slate curb cap",(0,0,.028),(.887,.202,.043),"cw-steel",.014)
    for x in (-.333,.333):
        box(group,"Bronze curb fitting",(x,0,.051),(.060,.181,.021),"cw-brass-dark",.005)
