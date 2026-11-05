"""Forge the Shieldbearer's armor, sword and five shield silhouettes."""

from characters import CAPES, HUMAN_FACE, face_transform


def shieldbearer(model, tier, branch, *, production=False):
    bulwark = tier > 0 and branch == 0
    vanguard = tier > 0 and branch == 1
    metal = "steel" if bulwark else "steel_dark"
    trim = "gold" if tier == 2 else "brass"
    armor(model, metal, trim, bulwark)
    helmet(model, tier, bulwark, vanguard, trim)
    shield(model, tier, bulwark, vanguard, trim)
    sword(model, tier, vanguard, trim)
    if vanguard:
        model.loft("Split cavalry mantle", "BackSpring", "wine", CAPES, n=12,
                   opening=1.18, fold=.10,
                   transform=lambda p: (p[0] * 1.1, p[1] * (1 + tier * .16), p[2]))


def armor(model, metal, trim, bulwark):
    model.loft("Quilted gambeson", "Chest", "linen", [
        (-.04,.134,.102,0), (.06,.152,.113,0), (.20,.191,.115,0),
        (.296,.181,.107,0), (.348,.097,.077,0),
    ], n=12)
    model.loft("Curved cuirass", "Chest", metal, [
        (.042,.154,.118,0), (.103,.176,.133,0), (.205,.196,.142,0),
        (.268,.192,.126,0), (.300,.158,.108,0),
    ], n=12)
    model.tube("Raised breastplate keel", "Chest", "steel", [
        (0,.063,.124), (0,.142,.138), (0,.215,.143), (0,.281,.120),
    ], [.007,.012,.012,.006], n=5)
    model.loft("Padded skirt", "Hips", "hide", [
        (-.146,.164,.120,0), (-.12,.177,.127,0), (.038,.134,.103,0),
    ], n=12)
    model.loft("Waist belt", "Hips", "leather", [(.012,.146,.112,0),(.055,.142,.110,0)], n=12)
    model.box("Forged belt buckle", "Hips", trim, (0,.033,.114), (.053,.043,.022))
    for side, sign in (("L",1), ("R",-1)):
        model.tube("Padded shoulder bridge", "Chest", "linen", [
            (sign*.122,.271,0), (sign*.177,.299,0), (sign*.244,.303,0),
        ], [.060,.059,.047])
        upper, lower = f"UpperArm{side}", f"Forearm{side}"
        model.loft("Quilted sleeve", upper, "linen", [
            (-.172,.044,.045,0), (-.070,.061,.065,0), (.031,.048,.049,0),
        ], n=9)
        model.ell("Rolled shoulder plate", upper, metal, (sign*.015,-.015,0),
                  (.105 if bulwark else .084,.073,.085), segments=10)
        model.loft("Articulated vambrace", lower, "steel", [
            (-.150,.041,.049,0), (-.108,.055,.060,0), (-.033,.059,.058,0), (.023,.048,.052,0),
        ], n=9)
        model.ell("Gloved grip", f"Hand{side}", "hide", (0,-.015,.008), (.049,.056,.045), segments=9)
        model.loft("Padded breeches", f"Thigh{side}", "hide", [
            (-.194,.050,.053,0), (-.094,.064,.066,0), (.038,.063,.065,0),
        ], n=9)
        model.loft("Steel greave", f"Shin{side}", metal, [
            (-.193,.044,.052,0), (-.113,.055,.063,0), (-.011,.061,.069,0), (.041,.055,.060,0),
        ], n=9)
        model.ell("Steel toe and leather heel", f"Foot{side}", "steel_dark",
                  (0,-.018,.045), (.059,.050,.092), segments=10)
        if bulwark:
            x = sign * .080
            model.plate("Overlapping hip plate", "Hips", metal,
                        [(x-.066,-.145),(x+.058,-.145),(x+.059,.010),(x-.052,.010)],
                        (.107,.136), bevel=.009)


