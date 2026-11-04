"""Editable War Auxiliary construction groups, moving joints and keyframes."""

from math import atan2, cos, hypot, radians, sin, tau
from random import Random

from mathutils import Vector

from hold_parts import braces, disc, empty, layer
from mesh import box, pebble, surface, tube

FPS = 16
END = 29
SEA = -1.28


def key(obj,path,seconds,value):
    setattr(obj,path,value)
    obj.keyframe_insert(data_path=path,frame=round(seconds*FPS)+1)


def scale(obj,seconds,value):
    key(obj,"scale",seconds,(value,)*3)


def pose(obj,seconds,angles):
    key(obj,"rotation_euler",seconds,tuple(radians(v) for v in angles))


def slide(obj,seconds,at):
    key(obj,"location",seconds,at)


def reveal(obj,start,end,*,drop=0):
    home = obj.location.copy()
    scale(obj,0,.0001)
    scale(obj,start,.0001)
    scale(obj,end,1)
    if drop:
        slide(obj,start,home+Vector((0,0,drop)))
        slide(obj,end,home)


def component(group,label,at,build,parent=None):
    parts = layer(group,label)
    root = empty(parts,label+" pivot",at,parent)
    build(parts)
    for obj in tuple(parts.objects):
        if obj!=root:
            obj.parent = root
    return root


def harbor(group,root,bridge,half,seconds):
    """Start with shore piles and sequential treads, then raise the work pad."""
    hx,hy = half
    ax,ay,bx,by = bridge
    dx,dy = bx-ax,by-ay
    span,angle = hypot(dx,dy),atan2(dy,dx)
    c,s = cos(angle),sin(angle)
    approach = layer(group,"Construction · shore bridge")
    approach_end = seconds*.31
    count = max(4,round(span/.17))
    for i in range(count):
        t = (i+.5)/count
        at = (ax+dx*t,ay+dy*t,0)
        def tread(parts,i=i):
            box(parts,"Laid bridge tread",(0,0,-.041),(span/count+.013,.53,.104),
                "wood-end" if i%3==0 else "wood",.014,angle)
        plank = component(approach,"Bridge tread",at,tread,root)
        reveal(plank,.10+approach_end*.78*t,.29+approach_end*.90*t,drop=.23)
    for i,t in enumerate((.12,.52,.91)):
        for side in (-1,1):
            x,y = ax+dx*t-side*.23*s,ay+dy*t+side*.23*c
            def pile(parts):
                tube(parts,"Driven shore post",[(0,0,SEA-.09),(0,0,.32)],[.067,.044],
                     "oak",sides=7,accent="wood")
            post = component(approach,"Bridge support post",(x,y,0),pile,root)
            reveal(post,i*.065,.22+approach_end*.58*t)
    for side in (-1,1):
        def rail(parts,side=side):
            box(parts,"Split timber handrail",(0,0,.224),(span,.057,.064),"oak",.012,angle)
        bar = component(approach,"Bridge handrail",
                        ((ax+bx)/2-side*.23*s,(ay+by)/2+side*.23*c,0),rail,root)
        reveal(bar,approach_end*.75,approach_end+.19)
    pad = layer(group,"Construction · work platform")
    def under(parts):
        for x in (-hx+.19,hx-.19):
            for y in (-hy+.15,hy-.15):
                tube(parts,"Work platform piling",[(x,y,SEA-.08),(x,y,-.04)],
                     [.114,.083],"oak",sides=8,accent="wood")
        for y in (-hy+.16,hy-.16):
            braces(parts,(-hx+.18,y,-1.03),(hx-.18,y,-.26),width=.059)
            braces(parts,(hx-.18,y,-1.01),(-hx+.18,y,-.25),width=.059)
    foundation = component(pad,"Braced pilings",(0,0,0),under,root)
    reveal(foundation,approach_end*.68,approach_end+seconds*.11)
    for i in range(11):
        y = (i-5)*hy*2/11
        def make(parts,i=i):
            box(parts,"Platform deck board",(0,0,-.056),(hx*2,hy*2/11-.009,.112),
                "wood" if i%3 else "wood-end",.014)
        board = component(pad,"Work platform tread",(0,y,0),make,root)
        reveal(board,seconds*.30+i*seconds*.012,seconds*.41+i*seconds*.012,drop=.17)


