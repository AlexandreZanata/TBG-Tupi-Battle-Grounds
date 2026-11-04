"""Sculpt the ogre gunner and machine the five shoulder bombard forms."""

from math import cos, sin, tau

from characters import HUMAN_FACE, face_transform

GUN_LENGTH = ((.86, 1.08, .98), (.86, .80, .90))


def ogre_face(model, brow, eye):
    model.loft("Broad ogre face", "Head", "ogre", HUMAN_FACE, n=19,
               transform=lambda p: face_transform((p[0]*1.29,p[1]*.91,p[2]*1.21), False))
    model.ell("Square underbite", "Head", "ogre_shade", (0,-.078,.068), (.118,.057,.080), segments=12)
    model.ell("Broad crooked nose", "Head", "ogre", (0,-.004,.141), (.039,.037,.043), segments=10)
    model.tube("Underslung mouth", "Head", "hide",
               [(-.062,-.069,.130),(0,-.058,.153),(.057,-.065,.132)], [.007,.009,.006], n=6)
    for s in (-1,1):
        model.tube("Lower tusk", "Head", "bone", [
            (s*.071,-.087,.117),(s*.077,-.047,.143),(s*.069,-.015,.148),
        ], [.022,.015,.003], n=7)
        model.tube("Pointed ogre ear", "Head", "ogre", [
            (s*.103,.015,.011),(s*.146,.026,.008),(s*.187,.062,-.008),
        ], [.037,.028,.003], n=7)
        model.ell("Deep-set eye", "Head", "ink", (s*.064,.032,.128), (.024,.015,.014), segments=9)
        model.ell("Ogre pupil", "Head", eye, (s*.063,.032,.141), (.011,.009,.006), segments=8)
        model.tube("Heavy brow", "Head", brow, [
            (s*.026,.046,.139),(s*.063,.061,.130),(s*.104,.056,.101),
        ], [.012,.021,.010], n=6)


def siegewalker(model, tier, branch, *, production=False):
    longshot = tier > 0 and branch == 0
    shrapnel = tier > 0 and branch == 1
    outfit(model, tier, longshot, shrapnel)
    headgear(model, tier, longshot, shrapnel)
    supply(model, tier, longshot)
    bombard(model, tier, branch)


