"""Shape the slate coast, quiet paved lane and wind-marked sea."""

from math import cos, sin
from random import Random

from mesh import PALETTE, box, color, material, pebble, surface
from terrain import CENTER, LANE_LENGTH, LANE_WIDTH, rim, rows

WATERLINE = -1.28
PALETTE_STORM = {
    "storm-heath": "5a6765", "storm-heath-dark": "505c62", "storm-heath-light": "687471",
    "storm-slate": "48596a", "storm-slate-deep": "354456", "storm-slate-light": "6f8290",
    "storm-slate-edge": "596a76", "storm-shale": "75838c", "storm-lane-bed": "424a58",
    "storm-slab": "7d8997", "storm-slab-cool": "788797", "storm-slab-light": "84909b",
    "storm-iron": "3c4d60", "storm-steel": "6b7f8f", "storm-copper": "a97952",
    "storm-brass": "c0a174", "storm-cyan": "6da6bb", "storm-glass": "8ec5d2",
    "storm-glow": "adf0f3", "storm-amber": "f6bd76", "storm-char": "303d4a",
    "storm-pine": "48655f", "storm-pine-light": "657d73", "storm-pine-dark": "364e4a",
    "storm-grass": "9baca6", "storm-grass-dark": "748887", "storm-grass-light": "b7c1b7",
    "storm-oak": "534f4a", "storm-wood": "76716a", "storm-grain": "a29886",
    "storm-oilcloth": "526e7b", "storm-cloth": "919c97", "storm-ore": "6599ac",
    "storm-ore-light": "a6c5ce", "water-storm": "204052", "storm-sea-deep": "233c56",
    "storm-sea-light": "41687c", "storm-swell": "587d8b", "storm-foam": "98b1b6",
    "storm-shelf": "486878", "storm-shallows": "627f87",
    "storm-cloud": "607186", "storm-cloud-dark": "3c4a60", "storm-cloud-edge": "748498",
}


def register_palette():
    PALETTE.update(PALETTE_STORM)
    for key in ("storm-iron", "storm-steel", "storm-copper", "storm-brass"):
        shader = material(key).node_tree.nodes["Principled BSDF"]
        shader.inputs["Metallic"].default_value = .54
        shader.inputs["Roughness"].default_value = .45
    for key, strength in (("storm-glow", .75), ("storm-amber", 1.4)):
        mat = material(key)
        shader = mat.node_tree.nodes["Principled BSDF"]
        shader.inputs["Emission Color"].default_value = mat.diffuse_color
        shader.inputs["Emission Strength"].default_value = strength
    for key, opacity in (("storm-cloud",.43),("storm-cloud-dark",.57),("storm-cloud-edge",.34)):
        cloud_material(key,opacity)


def cloud_material(key, opacity):
    mat = material(key)
    tree = mat.node_tree
    tree.nodes.remove(tree.nodes["Principled BSDF"])
    clear = tree.nodes.new("ShaderNodeBsdfTransparent")
    tint = tree.nodes.new("ShaderNodeEmission")
    tint.inputs["Color"].default_value = mat.diffuse_color
    blend = tree.nodes.new("ShaderNodeMixShader")
    blend.inputs[0].default_value = opacity
    tree.links.new(clear.outputs[0],blend.inputs[1])
    tree.links.new(tint.outputs[0],blend.inputs[2])
    tree.links.new(blend.outputs[0],tree.nodes["Material Output"].inputs["Surface"])


def heath_height(y, i):
    return -.077 + max(0, abs(y)-2.64) * (.035+.026*sin(i*.37))


