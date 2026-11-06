"""Model the Longaxe's quilted armor and five forged polearm heads."""

from characters import CAPES, HUMAN_FACE, face_transform

HEAD_Y = 2.40 * .72


def longaxe(model, tier, branch, *, production=False):
    executioner = tier > 0 and branch == 0
    sweeper = tier > 0 and branch == 1
    garment(model, tier, executioner, sweeper)
    helmet(model, tier, executioner, sweeper)
    poleaxe(model, tier, executioner, sweeper)


def garment(model, tier, executioner, sweeper):
    cloth = "wine" if sweeper else "ochre_cloth"
    model.loft("Quilted polearm jacket","Chest",cloth,[
        (-.039,.119,.092,0),(.055,.150,.111,0),(.154,.178,.118,0),
        (.259,.181,.112,-.005),(.319,.149,.094,-.010),(.349,.081,.073,-.012),
    ],n=13,fold=.043)
    model.loft("Layered coat skirt","Hips",cloth,[
        (-.151,.166,.113,.006),(-.129,.183,.123,.002),(.011,.137,.105,0),(.047,.120,.093,0),
    ],n=13,fold=.045)
    model.loft("Leather sword belt","Hips","hide",[
        (.015,.145,.112,0),(.048,.146,.111,0),
    ],n=11)
    model.box("Square belt clasp","Hips","brass",(0,.032,.113),(.062,.041,.022),bevel=.006)
    if executioner:
        model.loft("Rounded infantry breastplate","Chest","steel" if tier==2 else "iron",[
            (.064,.152,.117,0),(.121,.170,.128,0),(.225,.188,.127,0),
            (.271,.177,.117,0),(.298,.148,.107,0),
        ],n=13)
    else:
        for x in (-.092,-.046,0,.046,.092):
            model.tube("Gambeson stitching","Chest","hide",[
                (x*.88,.069,.104),(x,.171,.116),(x*1.10,.269,.110),
            ],[.0027,.004,.003],n=5)
    if tier==2:
        model.loft("Split field mantle","BackSpring","wine" if executioner else "hide",
                   CAPES,n=13,opening=1.34,fold=.10)
    for side,s in (("L",1),("R",-1)):
        upper,fore,hand = (f"{part}{side}" for part in ("UpperArm","Forearm","Hand"))
        model.tube("Padded shoulder join","Chest",cloth,[
            (s*.115,.264,0),(s*.174,.293,0),(s*.224,.303,0),
        ],[.053,.058,.044],n=7)
        model.loft("Long padded sleeve",upper,cloth,[
            (-.252,.047,.049,0),(-.145,.057,.061,0),(-.056,.070,.065,0),(.027,.049,.048,0),
        ],n=9)
        model.ell("Riveted shoulder saddle",upper,
                  "steel" if tier else "leather",(s*.014,.006,0),
                  (.080,.055,.074),segments=10,rings=5)
        model.loft("Long leather forearm guard",fore,"hide",[
            (-.226,.038,.044,0),(-.184,.050,.052,0),(-.053,.056,.052,0),(.021,.048,.048,0),
        ],n=9)
        model.ell("Wrapped polearm hand",hand,"leather",(0,-.022,.011),
                  (.047,.054,.046),segments=9)
        for joint,color in ((f"Thigh{side}","hide"),(f"Shin{side}","iron" if tier==2 else "leather")):
            model.loft("Fitted lower armor",joint,color,[
                (-.192,.047,.048,0),(-.098,.057,.059,0),(.020,.063,.062,0),
            ],n=9)
        model.ell("Reinforced leather boot",f"Foot{side}","hide",
                  (0,-.021,.048),(.057,.047,.093),segments=10)


def helmet(model, tier, executioner, sweeper):
    model.loft("Sculpted infantry face","Head","skin",HUMAN_FACE,n=18,
               transform=lambda p:face_transform(p,False))
    model.loft("Pointed oak beard","Head","darkwood",[
        (-.144,.027,.029,.042),(-.111,.067,.055,.041),(-.068,.085,.059,.032),
        (-.035,.080,.044,.035),
    ],n=11,opening=0,fold=.052)
    for s in (-1,1):
        model.ell("Focused eye","Head","ink",(s*.046,.032,.096),
                  (.016,.007,.008),segments=8,rings=4)
        model.tube("Handlebar moustache","Head","darkwood",
                   [(0,-.036,.117),(s*.038,-.040,.105),(s*.064,-.057,.092)],
                   [.009,.012,.005],n=6)
    brim = .135 if executioner else .178
    model.loft("Rolled kettle helmet","Head","steel" if executioner else "steel_dark",[
        (.071,.120,.114,-.004),(.074,brim,brim*.90,-.006),
        (.090,brim,brim*.90,-.008),(.114,.134,.127,-.008),
        (.151,.105,.100,-.014),(.175,.042,.048,-.017),
    ],n=14)
    if tier==2 and sweeper:
        for s in (-1,1):
            model.tube("Forked horn","HeadGear","bone",[
                (s*.113,.116,-.006),(s*.174,.153,-.013),(s*.183,.232,-.027),
            ],[.026,.018,.003],n=7)
    if tier==2 and executioner:
        model.plate("Swept helmet crest","HeadGear","wine",
                    [(.117,-.055),(.238,-.069),(.265,-.119),(.179,-.144),(.128,-.104)],
                    (-.019,.019),axis="x",bevel=.006)


def poleaxe(model, tier, executioner, sweeper):
    node = "WeaponR"
    model.tube("Tapered ash haft",node,"wood",[
        (0,-.669,0),(0,-.510,0),(0,.34,0),(0,1.31,0),(0,HEAD_Y+.125,0),
    ],[.021,.026,.023,.025,.025],n=8)
    model.tube("Both hands' leather binding",node,"hide",
               [(0,-.78,0),(0,-.44,0)],[.028,.028],n=8)
    model.tube("Lower steel ferrule",node,"steel_dark",
               [(0,-.688,0),(0,-.626,0)],[.027,.028],n=8)
    model.tube("Forged head socket",node,"iron",
               [(0,HEAD_Y-.23,0),(0,HEAD_Y+.19,0)],[.037,.031],n=8)
    model.tube("Pike tip",node,"edge",[
        (0,HEAD_Y+.137,0),(0,HEAD_Y+.212,0),(0,HEAD_Y+(.388 if tier==2 and sweeper else .285),0),
    ],[.029,.025,.002],n=7)
    radius = .289 if tier==2 else .219 if executioner else .176
    blade(model, radius, 1, "gold" if executioner and tier==2 else "edge")
    if sweeper and tier==1:
        blade(model,.166,-1,"edge")
    else:
        model.tube("Rear hook" if sweeper else "Hammer poll",node,"iron",[
            (0,HEAD_Y+.02,-.019),(0,HEAD_Y+.019,-.117),
            (0,HEAD_Y-.093 if sweeper else HEAD_Y+.03,-.214 if sweeper else -.142),
        ],[.039,.034,.008 if sweeper else .027],n=7)


def blade(model, height, side, edge):
    profile = [(-.138,.009),(.126,.010),(height,.144),
               (height*.50,.208),(0,.220),(-height*.76,.201),(-height,.143)]
    outline = [(HEAD_Y+y,side*z) for y,z in profile]
    model.plate("Forged curved axe plate","WeaponR","steel_dark",
                outline[::-1] if side>0 else outline,(-.017,.017),axis="x",bevel=.006)
    arc = [(0,HEAD_Y+y,side*z) for y,z in profile[2:]]
    model.tube("Polished cutting edge","WeaponR",edge,arc,[.010,.012,.014,.012,.009],n=6)
