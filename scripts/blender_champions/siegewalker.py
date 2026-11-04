"""Sculpt the ogre gunner and machine the five shoulder bombard forms."""

from math import cos, exp, sin, tau

from characters import HUMAN_FACE, face_transform

GUN_LENGTH = ((.86, 1.08, .98), (.86, .80, .90))
BORE_Y = .26
GUNNER_FACE = [
    (-.132,.066,.066,.032),(-.117,.094,.090,.024),(-.092,.123,.106,.018),
    (-.066,.134,.114,.008),(-.036,.135,.125,.006),(.002,.139,.128,.002),
    (.038,.141,.124,0),(.067,.139,.120,-.004),(.097,.127,.105,-.011),
    (.127,.100,.086,-.015),(.149,.064,.064,-.021),(.155,.014,.022,-.024),
]


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


def gunner_face(model, eye):
    def planes(point):
        x,y,z = point
        front = max(0,min(1,(z-.022)/.094))**3

        def ridge(a,b,w,h):
            return exp(-((x-a)/w)**2-((y-b)/h)**2)

        nose = .033*ridge(0,.021,.031,.048) + .036*ridge(0,-.012,.047,.026)
        cheek = .012*(ridge(.095,-.014,.030,.029)+ridge(-.095,-.014,.030,.029))
        socket = .010*(ridge(.062,.038,.032,.022)+ridge(-.062,.038,.032,.022))
        lip = .015*ridge(0,-.063,.090,.017) - .008*ridge(0,-.050,.076,.008)
        return x,y,z+front*(nose+cheek-socket+lip)

    model.loft("Sculpted gunner face and squared jaw","Head","ogre",GUNNER_FACE,
               n=26,transform=planes)
    model.tube("Recessed lower lip","Head","ogre_shade",[
        (-.062,-.050,.113),(-.033,-.054,.128),(0,-.052,.137),
        (.036,-.054,.128),(.062,-.050,.113),
    ],[.003,.005,.006,.005,.003],n=6)
    for s in (-1,1):
        model.tube("Fitted lower tusk","Head","bone",[
            (s*.072,-.082,.101),(s*.080,-.054,.123),(s*.077,-.025,.131),
        ],[.019,.014,.0025],n=8)
        model.tube("Sculpted ear rim","Head","ogre",[
            (s*.118,.018,.015),(s*.151,.030,.011),(s*.181,.052,-.003),
        ],[.031,.024,.003],n=8)
        model.ell("Recessed ear","Head","ogre_shade",(s*.148,.033,.026),
                  (.023,.011,.007),segments=8,rings=3)
        model.ell("Aligned eye socket","Head","ogre_shade",(s*.063,.038,.114),
                  (.029,.016,.014),segments=10,rings=4)
        model.ell("Shadowed eye","Head","ink",(s*.062,.038,.126),
                  (.021,.010,.006),segments=9,rings=4)
        model.ell("Warm iris","Head",eye,(s*.059,.038,.132),(.008,.008,.004),segments=8,rings=3)
        model.tube("Raised brow plane","Head","ogre_shade",[
            (s*.029,.052,.126),(s*.061,.058,.122),(s*.101,.052,.098),
        ],[.008,.011,.004],n=7)
        model.ell("Nostril crease","Head","ogre_shade",(s*.027,-.021,.154),
                  (.011,.005,.006),segments=8,rings=3)


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
    for s in (-1,1):
        model.box("Stitched waist keeper","Hips","hide",(s*.094,.072,.148),
                  (.025,.064,.019),bevel=0,rotation=s*.13)
    if longshot:
        model.loft("Convex riveted cuirass", "Chest", "steel", [
            (.057,.219,.174,.027),(.153,.248,.180,.016),(.259,.238,.154,.002),(.302,.204,.144,0),
        ], n=14, opening=0)
        for s in (-1,1):
            model.tube("Rolled cuirass edge","Chest","steel_dark",[
                (s*.087,.269,.154),(s*.139,.241,.142),(s*.175,.182,.129),
            ],[.007,.008,.007],n=6)
            model.ell("Countersunk chest rivet","Chest","brass",
                      (s*.137,.227,.145),(.013,.013,.008),segments=8,rings=3)
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
        model.loft("Bracer cuff",lower,"copper" if tier==2 else "leather",[
            (-.178,.080,.085,.005),(-.158,.086,.089,.005),(-.144,.084,.088,.005),
        ],n=9)
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
    gunner_face(model,"brass")
    metal = "steel" if longshot else "steel_dark"
    trim = "gold" if tier==2 and longshot else "copper"
    model.loft("Fitted artillery helmet","Head",metal,[
        (.071,.140,.131,-.010),(.086,.154,.142,-.010),
        (.124,.149,.138,-.014),(.160,.113,.103,-.019),
        (.182,.058,.062,-.025),(.185,.022,.027,-.026),
    ], n=13)
    model.tube("Rolled helmet visor","Head",metal,[
        (-.140,.083,.056),(-.094,.079,.107),(-.054,.077,.133),
        (0,.077,.143),(.054,.077,.133),(.094,.079,.107),(.140,.083,.056),
    ],[.009,.014,.016,.016,.016,.014,.009],n=7)
    for s in (-1,1):
        model.ell("Temple fastener","Head",trim,(s*.139,.096,.074),
                  (.012,.013,.011),segments=8,rings=3)
    if longshot and tier==1:
        model.tube("Monocle hinge","Head","steel_dark",
                   [(-.131,.085,.086),(-.101,.066,.111),(-.082,.049,.130)],
                   [.009,.008,.006],n=6)
        model.tube("Fitted sighting monocle","Head","brass",
                   [(-.062,.038,.132),(-.062,.038,.147)],[.025,.024],n=12)
        model.ell("Inset smoked lens","Head","steel_dark",(-.062,.038,.148),
                  (.017,.017,.004),segments=10,rings=3)
    if shrapnel and tier==2:
        cheek = [(.086,-.097),(.114,-.094),(.142,-.050),(.136,.008),(.114,.020),(.099,-.029)]
        for s in (-1,1):
            outline = [(s*x,y) for x,y in cheek]
            model.plate("Fitted cheek guard","Head","steel_dark",
                        outline if s>0 else outline[::-1],(.089,.119),bevel=.008)
            model.tube("Small cheek-guard vent","Head","copper",
                       [(s*.106,-.063,.119),(s*.122,-.037,.123)],[.003,.004],n=5)


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


