"""Shape the warm desert plateau, sandstone cliffs and clear coastal shallows."""

from math import cos, sin
from random import Random

from mesh import PALETTE, box, color, material, surface
from terrain import CENTER, LANE_LENGTH, LANE_WIDTH, rim, rows

WATERLINE = -1.28
DESERT = {
    "dr-sand": "bca078", "dr-sand-light": "cdb18a", "dr-sand-dark": "a88d69",
    "dr-stone": "bc966b", "dr-stone-light": "d4b082", "dr-stone-dark": "9f7d57",
    "dr-stratum": "c4a587", "dr-shadow": "6e614f", "dr-grout": "9b835f",
    "dr-slab": "c7b28f", "dr-slab-light": "ceba9c", "dr-slab-cool": "bfb299",
    "dr-ivory": "e2cc9e", "dr-turquoise": "3d918e", "dr-turquoise-light": "7ab5a7",
    "dr-turquoise-dark": "286961", "dr-gold": "bb9051", "dr-gold-light": "d6b576",
    "dr-gold-dark": "956c3f", "dr-bronze": "87734f", "dr-amber": "f4ca7c",
    "dr-palm": "59735a", "dr-palm-light": "82906a", "dr-palm-dark": "42644e",
    "dr-trunk": "8b7151", "dr-fruit": "7c5b41", "dr-reed": "a5a16b",
    "dr-clay": "b37350", "dr-clay-light": "d2976c", "dr-clay-dark": "8e5d45",
    "dr-wood": "927554", "dr-oak": "6c5943", "dr-grain": "b49b71",
    "dr-canvas": "d3b57f", "dr-linen": "e3d0aa", "dr-cloth": "a88d58",
    "dr-ore": "4f9d95", "dr-ore-light": "91c5b4",
    "water-desert": "30789a", "dr-sea-deep": "2b698c", "dr-sea-light": "5a9eaf",
    "dr-shelf": "72a5ac", "dr-shallows": "a3c6b9", "dr-ripple": "c8ded0",
}


def register_palette():
    PALETTE.update(DESERT)
    for key in ("dr-gold","dr-gold-light","dr-gold-dark","dr-bronze"):
        shader = material(key).node_tree.nodes["Principled BSDF"]
        shader.inputs["Metallic"].default_value = .43
        shader.inputs["Roughness"].default_value = .47
    shader = material("dr-turquoise").node_tree.nodes["Principled BSDF"]
    shader.inputs["Roughness"].default_value = .39
    mat = material("dr-amber")
    shader = mat.node_tree.nodes["Principled BSDF"]
    shader.inputs["Emission Color"].default_value = mat.diffuse_color
    shader.inputs["Emission Strength"].default_value = .92


def ground_height(y,i):
    return -.078+max(0,abs(y)-2.66)*(.028+.021*sin(i*.46))


