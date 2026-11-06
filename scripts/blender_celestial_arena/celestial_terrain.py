"""Shape the ivory BattleLine, terraced coast and clear blue sea."""

from math import cos, sin
from random import Random

from mesh import PALETTE, box, color, material, surface
from terrain import CENTER, LANE_LENGTH, LANE_WIDTH, rim, rows

WATERLINE = -1.28
CELESTIAL = {
    "cel-grass": "839478", "cel-grass-light": "96a486", "cel-grass-dark": "748b78",
    "cel-stone": "c5bdab", "cel-ivory": "e2dac4", "cel-ivory-shade": "b4aca0",
    "cel-cliff": "9b9fa6", "cel-shadow": "4b596b", "cel-grout": "7f796f",
    "cel-slab": "c4c1b5", "cel-slab-light": "cfcbc0", "cel-slab-cool": "bfc2bb",
    "cel-gold": "bd9255", "cel-gold-light": "dbba79", "cel-gold-dark": "8d6743",
    "cel-azure-deep": "244d72", "cel-azure": "34678c", "cel-azure-light": "548fb0",
    "cel-crystal": "4689bf", "cel-crystal-light": "9ed0e3", "cel-crystal-dark": "356aa1",
    "cel-lit": "a9e0ee", "cel-flame": "f8da97",
    "cel-leaf": "5f816d", "cel-leaf-light": "83a084", "cel-leaf-dark": "416a60",
    "cel-bloom": "d7b4c2", "cel-bloom-light": "e9d3d4", "cel-soil": "776e5d",
    "cel-oak": "63544b", "cel-wood": "907861", "cel-grain": "b89b76",
    "cel-cloth": "e6d6b8", "cel-canvas": "517595",
    "water-celestial": "397d9e", "cel-sea-deep": "296c91", "cel-sea-light": "51a0b0",
    "cel-shelf": "5e9dad", "cel-shelf-light": "82b6b7", "cel-ripple": "a8d3d5",
    "cel-waterfall": "82bbcc", "cel-fall-shadow": "5596b2", "cel-foam": "c3e1dd",
}


def register_palette():
    PALETTE.update(CELESTIAL)
    for key in ("cel-gold","cel-gold-light","cel-gold-dark"):
        shader = material(key).node_tree.nodes["Principled BSDF"]
        shader.inputs["Metallic"].default_value = .47
        shader.inputs["Roughness"].default_value = .42
    for key in ("cel-crystal","cel-crystal-light","cel-crystal-dark"):
        shader = material(key).node_tree.nodes["Principled BSDF"]
        shader.inputs["Roughness"].default_value = .32
        shader.inputs["Coat Weight"].default_value = .30
    for key,strength in (("cel-lit",.71),("cel-flame",1.04)):
        mat = material(key)
        shader = mat.node_tree.nodes["Principled BSDF"]
        shader.inputs["Emission Color"].default_value = mat.diffuse_color
        shader.inputs["Emission Strength"].default_value = strength


def ground_height(y,i):
    return -.077+max(0,abs(y)-2.65)*(.032+.021*sin(i*.51))


