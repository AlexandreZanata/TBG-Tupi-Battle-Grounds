"""Forge the Bombard's foundry armor and joint-local hip mortar."""

from math import cos, sin, tau


def bombard(model, tier, branch, *, production=False):
    heavy = tier > 0 and branch == 1
    heat = tier > 0 and branch == 0
    shell = "steel_dark" if heavy else "copper"
    trim = "edge" if heavy else "brass"
    armor(model, shell, trim, heavy, tier)
    head(model, shell, trim, heavy, heat, tier)
    limbs(model, shell, trim, heavy)
    supplies(model, shell, trim, heat, tier)
    mortar(model, shell, trim, heavy, heat, tier)


def armor(model, shell, trim, heavy, tier):
    model.loft("Quilted foundry coat", "Chest", "hide", [
        (-.054,.176,.122,0),(.047,.213,.143,0),(.175,.235,.157,0),
        (.266,.246,.141,0),(.328,.193,.122,0),(.364,.103,.094,0),
    ], n=14, fold=.035)
    model.loft("Hammered cuirass", "Chest", shell, [
        (.048,.187,.146,.012),(.076,.226,.167,.012),(.173,.245,.174,.004),
        (.268,.250,.154,.002),(.317,.194,.127,.005),(.335,.131,.111,.005),
    ], n=16)
    model.loft("Rolled breastplate edge", "Chest", trim, [
        (.067,.219,.165,.013),(.076,.230,.171,.013),(.091,.231,.171,.012),
    ], n=16)
    for side in (-1, 1):
        model.tube("Reinforced shoulder seam", "Chest", "leather", [
            (side*.257,.291,-.017),(side*.220,.305,.097),(side*.170,.240,.151),
            (side*.156,.143,.169),(side*.127,.070,.154),
        ], [.033,.031,.022,.020,.017], n=7)
        for x,y,z in ((.156,.135,.161),(.181,.236,.119),(.092,.079,.168)):
            model.ell("Hand-peened cuirass rivet", "Chest", trim, (side*x,y,z),
                      (.014,.014,.012), segments=8, rings=4)
        if heavy:
            model.plate("Overlapping hull tasset", "Hips", shell,
                        [(side*.110,-.117),(side*.215,-.107),(side*.219,.028),(side*.099,.035)],
                        (.080,.123), bevel=.008)
    model.loft("Split leather waist apron", "Hips", "hide", [
        (-.130,.189,.118,0),(-.101,.208,.133,0),(-.040,.206,.144,0),(.026,.203,.133,0),
    ], n=12, fold=.08)
    model.loft("Mortar belt", "Hips", "leather", [
        (-.008,.199,.145,0),(.003,.216,.151,0),(.049,.216,.151,0),(.056,.201,.145,0),
    ], n=14)
    model.box("Foundry belt buckle", "Hips", trim, (0,.027,.151),(.077,.056,.024),.009)
    if heavy and tier == 2:
        model.plate("Faceted breastplate keel", "Chest", "steel",
                    [(-.107,.102),(.107,.102),(.139,.230),(0,.274),(-.139,.230)],
                    (.125,.181), bevel=.012)