def barrel_surface(model, name, color, x, profile):
    sides = 16
    points = [(x+r*cos(i*tau/sides),BORE_Y+r*sin(i*tau/sides),z)
              for z,r in profile for i in range(sides)]
    faces = [(a*sides+i,a*sides+(i+1)%sides,(a+1)*sides+(i+1)%sides,(a+1)*sides+i)
             for a in range(len(profile)-1) for i in range(sides)]
    model.mesh(name,"WeaponR",color,points,faces)


def bore(model, x, start, mouth, radius, lip, casing, band):
    barrel_surface(model,"Tapered steel barrel",casing,x,[
        (start,radius*1.04),(start+.069,radius*1.06),(mouth-.173,radius*.93),
        (mouth-.102,radius*.96),(mouth-.042,lip*.94),
    ])
    barrel_surface(model,"Machined muzzle collar",band,x,[
        (mouth-.046,lip*.94),(mouth-.032,lip),(mouth-.010,lip),
        (mouth,lip*.95),(mouth,lip*.73),(mouth-.012,lip*.68),
    ])
    barrel_surface(model,"Recessed dark bore liner","ink",x,[
        (mouth-.011,lip*.68),(mouth-.059,lip*.62),(mouth-.151,min(radius*.68,lip*.62)),
    ])
    model.tube("Deep bore floor","WeaponR","ink",
               [(x,BORE_Y,mouth-.161),(x,BORE_Y,mouth-.150)],
               [radius*.70]*2,n=12)
    for z,r in ((start+.090,radius*1.10),(mouth-.177,radius*.98)):
        model.tube("Narrow barrel ferrule","WeaponR",band,
                   [(x,BORE_Y,z-.011),(x,BORE_Y,z+.011)],[r,r],n=12)


