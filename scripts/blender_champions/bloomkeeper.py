"""Build the Bloomkeeper from carved wood, folded leaves and layered flowers."""

from math import cos, sin, tau

from mathutils import Vector

from characters import HUMAN_FACE, face_transform


def bloomkeeper(model, tier, branch, *, production=False):
    flowering = tier > 0 and branch == 0
    bramble = tier > 0 and branch == 1
    thorns = bramble and tier == 2
    dress(model, tier, flowering, bramble)
    head(model, tier, flowering, thorns)
    limbs(model, tier, bramble)
    staff(model, tier, flowering, bramble)
    if flowering and tier == 2:
        bough(model)


def leaf(model, name, node, color, base, tip, width, *, facing=(0,0,1)):
    root, end, normal = Vector(base), Vector(tip), Vector(facing)
    axis = (end - root).normalized()
    side = axis.cross(normal).normalized()
    ridge = side.cross(axis).normalized()
    mid = root.lerp(end, .46)
    vertices = [root, mid + width*side, end, mid - width*side,
                mid + width*.43*ridge, mid - width*.12*ridge]
    faces = [(i,(i+1)%4,4) for i in range(4)]
    faces.extend(((i+1)%4,i,5) for i in range(4))
    model.mesh(name, node, color, vertices, faces)


def flower(model, node, center, radius, *, glow=False):
    x,y,z = center
    for index in range(5):
        angle = (index + .25) * tau / 5
        dx,dy = cos(angle),sin(angle)
        leaf(model, "Folded blossom petal", node, "petal_light" if index%2 else "petal",
             (x+dx*radius*.13,y+dy*radius*.13,z),
             (x+dx*radius,y+dy*radius,z+.030), radius*.34)
    model.ell("Flower heart", node, "sap" if glow else "brass",
              (x,y,z+.009), (radius*.25,radius*.25,radius*.22), segments=9)


def dress(model, tier, flowering, bramble):
    main = "wood" if bramble else "leaf"
    model.loft("Living bark bodice", "Chest", main, [
        (-.041,.095,.081,0),(.055,.118,.093,0),(.149,.138,.116,0),
        (.234,.153,.109,0),(.295,.131,.082,0),(.339,.056,.051,0),
    ], n=14, fold=.07)
    for s in (-1,1):
        leaf(model, "Crossed bodice leaf", "Chest", "pine" if bramble else "sprout",
             (s*.027,.056,.103),(-s*.126,.278,.088),.041)
        model.tube("Stem at shoulder", "Chest", "leaf", [
            (s*.117,.270,0),(s*.176,.298,0),(s*.227,.303,0),
        ], [.038,.041,.033], n=7)
    model.loft("Waist woven from vine", "Hips", "darkwood", [
        (-.025,.124,.103,0),(.011,.120,.104,0),(.045,.110,.090,0),
    ], n=11)
    model.loft("Layer under the petal skirt", "Hips", "pine", [
        (-.171,.162,.110,0),(-.132,.158,.108,0),(.018,.117,.098,0),
    ], n=12)
    for index in range(8):
        angle = index*tau/8
        x,z = sin(angle),cos(angle)
        color = ("wood" if index%2 else "leaf") if bramble else ("petal_light" if index%2 else "petal")
        leaf(model, "Folded skirt lobe", "Hips", color, (x*.105,.010,z*.101),
             (x*(.187+tier*.009),-.212-.018*(index%3),z*.150), .074, facing=(x,0,z))
    if flowering:
        for side in ("L","R"):
            flower(model, f"UpperArm{side}", (0,.053,.040), .071+.009*tier, glow=tier==2)


def head(model, tier, flowering, thorns):
    model.loft("Carved lightwood face", "Head", "linen", HUMAN_FACE, n=18,
               transform=lambda p: face_transform((p[0]*.94,p[1],p[2]),False))
    model.loft("Leafy hair crown", "Head", "pine", [
        (-.094,.112,.099,-.027),(-.045,.130,.124,-.022),(.027,.130,.134,-.015),
        (.092,.124,.125,-.012),(.150,.084,.085,-.008),(.177,.015,.016,-.014),
    ], n=15, opening=.87, fold=.11)
    for s in (-1,1):
        model.ell("Eye carved into wood", "Head", "pine", (s*.045,.030,.099),(.017,.008,.009),segments=9)
        model.ell("Sap in the iris", "Head", "brass", (s*.043,.030,.105),(.006,.005,.005),segments=7)
        leaf(model, "Pointed leaf ear", "Head", "leaf",
             (s*.087,-.013,.010),(s*.179,.062,.014),.031)
        leaf(model, "Trailing hair leaf", "HeadGear", "sprout",
             (s*.061,.082,-.103),(s*.104,-.181,-.126),.050)
        leaf(model, "Framing hair leaf", "Head", "leaf",
             (s*.042,.139,.092),(s*.124,-.056,.068),.034)
    model.tube("Carved smile", "Head", "wood",
               [(-.026,-.063,.103),(0,-.066,.108),(.027,-.058,.103)],[.002,.003,.002], n=5)
    if thorns:
        for s in (-1,1):
            model.tube("Branching thorn crown", "Head", "darkwood", [
                (s*.051,.122,.072),(s*.115,.151,.064),(s*.143,.210,.049),(s*.156,.264,.026),
            ], [.023,.021,.013,.002], n=7)
            model.tube("Split crown thorn", "Head", "wood", [
                (s*.117,.168,.053),(s*.164,.199,.061),(s*.189,.215,.067),
            ], [.012,.009,.002], n=6)
    else:
        for x,y,z in ((-.096,.105,.102),(0,.149,.111),(.089,.115,.106)):
            flower(model, "Head", (x,y,z), .040+.009*tier if flowering else .037)