def head(model, shell, trim, heavy, heat, tier):
    model.loft("Square ogre jaw and brow", "Head", "ogre", [
        (-.125,.061,.072,.029),(-.096,.104,.098,.025),(-.047,.125,.111,.013),
        (.026,.130,.111,0),(.081,.124,.099,-.009),(.122,.097,.085,-.009),
    ], n=16)
    model.loft("Ribbed foundry hood", "HeadGear", "hide", [
        (-.093,.105,.106,-.027),(.003,.134,.112,-.025),(.100,.136,.115,-.016),
        (.130,.117,.099,-.015),(.152,.074,.075,-.018),
    ], n=14, opening=.94)
    model.loft("Beveled gunner helmet", "HeadGear", shell, [
        (.065,.134,.119,-.003),(.080,.146,.127,-.002),(.130,.135,.114,-.012),
        (.163,.096,.082,-.016),(.176,.058,.060,-.015),
    ], n=16)
    model.tube("Helmet crest seam", "HeadGear", trim,
               [(0,.082,.123),(0,.144,.087),(0,.174,.042),(0,.178,-.043)],
               [.015,.013,.010,.007], n=7)
    model.ell("Ogre nose", "Head", "ogre_shade", (0,-.007,.120),(.038,.038,.030), segments=10)
    model.ell("Mouth crease", "Head", "ink", (0,-.071,.119),(.059,.011,.009), segments=10, rings=4)
    for side in (-1, 1):
        x = side*.064
        model.tube("Short ivory tusk", "Head", "bone",
                   [(side*.065,-.084,.115),(side*.072,-.064,.131),(side*.074,-.043,.126)],
                   [.015,.011,.002], n=7)
        model.ell("Goggle cup", "Head", "hide", (x,.035,.112),(.051,.046,.025),segments=12)
        model.ell("Machined lens bezel", "Head", trim, (x,.036,.135),(.043,.039,.017),segments=12)
        lens = "forge" if heat and tier == 2 else "copper" if heavy else "darkwood"
        model.ell("Smoked amber lens", "Head", lens, (x,.036,.148),(.032,.029,.009),segments=12)
        model.ell("Lens glint", "Head", "gold", (x-.009,.046,.156),(.009,.005,.002),segments=8,rings=3)
    model.tube("Goggle bridge", "Head", trim,
               [(-.024,.039,.134),(0,.047,.145),(.024,.039,.134)], [.010,.012,.010], n=7)
    if heavy:
        for side in (-1,1):
            model.plate("Visor cheek plate", "HeadGear", shell,
                        [(side*.091,-.027),(side*.132,-.031),(side*.151,.061),(side*.098,.078)],
                        (.081,.113), bevel=.008)


def limbs(model, shell, trim, heavy):
    for side, sign in (("R",-1),("L",1)):
        upper, fore, hand = f"UpperArm{side}", f"Forearm{side}", f"Hand{side}"
        model.tube("Broad coat sleeve", upper, "leather",
                   [(0,.041,0),(0,-.066,0),(sign*.008,-.182,.008),(0,-.342,0)],
                   [.075,.105,.086,.055], n=10)
        model.loft("Rounded foundry shoulder", upper, shell, [
            (-.084,.092,.091,0),(-.065,.122,.116,0),(.005,.120,.110,0),(.061,.081,.079,0),
        ], n=12)
        model.tube("Work glove sleeve", fore, "hide",
                   [(0,.022,0),(0,-.089,.003),(0,-.206,0),(0,-.307,0)],
                   [.061,.074,.060,.049], n=10)
        model.loft("Riveted forearm guard", fore, "steel" if heavy else trim, [
            (-.233,.054,.049,0),(-.208,.061,.056,0),(-.106,.080,.066,0),(-.082,.074,.063,0),
        ], n=10)
        model.ell("Leather grip fist", hand, "hide", (0,-.012,.015),(.059,.062,.056),segments=10)
        model.tube("Thumb over the grip", hand, "ogre_shade",
                   [(sign*.040,-.026,.030),(sign*.030,.017,.051),(sign*.005,.022,.050)],
                   [.020,.017,.010], n=7)
        model.tube("Heavy trousers", f"Thigh{side}", "cloth", [(0,.020,0),(0,-.152,0),(0,-.184,0)],
                   [.087,.080,.067], n=10)
        model.loft("Tapered iron greave", f"Shin{side}", "steel_dark", [
            (-.173,.065,.058,0),(-.133,.074,.066,0),(-.002,.081,.072,0),(.031,.065,.055,0),
        ], n=10)
        model.loft("Reinforced boot", f"Foot{side}", "hide", [
            (-.056,.091,.132,.053),(-.035,.097,.134,.055),(.010,.086,.115,.040),(.060,.070,.073,0),
        ], n=12)
        model.box("Toe plate", f"Foot{side}", shell, (0,-.023,.124),(.169,.052,.101),.011)


