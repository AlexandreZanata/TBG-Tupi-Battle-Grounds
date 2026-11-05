"""Tailor Finn's oilskin, creel and rod around the native fish release."""

from math import cos, sin, tau


TIP = .902


def oldfinn(model, tier, branch, *, production=False):
    big = tier > 0 and branch == 0
    net = tier > 0 and branch == 1
    coat(model, tier, big, net)
    face(model, tier, net)
    creel(model, tier, big, net)
    rod(model, tier, big, net)


def coat(model, tier, big, net):
    oil = ("finn_storm" if tier == 1 else "finn_spray") if net else ("oilshade" if big else "oilskin")
    cuff = "finn_storm" if net else "oilshade"
    model.loft("Round waxed fisherman coat", "Chest", oil, [
        (-.056,.132,.102,0),(.043,.182,.145,.018),(.155,.190,.157,.016),
        (.254,.167,.129,0),(.324,.134,.101,0),(.363,.079,.070,0),
    ], n=16, fold=.034)
    model.loft("Split oilskin coat tails", "Hips", oil, [
        (-.163,.155,.107,0),(-.115,.174,.124,0),(-.022,.163,.132,0),(.058,.151,.121,0),
    ], n=14, opening=.14, fold=.07)
    model.loft("Heavy leather belt", "Hips", "hide", [
        (-.010,.163,.130,0),(.005,.168,.138,0),(.041,.167,.137,0),
    ], n=14)
    model.box("Worn brass buckle", "Hips", "brass", (0,.025,.144),(.065,.044,.019),.005)
    if net:
        model.plate("Oilcloth fishing bib", "Chest", "finn_spray" if tier == 1 else "finn_storm",
                    [(-.116,.014),(.113,.014),(.111,.224),(.078,.272),(-.080,.272),(-.112,.224)],
                    (.141,.172),bevel=.009)
    for s,side in ((-1,"R"),(1,"L")):
        arm,fore,hand = f"UpperArm{side}",f"Forearm{side}",f"Hand{side}"
        model.tube("Creel harness strap", "Chest", "leather",[
            (s*.093,.073,.156),(s*.117,.194,.155),(s*.136,.305,.092),(s*.132,.339,-.037),
            (s*.126,.269,-.116)], [.019,.019,.020,.022,.021],n=7)
        model.tube("Filled oilskin sleeve",arm,oil,[(0,.028,0),(-s*.018,-.049,0),(0,-.213,0)],
                   [.060,.076,.051],n=9)
        model.tube("Folded sleeve cuff",fore,cuff,[(0,.024,0),(0,-.088,.008),(0,-.194,0)],
                   [.055,.062,.047],n=9)
        model.ell("Grip mitten",hand,"wine" if big else "hide",(0,-.008,.011),
                  (.046,.057,.043),segments=10,rings=5)
        for joint in (f"Thigh{side}",f"Shin{side}"):
            model.tube("Seafarer trousers",joint,"field_slate",[(0,.027,0),(0,-.092,0),(0,-.180,0)],
                       [.071,.065,.054],n=8)
        model.loft("Wide storm boot",f"Foot{side}","hide",[
            (-.060,.072,.112,.053),(-.047,.080,.118,.054),(.007,.074,.105,.040),(.053,.057,.069,.005),
        ],n=12)
    for y in (.094,.168,.244):
        model.ell("Coat rivet","Chest","edge" if net else "brass",(0,y,.178),(.015,.015,.008),segments=8,rings=4)
    if net:
        for i in range(3):
            x = .131 + i*.013
            points = [(x+.036*cos(j*tau/14),-.060+.064*sin(j*tau/14),.131+i*.009) for j in range(15)]
            model.tube("Working net rope coil","Hips","linen",points,[.007]*15,n=5)
    if big:
        model.ell("Cork hauling float","Hips","brass",(.129,-.063,.115),
                  (.041,.054,.028),segments=9,rings=5)


def face(model, tier, net):
    model.loft("Weathered cheeks","Head","skin",[
        (-.117,.061,.051,.020),(-.071,.111,.080,.018),(.023,.125,.103,0),
        (.084,.118,.092,-.004),(.116,.092,.076,-.011),
    ],n=14)
    model.ell("Salt-reddened nose","Head","wood",(0,-.010,.115),(.041,.035,.036),segments=12)
    for s in (-1,1):
        model.ell("Deep-set eye","Head","ink",(s*.052,.031,.099),(.021,.012,.010),segments=8,rings=4)
        model.tube("Thick pale brow","Head","bone",[(s*.026,.053,.102),(s*.055,.058,.110),(s*.082,.049,.101)],
                   [.012,.015,.007],n=7)
        model.tube("Fork of salt-white beard","Head","linen",[
            (s*.087,-.043,.075),(s*.048,-.083,.129),(s*.057,-.144,.115),(s*.078,-.208,.105),
        ],[.028,.049,.032,.004],n=10)
        model.tube("Swept moustache","Head","bone",[(s*.012,-.041,.140),(s*.042,-.053,.147),(s*.085,-.049,.114)],
                   [.012,.022,.004],n=8)
    def brim(p):
        x,y,z = p
        return x,y+.12*z-.14*max(0,-z),z
    hat = ("finn_storm" if tier == 1 else "finn_spray") if net else "oilskin"
    model.loft("Downturned sou'wester brim","HeadGear",hat,[
        (.055,.188,.200,-.025),(.070,.220,.228,-.033),(.082,.198,.208,-.032),
    ],n=18,transform=brim)
    model.loft("Sou'wester crown","HeadGear",hat,[
        (.077,.147,.131,-.012),(.127,.136,.123,-.024),(.173,.080,.076,-.041),
    ],n=16)
    if net:
        model.loft("Oilskin hat band","HeadGear","hide",[
            (.084,.151,.139,-.015),(.106,.148,.136,-.016),
        ],n=16)
    if net and tier == 2:
        model.ell("Hat-band brass clasp","HeadGear","gold",(.124,.098,.069),
                  (.030,.020,.015),segments=8,rings=4)