def work_rig(group,parent,name,at,kind,*,size=1):
    rig = layer(group,name)
    root = empty(rig,name+" root",at,parent)
    root["Animation role"] = kind
    root.scale = (size,)*3
    clothes = "hood" if kind=="miner" else "merchant-cloth"

    def torso(parts):
        tube(parts,"Fitted work coat",[(0,0,-.03),(0,0,.09),(-.022,0,.20),(-.015,0,.28)],
             [.104,.140,.126,.074],clothes,sides=10,accent="oak")
        disc(parts,"Leather utility belt",(0,0,-.003),.123,.053,"wood-dark",axis="z",sides=10)
        box(parts,"Tool pouch",(-.024,-.102,-.04),(.14,.082,.133),"oak",.021)
        box(parts,"Shoulder satchel",(-.113,.056,.105),(.13,.14,.18),"wood-dark",.026)
        braces(parts,(.074,-.087,.224),(.096,.072,.020),key="wood",width=.020)
    chest = component(rig,"Spine",(0,0,.34),torso,root)

    def head(parts):
        pebble(parts,"Sculpted face",(.009,0,.072),(.095,.095,.105),"skin",sides=10)
        pebble(parts,"Nose and cheek",(.092,-.002,.054),(.038,.041,.034),"skin",sides=7)
        for y in (-.056,.056):
            pebble(parts,"Eye in shadow",(.079,y,.095),(.017,.014,.016),"soot",sides=6)
        pebble(parts,"Short beard",(.053,0,.018),(.075,.082,.046),"wood-dark",sides=9)
        if kind=="miner":
            disc(parts,"Metal helmet brim",(-.013,0,.135),.129,.046,"gunmetal",axis="z",sides=12)
            tube(parts,"Domed mining helmet",[(-.015,0,.14),(-.02,0,.19),(-.02,0,.226)],
                 [.110,.086,.037],"gunmetal",12,accent="bronze")
            box(parts,"Forehead lamp",(.097,0,.154),(.062,.083,.063),"bronze",.013)
            disc(parts,"Warm lamp lens",(.130,0,.154),.026,.018,"glow",axis="x",sides=8)
        else:
            disc(parts,"Weathered broad hat brim",(-.019,0,.14),.167,.031,"oak",axis="z",sides=13)
            tube(parts,"Tall travel hat",[(-.02,0,.151),(-.035,0,.228),(-.040,0,.245)],
                 [.103,.081,.063],"oak",sides=12,accent="wood")
            disc(parts,"Hat leather band",(-.026,0,.171),.104,.027,"tile",axis="z",sides=12)
            tube(parts,"Tied coat scarf",[(0,-.074,-.005),(-.075,-.073,-.056),(-.142,-.08,-.103)],
                 [.038,.033,.008],"tile",sides=8)
    neck = component(rig,"Head",(-.010,0,.286),head,chest)

    arms = []
    for side in (-1,1):
        def upper(parts):
            tube(parts,"Tailored sleeve",[(0,0,.01),(.024,0,-.072),(.059,0,-.153)],
                 [.059,.059,.047],clothes,8)
        shoulder = component(rig,"Shoulder",(0,side*.122,.213),upper,chest)
        def lower(parts):
            tube(parts,"Rolled work cuff",[(0,0,.015),(.048,0,-.050),(.121,0,-.108)],
                 [.043,.041,.032],"oak",8)
            pebble(parts,"Work glove",(.125,0,-.112),(.047,.039,.037),"wood-dark",sides=8)
        forearm = component(rig,"Elbow",(.059,0,-.153),lower,shoulder)
        wrist = empty(rig,"Tool grip",(.125,0,-.112),forearm)
        arms.append((shoulder,forearm,wrist))
    legs = []
    for side in (-1,1):
        def leg(parts):
            tube(parts,"Heavy work trousers",[(0,0,0),(-.012,0,-.13),(.01,0,-.258)],
                 [.057,.057,.041],"oak",8)
            box(parts,"Leather work boot",(.05,0,-.271),(.175,.109,.092),"wood-dark",.018)
        legs.append(component(rig,"Hip",(0,side*.066,.321),leg,root))
    return root,chest,neck,arms,legs


def walk(rig,start,end,a,b,heading=0):
    root,chest,_,arms,legs = rig
    a,b = Vector(a),Vector(b)
    for i in range(9):
        u = i/8
        now = start+(end-start)*u
        step = 26*sin(u*2*tau)
        p = a.lerp(b,u)
        p.z += .033*abs(sin(u*2*tau))
        slide(root,now,p)
        pose(root,now,(0,0,heading))
        pose(chest,now,(0,8 if i not in (0,8) else 0,0))
        for j,(arm,_,_) in enumerate(arms):
            angle = (-1 if j==0 else 1)*step
            pose(arm,now,(0,-angle*.52,0))
            pose(legs[j],now,(0,angle,0))


def sweep(group,parent,t,extent,seconds=.82):
    def circle(parts):
        verts = [(r*cos(i*tau/32)*extent[0],r*sin(i*tau/32)*extent[1],0)
                 for r in (.94,1) for i in range(32)]
        faces = [(i,i+32,(i+1)%32+32,(i+1)%32) for i in range(32)]
        surface(parts,"Gold construction sweep",verts,faces,["glow"]*32)
    ring = component(group,"Upgrade light sweep",(0,0,.046),circle,parent)
    for time,value in ((0,.0001),(t-.04,.0001),(t,.45),(t+seconds*.55,1),
                       (t+seconds,.0001)):
        scale(ring,time,value)


def burst(group,parent,at,times,*,coins=False):
    rng = Random(73 if coins else 38)
    for i in range(7 if coins else 5):
        def shape(parts):
            if coins:
                disc(parts,"Flying gold coin",(0,0,0),.050,.016,"gold",sides=8)
            else:
                pebble(parts,"Hot ore chip",(0,0,0),(.026,.019,.035),"glow",sides=5)
        part = component(group,"Payout coin" if coins else "Pick strike spark",at,shape,parent)
        scale(part,0,.0001)
        angle = i*tau/(7 if coins else 5)
        reach = rng.uniform(.12,.34)
        duration = .65 if coins else .24
        for t in times:
            scale(part,t-.06,.0001)
            scale(part,t,1)
            slide(part,t,at)
            slide(part,t+duration*.45,
                  (at[0]+cos(angle)*reach*.65,at[1]+sin(angle)*reach*.65,
                   at[2]+(.36 if coins else .12)))
            pose(part,t,(0,i*40,0))
            pose(part,t+duration,(60,i*40+170,90))
            slide(part,t+duration,(at[0]+cos(angle)*reach,at[1]+sin(angle)*reach,at[2]-.03))
            scale(part,t+duration,.0001)