def bomb(model, node, x, y, z):
    model.ell("Cast bomb shell", node, "slag", (x,y,z),(.049,.051,.049),segments=10,rings=5)
    model.tube("Twisted bomb fuse", node, "linen",
               [(x,y+.043,z),(x+.004,y+.068,z),(x+.018,y+.073,z)], [.010,.008,.005], n=6)
    model.ell("Brass fuse collar", node, "brass", (x,y+.038,z),(.020,.015,.020),segments=8,rings=4)


def supplies(model, shell, trim, heat, tier):
    bomb(model, "Hips", .162,-.050,.101)
    if tier == 0:
        return
    if heat:
        for x in (-.122,0,.122):
            bomb(model, "Back", x,.070,-.103)
        model.box("Ammunition rack", "Back", shell, (0,.023,-.086),(.327,.063,.118),.011)
        if tier == 2:
            model.box("Furnace pack", "Back", "slag", (0,-.084,-.080),(.224,.143,.123),.014)
            for x in (-.061,0,.061):
                model.box("Furnace vent", "Back", "forge", (x,-.085,-.147),(.014,.084,.007),.002)
    else:
        model.loft("Rear ammunition drum", "Back", shell, [
            (-.183,.103,.082,-.068),(-.158,.127,.109,-.080),
            (.058,.127,.109,-.080),(.086,.103,.084,-.067),
        ], n=14)
        for y in (-.151,.045):
            model.loft("Ammunition drum binding", "Back", trim, [
                (y-.014,.129,.111,-.080),(y+.014,.129,.111,-.080),
            ], n=14)


def bore(model, name, x, swatch, rings):
    n = 16
    points = [(x + radius*cos(tau*i/n), .10 + radius*sin(tau*i/n), z)
              for z,radius in rings for i in range(n)]
    quads = [(row*n+i,row*n+(i+1)%n,(row+1)*n+(i+1)%n,(row+1)*n+i)
             for row in range(len(rings)-1) for i in range(n)]
    model.mesh(name, "WeaponR", swatch, points, quads)


def mortar(model, shell, trim, heavy, heat, tier):
    end, radius = ((.45,.100),(.51,.112))[tier-1] if heat else (
        (.47,.070) if heavy and tier == 2 else (.43,.088)
    )
    barrels = (-.080,.080) if heavy and tier == 2 else (0,)
    model.tube("Right-hand mortar grip", "WeaponR", "darkwood",
               [(0,-.067,-.016),(0,-.020,-.007),(0,.056,.018)], [.025,.025,.029], n=8)
    model.tube("Fore support grip", "WeaponR", "darkwood",
               [(0,-.054,.181),(0,-.030,.240),(0,-.031,.283)], [.025,.031,.023], n=8)
    model.box("Mortar cradle", "WeaponR", "slag", (0,.004,.149),(.123,.073,.269),.011)
    for x in barrels:
        bore(model, "Forged mortar body", x, shell,
             [(-.074,.025),(-.056,radius*.78),(.030,radius*.89),(.219,radius*.81),
              (end-.102,radius*.90),(end-.053,radius)])
        bore(model, "Machined open muzzle lip", x, trim,
             [(end-.066,radius*.99),(end-.050,radius*1.10),(end-.012,radius*1.12),
              (end,radius*.96),(end,radius*.71)])
        bore(model, "Recessed mortar chamber", x, "ink",
             [(end,radius*.71),(end-.034,radius*.67),(end-.150,radius*.55),(end-.166,.003)])
        bore(model, "Reinforced trunnion ring", x, trim,
             [(.137,radius*.83),(.144,radius*.94),(.179,radius*.94),(.187,radius*.83)])
        if heat and tier == 2:
            bore(model, "Hot collar under the muzzle", x, "forge",
                 [(end-.078,radius*.982),(end-.065,radius*1.015)])
    if heavy and tier == 1:
        model.plate("Mortar hull shield", "WeaponR", "steel",
                    [(-.144,.031),(.145,.031),(.157,.169),(.103,.210),(-.103,.210),(-.157,.169)],
                    (.112,.143), bevel=.010)
