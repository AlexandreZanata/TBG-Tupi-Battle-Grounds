"""Build the Farwatch's fitted watch coat and socket-local arbalest."""

from characters import CAPES, HOOD, HUMAN_FACE, face_transform


def farwatch(model, tier, branch, *, production=False):
    marksman = tier > 0 and branch == 0
    wounding = tier > 0 and branch == 1
    head(model, tier, marksman, wounding)
    coat(model, tier, marksman, wounding)
    arbalest(model, tier, marksman, wounding)


def head(model, tier, marksman, wounding):
    model.loft("Carved field watcher's face","Head","skin",HUMAN_FACE,n=19,
               transform=lambda p:face_transform((p[0]*.93,p[1],p[2]),False))
    if marksman and tier==2:
        model.loft("Wide buckled field hat","Head","hide",[
            (.067,.124,.116,0),(.071,.186,.161,-.008),(.085,.187,.164,-.008),
            (.096,.133,.134,-.013),(.154,.106,.100,-.016),(.172,.057,.060,-.016),
        ],n=15)
        model.plate("Bent feather","HeadGear","linen",
                    [(.084,.104),(.111,.105),(.172,.218),(.190,.269),(.139,.245),(.108,.177)],
                    (-.039,-.024),bevel=.004)
    else:
        model.loft("Low weather hood","Head","plum" if wounding else "field_slate",
                   HOOD,n=16,opening=.80,fold=.069)
    for s in (-1,1):
        model.ell("Farwatch eye","Head","forge" if tier==2 and wounding else "ink",
                  (s*.044,.031,.100),(.015,.008,.007),segments=8,rings=4)
        model.tube("Narrow brow","Head","hide",
                   [(s*.022,.050,.109),(s*.047,.054,.100),(s*.068,.049,.088)],
                   [.0025,.004,.0025],n=5)
    if wounding:
        model.loft("Folded scarf over the jaw","Head","hide",[
            (-.113,.078,.057,.030),(-.081,.104,.080,.028),(-.045,.107,.092,.019),
            (-.018,.103,.102,.009),(-.011,.083,.092,.005),
        ],n=13,fold=.07)


def coat(model, tier, marksman, wounding):
    cloth = "plum" if wounding else "field_slate"
    model.loft("Fitted double-breasted coat","Chest",cloth,[
        (-.045,.123,.100,0),(.032,.143,.109,0),(.140,.163,.118,0),
        (.248,.179,.109,0),(.308,.143,.094,0),(.363,.081,.073,0),
    ],n=14,fold=.045)
    model.loft("Riding coat tails","Hips",cloth,[
        (-.217,.176,.115,-.008),(-.186,.193,.130,-.003),(-.041,.147,.116,0),
        (.043,.132,.101,0),
    ],n=14,opening=.23,fold=.078)
    model.loft("Turned coat collar","Chest","linen" if not wounding else "hide",[
        (.276,.104,.091,0),(.315,.111,.092,0),(.355,.093,.080,0),
    ],n=11,opening=.40)
    model.loft("Thin utility belt","Hips","hide",[(.009,.148,.116,0),(.044,.146,.114,0)],n=12)
    model.box("Bolt pouch","Hips","leather",(.104,-.039,.095),(.096,.100,.054),
              bevel=.008,rotation=.21)
    for s in (-1,1):
        model.tube("Turned coat lapel","Chest","hide",[
            (s*.031,.095,.105),(s*.048,.213,.114),(s*.086,.302,.081),
        ],[.005,.010,.009],n=6)
        for y in (.095,.144,.194):
            model.ell("Coat button","Chest","brass",(s*.028,y,.119),(.007,.007,.006),
                      segments=7,rings=3)
    if tier==2:
        model.loft("Weather cape","BackSpring","wine" if wounding else "leather",
                   CAPES,n=12,opening=1.32,fold=.10)
    for side,s in (("L",1),("R",-1)):
        upper,fore,hand = (f"{part}{side}" for part in ("UpperArm","Forearm","Hand"))
        model.tube("Sleeve attachment","Chest",cloth,[
            (s*.127,.270,0),(s*.176,.291,0),(s*.221,.306,0),
        ],[.052,.049,.043],n=7)
        model.loft("Cut wool upper sleeve",upper,cloth,[
            (-.229,.046,.050,0),(-.118,.060,.063,0),(-.029,.070,.064,0),(.025,.046,.047,0),
        ],n=9)
        model.loft("Leather archery forearm guard",fore,"hide",[
            (-.202,.039,.044,0),(-.158,.053,.059,0),(-.041,.057,.057,0),(.022,.048,.051,0),
        ],n=9)
        model.ell("Gripping glove",hand,"leather",(0,-.017,.014),(.046,.052,.043),segments=9)
        for joint,color in ((f"Thigh{side}","hide"),(f"Shin{side}","leather")):
            model.tube("Fitted riding leg",joint,color,[(0,.020,0),(0,-.093,0),(0,-.191,0)],
                       [.055,.053,.045],n=8)
        model.ell("Closed leather boot",f"Foot{side}","hide",(0,-.024,.048),
                  (.054,.045,.092),segments=10)
    model.tube("Rolled leather bolt case","Back","hide",[
        (-.018,-.154,-.069),(.021,-.030,-.078),(.067,.130,-.083),
    ],[.041,.053,.049],n=9)
    for x in (.030,.057,.081):
        model.tube("Spare bolt","Back","wood",[(x,-.023,-.074),(x+.025,.232,-.084)],
                   [.005,.004],n=5)
        model.plate("Neutral bolt fletching","Back","linen",
                    [(x+.018,.183),(x+.038,.191),(x+.032,.233),(x+.023,.230)],
                    (-.086,-.079),bevel=0)


