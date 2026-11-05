"""Grow Rotcap's asymmetric mushroom anatomy around the native spit anchor."""

from math import cos, sin, tau

from rootcaller import cap as small_cap


MOUTH = (0, -.1365*.38, .1365*1.02)


def rotcap(model, tier, branch, *, production=False):
    toxin = tier > 0 and branch == 0
    spores = tier > 0 and branch == 1
    stem(model, tier, toxin, spores)
    face(model, toxin and tier == 2)
    main_cap(model, tier, toxin)
    if toxin:
        for x in (-.143,.143):
            flask(model,x,tier)
    if spores and tier == 2:
        for x,y,z,radius in ((-.128,.139,-.135,.115),(.161,.192,-.110,.141),(-.023,.260,-.144,.099)):
            small_cap(model,"BackSpring",(x,y,z),radius,True)
        for x,y,z in ((-.172,.096,-.172),(.172,.135,-.182),(.002,.053,-.224)):
            pod(model,"BackSpring",(x,y,z),.045)


def stem(model, tier, toxin, spores):
    model.loft("Pear-shaped fungal stem", "Chest", "linen", [
        (-.072,.125,.092,.003),(.012,.174,.127,.010),(.101,.207,.149,.018),
        (.193,.191,.135,.006),(.265,.142,.111,-.014),(.321,.109,.102,-.013),
        (.354,.077,.078,0),
    ], n=16, fold=.068)
    model.loft("Scalloped stem collar", "Chest", "bone", [
        (.251,.150,.122,-.003),(.261,.170,.134,.003),(.288,.175,.131,-.002),
        (.309,.137,.112,-.004),(.318,.107,.100,-.012),
    ], n=16, fold=.095)
    model.loft("Fibrous waist", "Hips", "oldbone", [
        (-.075,.147,.108,0),(-.047,.173,.127,0),(.017,.164,.125,0),(.057,.132,.104,0),
    ], n=12, fold=.075)
    for x in (-.118,-.061,.027,.092):
        model.tube("Growing stem seam", "Chest", "bone", [
            (x*.80,.003,.118),(x,.081,.145),(x*.88,.170,.151),(x*.66,.238,.124),
        ], [.006,.012,.010,.004], n=6)
    for side,sign in (("L",1),("R",-1)):
        upper,fore,hand = f"UpperArm{side}",f"Forearm{side}",f"Hand{side}"
        model.tube("Shoulder stem bridge", "Chest", "linen",
                   [(sign*.099,.282,.011),(sign*.177,.274,.007),(sign*.243,.303,.007)],
                   [.059,.054,.042], n=8)
        model.tube("Tapered upper stem", upper, "bone",
                   [(0,.026,0),(sign*.014,-.059,.016),(sign*.010,-.120,.008),(0,-.175,0)],
                   [.051,.060,.047,.033], n=9)
        model.tube("Bent forearm sprout", fore, "linen",
                   [(0,.026,0),(sign*.024,-.052,.014),(sign*.017,-.109,.006),(0,-.161,0)],
                   [.038,.051,.042,.036], n=9)
        model.ell("Fungal mitten", hand, "oldbone", (0,-.024,.009),(.047,.058,.045),segments=10)
        model.tube("Short stem thumb", hand, "bone",
                   [(-sign*.025,-.010,.023),(-sign*.042,-.031,.038),(-sign*.037,-.061,.039)],
                   [.021,.017,.008], n=7)
        for joint, color in ((f"Thigh{side}","oldbone"),(f"Shin{side}","linen")):
            model.tube("Short stem leg", joint, color,
                       [(0,.019,0),(sign*.010,-.076,.014),(0,-.161,0)], [.063,.062,.051], n=8)
        model.loft("Splayed fungal pad", f"Foot{side}", "mushroom", [
            (-.060,.074,.098,.042),(-.049,.096,.113,.047),(.010,.084,.100,.044),(.049,.048,.053,0),
        ], n=12, fold=.085)
        if spores:
            small_cap(model,upper,(sign*.032,.066,-.006),.089+.016*tier,tier==2)
    if toxin:
        model.loft("Spore-vial girdle", "Hips", "pine", [
            (-.025,.169,.133,0),(-.016,.176,.138,0),(.024,.172,.135,0),(.036,.161,.121,0),
        ], n=14)