def outfit(model, tier, longshot, shrapnel):
    model.loft("Heavy ogre torso", "Chest", "ogre", [
        (-.04,.163,.120,.018),(.051,.223,.167,.025),(.148,.241,.169,.020),
        (.260,.233,.151,0),(.324,.208,.128,-.011),(.375,.111,.088,0),
    ], n=14)
    model.loft("Layered leather waist", "Hips", "hide", [
        (-.135,.203,.133,0),(-.087,.224,.151,0),(.066,.193,.146,0),(.096,.175,.136,0),
    ], n=12)
    model.loft("Double leather belt", "Hips", "leather", [
        (.052,.211,.160,0),(.092,.211,.160,0),
    ], n=12)
    model.box("Heavy belt clasp", "Hips", "brass", (0,.073,.165),(.079,.061,.032),bevel=.009)
    if longshot:
        model.loft("Convex riveted cuirass", "Chest", "steel", [
            (.057,.219,.174,.027),(.153,.248,.180,.016),(.259,.238,.154,.002),(.302,.204,.144,0),
        ], n=14, opening=0)
    else:
        model.tube("Bombard harness", "Chest", "leather", [
            (-.158,.323,.090),(-.069,.239,.166),(.042,.145,.195),(.157,.061,.171),
        ], [.025,.029,.029,.026], n=7)
    if shrapnel:
        for i in range(4):
            x = -.105 + i*.065
            y = .105 + i*.047
            model.tube("Bandolier shell", "Chest", "brass", [
                (x+.018,y-.027,.185),(x,y+.020,.199),(x-.009,y+.046,.191),
            ], [.023,.024,.013], n=7)
    for side,s in (("L",1),("R",-1)):
        upper, lower = f"UpperArm{side}", f"Forearm{side}"
        model.tube("Shoulder and neck bridge", "Chest", "ogre", [
            (s*.154,.282,0),(s*.222,.313,0),(s*.284,.319,0),
        ], [.071,.070,.060], n=8)
        model.loft("Ogre bicep", upper, "ogre", [
            (-.252,.068,.075,0),(-.162,.091,.087,.006),(-.067,.107,.093,0),(.033,.079,.074,0),
        ], n=10)
        model.loft("Layered gunner bracer", lower, "steel_dark" if tier==2 else "hide", [
            (-.225,.060,.066,0),(-.159,.083,.086,.005),(-.055,.082,.078,0),(.024,.067,.067,0),
        ], n=10)
        model.ell("Ogre grip", f"Hand{side}", "ogre_shade" if tier<2 else "iron",
                  (0,-.041,.013),(.069,.068,.066),segments=10)
        if side=="R" or shrapnel:
            model.ell("Bombard shoulder saddle", upper, "steel" if longshot else "steel_dark",
                      (s*.019,.034,-.005),(.131,.067,.114),segments=11)
        if shrapnel and side=="L":
            model.tube("Raked shoulder spike", upper, "edge",
                       [(s*.045,.052,.005),(s*.085,.119,.005),(s*.107,.157,-.007)],
                       [.031,.022,.002], n=7)
        for joint, color in ((f"Thigh{side}","hide"),(f"Shin{side}","steel_dark")):
            model.loft("Reinforced leg wrap", joint, color, [
                (-.172,.067,.071,0),(-.101,.081,.084,0),(.020,.086,.087,0),
            ], n=9)
        model.box("Heavy square boot", f"Foot{side}", "hide",
                  (0,-.025,.058),(.161,.090,.220),bevel=.023)
        model.box("Steel boot cap", f"Foot{side}", "iron",
                  (0,-.030,.133),(.161,.072,.084),bevel=.014)


def headgear(model, tier, longshot, shrapnel):
    ogre_face(model, "ogre_shade", "forge" if tier==2 else "brass")
    model.loft("Riveted artillery helmet", "Head", "steel" if longshot else "steel_dark", [
        (.056,.142,.138,-.009),(.083,.152,.150,-.015),
        (.131,.133,.133,-.017),(.160,.072,.084,-.021),(.167,.037,.049,-.024),
    ], n=13)
    model.tube("Helmet rolled visor", "Head", "gold" if tier==2 and longshot else "copper", [
        (-.134,.075,.063),(-.088,.072,.117),(0,.066,.140),(.089,.074,.114),(.134,.075,.063),
    ], [.010,.012,.013,.012,.010], n=7)
    if longshot and tier==1:
        model.tube("Sighting monocle", "Head", "brass",
                   [(-.063,.032,.146),(-.063,.032,.155)], [.030,.030], n=10)
        model.ell("Smoked glass", "Head", "steel_dark", (-.063,.033,.157),(.020,.018,.005),segments=9)
    if shrapnel and tier==2:
        model.plate("Vented cheek guard", "Head", "steel_dark",
                    [(-.126,-.096),(.124,-.096),(.115,-.047),(.081,-.032),(-.082,-.032),(-.116,-.047)],
                    (.144,.166),bevel=.009)
        model.tube("Furnace face vents", "Head", "forge",
                   [(-.070,-.060,.168),(0,-.064,.171),(.070,-.060,.168)], [.005,.008,.005], n=6)