def fish(model, at, size, catfish=False):
    x,y,z = at
    node = "BackSpring"
    model.loft("Silver belly of the catch",node,"fish_silver",[
        (y-.102*size,.015*size,.018*size,z),(y-.063*size,.051*size,.045*size,z),
        (y+.028*size,.052*size,.038*size,z),(y+.104*size,.014*size,.018*size,z),
    ],n=10,transform=lambda p:(p[0]+x,p[1],p[2]))
    model.tube("Slate dorsal stripe",node,"fish_back",[(x,y-.068*size,z+.027*size),
               (x,y+.014*size,z+.040*size),(x,y+.105*size,z+.008*size)],
               [.022*size,.034*size,.012*size],n=8)
    for s in (-1,1):
        model.tube("Forked fish tail",node,"fish_back",[
            (x,y+.093*size,z),(x+s*.026*size,y+.131*size,z),(x+s*.052*size,y+.172*size,z),
        ],[.018*size,.026*size,.003],n=7)
        if catfish:
            model.tube("Curved catfish barbel",node,"bone",[
                (x+s*.026*size,y-.084*size,z+.015*size),
                (x+s*.080*size,y-.107*size,z+.059*size),
                (x+s*.119*size,y-.141*size,z+.079*size),
            ],[.008,.006,.002],n=6)
    model.ell("Visible fish eye",node,"ink",(x+.034*size,y-.061*size,z+.029*size),
              (.009*size,)*3,segments=8,rings=4)


def creel(model, tier, big, net):
    node = "BackSpring"
    w = .185 if big else .154
    model.loft("Rounded reed creel",node,"wicker",[
        (-.189,w*.72,.070,-.117),(-.151,w,.092,-.126),(.062,w*.92,.095,-.119),
    ],n=14)
    for y in (-.147,-.090,-.031,.027,.064):
        points = [(w*.99*sin(i*tau/16),y,-.120+.097*cos(i*tau/16)) for i in range(17)]
        model.tube("Split reed weave",node,"wood",points,[.008]*17,n=5)
    fish(model,(.102,.153,-.142),.91 if big else 1)
    if big:
        fish(model,(.230,.218,-.082) if tier==2 else (-.139,.180,-.120),
             1.70 if tier == 2 else 1.20,tier == 2)
    if net:
        model.ell("Gathered fishing net",node,"linen",(-.04,-.190,-.103),
                  (.190,.087,.083),segments=12)
        for s in (-1,1):
            model.tube("Net lash and floats",node,"wood",[
                (s*.141,.020,-.115),(s*.171,-.115,-.181),(s*.132,-.233,-.160),
            ],[.014,.017,.012],n=7)
        if tier == 2:
            fish(model,(-.121,.148,-.120),.95)
            fish(model,(-.225,.113,-.124),.77)
            for i in range(3):
                model.ell("Pale net float",node,"bone",(.178,.03-i*.055,-.143),
                          (.021,.031,.023),segments=8,rings=4)


def rod(model, tier, big, net):
    node = "WeaponR"
    model.tube("Ash fishing rod",node,"wood",[(0,-.156,0),(0,.162,0),(0,.502,-.016),
               (0,.708,-.032),(0,TIP,0)], [.027,.022,.016,.011,.004],n=9)
    model.tube("Leather handle binding",node,"hide",[(0,-.090,0),(0,.117,0)],[.031,.030],n=9)
    for s in (-1,1):
        model.tube("Turned reel flange",node,"gold" if big else "brass",[
            (s*.021,.145,.041),(s*.047,.145,.041)], [.065,.061],n=14)
    model.tube("Wound spool",node,"hide",[(-.027,.145,.041),(.028,.145,.041)],[.049,.049],n=12)
    model.box("Reel crank",node,"iron",(.055,.131,.044),(.036,.076,.018),.004)
    model.ell("Crank knob",node,"wood",(.077,.104,.044),(.018,.016,.016),segments=8,rings=4)
    model.tube("Taut fishing line",node,"linen",[(.020,.154,.097),(0,.404,.051),
               (0,.699,.024),(0,TIP,0)], [.004,.003,.003,.003],n=5)
    if tier:
        model.ell("Knotted cork float",node,"wine" if net else "wood",(0,.745,.056),
                  (.018,.035,.018),segments=8,rings=5)