def limbs(model, tier, bramble):
    for side, s in (("L",1),("R",-1)):
        upper, lower, hand = (f"{part}{side}" for part in ("UpperArm","Forearm","Hand"))
        model.loft("Living branch upper arm", upper, "oldbone", [
            (-.166,.035,.037,0),(-.079,.048,.049,0),(.009,.049,.050,0),
        ], n=8)
        leaf(model, "Shoulder leaf armor", upper, "wood" if bramble else "sprout",
             (-s*.035,.027,.017),(s*.080,-.068,.039),.047)
        model.tube("Tapered forearm", lower, "oldbone", [(0,.017,0),(0,-.064,.004),(0,-.151,.004)],
                   [.043,.038,.027], n=8)
        leaf(model, "Forearm vine", lower, "leaf",
             (0,.014,.041),(s*.037,-.130,.028),.028)
        model.ell("Carved wooden palm", hand, "oldbone", (0,-.022,.018),(.041,.049,.034),segments=9)
        if side == "L":
            for x in (-.022,0,.022):
                model.tube("Blessing fingers", hand, "linen",
                           [(x,-.036,.014),(x,-.074,.015),(x,-.090,.027)],
                           [.010,.008,.004], n=5)
        if bramble:
            for joint,y in ((upper,.003),(lower,-.071)):
                model.tube("Bark armor thorn", joint, "darkwood", [
                    (s*.035,y,.030),(s*.076,y+.031,.035),(s*.097,y+.045,.036),
                ], [.028,.016,.002], n=6)
        model.tube("Gnarled thigh", f"Thigh{side}", "darkwood",
                   [(0,.021,0),(0,-.089,0),(0,-.181,0)], [.052,.046,.038], n=8)
        model.tube("Gnarled shin", f"Shin{side}", "wood",
                   [(0,.019,0),(0,-.109,.004),(0,-.188,.004)], [.040,.037,.040], n=8)
        model.ell("Root foot", f"Foot{side}", "darkwood",
                  (0,-.022,.052),(.054,.048,.092),segments=10)
        if bramble and tier == 2:
            for x in (-.035,.035):
                model.tube("Forward root toe", f"Foot{side}", "wood", [
                    (x*.8,-.015,.046),(x,-.026,.113),(x*1.8,-.041,.158),
                ], [.022,.014,.002], n=6)


def staff(model, tier, flowering, bramble):
    node = "WeaponR"
    model.tube("Twisted branch staff", node, "wood", [
        (-.022,-.226,.032),(0,-.118,.034),(0,.092,.030),(.035,.341,.030),
        (-.027,.572,.022),(.026,.727,.024),(0,.889,.030),
    ], [.022,.023,.022,.024,.020,.024,.016], n=8)
    for s in (-1,1):
        leaf(model, "Staff side leaf", node, "leaf",
             (.021,.690,.024),(s*.146,.759,.044),.044)
    if bramble:
        for i in range(4):
            s = (-1)**i
            y = .22+i*.14
            model.tube("Curled staff thorn", node, "darkwood", [
                (0,y,.026),(s*.047,y+.029,.033),(s*.057,y+.086,.040),
            ], [.018,.012,.002], n=6)
    if bramble and tier == 2:
        model.ell("Amber seed at the cast point", node, "sap", (0,.900,.030),(.047,.053,.045),segments=11)
        for i in range(3):
            angle = i*tau/3
            x,z = cos(angle),sin(angle)
            model.tube("Thorn cage around the seed", node, "darkwood", [
                (.025*x,.813,.030+.025*z),(.067*x,.881,.030+.067*z),
                (.068*x,.933,.030+.068*z),(.033*x,.978,.030+.033*z),
            ], [.021,.017,.012,.002], n=7)
    else:
        flower(model, node, (0,.900,.030), .119 if flowering else .105, glow=flowering)


def bough(model):
    node = "BackSpring"
    model.tube("Living bough on the back spring", node, "wood", [
        (0,-.123,-.039),(0,.145,-.090),(.016,.315,-.151),(0,.447,-.155),
    ], [.042,.037,.029,.011], n=8)
    for x,y in ((-.240,.402),(.216,.450),(0,.538)):
        model.tube("Flowering fork", node, "wood", [
            (0,.227,-.123),(x*.59,y-.074,-.155),(x,y,-.153),
        ], [.024,.018,.005], n=6)
        leaf(model, "Upper bough leaf", node, "leaf", (x,y-.10,-.154),(x+.080,y+.051,-.145),.044)
        flower(model, node, (x,y,-.149), .088, glow=True)
