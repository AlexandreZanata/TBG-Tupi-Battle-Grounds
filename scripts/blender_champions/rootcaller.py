"""Grow the Rootcaller's jointed bark, exact claw tips and fungal forms."""

from math import cos, sin, tau

from bloomkeeper import leaf


def rootcaller(model, tier, branch, *, production=False):
    roots = tier > 0 and branch == 0
    spores = tier > 0 and branch == 1
    trunk(model, tier, roots, spores)
    face(model, tier, roots, spores)
    limbs(model, tier, roots, spores)
    if spores and tier == 2:
        for x,y,z,radius in ((-.152,.213,-.082,.085),(.145,.277,-.09,.104),(0,.343,-.105,.081)):
            cap(model,"BackSpring",(x,y,z),radius,True)


def trunk(model, tier, roots, spores):
    model.loft("Fluted heartwood trunk", "Chest", "wood", [
        (-.054,.125,.104,.016),(.049,.169,.132,.006),(.153,.179,.134,0),
        (.255,.179,.117,-.009),(.321,.165,.104,-.014),(.369,.089,.081,-.012),
    ], n=14, fold=.10)
    model.loft("Flared root skirt", "Hips", "darkwood", [
        (-.139,.180,.126,0),(-.108,.188,.132,0),(-.031,.149,.116,0),(.032,.130,.101,0),
    ], n=12,fold=.16)
    for s in (-1,1):
        model.tube("Twisting bark ridge", "Chest", "darkwood", [
            (s*.080,-.029,.088),(s*.111,.084,.114),(s*.093,.183,.133),
            (s*.147,.272,.098),(s*.207,.317,.032),
        ], [.026,.029,.024,.033,.018], n=7)
        model.tube("Shoulder branch bridge", "Chest", "wood", [
            (s*.116,.270,0),(s*.179,.310,0),(s*.229,.325,0),
        ], [.050,.048,.040], n=7)
        leaf(model,"Shoulder moss lobe","Chest","sprout" if spores else "leaf",
             (s*.105,.348,.077),(s*(.234+.016*tier),.302,.064),.053)
    if roots:
        for offset in (.060,.168):
            model.tube("Climbing vine", "Chest", "leaf", [
                (-.149,offset+.083,.070),(-.081,offset+.042,.131),
                (.017,offset+.016,.143),(.106,offset-.003,.124),(.160,offset-.016,.062),
            ], [.009,.014,.015,.013,.008], n=7)
    if roots and tier==2:
        model.ell("Dark hollow in the trunk", "Chest", "ink", (0,.177,.135),(.059,.080,.018),segments=10)
        model.ell("Amber knot in the hollow", "Chest", "sap", (0,.178,.155),(.033,.055,.017),segments=10)


def face(model, tier, roots, spores):
    model.loft("Split bark mask", "Head", "wood", [
        (-.152,.058,.048,.036),(-.090,.100,.082,.025),(-.029,.134,.114,.013),
        (.039,.141,.117,.004),(.095,.127,.099,-.005),(.150,.103,.086,-.009),
        (.172,.074,.073,-.014),
    ], n=15, fold=.12)
    for s in (-1,1):
        model.ell("Shadow under the brow", "Head", "ink", (s*.064,.039,.118),
                  (.041,.026,.014),segments=9)
        model.ell("Sap eye", "Head", "sap", (s*.061,.037,.134),(.025,.012,.007),segments=9)
        model.tube("Carved bark brow", "Head", "darkwood", [
            (s*.029,.060,.126),(s*.064,.075,.128),(s*.111,.068,.093),
        ], [.014,.019,.008], n=7)
        leaf(model,"Moss beard lobe","Head","pine",(s*.063,-.058,.100),
             (s*.050,-.173,.128),.044)
    model.tube("Knotted nose", "Head", "wood", [
        (0,.050,.104),(0,.001,.144),(.008,-.035,.136),
    ], [.029,.030,.014], n=7)
    leaf(model,"Moss beard point","Head","leaf",(0,-.047,.118),(.019,-.204,.118),.043)
    if spores:
        cap(model,"HeadGear",(0,.149,-.025),.281 if tier==2 else .220,tier==2)
    else:
        crown(model,tier,roots)