def helmet(model, tier, bulwark, vanguard, trim):
    if bulwark and tier == 2:
        model.loft("Closed great helm", "Head", "steel", [
            (-.116,.108,.116,0), (-.096,.131,.137,0), (.116,.132,.138,0),
            (.158,.112,.118,0), (.172,.060,.065,0),
        ], n=14)
        model.box("Recessed eye slit", "Head", "ink", (0,.044,.131), (.216,.026,.035), bevel=.005)
        model.box("Great helm nose reinforcement", "Head", trim, (0,-.016,.143), (.026,.156,.017))
        return
    model.loft("Sculpted face", "Head", "skin", HUMAN_FACE, n=18,
               transform=lambda p: face_transform(p,False))
    model.loft("Open sallet shell", "Head", "steel", [
        (-.107,.117,.112,-.016), (.019,.133,.136,-.010), (.083,.136,.137,-.004),
        (.112,.126,.130,-.006),
    ], n=13, opening=.84)
    model.loft("Forged sallet crown", "Head", "steel", [
        (.092,.130,.134,-.005), (.119,.128,.132,-.006), (.153,.090,.098,-.012),
        (.174,.031,.041,-.011), (.179,.008,.011,-.011),
    ], n=13)
    model.tube("Rolled brow rim", "Head", trim,
               [(-.111,.079,.089),(-.067,.086,.117),(0,.087,.135),(.067,.086,.117),(.111,.079,.089)],
               [.007]*5, n=6)
    model.plate("Nasal guard", "Head", "steel", [(-.018,.088),(-.012,-.021),(0,-.032),(.012,-.021),(.018,.088)], (.119,.143), bevel=.004)
    for s in (-1,1):
        model.ell("Visible eye", "Head", "ink", (s*.045,.032,.098), (.014,.007,.008), segments=8)
    if bulwark:
        model.loft("Kettle helm rolled brim", "Head", "steel_dark", [
            (.075,.132,.136,0), (.079,.190,.171,0), (.095,.192,.174,0), (.119,.132,.132,0),
        ], n=15)
    if vanguard:
        if tier == 2:
            wing = [(.081,.134),(.119,.143),(.170,.210),(.208,.296),(.154,.262),(.103,.228)]
            for s in (-1,1):
                outline = [(s*x,y) for x,y in wing]
                model.plate("Wing crest", "HeadGear", "linen",
                            outline if s > 0 else outline[::-1], (-.073,-.052), bevel=.005)
        else:
            model.plate("Cavalry plume", "HeadGear", "wine",
                        [(-.032,.125),(.029,.125),(.035,.231),(.016,.310),(-.025,.267)],
                        (-.073,-.052), bevel=.005)


def shield(model, tier, bulwark, vanguard, trim):
    tower = bulwark and tier == 2
    if tower:
        outline = [(-.44,-.18),(-.34,-.235),(.29,-.235),(.36,-.235),(.36,-.13),
                   (.30,-.13),(.30,-.05),(.36,-.05),(.36,.05),(.30,.05),
                   (.30,.13),(.36,.13),(.36,.235),(-.34,.235),(-.44,.18)]
    else:
        height = .33 if bulwark else .27
        width = .20 if bulwark else .17
        outline = [(-height,0),(-height*.62,-width*.62),(-.04,-width),(.15,-width),
                   (height,-width*.55),(height,width*.55),(.15,width),(-.04,width),
                   (-height*.62,width*.62)]
    # Keep the canonical hand-normal (+X), so the rim turns with the shield guard.
    model.plate("Forged shield rim", "OffhandL", trim, outline, (.050,.103), axis="x", bevel=.009)
    field = [(y*.89,z*.84) for y,z in outline]
    model.plate("Recessed shield field", "OffhandL", "wood" if vanguard else "steel_dark",
                field, (.097,.116), axis="x", bevel=.006)
    model.ell("Hammered central shield boss", "OffhandL", "steel",
              (.119,.025,0), (.033,.065,.065), segments=10)
    for z in (-.085,.085):
        model.ell("Shield rivet", "OffhandL", trim, (.123,.13,z), (.009,.012,.012), segments=7)
    model.tube("Shield hand brace", "OffhandL", "hide", [
        (.057,-.070,0), (0,-.040,0), (0,.039,0), (.057,.066,0),
    ], [.021]*4, n=7)


def sword(model, tier, vanguard, trim):
    tip = .82 if vanguard and tier == 2 else .72 if vanguard else .62
    model.tube("Wrapped arming sword grip", "WeaponR", "hide",
               [(0,0,-.092),(0,0,.075)], [.023,.023], n=8)
    model.ell("Counterweight pommel", "WeaponR", trim, (0,0,-.105), (.034,.025,.035), segments=9)
    model.tube("Swept crossguard", "WeaponR", trim, [
        (-.104,0,.082), (-.065,0,.101), (0,0,.091), (.065,0,.101), (.104,0,.082),
    ], [.012,.015,.019,.015,.012], n=7)
    sections = [(.095,.039,.015), (.14,.042,.016), (tip-.107,.027,.012)]
    blade = [(x,y,z) for z,w,h in sections for x,y in ((-w,0),(0,-h),(w,0),(0,h))]
    faces = [(3,2,1,0)]
    for row in (0,4):
        faces.extend((row+i,row+(i+1)%4,row+(i+1)%4+4,row+i+4) for i in range(4))
    faces.extend((8+i,8+(i+1)%4,12) for i in range(4))
    model.mesh("Diamond-section steel blade", "WeaponR", "edge", [*blade,(0,0,tip)], faces, smooth=False)
    if vanguard and tier == 2:
        model.tube("Tempered gold fuller", "WeaponR", "gold",
                   [(0,.017,.17),(0,.016,.39),(0,.014,tip-.13)], [.004,.005,.003], n=5)