def plateau(group):
    bands = [
        [(CENTER[0]+(x-CENTER[0])*s, y*s, heath_height(y*s,i)) for i,(x,y) in enumerate(rim())]
        for s in (1,.83,.65,.43,.26,.08)
    ]
    tones = ("storm-heath","storm-heath","storm-heath-light","storm-heath","storm-heath-dark")
    rows(group,"Wind-scoured heath",bands,lambda band,i: tones[(i//3+2*band)%len(tones)],floor=True)


def cliffs(group):
    edge = rim()
    profiles = ((1,0),(1.009,-.18),(1.014,-.42),(1.002,-.62),(1.044,-.97),(1.050,-1.54))
    bands = [
        [(CENTER[0]+(x-CENTER[0])*s,y*s,z+.068*sin(i*.86+b)) for i,(x,y) in enumerate(edge)]
        for b,(s,z) in enumerate(profiles)
    ]
    bands[0] = [(x,y,heath_height(y,i)) for i,(x,y) in enumerate(edge)]
    keys = ("storm-slate-edge","storm-shale","storm-slate","storm-slate-deep","storm-slate-edge")
    rows(group,"Split blue slate escarpment",bands,
         lambda band,i: "storm-slate-light" if (i+2*band)%11==0 else keys[band],reverse=True)
    for i in (3,9,14,22,29,38,43,49,57,61):
        x,y = edge[i]
        pebble(group,"Sheared rock at the cliff edge",(x,y,-.50),(.48,.32,.81),
               "storm-slate",sides=7,accent="storm-slate-light",phase=.41*i)


def sea(group):
    side = 27
    vertices = [(8+(x-13)*4+.19*sin(x+y),(y-12)*4+.18*cos(x-y),WATERLINE)
                for y in range(side) for x in range(side)]
    faces = [(i,i+1,i+1+side,i+side) for y in range(side-1) for x in range(side-1)
             for i in (y*side+x,)]
    obj = surface(group,"Open dark sea",vertices,faces,["water-storm"]*len(faces))
    tones = obj.data.color_attributes.new(name="Sea depth",type="FLOAT_COLOR",domain="POINT")
    graph = obj.data.materials[0].node_tree
    attribute = graph.nodes.new("ShaderNodeVertexColor")
    attribute.layer_name = tones.name
    graph.links.new(attribute.outputs["Color"],graph.nodes["Principled BSDF"].inputs["Base Color"])
    low,high = (color(PALETTE[k]) for k in ("storm-sea-deep","storm-sea-light"))
    for vertex,rgba in zip(obj.data.vertices,tones.data,strict=True):
        x,y,_ = vertex.co
        t = .46+.19*sin(x*.47+y*.59)+.14*cos(x*.58-y*.36)
        rgba.color = tuple(a+(b-a)*t for a,b in zip(low,high,strict=True))


def shelf(group):
    edge = rim()
    rings = [
        [(CENTER[0]+(x-CENTER[0])*(s+.009*cos(i*.87+band)),
          y*(s+.008*sin(i*1.41+band)),WATERLINE+.015+.012*band)
         for i,(x,y) in enumerate(edge)]
        for band,s in enumerate((1.152,1.112,1.075,1.026))
    ]
    keys = ("storm-shelf","storm-shallows","storm-slate-edge")
    rows(group,"Slate shelf below the surf",rings,lambda band,i:keys[band])
    for i in range(0,len(edge),2):
        a,b = rings[1][i],rings[1][(i+1)%len(edge)]
        x,y,z = a
        u,v,_ = b
        outline = [(x,y,z+.018),(u,v,z+.018),(u+.021,v+.037,z+.018),
                   ((x+u)*.5,(y+v)*.5+.04,z+.018),(x+.029,y+.016,z+.018)]
        surface(group,"Broken tide line across the shale shelf",outline,[(0,1,2,3,4)],
                ["storm-swell" if i%3 else "storm-foam"])


def road_bed(group):
    box(group,"Compacted slate road foundation",(0,0,-.063),
        (LANE_LENGTH+2.16,LANE_WIDTH+.23,.107),"storm-lane-bed",.04)


def slab(group, variant):
    rng = Random(841+variant)
    x,y = .487,.608
    outline = [(-x+.081,-y),(x-.071,-y),(x,-y+.083),(x,y-.074),
               (x-.088,y),(-x+.071,y),(-x,y-.085),(-x,-y+.073)]
    wear = [(rng.uniform(-.012,.012),rng.uniform(-.011,.011)) for _ in outline]
    vertices = [(a*s+u,b*s+v,z) for z,s in ((-.054,1),(-.021,1),(.005,.970))
                for (a,b),(u,v) in zip(outline,wear,strict=True)]
    faces = [(j*8+i,j*8+(i+1)%8,(j+1)*8+(i+1)%8,(j+1)*8+i) for j in (0,1) for i in range(8)]
    faces.append(tuple(range(16,24)))
    tones = ("storm-slab","storm-slab-cool","storm-slab","storm-slab-light")
    surface(group,"Individually dressed silver-blue slate",vertices,faces,
            ["storm-lane-bed"]*8+["storm-slate-edge"]*8+[tones[variant]])


def curb(group):
    box(group,"Slate curb foundation",(0,0,-.021),(.95,.262,.102),"storm-slate-deep",.016)
    box(group,"Beveled silvered curb",(0,0,.027),(.907,.205,.045),"storm-shale",.012)