def crown(model, tier, roots):
    node = "HeadGear"
    for i in range(5):
        a = i*tau/5
        x,z = sin(a),cos(a)
        if roots and tier==2:
            end = (x*.165,.289+.022*(i%2),z*.140)
            model.tube("Crown of twisted roots",node,"darkwood", [
                (x*.077,.122,z*.065),(x*.139,.189,z*.107),end,
            ], [.024,.021,.005], n=7)
            model.ell("Sap at a root tip",node,"sap",end,(.013,.013,.013),segments=8)
        else:
            leaf(model,"Leaf crown",node,"sprout" if i%2 else "leaf",
                 (x*.061,.130,z*.055),(x*.148,.205+.029*(i%2),z*.128),
                 .045,facing=(x,0,z))
    if not (roots and tier==2):
        model.tube("New twig at the crown",node,"wood",
                   [(.018,.135,-.025),(.033,.209,-.034),(.077,.251,-.044)],[.023,.016,.003],n=7)


def cap(model, node, center, radius, luminous):
    x,y,z = center
    model.tube("Fungal stem",node,"linen",[(x,y-radius*.27,z),(x,y+radius*.04,z)],
               [radius*.25,radius*.23],n=8)
    model.loft("Pale mushroom gills",node,"linen", [
        (y-radius*.016,radius*.23,radius*.21,z),(y+radius*.087,radius*.94,radius*.86,z),
    ], n=16,transform=lambda p:(p[0]+x,p[1],p[2]))
    model.loft("Wavy mushroom cap",node,"mushroom", [
        (y+radius*.055,radius*.93,radius*.86,z),(y+radius*.116,radius,radius*.91,z),
        (y+radius*.256,radius*.87,radius*.78,z-.012),
        (y+radius*.385,radius*.56,radius*.51,z-.022),(y+radius*.435,radius*.18,radius*.19,z-.018),
    ], n=16,fold=.054,transform=lambda p:(p[0]+x,p[1],p[2]))
    for dx,dz,h in ((-.48,.28,.33),(.44,.23,.35),(.10,.53,.31),(.06,-.40,.39)):
        model.ell("Cap spore spot",node,"sap" if luminous else "linen",
                  (x+radius*dx,y+radius*h,z+radius*dz),
                  (radius*.12,radius*.04,radius*.10),segments=8)


def limbs(model, tier, roots, spores):
    length = (.19,.26,.30)[tier] if roots else .19
    for side,s in (("L",1),("R",-1)):
        upper, lower, hand = (f"{part}{side}" for part in ("UpperArm","Forearm","Hand"))
        grip = "OffhandL" if side=="L" else "WeaponR"
        model.tube("Knotted branch upper arm",upper,"wood", [
            (0,.023,0),(s*.012,-.063,-.005),(-s*.014,-.141,.005),(0,-.213,0),
        ], [.057,.059,.052,.046],n=8)
        model.tube("Bark forearm",lower,"darkwood", [
            (0,.027,0),(s*.016,-.079,.010),(s*.004,-.187,.004),
        ], [.052,.061,.046],n=8)
        model.ell("Root knuckles",hand,"wood",(0,-.035,.013),(.064,.057,.049),segments=10)
        for x in (-.038,0,.038):
            model.tube("Curved root claw",grip,"darkwood", [
                (x,-.045,.016),(x*1.27,-.102,.025),
                (x*1.49,-.060-length*.70,.046),(x*1.53,-.060-length,.040),
            ], [.024,.020,.013,.002],n=7)
        if roots:
            leaf(model,"Forearm vine leaf",lower,"leaf",(-s*.037,-.022,.031),
                 (s*.080,-.120,.057),.037)
        if spores:
            cap(model,upper,(s*.040,.031,.009),.073+.009*tier,tier==2)
        for joint,color in ((f"Thigh{side}","wood"),(f"Shin{side}","darkwood")):
            model.tube("Gnarled leg",joint,color,[(0,.020,0),(s*.010,-.074,.008),(0,-.151,0)],
                       [.063,.054,.052],n=8)
        for x in (-.039,0,.039):
            model.tube("Splayed root toe",f"Foot{side}","wood", [
                (x*.59,-.006,-.023),(x,-.026,.053),(x*1.54,-.042,.135),
            ], [.035,.029,.006],n=7)
        if roots:
            model.tube("Trailing heel root",f"Foot{side}","darkwood", [
                (0,-.012,-.029),(s*.035,-.036,-.089),(s*.050,-.049,-.153),
            ], [.033,.023,.003],n=7)