def arbalest(model, tier, marksman, wounding):
    node = "WeaponR"
    trim = "gold" if marksman and tier==2 else "steel"
    half = .31 if marksman and tier==2 else .25
    model.plate("Carved arbalest stock",node,"darkwood",[
        (-.020,-.166),(.060,-.137),(.070,.039),(.040,.515),
        (-.010,.518),(-.034,.335),(-.023,.099),(-.061,-.087),
    ][::-1],(-.032,.032),axis="x",bevel=.011)
    model.tube("Wrapped arbalest grip",node,"leather",[
        (0,-.093,-.043),(0,-.028,-.026),(0,.020,-.006),
    ],[.025,.026,.027],n=8)
    model.box("Fore-stock under the support hand",node,"wood",(.010,.003,.141),
              (.081,.056,.185),bevel=.009)
    model.box("Forged bolt track",node,trim,(0,.035,.297),(.058,.021,.445),bevel=.005)
    model.tube("Trigger guard",node,"brass",[
        (0,-.004,-.024),(0,-.049,.009),(0,-.048,.058),(0,-.006,.080),
    ],[.007]*4,n=6)
    for s in (-1,1):
        points = [(0,.051,.474),(s*half*.42,.053,.463),(s*half*.77,.052,.429),(s*half,.052,.376)]
        model.tube("Tapered spring-steel bow limb",node,"edge",points,
                   [.030,.024,.016,.010],n=8)
        x,y,z = points[-1]
        model.tube("Taut linen drawstring",node,"linen",[(x,y,z),(0,.052,.272)],
                   [.004,.004],n=5)
    model.tube("Loaded bolt shaft",node,"wood",[(0,.050,.252),(0,.050,.535)],
               [.005,.006],n=6)
    model.tube("Loaded bolt point",node,"forge" if wounding and tier==2 else "edge",
               [(0,.050,.530),(0,.050,.549),(0,.050,.580)],[.013,.012,.001],n=7)
    model.tube("Winding crank axle",node,"brass",[(.031,.018,.021),(.075,.018,.021)],
               [.018,.018],n=8)
    model.tube("Turned crank handle",node,"iron",[(.069,.018,.021),(.071,-.019,.002),(.095,-.019,.002)],
               [.010,.010,.009],n=6)
    if marksman:
        for z in (.085,.279):
            model.box("Scope saddle",node,"iron",(0,.079,z),(.043,.069,.034),bevel=.004)
        model.tube("Centered arbalest scope",node,"steel_dark",[(0,.127,.041),(0,.127,.366)],
                   [.021,.025],n=10)
        model.tube("Objective bezel",node,trim,[(0,.127,.347),(0,.127,.375)],
                   [.030,.029],n=10)
        model.ell("Smoked objective glass",node,"ink",(0,.127,.376),
                  (.022,.022,.005),segments=9,rings=3)