def plateau(group):
    rings = [[(CENTER[0]+(x-CENTER[0])*s,y*s,ground_height(y*s,i))
              for i,(x,y) in enumerate(rim())] for s in (1,.84,.65,.44,.26,.08)]
    shades = ("dr-sand","dr-sand-light","dr-sand","dr-sand-dark","dr-sand")
    rows(group,"Wind-shaped coastal sand",rings,
         lambda band,i:shades[(i//4+band*2)%len(shades)],floor=True)


def cliffs(group):
    edge = rim()
    profiles = ((1,0),(1.009,-.19),(.997,-.43),(1.032,-.65),(1.025,-.96),(1.051,-1.59))
    rings = [[(CENTER[0]+(x-CENTER[0])*s,y*s,z+.044*sin(i*.79+band))
              for i,(x,y) in enumerate(edge)] for band,(s,z) in enumerate(profiles)]
    rings[0] = [(x,y,ground_height(y,i)) for i,(x,y) in enumerate(edge)]
    keys = ("dr-stone-light","dr-stratum","dr-stone","dr-stone-dark","dr-shadow")
    rows(group,"Weathered bands of sandstone",rings,
         lambda band,i:"dr-stone" if band<4 and (i+band)%9==0 else keys[band],reverse=True)


def sea(group):
    side = 25
    verts = [(8+(x-12)*4.2+.16*sin(x+y),(y-11)*4.2+.16*cos(x-y),WATERLINE)
             for y in range(side) for x in range(side)]
    faces = [(i,i+1,i+side+1,i+side) for y in range(side-1) for x in range(side-1)
             for i in (y*side+x,)]
    obj = surface(group,"Clear blue desert sea",verts,faces,["water-desert"]*len(faces))
    tones = obj.data.color_attributes.new(name="Coastal depth",type="FLOAT_COLOR",domain="POINT")
    graph = obj.data.materials[0].node_tree
    attr = graph.nodes.new("ShaderNodeVertexColor")
    attr.layer_name = tones.name
    graph.links.new(attr.outputs["Color"],graph.nodes["Principled BSDF"].inputs["Base Color"])
    low,high = (color(PALETTE[key]) for key in ("dr-sea-deep","dr-sea-light"))
    for vertex,rgba in zip(obj.data.vertices,tones.data,strict=True):
        x,y,_ = vertex.co
        t = .46+.19*sin(x*.33+y*.43)+.14*cos(x*.52-y*.31)
        rgba.color = tuple(a+(b-a)*t for a,b in zip(low,high,strict=True))


def shelf(group):
    rings = [[(CENTER[0]+(x-CENTER[0])*(s+.006*cos(i+band)),
               y*(s+.008*sin(i*.94+band)),WATERLINE+.015+band*.011)
              for i,(x,y) in enumerate(rim())] for band,s in enumerate((1.148,1.107,1.068,1.024))]
    keys = ("dr-shelf","dr-shallows","dr-stratum")
    rows(group,"Pale sand under the shallows",rings,lambda band,i:keys[band])
    for i in range(1,len(rings[0]),3):
        x,y,z = rings[1][i]
        a,b,_ = rings[1][(i+1)%len(rings[0])]
        surface(group,"Broken desert coastline ripple",[(x,y,z+.016),(a,b,z+.016),
                (a+.027,b+.033,z+.016),(x+.016,y+.023,z+.016)],[(0,1,2,3)],["dr-ripple"])


def road_bed(group):
    box(group,"Packed sand under the BattleLine",(0,0,-.064),
        (LANE_LENGTH+2.16,LANE_WIDTH+.23,.103),"dr-grout",.039)


def slab(group,variant):
    rng = Random(835+variant)
    x,y = .485,.607
    outline = [(-x+.076,-y),(x-.070,-y),(x,-y+.071),(x,y-.074),
               (x-.082,y),(-x+.074,y),(-x,y-.069),(-x,-y+.079)]
    wear = [(rng.uniform(-.012,.012),rng.uniform(-.010,.010)) for _ in outline]
    verts = [(a*s+u,b*s+v,z) for z,s in ((-.055,1),(-.021,1),(.004,.972))
             for (a,b),(u,v) in zip(outline,wear,strict=True)]
    faces = [(j*8+i,j*8+(i+1)%8,(j+1)*8+(i+1)%8,(j+1)*8+i) for j in (0,1) for i in range(8)]
    faces.append(tuple(range(16,24)))
    key = ("dr-slab","dr-slab-light","dr-slab","dr-slab-cool")[variant]
    surface(group,"Dressed muted sandstone flagstone",verts,faces,
            ["dr-grout"]*8+["dr-stone-dark"]*8+[key])


def curb(group):
    box(group,"Sandstone curb footing",(0,0,-.021),(.93,.248,.103),"dr-stone-dark",.018)
    box(group,"Chipped pale sandstone curb cap",(0,0,.028),(.89,.204,.044),"dr-stone-light",.014)