def bombard(model, tier, branch):
    node = "WeaponR"
    back = -.32
    flared = tier==1 and branch==1
    muzzle = back+GUN_LENGTH[branch][tier]+(.16 if flared else .08)
    twin = tier==2 and branch==0
    radius = .062 if tier==1 and branch==0 else .078
    if twin:
        for x in (-.074,.074):
            bore(model,x,back,muzzle,.057,.069,"steel_dark","brass")
    else:
        lip = .156 if flared else radius*1.24
        bore(model,0,back,muzzle,radius,lip,"steel_dark","copper" if flared else "brass")
    receiver(model,twin)
    sights(model,muzzle,tier==1 and branch==0,flared,twin)
    if tier==2 and branch==1:
        model.tube("Hammer shell drum",node,"steel_dark",[(-.112,.240,-.029),(.112,.240,-.029)],
                   [.109,.109],n=12)
        for x in (-.116,.116):
            model.ell("Recessed drum cap",node,"iron",(x,.240,-.029),(.012,.079,.079),segments=10)
            model.ell("Drum furnace hub",node,"forge",(x*1.10,.240,-.029),
                      (.009,.031,.031),segments=9,rings=4)


def receiver(model, twin):
    node = "WeaponR"
    model.box("Reciprocating breech housing", node, "steel_dark",
              (0,.253,-.286),(.265 if twin else .176,.154,.192),bevel=.017)
    model.tube("Backblast throat", node, "slag", [(0,.26,-.42),(0,.26,-.349)],
               [.079,.065], n=11)
    model.plate("Carved shoulder stock",node,"darkwood",[
        (.055,-.463),(.157,-.463),(.182,-.343),(.184,.149),(.141,.304),
        (.107,.271),(.106,-.014),(.095,-.261),(.047,-.385),
    ][::-1],(-.057,.057),axis="x",bevel=.016)
    model.box("Steel butt plate",node,"iron",(0,.105,-.463),(.122,.115,.026),bevel=.013)
    model.box("Raised receiver bedding",node,"iron",(0,.170,-.101),(.126,.048,.438),bevel=.010)
    model.tube("Wrapped pistol grip",node,"leather",[
        (0,-.070,-.025),(0,-.018,-.012),(0,.054,.012),(0,.134,.037),
    ],[.032,.036,.036,.041],n=9)
    model.tube("Forged trigger guard",node,"brass",[
        (0,.092,-.025),(0,.037,-.036),(0,-.006,.026),(0,.005,.071),(0,.092,.096),
    ],[.008,.009,.010,.009,.008],n=7)
    model.tube("Support-rail gusset",node,"steel",[
        (.045,.154,.232),(.131,.145,.278),(.220,.134,.335),(.280,.129,.360),
    ],[.028,.024,.020,.019],n=7)
    model.tube("Knurled support grip",node,"wood",[(.28,.047,.36),(.28,.149,.36)],
               [.025,.025],n=9)
    model.box("Loading lever guide",node,"brass",(.306,.104,.400),(.029,.029,.138),bevel=0)
    model.tube("Guide brace",node,"iron",[(.226,.140,.345),(.292,.116,.369)],
               [.016,.016],n=7)
    for s in (-1,1):
        x = s*(.135 if twin else .090)
        for z in (-.337,-.236):
            model.ell("Receiver fastener",node,"brass",(x,.272,z),
                      (.009,.012,.012),segments=8,rings=3)


def sights(model, muzzle, longshot, flared, twin):
    node = "WeaponR"
    if longshot:
        for z in (-.104,.184):
            model.box("Scope foot",node,"iron",(0,.346,z),(.052,.063,.045),bevel=0)
        model.tube("Coaxial telescope body",node,"steel_dark",
                   [(0,.387,-.177),(0,.387,.252)],[.022,.026],n=11)
        for z in (-.172,.242):
            model.tube("Telescope ferrule",node,"brass",[(0,.387,z-.013),(0,.387,z+.013)],
                       [.029,.029],n=11)
        model.ell("Smoked scope objective",node,"ink",(0,.387,.256),
                  (.020,.020,.006),segments=10,rings=3)
        return
    height = .383 if flared else .363
    bridge = .170 if twin else .061
    model.box("Front sight saddle",node,"iron",(0,height-.030,muzzle-.199),
              (bridge,.060,.026),bevel=0)
    model.ell("Front sight bead",node,"brass",(0,height,muzzle-.199),
              (.008,.009,.007),segments=8,rings=3)
    model.box("Rear sight saddle",node,"iron",(0,height-.029,-.204),
              (bridge,.047,.044),bevel=0)
    for x in (-.025,.025):
        model.box("Slotted rear sight ear",node,"steel",(x,height-.006,-.204),
                  (.017,.033,.034),bevel=0)