def plateau(group):
    bands = [[(CENTER[0]+(x-CENTER[0])*s,y*s,ground_height(y*s,i))
              for i,(x,y) in enumerate(rim())] for s in (1,.86,.68,.48,.26,.08)]
    keys = ("cel-grass","cel-grass-light","cel-grass","cel-grass-dark","cel-grass")
    rows(group,"High garden meadow",bands,lambda band,i:keys[(i//4+band*2)%len(keys)],floor=True)


def cliffs(group):
    edge = rim()
    profiles = ((1,0),(1.003,-.22),(1.029,-.29),(1.013,-.57),(1.041,-.68),(1.036,-1.04),(1.052,-1.60))
    bands = [[(CENTER[0]+(x-CENTER[0])*s,y*s,z+.033*sin(i*.72+b)) for i,(x,y) in enumerate(edge)]
             for b,(s,z) in enumerate(profiles)]
    bands[0] = [(x,y,ground_height(y,i)) for i,(x,y) in enumerate(edge)]
    keys = ("cel-stone","cel-ivory","cel-ivory-shade","cel-stone","cel-cliff","cel-shadow")
    rows(group,"Pale terraced coastal bluffs",bands,
         lambda band,i:"cel-stone" if band in (2,4) and (i+band)%8==0 else keys[band],reverse=True)


def sea(group):
    side = 25
    vertices = [(8+(x-12)*4.2+.17*sin(x+y),(y-11)*4.2+.17*cos(x-y),WATERLINE)
                for y in range(side) for x in range(side)]
    faces = [(i,i+1,i+side+1,i+side) for y in range(side-1) for x in range(side-1)
             for i in (y*side+x,)]
    obj = surface(group,"Open celestial sea",vertices,faces,["water-celestial"]*len(faces))
    tones = obj.data.color_attributes.new(name="Sea depth",type="FLOAT_COLOR",domain="POINT")
    tree = obj.data.materials[0].node_tree
    attribute = tree.nodes.new("ShaderNodeVertexColor")
    attribute.layer_name = tones.name
    tree.links.new(attribute.outputs["Color"],tree.nodes["Principled BSDF"].inputs["Base Color"])
    low,high = (color(PALETTE[k]) for k in ("cel-sea-deep","cel-sea-light"))
    for vertex,rgba in zip(obj.data.vertices,tones.data,strict=True):
        x,y,_ = vertex.co
        t = .43+.17*sin(x*.29+y*.41)+.16*cos(x*.61-y*.27)
        rgba.color = tuple(a+(b-a)*t for a,b in zip(low,high,strict=True))


def shelf(group):
    edge = rim()
    bands = [[(CENTER[0]+(x-CENTER[0])*(s+.006*cos(i+band)),y*(s+.007*sin(i*.96+band)),
               WATERLINE+.015+band*.011) for i,(x,y) in enumerate(edge)]
             for band,s in enumerate((1.145,1.106,1.067,1.024))]
    keys = ("cel-shelf","cel-shelf-light","cel-cliff")
    rows(group,"Sunlit sand under the shallows",bands,lambda band,i:keys[band])
    for i in range(2,len(edge),3):
        x,y,z = bands[1][i]
        a,b,_ = bands[1][(i+1)%len(edge)]
        surface(group,"Thin coastline ripple",[(x,y,z+.015),(a,b,z+.015),(a+.035,b+.027,z+.015),
                (x+.016,y+.023,z+.015)],[(0,1,2,3)],["cel-ripple"])


def road_bed(group):
    box(group,"Recessed limestone lane bed",(0,0,-.066),
        (LANE_LENGTH+2.16,LANE_WIDTH+.23,.104),"cel-grout",.041)


def slab(group,variant):
    rng = Random(621+variant)
    x,y = .486,.606
    outline = [(-x+.075,-y),(x-.074,-y),(x,-y+.069),(x,y-.073),
               (x-.083,y),(-x+.070,y),(-x,y-.067),(-x,-y+.080)]
    wear = [(rng.uniform(-.011,.011),rng.uniform(-.010,.010)) for _ in outline]
    verts = [(a*s+u,b*s+v,z) for z,s in ((-.055,1),(-.021,1),(.004,.973))
             for (a,b),(u,v) in zip(outline,wear,strict=True)]
    faces = [(j*8+i,j*8+(i+1)%8,(j+1)*8+(i+1)%8,(j+1)*8+i) for j in (0,1) for i in range(8)]
    faces.append(tuple(range(16,24)))
    key = ("cel-slab","cel-slab-light","cel-slab","cel-slab-cool")[variant]
    surface(group,"Chamfered pale limestone flagstone",verts,faces,
            ["cel-grout"]*8+["cel-ivory-shade"]*8+[key])


def curb(group):
    box(group,"Dressed ivory curb base",(0,0,-.020),(.93,.244,.102),"cel-ivory-shade",.017)
    box(group,"Separate polished curb cap",(0,0,.028),(.89,.206,.046),"cel-stone",.017)