def supply(model, tier, longshot):
    node = "BackSpring"
    color = "steel_dark" if tier==2 else "darkwood"
    model.loft("Powder vessel", node, color, [
        (-.161,.089,.075,-.094),(-.124,.113,.094,-.105),
        (.075,.114,.096,-.105),(.132,.082,.078,-.094),
    ], n=12)
    for y in (-.097,.068):
        model.loft("Powder vessel hoop", node, "brass", [
            (y-.014,.116,.100,-.104),(y+.012,.117,.101,-.104),
        ], n=12)
    if tier==2:
        model.tube("Furnace feed pipe", node, "copper",
                   [(.05,.059,-.119),(.084,.173,-.139),(.089,.234,-.111)], [.028,.034,.030], n=8)
        model.ell("Furnace at the pack outlet", node, "forge", (.087,.232,-.108),(.022,.014,.022),segments=9)
    elif longshot:
        for x in (-.093,0,.092):
            model.tube("Spare long shell", node, "brass",
                       [(x,-.069,-.172),(x,.145,-.172),(x,.205,-.168)], [.026,.029,.005], n=7)


def bore(model, x, start, mouth, radius, lip, casing, band):
    # A recessed dark plug leaves a readable hole at the actual release plane.
    node = "WeaponR"
    profile = [(start,radius*.91),(start+.052,radius),(mouth-.11,radius),
               (mouth-.028,lip),(mouth,lip),(mouth,lip*.68),(mouth-.060,lip*.65)]
    points = [(x+r*cos(i*tau/14),.26+r*sin(i*tau/14),z) for z,r in profile for i in range(14)]
    faces = [(a*14+i,a*14+(i+1)%14,(a+1)*14+(i+1)%14,(a+1)*14+i)
             for a in range(len(profile)-1) for i in range(14)]
    model.mesh("Bored bombard barrel", node, casing, points, faces)
    model.tube("Recessed bore floor", node, "ink", [(x,.26,mouth-.082),(x,.26,mouth-.061)],
               [lip*.64,lip*.64], n=14)
    for z in (start+.17,mouth-.161,mouth-.037):
        r = lip if z==mouth-.037 else radius
        model.tube("Forged barrel band", node, band, [(x,.26,z-.013),(x,.26,z+.013)],
                   [r*1.04,r*1.04], n=12)


def bombard(model, tier, branch):
    node = "WeaponR"
    back = -.32
    muzzle = back+GUN_LENGTH[branch][tier]+(.16 if tier==1 and branch==1 else .08)
    twin = tier==2 and branch==0
    radius = .064 if tier==1 and branch==0 else .084
    if twin:
        for x in (-.067,.067):
            bore(model,x,back,muzzle,.059,.069,"steel_dark","gold")
    else:
        lip = .177 if tier==1 and branch==1 else radius*1.32
        bore(model,0,back,muzzle,radius,lip,"steel_dark" if tier==2 else "copper","brass")
    model.box("Reciprocating breech housing", node, "steel_dark",
              (0,.249,-.305),(.233 if twin else .184,.174,.157),bevel=.022)
    model.tube("Backblast throat", node, "slag", [(0,.26,-.42),(0,.26,-.349)],
               [.091,.066], n=10)
    model.tube("Main grip and stock", node, "darkwood", [(0,-.062,0),(0,.047,0),(0,.205,.018)],
               [.030,.037,.043], n=8)
    model.box("Foregrip rail", node, "steel", (.147,.132,.36),(.308,.039,.124),bevel=.009)
    model.tube("Support hand grip", node, "wood", [(.28,.047,.36),(.28,.149,.36)],
               [.025,.025], n=8)
    model.box("Side loading guide", node, "brass", (.306,.105,.400),(.028,.029,.184),bevel=.005)
    if tier==1 and branch==0:
        model.tube("Longshot telescope", node, "brass", [(.060,.376,.020),(.060,.376,.466)],
                   [.024,.028], n=9)
    if tier==2 and branch==1:
        model.tube("Hammer shell drum", node, "steel_dark", [(-.112,.240,-.029),(.112,.240,-.029)],
                   [.113,.113], n=12)
        for x in (-.116,.116):
            model.ell("Drum furnace hub", node, "forge", (x,.240,-.029),(.012,.060,.060),segments=10)
