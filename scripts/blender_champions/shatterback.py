"""Sculpt a four-legged turtle with a separately detachable slate shell."""

from math import cos, sin, tau


def shatterback(model, tier, branch, *, production=False):
    shards = tier > 0 and branch == 0
    fury = tier > 0 and branch == 1
    carapace(model, tier, shards, fury)
    head(model, tier, shards, fury)
    paws(model)


def carapace(model, tier, shards, fury):
    model.loft("Low terrapin body", "Chest", "moss", [
        (-.151,.271,.385,-.122),(-.082,.413,.522,-.123),(.071,.482,.564,-.105),
        (.208,.445,.501,-.095),(.267,.327,.396,-.088),
    ], n=20, fold=.053)
    model.loft("Exposed inner carapace", "Chest", "pine", [
        (.074,.456,.549,-.106),(.166,.455,.526,-.112),(.280,.372,.442,-.106),
        (.337,.246,.353,-.095),(.354,.091,.159,-.098),
    ], n=20, fold=.046)
    for s in (-1,1):
        model.tube("Ribbed shell rim", "Chest", "oldbone" if shards else "wood", [
            (s*.105,.052,.381),(s*.294,.096,.339),(s*.438,.125,.157),
            (s*.469,.118,-.123),(s*.401,.117,-.402),(s*.195,.093,-.597),
        ], [.031,.046,.047,.048,.046,.029], n=9)
        model.tube("Forelimb shell bridge", "Chest", "moss", [
            (s*.262,.063,.106),(s*.339,.120,.063),(s*.365,.135,.013),
        ], [.099,.105,.095], n=9)
        if fury:
            model.tube("Exposed heated shell seam","Chest","forge",[
                (s*.021,.316,.233),(s*.127,.346,.103),(s*.239,.321,-.042),
                (s*.287,.286,-.226),(s*.242,.238,-.407),
            ], [.008,.014,.018,.012,.005], n=7)
    model.tube("Flexible neck base","Chest","moss",[(0,.134,.197),(0,.180,.284),(0,.201,.361)],
               [.112,.111,.093],n=12)
    model.tube("Tapered armored tail","Hips","pine",[(0,-.046,-.417),(0,-.067,-.575),
               (.048,-.058,-.698),(.077,-.040,-.774)],[.069,.060,.039,.004],n=8)
    if tier:
        model.tube("Front shell clasp", "Chest", "brass" if shards else "iron", [
            (-.154,.158,.383),(0,.195,.407),(.154,.158,.383),
        ], [.018,.024,.018], n=8)


def head(model, tier, shards, fury):
    node = "Head"
    model.loft("Wrinkled terrapin head",node,"moss",[
        (-.113,.049,.061,.096),(-.074,.093,.115,.083),(-.006,.111,.131,.083),
        (.059,.104,.099,.059),(.105,.072,.079,.048),
    ],n=14,fold=.043)
    model.tube("Hooked bone beak",node,"bone",[(0,-.028,.112),(0,-.023,.173),
               (0,-.033,.219),(0,-.058,.244)],[.070,.071,.042,.004],n=10)
    model.tube("Lower horn beak",node,"oldbone",[(0,-.069,.113),(0,-.076,.172),(0,-.065,.209)],
               [.058,.050,.007],n=9)
    for s in (-1,1):
        model.ell("Sheltered reptile eye",node,"ink",(s*.089,.028,.134),(.030,.025,.018),
                  segments=10,rings=5)
        model.ell("Amber reptile iris",node,"forge" if fury else "sap",
                  (s*.099,.025,.147),(.014,.013,.009),segments=8,rings=4)
        model.tube("Heavy shell brow",node,"pine",[(s*.061,.055,.143),
                   (s*.101,.063,.132),(s*.107,.055,.079)],[.021,.025,.011],n=8)
    if tier == 2:
        model.plate("Forged brow shield",node,"brass" if shards else "iron",
                    [(-.063,.069),(0,.116),(.063,.069),(0,.034)],(.091,.119),bevel=.004)


def paws(model):
    for side in ("L","R"):
        for joint,length,r in ((f"UpperArm{side}",.26,.103),(f"Forearm{side}",.232,.070),
                               (f"Thigh{side}",.114,.105),(f"Shin{side}",.114,.079)):
            model.tube("Pillar limb",joint,"moss",[(0,.044,0),(0,-length*.44,.012),(0,-length,0)],
                       [r*.87,r,r*.73],n=10)
        for node in (f"Hand{side}",f"Foot{side}"):
            model.loft("Broad planted paw",node,"pine",[
                (-.060,.101,.128,.058),(-.044,.119,.135,.058),(.015,.105,.125,.052),(.068,.068,.083,.033),
            ],n=12)
            for x in (-.073,0,.073):
                model.tube("Blunt terrapin claw",node,"bone",[
                    (x,-.009,.148),(x,-.003,.205),(x,0 if node=="HandR" and x==0 else -.025,.240),
                ],[.029,.025,.002],n=7)


def plate(model, tier, branch, index):
    node = "Chest"
    side = -1 if index%2==0 else 1
    front = index<2
    x,z = side*.223, .137 if front else -.282
    shards = tier>0 and branch==0
    fury = tier>0 and branch==1
    rx,rz = .223, .215 if front else .237
    n = 9
    rings = ((.80,.161),(1,.228),(.97,.270),(.79,.350),(.43,.402),(.12,.425))
    points = []
    for scale,y in rings:
        for i in range(n):
            angle = tau*i/n
            ripple = 1+.074*sin(3*angle+index*.91)+.051*cos(5*angle)
            points.append((x+rx*scale*ripple*sin(angle),
                           y-side*.094*scale*sin(angle)+.010*cos(3*angle),
                           z+rz*scale*ripple*cos(angle)))
    quads = [(r*n+i,r*n+(i+1)%n,(r+1)*n+(i+1)%n,(r+1)*n+i)
             for r in range(len(rings)-1) for i in range(n)]
    model.mesh("Beveled detachable slate plate",node,"slag" if fury else "oldbone" if shards else "slate",
               points,quads+[tuple(reversed(range(n))),tuple(range(len(points)-n,len(points)))])
    model.tube("Mineral growth on slate",node,"ember_stone" if fury else "brass" if shards else "leaf",[
        (x-side*.129,.362,z+.088),(x-side*.045,.426,z+.107),(x+.026,.434,z+.033),
    ],[.013,.024,.009],n=7)
    if shards:
        rise = .246 if tier==2 else .143
        for offset in (-.064,.073):
            model.tube("Grown shell shard",node,"bone",[
                (x+offset,.398,z-.031),(x+offset,.442,z-.043),
                (x+offset+side*.026,.441+rise*.51,z-.077),
                (x+offset+side*.061,.441+rise,z-.098),
            ],[.044,.040,.023,.003],n=8)
    if fury:
        model.tube("Fracture across the plate",node,"ember_stone",[
            (x-side*.150,.356,z+.075),(x-side*.057,.414,z+.015),(x+.031,.430,z+.046),
            (x+side*.072,.414,z-.048),(x+side*.123,.366,z-.061),
        ],[.006,.012,.015,.012,.006] if tier==2 else [.004,.006,.008,.006,.004],n=6)
        if tier==2:
            model.tube("Raised obsidian ridge",node,"iron",[
                (x+side*.143,.290,z+.089),(x+side*.103,.376,z+.024),
                (x+side*.056,.438,z-.032),(x+side*.038,.456,z-.119),
            ],[.028,.043,.034,.004],n=8)