def face(model, bright):
    model.loft("Soft stem face", "Head", "linen", [
        (-.130,.072,.060,.015),(-.100,.111,.086,.022),(-.052,.133,.102,.017),
        (.007,.140,.114,.003),(.063,.143,.112,-.005),(.103,.128,.098,-.003),
        (.133,.104,.089,-.006),
    ], n=16, fold=.040)
    x,y,z = MOUTH
    model.ell("Recessed throat", "Head", "ink", (x,y,z-.017),(.046,.037,.010),
              segments=14,rings=5)
    lip = [(x+.046*cos(tau*i/20),y+.037*sin(tau*i/20),z-.004) for i in range(21)]
    model.tube("Spore nozzle rim", "Head", "oldbone", lip, [.011]*len(lip), n=7)
    for sign in (-1,1):
        model.ell("Half-lidded eye socket", "Head", "hide", (sign*.064,.030,.105),
                  (.029,.024,.013),segments=10,rings=5)
        model.ell("Fungal eye", "Head", "venom" if bright else "bone",
                  (sign*.059,.027,.118),(.017,.014,.006),segments=10,rings=5)
        model.ell("Narrow pupil", "Head", "ink", (sign*.057,.028,.124),(.005,.009,.002),
                  segments=8,rings=4)
        model.tube("Overgrown eyelid", "Head", "bone",
                   [(sign*.035,.043,.109),(sign*.062,.048,.113),(sign*.094,.042,.096)],
                   [.008,.010,.006], n=7)


def cap_surface(angle, ratio, radius, height):
    ripple = 1 + .048*cos(5*angle) + .039*sin(3*angle+.8)
    return (
        radius*ratio*ripple*sin(angle) + .019*(1-ratio),
        .101 + height*radius + radius*ratio*(.044*sin(angle+.4)+.026*cos(5*angle)),
        radius*.89*ratio*ripple*cos(angle)-.026,
    )


def main_cap(model, tier, toxin):
    node = "HeadGear"
    radius = .410 if toxin and tier == 2 else .353
    skin = ("wine" if tier == 1 else "violet_dark") if toxin else "mushroom"
    n = 32
    rings = [(.010,-.032),(.29,-.039),(.78,-.009),(.97,.001),(1,.048),
             (.95,.150),(.82,.289),(.58,.401),(.30,.450),(.010,.462)]
    points = [cap_surface(tau*i/n,r,radius,h) for r,h in rings for i in range(n)]
    faces = [(row*n+i,row*n+(i+1)%n,(row+1)*n+(i+1)%n,(row+1)*n+i)
             for row in range(len(rings)-1) for i in range(n)]
    # Keep the brim skin and gill membrane as independently editable pieces.
    skin_points = points[n*3:]
    skin_faces = [tuple(i-n*3 for i in face) for face in faces[n*3:]]
    model.mesh("Sculpted fungal cap skin",node,skin,skin_points,
               skin_faces+[tuple(range(len(skin_points)-n,len(skin_points)))])
    model.mesh("Fungal cap gill membrane",node,"oldbone",points[:n*4],
               faces[:n*3]+[tuple(reversed(range(n)))])
    for i in range(24):
        angle = tau*i/24
        rib = [cap_surface(angle+turn,r,radius,h) for turn,r,h in
               ((0,.29,-.054),(.047,.55,-.043),(.078,.81,-.022),(.078,.955,-.010))]
        model.tube("Radial gill",node,"bone" if i%3==0 else "mushroom",rib,
                   [.004,.009,.009,.002],n=5)
    for angle,ratio,height,width in (
        (.23,.79,.294,.10),(1.01,.54,.413,.11),(1.89,.81,.291,.082),
        (2.69,.59,.401,.11),(3.72,.75,.338,.12),(4.35,.46,.428,.075),
        (5.01,.84,.265,.088),(5.72,.51,.415,.104),(.70,.20,.461,.082),
    ):
        model.ell("Raised spore freckle",node,"venom" if toxin else "bone",
                  cap_surface(angle,ratio,radius,height),
                  (radius*width,radius*.026,radius*width*.89),segments=8,rings=4)
    if toxin and tier == 2:
        for angle in (.51,1.36,2.55,3.87,5.49):
            x,y,z = cap_surface(angle,.91,radius,.002)
            model.tube("Hanging venom bead",node,"venom",
                       [(x,y+.015,z),(x,y-.023,z),(x*.98,y-.051,z*.98),(x*.96,y-.080,z*.96)],
                       [.018,.017,.020,.003],n=7)


def flask(model, x, tier):
    node = "Hips"
    model.tube("Vial leather loop",node,"leather",
               [(x,.027,.097),(x,.014,.151),(x,-.019,.153)], [.014,.017,.015],n=7)
    model.loft("Banded ceramic toxin vial",node,"pine", [
        (-.102,.019,.022,.148),(-.091,.035,.033,.148),(-.046,.035,.034,.148),
        (-.026,.017,.018,.148),(-.013,.017,.018,.148),
    ],n=10,transform=lambda p:(p[0]+x,p[1],p[2]))
    model.ell("Toxin in the vial opening",node,"venom" if tier==2 else "sprout",
              (x,-.062,.177),(.021,.028,.009),segments=8,rings=4)


def pod(model, node, center, radius):
    x,y,z = center
    model.ell("Ribbed spore pod",node,"sprout",center,(radius,radius*1.16,radius),segments=10)
    for sign in (-1,1):
        model.tube("Bioluminescent spore seam",node,"sap",[
            (x+sign*radius*.35,y+radius*.88,z-radius*.30),
            (x+sign*radius*.50,y,z-radius*.84),
            (x+sign*radius*.28,y-radius*.71,z-radius*.50),
        ],[.004,.007,.002],n=6)
